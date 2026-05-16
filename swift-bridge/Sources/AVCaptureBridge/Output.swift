import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

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
        owner?.callbackBox?.emit(
            sampleBuffer: sampleBuffer,
            pixelBuffer: CMSampleBufferGetImageBuffer(sampleBuffer)
        )
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

final class VideoDataOutputBox {
    let output = AVCaptureVideoDataOutput()
    fileprivate var callbackBox: VideoSampleCallbackBox?
    private var delegate: VideoSampleDelegate?
    private var callbackQueue: DispatchQueue?

    deinit {
        clearCallback()
    }

    func infoPayload() -> VideoDataOutputInfoPayload {
        let availableFormats: [UInt32]
        #if os(macOS)
        availableFormats = []
        #else
        availableFormats = output.availableVideoCVPixelFormatTypes.map(\.uint32Value)
        #endif
        return VideoDataOutputInfoPayload(
            connectionCount: output.connections.count,
            alwaysDiscardsLateVideoFrames: output.alwaysDiscardsLateVideoFrames,
            availableVideoCvPixelFormatTypes: availableFormats,
            callbackInstalled: callbackBox != nil,
            videoSettings: avcEncodeVideoSettings(output.videoSettings)
        )
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
        output.setSampleBufferDelegate(delegate, queue: queue)
        callbackBox = box
        self.delegate = delegate
        callbackQueue = queue
    }

    func clearCallback() {
        output.setSampleBufferDelegate(nil, queue: nil)
        delegate = nil
        callbackQueue = nil
        callbackBox?.dispose()
        callbackBox = nil
    }
}

final class AudioDataOutputBox {
    let output = AVCaptureAudioDataOutput()
    fileprivate var callbackBox: AudioSampleCallbackBox?
    private var delegate: AudioSampleDelegate?
    private var callbackQueue: DispatchQueue?

    deinit {
        clearCallback()
    }

    func infoPayload() -> AudioDataOutputInfoPayload {
        AudioDataOutputInfoPayload(
            connectionCount: output.connections.count,
            callbackInstalled: callbackBox != nil,
            audioSettings: avcEncodeAudioSettings(output.audioSettings)
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
        output.setSampleBufferDelegate(delegate, queue: queue)
        callbackBox = box
        self.delegate = delegate
        callbackQueue = queue
    }

    func clearCallback() {
        output.setSampleBufferDelegate(nil, queue: nil)
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
    Unmanaged.passRetained(VideoDataOutputBox()).toOpaque()
}

@_cdecl("av_capture_video_output_release")
public func av_capture_video_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    guard let outputPtr else { return }
    Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).release()
}

@_cdecl("av_capture_video_output_info_json")
public func av_capture_video_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
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
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    do {
        let payload = try avcDecodeJSON(settingsJson, as: VideoOutputSettingsPayload.self)
        output.output.videoSettings = avcVideoSettingsDictionary(from: payload)
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
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    output.output.alwaysDiscardsLateVideoFrames = enabled
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
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let queueLabel = String(cString: queueLabelPtr)
    output.setCallback(callback: callback, userData: userData, dropUserData: dropUserData, queueLabel: queueLabel)
    return AVC_OK
}

@_cdecl("av_capture_video_output_clear_sample_buffer_callback")
public func av_capture_video_output_clear_sample_buffer_callback(_ outputPtr: UnsafeMutableRawPointer) {
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    output.clearCallback()
}

@_cdecl("av_capture_audio_output_create")
public func av_capture_audio_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    Unmanaged.passRetained(AudioDataOutputBox()).toOpaque()
}

@_cdecl("av_capture_audio_output_release")
public func av_capture_audio_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    guard let outputPtr else { return }
    Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).release()
}

@_cdecl("av_capture_audio_output_info_json")
public func av_capture_audio_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
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
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    do {
        let payload = try avcDecodeJSON(settingsJson, as: AudioOutputSettingsPayload.self)
        output.output.audioSettings = avcAudioSettingsDictionary(from: payload)
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
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    let queueLabel = String(cString: queueLabelPtr)
    output.setCallback(callback: callback, userData: userData, dropUserData: dropUserData, queueLabel: queueLabel)
    return AVC_OK
}

@_cdecl("av_capture_audio_output_clear_sample_buffer_callback")
public func av_capture_audio_output_clear_sample_buffer_callback(_ outputPtr: UnsafeMutableRawPointer) {
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue()
    output.clearCallback()
}
