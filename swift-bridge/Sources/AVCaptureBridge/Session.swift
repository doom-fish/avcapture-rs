import AVFoundation
import Foundation

@_cdecl("av_capture_session_create")
public func av_capture_session_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(SessionBox())
}

@_cdecl("av_capture_session_release")
public func av_capture_session_release(_ sessionPtr: UnsafeMutableRawPointer?) {
    avcRelease(sessionPtr, as: SessionBox.self)
}

@_cdecl("av_capture_session_info_json")
public func av_capture_session_info_json(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let payload = avcSessionInfoPayload(from: avcSessionBox(sessionPtr))
    do {
        return ffiString(try avcEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_session_connections_count")
public func av_capture_session_connections_count(_ sessionPtr: UnsafeMutableRawPointer) -> Int {
    avcSessionBox(sessionPtr).session.connections.count
}

@_cdecl("av_capture_session_connection_at_index")
public func av_capture_session_connection_at_index(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let session = avcSessionBox(sessionPtr).session
    guard index >= 0, index < session.connections.count else {
        outErrorMessage?.pointee = ffiString("session connection index out of range")
        return nil
    }
    return avcRetain(ConnectionBox(session.connections[index]))
}

@_cdecl("av_capture_session_begin_configuration")
public func av_capture_session_begin_configuration(_ sessionPtr: UnsafeMutableRawPointer) {
    avcSessionBox(sessionPtr).session.beginConfiguration()
}

@_cdecl("av_capture_session_commit_configuration")
public func av_capture_session_commit_configuration(_ sessionPtr: UnsafeMutableRawPointer) {
    avcSessionBox(sessionPtr).session.commitConfiguration()
}

@_cdecl("av_capture_session_start_running")
public func av_capture_session_start_running(_ sessionPtr: UnsafeMutableRawPointer) {
    avcSessionBox(sessionPtr).session.startRunning()
}

@_cdecl("av_capture_session_stop_running")
public func av_capture_session_stop_running(_ sessionPtr: UnsafeMutableRawPointer) {
    avcSessionBox(sessionPtr).session.stopRunning()
}

@_cdecl("av_capture_session_can_set_preset")
public func av_capture_session_can_set_preset(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ presetPtr: UnsafePointer<CChar>
) -> Bool {
    let session = avcSessionBox(sessionPtr).session
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
    let session = avcSessionBox(sessionPtr).session
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
    let session = avcSessionBox(sessionPtr).session
    let input = avcInputBox(inputPtr).input
    return session.canAddInput(input)
}

@_cdecl("av_capture_session_add_input")
public func av_capture_session_add_input(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ inputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = avcSessionBox(sessionPtr).session
    let input = avcInputBox(inputPtr).input
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
    let session = avcSessionBox(sessionPtr).session
    session.removeInput(avcInputBox(inputPtr).input)
}

@_cdecl("av_capture_session_can_add_output")
public func av_capture_session_can_add_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    let session = avcSessionBox(sessionPtr).session
    return session.canAddOutput(avcOutputBox(outputPtr).output)
}

@_cdecl("av_capture_session_add_output")
public func av_capture_session_add_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let session = avcSessionBox(sessionPtr).session
    let output = avcOutputBox(outputPtr).output
    guard session.canAddOutput(output) else {
        outErrorMessage?.pointee = ffiString("session cannot add capture output")
        return AVC_SESSION_ERROR
    }
    session.addOutput(output)
    return AVC_OK
}

@_cdecl("av_capture_session_remove_output")
public func av_capture_session_remove_output(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outputPtr: UnsafeMutableRawPointer
) {
    let session = avcSessionBox(sessionPtr).session
    session.removeOutput(avcOutputBox(outputPtr).output)
}
