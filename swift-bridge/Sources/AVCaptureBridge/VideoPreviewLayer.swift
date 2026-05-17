import AVFoundation
import Foundation

private struct VideoPreviewLayerInfoPayload: Codable {
    let sessionAttached: Bool
    let connectionPresent: Bool
    let videoGravity: String
}

private struct CapturePointPayload: Codable {
    let x: Double
    let y: Double

    init(_ point: CGPoint) {
        x = point.x
        y = point.y
    }

    var point: CGPoint {
        CGPoint(x: x, y: y)
    }
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

@_cdecl("av_capture_video_preview_layer_set_session")
public func av_capture_video_preview_layer_set_session(
    _ layerPtr: UnsafeMutableRawPointer,
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    avcPreviewLayerBox(layerPtr).layer.session = avcSessionBox(sessionPtr).session
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_clear_session")
public func av_capture_video_preview_layer_clear_session(_ layerPtr: UnsafeMutableRawPointer) {
    avcPreviewLayerBox(layerPtr).layer.session = nil
}

@_cdecl("av_capture_video_preview_layer_set_session_with_no_connection")
public func av_capture_video_preview_layer_set_session_with_no_connection(
    _ layerPtr: UnsafeMutableRawPointer,
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let layer = avcPreviewLayerBox(layerPtr).layer
    let selector = NSSelectorFromString("setSessionWithNoConnection:")
    guard layer.responds(to: selector) else {
        outErrorMessage?.pointee = ffiString("preview layer no-connection reattach is unavailable on this macOS runtime")
        return AVC_OPERATION_FAILED
    }
    layer.setSessionWithNoConnection(avcSessionBox(sessionPtr).session)
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json")
public func av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ pointJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("preview layer point conversions require macOS 10.15 or newer")
        return nil
    }
    let layer = avcPreviewLayerBox(layerPtr).layer
    do {
        let point = try avcDecodeJSON(pointJson, as: CapturePointPayload.self)
        return ffiString(
            try avcEncodeJSON(
                CapturePointPayload(layer.captureDevicePointConverted(fromLayerPoint: point.point))
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_video_preview_layer_point_for_capture_device_point_of_interest_json")
public func av_capture_video_preview_layer_point_for_capture_device_point_of_interest_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ pointJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("preview layer point conversions require macOS 10.15 or newer")
        return nil
    }
    let layer = avcPreviewLayerBox(layerPtr).layer
    do {
        let point = try avcDecodeJSON(pointJson, as: CapturePointPayload.self)
        return ffiString(
            try avcEncodeJSON(
                CapturePointPayload(layer.layerPointConverted(fromCaptureDevicePoint: point.point))
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_video_preview_layer_metadata_output_rect_of_interest_for_rect_json")
public func av_capture_video_preview_layer_metadata_output_rect_of_interest_for_rect_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ rectJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("preview layer rect conversions require macOS 10.15 or newer")
        return nil
    }
    let layer = avcPreviewLayerBox(layerPtr).layer
    do {
        let rect = try avcDecodeJSON(rectJson, as: CaptureRectPayload.self)
        let converted = layer.metadataOutputRectConverted(
            fromLayerRect: CGRect(x: rect.x, y: rect.y, width: rect.width, height: rect.height)
        )
        return ffiString(try avcEncodeJSON(CaptureRectPayload(converted)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_video_preview_layer_rect_for_metadata_output_rect_of_interest_json")
public func av_capture_video_preview_layer_rect_for_metadata_output_rect_of_interest_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ rectJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("preview layer rect conversions require macOS 10.15 or newer")
        return nil
    }
    let layer = avcPreviewLayerBox(layerPtr).layer
    do {
        let rect = try avcDecodeJSON(rectJson, as: CaptureRectPayload.self)
        let converted = layer.layerRectConverted(
            fromMetadataOutputRect: CGRect(x: rect.x, y: rect.y, width: rect.width, height: rect.height)
        )
        return ffiString(try avcEncodeJSON(CaptureRectPayload(converted)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
