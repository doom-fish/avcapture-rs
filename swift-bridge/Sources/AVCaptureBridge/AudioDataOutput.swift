import AVFoundation
import CoreMedia
import Foundation

private struct AudioDataOutputInfoSnapshot: Codable {
    let connectionCount: Int
    let callbackInstalled: Bool
    let audioSettings: AudioOutputSettingsPayload?
    let droppedSampleCount: Int
    let lastDroppedSampleReason: String?
}

private struct AudioPreviewOutputInfoPayload: Codable {
    let connectionCount: Int
    let outputDeviceUniqueID: String?
    let volume: Float
}

private final class AudioSampleCallbackBox {
    private let callback: AVCAudioSampleCallback
    private let contextOwner: AVCCallbackContextOwner

    init(
        callback: @escaping AVCAudioSampleCallback,
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

    func emit(sampleBuffer: CMSampleBuffer) {
        let contextOwner = self.contextOwner
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(contextOwner.userData, sampleOpaque)
    }
}

private final class AudioSampleDelegate: NSObject, AVCaptureAudioDataOutputSampleBufferDelegate {
    private weak var owner: AudioDataOutputBox?

    init(owner: AudioDataOutputBox) {
        self.owner = owner
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitSample(sampleBuffer)
    }
}

final class AudioDataOutputBox: CaptureOutputBoxBase {
    let audioOutput = AVCaptureAudioDataOutput()
    private let callbackLock = NSLock()
    private let counterLock = NSLock()
    private let delegateSlot = AVCDelegateSlot("audio sample-buffer delegate")
    private var callbackBox: AudioSampleCallbackBox?
    private var delegate: AudioSampleDelegate?
    private var callbackQueue: AVCSerialCallbackQueue?
    private var droppedSampleCount = 0
    private var lastDroppedSampleReason: String?

    override var output: AVCaptureOutput {
        audioOutput
    }

    deinit {
        clearCallback()
    }

    fileprivate func infoPayload() -> AudioDataOutputInfoSnapshot {
        counterLock.lock()
        let droppedSampleCount = self.droppedSampleCount
        let lastDroppedSampleReason = self.lastDroppedSampleReason
        counterLock.unlock()
        return AudioDataOutputInfoSnapshot(
            connectionCount: audioOutput.connections.count,
            callbackInstalled: delegateSlot.isOccupied,
            audioSettings: avcEncodeAudioSettings(audioOutput.audioSettings),
            droppedSampleCount: droppedSampleCount,
            lastDroppedSampleReason: lastDroppedSampleReason
        )
    }

    func noteDroppedReasonIfPresent(_ sampleBuffer: CMSampleBuffer) {
        guard let reason = avcDroppedSampleReason(from: sampleBuffer) else { return }
        counterLock.lock()
        droppedSampleCount += 1
        lastDroppedSampleReason = reason
        counterLock.unlock()
    }

    fileprivate func emitSample(_ sampleBuffer: CMSampleBuffer) {
        noteDroppedReasonIfPresent(sampleBuffer)
        callbackLock.lock()
        let callbackBox = self.callbackBox
        callbackLock.unlock()
        callbackBox?.emit(sampleBuffer: sampleBuffer)
    }

    func setCallback(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        releaseUserData: AVCDropCallback?,
        queueLabel: String
    ) throws {
        let box = AudioSampleCallbackBox(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: releaseUserData
        )
        let delegate = AudioSampleDelegate(owner: self)
        let queue = AVCSerialCallbackQueue(label: queueLabel)
        try delegateSlot.acquire(box)
        audioOutput.setSampleBufferDelegate(delegate, queue: queue.queue)
        callbackLock.lock()
        callbackBox = box
        self.delegate = delegate
        callbackQueue = queue
        callbackLock.unlock()
    }

    func clearCallback() {
        callbackLock.lock()
        let box = callbackBox
        let delegate = self.delegate
        let queue = callbackQueue
        if let box, delegateSlot.release(box) {
            callbackBox = nil
            self.delegate = nil
            callbackQueue = nil
        }
        callbackLock.unlock()
        guard let delegate else {
            return
        }
        if audioOutput.sampleBufferDelegate === delegate {
            audioOutput.setSampleBufferDelegate(nil, queue: nil)
        }
        queue?.drain()
    }

    func installStreamDelegate(
        owner: AnyObject,
        delegate: AVCaptureAudioDataOutputSampleBufferDelegate,
        queue: AVCSerialCallbackQueue
    ) throws {
        try delegateSlot.acquire(owner)
        audioOutput.setSampleBufferDelegate(delegate, queue: queue.queue)
    }

    func removeStreamDelegate(
        owner: AnyObject,
        delegate: AVCaptureAudioDataOutputSampleBufferDelegate,
        queue: AVCSerialCallbackQueue
    ) {
        guard delegateSlot.release(owner) else {
            return
        }
        if audioOutput.sampleBufferDelegate === delegate {
            audioOutput.setSampleBufferDelegate(nil, queue: nil)
        }
        queue.drain()
    }
}

final class AudioPreviewOutputBox: CaptureOutputBoxBase {
    let previewOutput = AVCaptureAudioPreviewOutput()

    override var output: AVCaptureOutput {
        previewOutput
    }

    fileprivate func infoPayload() -> AudioPreviewOutputInfoPayload {
        AudioPreviewOutputInfoPayload(
            connectionCount: previewOutput.connections.count,
            outputDeviceUniqueID: previewOutput.outputDeviceUniqueID,
            volume: previewOutput.volume
        )
    }
}

@_cdecl("av_capture_audio_output_create")
public func av_capture_audio_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(AudioDataOutputBox())
}

@_cdecl("av_capture_audio_output_release")
public func av_capture_audio_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    avcRelease(outputPtr, as: AudioDataOutputBox.self)
}

@_cdecl("av_capture_audio_output_info_json")
public func av_capture_audio_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
    do {
        return ffiString(try avcEncodeJSON(output.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_audio_output_set_audio_settings_json")
public func av_capture_audio_output_set_audio_settings_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
    do {
        guard let settingsJson else {
            output.audioOutput.audioSettings = nil
            return AVC_OK
        }
        let payload = try avcDecodeJSON(settingsJson, as: AudioOutputSettingsPayload.self)
        output.audioOutput.audioSettings = avcAudioSettingsDictionary(from: payload)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_audio_output_set_sample_buffer_callback")
public func av_capture_audio_output_set_sample_buffer_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCAudioSampleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing audio sample callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
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

@_cdecl("av_capture_audio_output_clear_sample_buffer_callback")
public func av_capture_audio_output_clear_sample_buffer_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: AudioDataOutputBox.self).clearCallback()
}

@_cdecl("av_capture_audio_preview_output_create")
public func av_capture_audio_preview_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(AudioPreviewOutputBox())
}

@_cdecl("av_capture_audio_preview_output_release")
public func av_capture_audio_preview_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    avcRelease(outputPtr, as: AudioPreviewOutputBox.self)
}

@_cdecl("av_capture_audio_preview_output_info_json")
public func av_capture_audio_preview_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcUnretained(outputPtr, as: AudioPreviewOutputBox.self)
    do {
        return ffiString(try avcEncodeJSON(output.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_audio_preview_output_set_output_device_unique_id")
public func av_capture_audio_preview_output_set_output_device_unique_id(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outputDeviceUniqueIDPtr: UnsafePointer<CChar>?
) {
    let output = avcUnretained(outputPtr, as: AudioPreviewOutputBox.self)
    output.previewOutput.outputDeviceUniqueID = outputDeviceUniqueIDPtr.map { String(cString: $0) }
}

@_cdecl("av_capture_audio_preview_output_set_volume")
public func av_capture_audio_preview_output_set_volume(
    _ outputPtr: UnsafeMutableRawPointer,
    _ volume: Float
) {
    avcUnretained(outputPtr, as: AudioPreviewOutputBox.self).previewOutput.volume = volume
}
