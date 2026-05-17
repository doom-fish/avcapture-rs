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
    let callback: AVCAudioSampleCallback
    let userData: UnsafeMutableRawPointer?
    let dropUserData: AVCDropCallback?
    private var disposed = false

    init(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
    }

    func emit(sampleBuffer: CMSampleBuffer) {
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(userData, sampleOpaque)
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        if let userData, let dropUserData {
            dropUserData(userData)
        }
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
        owner?.noteDroppedReasonIfPresent(sampleBuffer)
        owner?.callbackBox?.emit(sampleBuffer: sampleBuffer)
    }
}

final class AudioDataOutputBox: CaptureOutputBoxBase {
    let audioOutput = AVCaptureAudioDataOutput()
    fileprivate var callbackBox: AudioSampleCallbackBox?
    private var delegate: AudioSampleDelegate?
    private var callbackQueue: DispatchQueue?
    private var droppedSampleCount = 0
    private var lastDroppedSampleReason: String?

    override var output: AVCaptureOutput {
        audioOutput
    }

    deinit {
        clearCallback()
    }

    fileprivate func infoPayload() -> AudioDataOutputInfoSnapshot {
        AudioDataOutputInfoSnapshot(
            connectionCount: audioOutput.connections.count,
            callbackInstalled: callbackBox != nil,
            audioSettings: avcEncodeAudioSettings(audioOutput.audioSettings),
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
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?,
        queueLabel: String
    ) {
        clearCallback()
        let box = AudioSampleCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = AudioSampleDelegate(owner: self)
        let queue = DispatchQueue(label: queueLabel)
        audioOutput.setSampleBufferDelegate(delegate, queue: queue)
        callbackBox = box
        self.delegate = delegate
        callbackQueue = queue
    }

    func clearCallback() {
        audioOutput.setSampleBufferDelegate(nil, queue: nil)
        delegate = nil
        callbackQueue = nil
        callbackBox?.dispose()
        callbackBox = nil
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
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing audio sample callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    output.setCallback(callback: callback, userData: userData, dropUserData: dropUserData, queueLabel: queueLabel)
    return AVC_OK
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
