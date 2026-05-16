import AVFoundation
import Foundation

@_cdecl("av_capture_session_create")
public func av_capture_session_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    Unmanaged.passRetained(AVCaptureSession()).toOpaque()
}

@_cdecl("av_capture_session_release")
public func av_capture_session_release(_ sessionPtr: UnsafeMutableRawPointer?) {
    guard let sessionPtr else { return }
    Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).release()
}

@_cdecl("av_capture_session_info_json")
public func av_capture_session_info_json(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let payload = CaptureSessionInfoPayload(
        sessionPreset: avcEncodeSessionPreset(session.sessionPreset),
        inputCount: session.inputs.count,
        outputCount: session.outputs.count,
        connectionCount: session.connections.count,
        isRunning: session.isRunning
    )
    do {
        return ffiString(try avcEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_session_begin_configuration")
public func av_capture_session_begin_configuration(_ sessionPtr: UnsafeMutableRawPointer) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    session.beginConfiguration()
}

@_cdecl("av_capture_session_commit_configuration")
public func av_capture_session_commit_configuration(_ sessionPtr: UnsafeMutableRawPointer) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    session.commitConfiguration()
}

@_cdecl("av_capture_session_start_running")
public func av_capture_session_start_running(_ sessionPtr: UnsafeMutableRawPointer) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    session.startRunning()
}

@_cdecl("av_capture_session_stop_running")
public func av_capture_session_stop_running(_ sessionPtr: UnsafeMutableRawPointer) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    session.stopRunning()
}

@_cdecl("av_capture_session_can_set_preset")
public func av_capture_session_can_set_preset(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ presetPtr: UnsafePointer<CChar>
) -> Bool {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let raw = String(cString: presetPtr)
    guard let preset = avcDecodeSessionPreset(raw) else {
        return false
    }
    return session.canSetSessionPreset(preset)
}

@_cdecl("av_capture_session_set_preset")
public func av_capture_session_set_preset(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ presetPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let raw = String(cString: presetPtr)
    guard let preset = avcDecodeSessionPreset(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported session preset: \(raw)")
        return AVC_INVALID_ARGUMENT
    }
    guard session.canSetSessionPreset(preset) else {
        outErrorMessage?.pointee = ffiString("session cannot use preset \(raw) with the current graph")
        return AVC_SESSION_ERROR
    }
    session.sessionPreset = preset
    return AVC_OK
}

@_cdecl("av_capture_session_can_add_input")
public func av_capture_session_can_add_input(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ inputPtr: UnsafeMutableRawPointer
) -> Bool {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let input = Unmanaged<AVCaptureDeviceInput>.fromOpaque(inputPtr).takeUnretainedValue()
    return session.canAddInput(input)
}

@_cdecl("av_capture_session_add_input")
public func av_capture_session_add_input(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ inputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let input = Unmanaged<AVCaptureDeviceInput>.fromOpaque(inputPtr).takeUnretainedValue()
    guard session.canAddInput(input) else {
        outErrorMessage?.pointee = ffiString("session cannot add capture input")
        return AVC_SESSION_ERROR
    }
    session.addInput(input)
    return AVC_OK
}

@_cdecl("av_capture_session_remove_input")
public func av_capture_session_remove_input(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ inputPtr: UnsafeMutableRawPointer
) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let input = Unmanaged<AVCaptureDeviceInput>.fromOpaque(inputPtr).takeUnretainedValue()
    session.removeInput(input)
}

@_cdecl("av_capture_session_can_add_video_output")
public func av_capture_session_can_add_video_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    return session.canAddOutput(output)
}

@_cdecl("av_capture_session_add_video_output")
public func av_capture_session_add_video_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    guard session.canAddOutput(output) else {
        outErrorMessage?.pointee = ffiString("session cannot add video data output")
        return AVC_SESSION_ERROR
    }
    session.addOutput(output)
    return AVC_OK
}

@_cdecl("av_capture_session_remove_video_output")
public func av_capture_session_remove_video_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let output = Unmanaged<VideoDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    session.removeOutput(output)
}

@_cdecl("av_capture_session_can_add_audio_output")
public func av_capture_session_can_add_audio_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    return session.canAddOutput(output)
}

@_cdecl("av_capture_session_add_audio_output")
public func av_capture_session_add_audio_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    guard session.canAddOutput(output) else {
        outErrorMessage?.pointee = ffiString("session cannot add audio data output")
        return AVC_SESSION_ERROR
    }
    session.addOutput(output)
    return AVC_OK
}

@_cdecl("av_capture_session_remove_audio_output")
public func av_capture_session_remove_audio_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) {
    let session = Unmanaged<AVCaptureSession>.fromOpaque(sessionPtr).takeUnretainedValue()
    let output = Unmanaged<AudioDataOutputBox>.fromOpaque(outputPtr).takeUnretainedValue().output
    session.removeOutput(output)
}
