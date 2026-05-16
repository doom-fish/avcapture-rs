import AVFoundation
import CoreMedia
import Foundation

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
        owner?.callbackBox?.emit(sampleBuffer: sampleBuffer)
    }
}

final class AudioDataOutputBox: CaptureOutputBoxBase {
    let audioOutput = AVCaptureAudioDataOutput()
    fileprivate var callbackBox: AudioSampleCallbackBox?
    private var delegate: AudioSampleDelegate?
    private var callbackQueue: DispatchQueue?

    override var output: AVCaptureOutput {
        audioOutput
    }

    deinit {
        clearCallback()
    }

    func infoPayload() -> AudioDataOutputInfoPayload {
        AudioDataOutputInfoPayload(
            connectionCount: audioOutput.connections.count,
            callbackInstalled: callbackBox != nil,
            audioSettings: avcEncodeAudioSettings(audioOutput.audioSettings)
        )
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
