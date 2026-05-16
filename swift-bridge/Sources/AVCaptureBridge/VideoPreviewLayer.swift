import AVFoundation
import Foundation

private struct VideoPreviewLayerInfoPayload: Codable {
    let sessionAttached: Bool
    let connectionPresent: Bool
    let videoGravity: String
}

final class PreviewLayerBox: NSObject {
    let layer: AVCaptureVideoPreviewLayer

    init(session: AVCaptureSession) {
        layer = AVCaptureVideoPreviewLayer(session: session)
    }
}

private func avcEncodeVideoGravity(_ videoGravity: AVLayerVideoGravity) -> String {
    switch videoGravity {
    case .resize:
        return "resize"
    case .resizeAspect:
        return "resizeAspect"
    case .resizeAspectFill:
        return "resizeAspectFill"
    default:
        return videoGravity.rawValue
    }
}

private func avcDecodeVideoGravity(_ raw: String) -> AVLayerVideoGravity? {
    switch raw {
    case "resize", AVLayerVideoGravity.resize.rawValue:
        return .resize
    case "resizeAspect", AVLayerVideoGravity.resizeAspect.rawValue:
        return .resizeAspect
    case "resizeAspectFill", AVLayerVideoGravity.resizeAspectFill.rawValue:
        return .resizeAspectFill
    default:
        return nil
    }
}

private func previewLayerInfoPayload(from layer: AVCaptureVideoPreviewLayer) -> VideoPreviewLayerInfoPayload {
    VideoPreviewLayerInfoPayload(
        sessionAttached: layer.session != nil,
        connectionPresent: layer.connection != nil,
        videoGravity: avcEncodeVideoGravity(layer.videoGravity)
    )
}

@_cdecl("av_capture_video_preview_layer_create")
public func av_capture_video_preview_layer_create(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(PreviewLayerBox(session: avcSessionBox(sessionPtr).session))
}

@_cdecl("av_capture_video_preview_layer_release")
public func av_capture_video_preview_layer_release(_ layerPtr: UnsafeMutableRawPointer?) {
    avcRelease(layerPtr, as: PreviewLayerBox.self)
}

@_cdecl("av_capture_video_preview_layer_info_json")
public func av_capture_video_preview_layer_info_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let layer = avcPreviewLayerBox(layerPtr).layer
    do {
        return ffiString(try avcEncodeJSON(previewLayerInfoPayload(from: layer)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_video_preview_layer_connection")
public func av_capture_video_preview_layer_connection(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let connection = avcPreviewLayerBox(layerPtr).layer.connection else {
        return nil
    }
    return avcRetain(ConnectionBox(connection))
}

@_cdecl("av_capture_video_preview_layer_set_video_gravity")
public func av_capture_video_preview_layer_set_video_gravity(
    _ layerPtr: UnsafeMutableRawPointer,
    _ videoGravityPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let raw = String(cString: videoGravityPtr)
    guard let videoGravity = avcDecodeVideoGravity(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported video gravity: \(raw)")
        return AVC_INVALID_ARGUMENT
    }
    avcPreviewLayerBox(layerPtr).layer.videoGravity = videoGravity
    return AVC_OK
}
