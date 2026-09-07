import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

private struct VideoDataOutputInfoSnapshot: Codable {
    let connectionCount: Int
    let alwaysDiscardsLateVideoFrames: Bool
    let availableVideoCVPixelFormatTypes: [UInt32]
    let callbackInstalled: Bool
    let videoSettings: VideoOutputSettingsPayload?
    let droppedSampleCount: Int
    let lastDroppedSampleReason: String?
}

private protocol VideoOutputCallbackBox: AnyObject {
    func emitSample(sampleBuffer: CMSampleBuffer, pixelBuffer: CVPixelBuffer?)
    func emitDropped(sampleBuffer: CMSampleBuffer, reason: String?, total: UInt64)
}

private final class VideoSampleCallbackBox: VideoOutputCallbackBox {
    private let callback: AVCVideoSampleCallback
    private let contextOwner: AVCCallbackContextOwner

    init(
        callback: @escaping AVCVideoSampleCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        releaseUserData: AVCDropCallback?
    ) {
        self.callback = callback
        contextOwner = AVCCallbackContextOwner(
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: releaseUserData
        )
    }

    func emitSample(sampleBuffer: CMSampleBuffer, pixelBuffer: CVPixelBuffer?) {
        let contextOwner = self.contextOwner
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        let pixelOpaque = pixelBuffer.map { Unmanaged.passRetained($0).toOpaque() }
        callback(contextOwner.userData, sampleOpaque, pixelOpaque)
    }

    func emitDropped(sampleBuffer: CMSampleBuffer, reason: String?, total: UInt64) {
    }
}

private final class VideoDataOutputEventCallbackBox: VideoOutputCallbackBox {
    private let callback: AVCVideoDataOutputEventCallback
    private let contextOwner: AVCCallbackContextOwner

    init(
        callback: @escaping AVCVideoDataOutputEventCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        releaseUserData: AVCDropCallback?
    ) {
        self.callback = callback
        contextOwner = AVCCallbackContextOwner(
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: releaseUserData
        )
    }

    func emitSample(sampleBuffer: CMSampleBuffer, pixelBuffer: CVPixelBuffer?) {
        let contextOwner = self.contextOwner
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        let pixelOpaque = pixelBuffer.map { Unmanaged.passRetained($0).toOpaque() }
        callback(contextOwner.userData, 0, sampleOpaque, pixelOpaque, nil, 0)
    }

    func emitDropped(sampleBuffer: CMSampleBuffer, reason: String?, total: UInt64) {
        let contextOwner = self.contextOwner
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(
            contextOwner.userData,
            1,
            sampleOpaque,
            nil,
            reason.flatMap(ffiString),
            total
        )
    }
}

private final class VideoSampleDelegate: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    private weak var owner: VideoDataOutputBox?

    init(owner: VideoDataOutputBox) {
        self.owner = owner
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitSample(
            sampleBuffer: sampleBuffer,
            pixelBuffer: CMSampleBufferGetImageBuffer(sampleBuffer)
        )
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didDrop sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitDropped(sampleBuffer: sampleBuffer)
    }
}

final class VideoDataOutputBox: CaptureOutputBoxBase {
    fileprivate enum CallbackKind {
        case sample
        case event
    }

    let videoOutput = AVCaptureVideoDataOutput()
    private let callbackLock = NSLock()
    private let counterLock = NSLock()
    private let delegateSlot = AVCDelegateSlot("video sample-buffer delegate")
    private var callbackBox: VideoOutputCallbackBox?
    private var callbackKind: CallbackKind?
    private var delegate: VideoSampleDelegate?
    private var callbackQueue: AVCSerialCallbackQueue?
    private var droppedSampleCount = 0
    private var lastDroppedSampleReason: String?

    override var output: AVCaptureOutput {
        videoOutput
    }

    deinit {
        clearCallback()
    }

    fileprivate func infoPayload() -> VideoDataOutputInfoSnapshot {
        let availableFormats = videoOutput.availableVideoPixelFormatTypes.map { UInt32($0) }
        counterLock.lock()
        let droppedSampleCount = self.droppedSampleCount
        let lastDroppedSampleReason = self.lastDroppedSampleReason
        counterLock.unlock()
        return VideoDataOutputInfoSnapshot(
            connectionCount: videoOutput.connections.count,
            alwaysDiscardsLateVideoFrames: videoOutput.alwaysDiscardsLateVideoFrames,
            availableVideoCVPixelFormatTypes: availableFormats,
            callbackInstalled: delegateSlot.isOccupied,
            videoSettings: avcEncodeVideoSettings(videoOutput.videoSettings),
            droppedSampleCount: droppedSampleCount,
            lastDroppedSampleReason: lastDroppedSampleReason
        )
    }

    func recordDroppedSample(_ sampleBuffer: CMSampleBuffer) -> (String?, UInt64) {
        recordDroppedReason(avcDroppedSampleReason(from: sampleBuffer))
    }

    func recordDroppedReason(_ reason: String?) -> (String?, UInt64) {
        counterLock.lock()
        droppedSampleCount += 1
        lastDroppedSampleReason = reason
        let total = UInt64(droppedSampleCount)
        counterLock.unlock()
        return (reason, total)
    }

    fileprivate func emitSample(sampleBuffer: CMSampleBuffer, pixelBuffer: CVPixelBuffer?) {
        callbackLock.lock()
        let callbackBox = self.callbackBox
        callbackLock.unlock()
        callbackBox?.emitSample(sampleBuffer: sampleBuffer, pixelBuffer: pixelBuffer)
    }

    fileprivate func emitDropped(sampleBuffer: CMSampleBuffer) {
        let (reason, total) = recordDroppedSample(sampleBuffer)
        callbackLock.lock()
        let callbackBox = self.callbackBox
        callbackLock.unlock()
        callbackBox?.emitDropped(sampleBuffer: sampleBuffer, reason: reason, total: total)
    }

    func setCallback(
        callback: @escaping AVCVideoSampleCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        releaseUserData: AVCDropCallback?,
        queueLabel: String
    ) throws {
        let box = VideoSampleCallbackBox(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: releaseUserData
        )
        let delegate = VideoSampleDelegate(owner: self)
        let queue = AVCSerialCallbackQueue(label: queueLabel)
        try installCallback(box: box, kind: .sample, delegate: delegate, queue: queue)
    }

    func setEventCallback(
        callback: @escaping AVCVideoDataOutputEventCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        releaseUserData: AVCDropCallback?,
        queueLabel: String
    ) throws {
        let box = VideoDataOutputEventCallbackBox(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: releaseUserData
        )
        let delegate = VideoSampleDelegate(owner: self)
        let queue = AVCSerialCallbackQueue(label: queueLabel)
        try installCallback(box: box, kind: .event, delegate: delegate, queue: queue)
    }

    private func installCallback(
        box: VideoOutputCallbackBox,
        kind: CallbackKind,
        delegate: VideoSampleDelegate,
        queue: AVCSerialCallbackQueue
    ) throws {
        try delegateSlot.acquire(box)
        videoOutput.setSampleBufferDelegate(delegate, queue: queue.queue)
        callbackLock.lock()
        callbackBox = box
        callbackKind = kind
        self.delegate = delegate
        callbackQueue = queue
        callbackLock.unlock()
    }

    fileprivate func clearCallback(_ expectedKind: CallbackKind? = nil) {
        callbackLock.lock()
        let box = callbackBox
        let delegate = self.delegate
        let queue = callbackQueue
        let kindMatches = expectedKind == nil || callbackKind == expectedKind
        if kindMatches, let box, delegateSlot.release(box) {
            callbackBox = nil
            callbackKind = nil
            self.delegate = nil
            callbackQueue = nil
        }
        callbackLock.unlock()
        guard let box, let delegate, !delegateSlot.isOwned(by: box) else {
            return
        }
        if videoOutput.sampleBufferDelegate === delegate {
            videoOutput.setSampleBufferDelegate(nil, queue: nil)
        }
        queue?.drain()
    }

    func installStreamDelegate(
        owner: AnyObject,
        delegate: AVCaptureVideoDataOutputSampleBufferDelegate,
        queue: AVCSerialCallbackQueue
    ) throws {
        try delegateSlot.acquire(owner)
        videoOutput.setSampleBufferDelegate(delegate, queue: queue.queue)
    }

    func removeStreamDelegate(
        owner: AnyObject,
        delegate: AVCaptureVideoDataOutputSampleBufferDelegate,
        queue: AVCSerialCallbackQueue
    ) {
        guard delegateSlot.release(owner) else {
            return
        }
        if videoOutput.sampleBufferDelegate === delegate {
            videoOutput.setSampleBufferDelegate(nil, queue: nil)
        }
        queue.drain()
    }
}

@_cdecl("av_capture_video_output_create")
public func av_capture_video_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(VideoDataOutputBox())
}

@_cdecl("av_capture_video_output_release")
public func av_capture_video_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    avcRelease(outputPtr, as: VideoDataOutputBox.self)
}

@_cdecl("av_capture_video_output_info_json")
public func av_capture_video_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    do {
        return ffiString(try avcEncodeJSON(output.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_video_output_set_video_settings_json")
public func av_capture_video_output_set_video_settings_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    do {
        guard let settingsJson else {
            output.videoOutput.videoSettings = nil
            return AVC_OK
        }
        let payload = try avcDecodeJSON(settingsJson, as: VideoOutputSettingsPayload.self)
        output.videoOutput.videoSettings = avcVideoSettingsDictionary(from: payload)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_video_output_set_always_discards_late_video_frames")
public func av_capture_video_output_set_always_discards_late_video_frames(
    _ outputPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    avcUnretained(outputPtr, as: VideoDataOutputBox.self).videoOutput.alwaysDiscardsLateVideoFrames = enabled
}

@_cdecl("av_capture_video_output_set_sample_buffer_callback")
public func av_capture_video_output_set_sample_buffer_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCVideoSampleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing video sample callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        try output.setCallback(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: dropUserData,
            queueLabel: queueLabel
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_CALLBACK_ERROR)
    }
}

@_cdecl("av_capture_video_output_clear_sample_buffer_callback")
public func av_capture_video_output_clear_sample_buffer_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: VideoDataOutputBox.self).clearCallback(.sample)
}

@_cdecl("av_capture_video_output_set_sample_buffer_event_callback")
public func av_capture_video_output_set_sample_buffer_event_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCVideoDataOutputEventCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing video data-output event callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        try output.setEventCallback(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: dropUserData,
            queueLabel: queueLabel
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_CALLBACK_ERROR)
    }
}

@_cdecl("av_capture_video_output_clear_sample_buffer_event_callback")
public func av_capture_video_output_clear_sample_buffer_event_callback(
    _ outputPtr: UnsafeMutableRawPointer
) {
    avcUnretained(outputPtr, as: VideoDataOutputBox.self).clearCallback(.event)
}

@_cdecl("av_capture_video_output_record_drop_for_testing")
public func av_capture_video_output_record_drop_for_testing(
    _ outputPtr: UnsafeMutableRawPointer,
    _ reasonPtr: UnsafePointer<CChar>?
) -> UInt64 {
    let reason = reasonPtr.map(String.init(cString:))
    return avcUnretained(outputPtr, as: VideoDataOutputBox.self)
        .recordDroppedReason(reason).1
}
