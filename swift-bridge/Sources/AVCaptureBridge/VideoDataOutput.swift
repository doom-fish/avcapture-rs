import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

private struct VideoDataOutputInfoSnapshot: Codable {
    let connectionCount: Int
    let alwaysDiscardsLateVideoFrames: Bool
    let availableVideoCvPixelFormatTypes: [UInt32]
    let callbackInstalled: Bool
    let videoSettings: VideoOutputSettingsPayload?
    let droppedSampleCount: Int
    let lastDroppedSampleReason: String?
}

private final class VideoSampleCallbackBox {
    let callback: AVCVideoSampleCallback
    let userData: UnsafeMutableRawPointer?
    let dropUserData: AVCDropCallback?
    private var disposed = false

    init(
        callback: @escaping AVCVideoSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
    }

    func emit(sampleBuffer: CMSampleBuffer, pixelBuffer: CVPixelBuffer?) {
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        let pixelOpaque = pixelBuffer.map { Unmanaged.passRetained($0).toOpaque() }
        callback(userData, sampleOpaque, pixelOpaque)
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        if let userData, let dropUserData {
            dropUserData(userData)
        }
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
        owner?.noteDroppedReasonIfPresent(sampleBuffer)
        owner?.callbackBox?.emit(
            sampleBuffer: sampleBuffer,
            pixelBuffer: CMSampleBufferGetImageBuffer(sampleBuffer)
        )
    }
}

final class VideoDataOutputBox: CaptureOutputBoxBase {
    let videoOutput = AVCaptureVideoDataOutput()
    fileprivate var callbackBox: VideoSampleCallbackBox?
    private var delegate: VideoSampleDelegate?
    private var callbackQueue: DispatchQueue?
    private var droppedSampleCount = 0
    private var lastDroppedSampleReason: String?

    override var output: AVCaptureOutput {
        videoOutput
    }

    deinit {
        clearCallback()
    }

    fileprivate func infoPayload() -> VideoDataOutputInfoSnapshot {
        let availableFormats: [UInt32]
        #if os(macOS)
        availableFormats = []
        #else
        availableFormats = videoOutput.availableVideoCVPixelFormatTypes.map(\.uint32Value)
        #endif
        return VideoDataOutputInfoSnapshot(
            connectionCount: videoOutput.connections.count,
            alwaysDiscardsLateVideoFrames: videoOutput.alwaysDiscardsLateVideoFrames,
            availableVideoCvPixelFormatTypes: availableFormats,
            callbackInstalled: callbackBox != nil,
            videoSettings: avcEncodeVideoSettings(videoOutput.videoSettings),
            droppedSampleCount: droppedSampleCount,
            lastDroppedSampleReason: lastDroppedSampleReason
        )
    }

    func noteDroppedReasonIfPresent(_ sampleBuffer: CMSampleBuffer) {
        guard let reason = avcDroppedSampleReason(from: sampleBuffer) else { return }
        droppedSampleCount += 1
        lastDroppedSampleReason = reason
    }

    func setCallback(
        callback: @escaping AVCVideoSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?,
        queueLabel: String
    ) {
        clearCallback()
        let box = VideoSampleCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = VideoSampleDelegate(owner: self)
        let queue = DispatchQueue(label: queueLabel)
        videoOutput.setSampleBufferDelegate(delegate, queue: queue)
        callbackBox = box
        self.delegate = delegate
        callbackQueue = queue
    }

    func clearCallback() {
        videoOutput.setSampleBufferDelegate(nil, queue: nil)
        delegate = nil
        callbackQueue = nil
        callbackBox?.dispose()
        callbackBox = nil
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
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing video sample callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    output.setCallback(callback: callback, userData: userData, dropUserData: dropUserData, queueLabel: queueLabel)
    return AVC_OK
}

@_cdecl("av_capture_video_output_clear_sample_buffer_callback")
public func av_capture_video_output_clear_sample_buffer_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: VideoDataOutputBox.self).clearCallback()
}
