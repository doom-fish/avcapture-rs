import AVFoundation
import Foundation

private struct CaptureConnectionInfoPayload: Codable {
    let inputPortCount: Int
    let mediaTypes: [String]
    let enabled: Bool
    let active: Bool
    let supportsVideoMirroring: Bool
    let videoMirrored: Bool
    let automaticallyAdjustsVideoMirroring: Bool
    let videoRotationAngle: Double?
    let supportsVideoMinFrameDuration: Bool
    let videoMinFrameDuration: CMTimePayload
    let supportsVideoMaxFrameDuration: Bool
    let videoMaxFrameDuration: CMTimePayload
}

private func avcConnectionInfoPayload(from connection: AVCaptureConnection) -> CaptureConnectionInfoPayload {
    var mediaTypes: [String] = []
    for port in connection.inputPorts {
        let mediaType = avcEncodeMediaType(port.mediaType)
        if !mediaTypes.contains(mediaType) {
            mediaTypes.append(mediaType)
        }
    }
    let videoRotationAngle: Double?
    if #available(macOS 14.0, *) {
        videoRotationAngle = Double(connection.videoRotationAngle)
    } else {
        videoRotationAngle = nil
    }
    return CaptureConnectionInfoPayload(
        inputPortCount: connection.inputPorts.count,
        mediaTypes: mediaTypes,
        enabled: connection.isEnabled,
        active: connection.isActive,
        supportsVideoMirroring: connection.isVideoMirroringSupported,
        videoMirrored: connection.isVideoMirrored,
        automaticallyAdjustsVideoMirroring: connection.automaticallyAdjustsVideoMirroring,
        videoRotationAngle: videoRotationAngle,
        supportsVideoMinFrameDuration: connection.isVideoMinFrameDurationSupported,
        videoMinFrameDuration: CMTimePayload(connection.videoMinFrameDuration),
        supportsVideoMaxFrameDuration: connection.isVideoMaxFrameDurationSupported,
        videoMaxFrameDuration: CMTimePayload(connection.videoMaxFrameDuration)
    )
}

@_cdecl("av_capture_connection_release")
public func av_capture_connection_release(_ connectionPtr: UnsafeMutableRawPointer?) {
    avcRelease(connectionPtr, as: ConnectionBox.self)
}

@_cdecl("av_capture_connection_info_json")
public func av_capture_connection_info_json(
    _ connectionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let connection = avcConnectionBox(connectionPtr).connection
    do {
        return ffiString(try avcEncodeJSON(avcConnectionInfoPayload(from: connection)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_connection_set_enabled")
public func av_capture_connection_set_enabled(_ connectionPtr: UnsafeMutableRawPointer, _ enabled: Bool) {
    avcConnectionBox(connectionPtr).connection.isEnabled = enabled
}

@_cdecl("av_capture_connection_set_automatically_adjusts_video_mirroring")
public func av_capture_connection_set_automatically_adjusts_video_mirroring(
    _ connectionPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    avcConnectionBox(connectionPtr).connection.automaticallyAdjustsVideoMirroring = enabled
}

@_cdecl("av_capture_connection_set_video_mirrored")
public func av_capture_connection_set_video_mirrored(
    _ connectionPtr: UnsafeMutableRawPointer,
    _ mirrored: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let connection = avcConnectionBox(connectionPtr).connection
    guard connection.isVideoMirroringSupported else {
        outErrorMessage?.pointee = ffiString("connection does not support video mirroring")
        return AVC_OPERATION_FAILED
    }
    connection.isVideoMirrored = mirrored
    return AVC_OK
}

@_cdecl("av_capture_connection_set_video_rotation_angle")
public func av_capture_connection_set_video_rotation_angle(
    _ connectionPtr: UnsafeMutableRawPointer,
    _ angle: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let connection = avcConnectionBox(connectionPtr).connection
    if #available(macOS 14.0, *) {
        let cgAngle = CGFloat(angle)
        guard connection.isVideoRotationAngleSupported(cgAngle) else {
            outErrorMessage?.pointee = ffiString("connection does not support video rotation angle \(angle)")
            return AVC_OPERATION_FAILED
        }
        connection.videoRotationAngle = cgAngle
        return AVC_OK
    }
    outErrorMessage?.pointee = ffiString("video rotation angle requires macOS 14.0 or newer")
    return AVC_OPERATION_FAILED
}
