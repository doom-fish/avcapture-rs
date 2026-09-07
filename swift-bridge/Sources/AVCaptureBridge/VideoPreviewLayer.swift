import AVFoundation
import Foundation
import QuartzCore

private struct VideoPreviewLayerInfoPayload: Codable {
    let sessionAttached: Bool
    let connectionPresent: Bool
    let videoGravity: String
    let frame: CaptureRectPayload
    let bounds: CaptureRectPayload
    let contentsScale: Double
    let hasSuperlayer: Bool
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
        videoGravity: avcEncodeVideoGravity(layer.videoGravity),
        frame: CaptureRectPayload(layer.frame),
        bounds: CaptureRectPayload(layer.bounds),
        contentsScale: layer.contentsScale,
        hasSuperlayer: layer.superlayer != nil
    )
}

private func avcRequirePreviewLayerMainThread(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Bool {
    guard Thread.isMainThread else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureVideoPreviewLayer operations must run on the main thread"
        )
        return false
    }
    return true
}

private func avcWithoutLayerActions(_ action: () -> Void) {
    CATransaction.begin()
    CATransaction.setDisableActions(true)
    action()
    CATransaction.commit()
}

@_cdecl("av_capture_video_preview_layer_create")
public func av_capture_video_preview_layer_create(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        outStatus.pointee = AVC_MAIN_THREAD_REQUIRED
        return nil
    }
    outStatus.pointee = AVC_OK
    return avcRetain(PreviewLayerBox(session: avcSessionBox(sessionPtr).session))
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
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
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
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
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
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    let raw = String(cString: videoGravityPtr)
    guard let videoGravity = avcDecodeVideoGravity(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported video gravity: \(raw)")
        return AVC_INVALID_ARGUMENT
    }
    avcWithoutLayerActions {
        avcPreviewLayerBox(layerPtr).layer.videoGravity = videoGravity
    }
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_set_session")
public func av_capture_video_preview_layer_set_session(
    _ layerPtr: UnsafeMutableRawPointer,
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    avcWithoutLayerActions {
        avcPreviewLayerBox(layerPtr).layer.session = avcSessionBox(sessionPtr).session
    }
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_clear_session")
public func av_capture_video_preview_layer_clear_session(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    avcWithoutLayerActions {
        avcPreviewLayerBox(layerPtr).layer.session = nil
    }
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_set_session_with_no_connection")
public func av_capture_video_preview_layer_set_session_with_no_connection(
    _ layerPtr: UnsafeMutableRawPointer,
    _ sessionPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    let layer = avcPreviewLayerBox(layerPtr).layer
    let selector = NSSelectorFromString("setSessionWithNoConnection:")
    guard layer.responds(to: selector) else {
        outErrorMessage?.pointee = ffiString("preview layer no-connection reattach is unavailable on this macOS runtime")
        return AVC_OPERATION_FAILED
    }
    avcWithoutLayerActions {
        layer.setSessionWithNoConnection(avcSessionBox(sessionPtr).session)
    }
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json")
public func av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ pointJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
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
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
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
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
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
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
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

@_cdecl("av_capture_video_preview_layer_native_layer")
public func av_capture_video_preview_layer_native_layer(
    _ layerPtr: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer {
    Unmanaged.passUnretained(avcPreviewLayerBox(layerPtr).layer).toOpaque()
}

@_cdecl("av_capture_video_preview_layer_native_layer_retained")
public func av_capture_video_preview_layer_native_layer_retained(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return nil
    }
    return Unmanaged.passRetained(avcPreviewLayerBox(layerPtr).layer).toOpaque()
}

@_cdecl("av_capture_video_preview_layer_native_layer_release")
public func av_capture_video_preview_layer_native_layer_release(
    _ nativeLayerPtr: UnsafeMutableRawPointer?
) {
    guard let nativeLayerPtr else { return }
    Unmanaged<CALayer>.fromOpaque(nativeLayerPtr).release()
}

@_cdecl("av_capture_video_preview_layer_set_frame_json")
public func av_capture_video_preview_layer_set_frame_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ rectJSON: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    do {
        let rect = try avcDecodeJSON(rectJSON, as: CaptureRectPayload.self)
        avcWithoutLayerActions {
            avcPreviewLayerBox(layerPtr).layer.frame = CGRect(
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height
            )
        }
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_video_preview_layer_set_bounds_json")
public func av_capture_video_preview_layer_set_bounds_json(
    _ layerPtr: UnsafeMutableRawPointer,
    _ rectJSON: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    do {
        let rect = try avcDecodeJSON(rectJSON, as: CaptureRectPayload.self)
        avcWithoutLayerActions {
            avcPreviewLayerBox(layerPtr).layer.bounds = CGRect(
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height
            )
        }
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_video_preview_layer_set_contents_scale")
public func av_capture_video_preview_layer_set_contents_scale(
    _ layerPtr: UnsafeMutableRawPointer,
    _ contentsScale: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    guard contentsScale.isFinite, contentsScale > 0 else {
        outErrorMessage?.pointee = ffiString("preview layer contents scale must be finite and positive")
        return AVC_INVALID_ARGUMENT
    }
    avcWithoutLayerActions {
        avcPreviewLayerBox(layerPtr).layer.contentsScale = contentsScale
    }
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_set_needs_layout")
public func av_capture_video_preview_layer_set_needs_layout(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    avcPreviewLayerBox(layerPtr).layer.setNeedsLayout()
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_layout_if_needed")
public func av_capture_video_preview_layer_layout_if_needed(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    avcPreviewLayerBox(layerPtr).layer.layoutIfNeeded()
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_attach_to_host_layer")
public func av_capture_video_preview_layer_attach_to_host_layer(
    _ layerPtr: UnsafeMutableRawPointer,
    _ hostLayerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    let layer = avcPreviewLayerBox(layerPtr).layer
    let hostLayer = Unmanaged<CALayer>.fromOpaque(hostLayerPtr).takeUnretainedValue()
    avcWithoutLayerActions {
        hostLayer.addSublayer(layer)
    }
    return AVC_OK
}

@_cdecl("av_capture_video_preview_layer_detach_from_host_layer")
public func av_capture_video_preview_layer_detach_from_host_layer(
    _ layerPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard avcRequirePreviewLayerMainThread(outErrorMessage) else {
        return AVC_MAIN_THREAD_REQUIRED
    }
    avcWithoutLayerActions {
        avcPreviewLayerBox(layerPtr).layer.removeFromSuperlayer()
    }
    return AVC_OK
}
