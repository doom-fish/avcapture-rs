import AVFoundation
import Foundation

struct MetadataOutputInfoPayload: Codable {
    let connectionCount: Int
    let metadataObjectTypes: [String]
    let availableMetadataObjectTypes: [String]
    let rectOfInterest: CaptureRectPayload
    let callbackInstalled: Bool
}

struct MetadataObjectPayload: Codable {
    let objectType: String
    let stringValue: String?
    let bounds: CaptureRectPayload
}

struct MetadataObjectsEventPayload: Codable {
    let objects: [MetadataObjectPayload]
}

@available(macOS 13.0, *)
private final class MetadataObjectsDelegate: NSObject, AVCaptureMetadataOutputObjectsDelegate {
    private weak var owner: MetadataOutputBox?

    init(owner: MetadataOutputBox) {
        self.owner = owner
    }

    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput metadataObjects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        owner?.emit(objects: metadataObjects)
    }
}

@available(macOS 13.0, *)
final class MetadataOutputBox: CaptureOutputBoxBase {
    let metadataOutput = AVCaptureMetadataOutput()
    fileprivate var callbackBox: AVCJsonCallbackBox?
    private var delegate: MetadataObjectsDelegate?
    private var callbackQueue: DispatchQueue?

    override var output: AVCaptureOutput {
        metadataOutput
    }

    deinit {
        clearCallback()
    }

    fileprivate func infoPayload() -> MetadataOutputInfoPayload {
        MetadataOutputInfoPayload(
            connectionCount: metadataOutput.connections.count,
            metadataObjectTypes: metadataOutput.metadataObjectTypes?.map(\.rawValue) ?? [],
            availableMetadataObjectTypes: metadataOutput.availableMetadataObjectTypes.map(\.rawValue),
            rectOfInterest: CaptureRectPayload(metadataOutput.rectOfInterest),
            callbackInstalled: callbackBox != nil
        )
    }

    func setCallback(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?,
        queueLabel: String
    ) {
        clearCallback()
        let callbackBox = AVCJsonCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = MetadataObjectsDelegate(owner: self)
        let queue = DispatchQueue(label: queueLabel)
        metadataOutput.setMetadataObjectsDelegate(delegate, queue: queue)
        self.callbackBox = callbackBox
        self.delegate = delegate
        callbackQueue = queue
    }

    func clearCallback() {
        metadataOutput.setMetadataObjectsDelegate(nil, queue: nil)
        delegate = nil
        callbackQueue = nil
        callbackBox?.dispose()
        callbackBox = nil
    }

    func emit(objects: [AVMetadataObject]) {
        let payload = MetadataObjectsEventPayload(objects: objects.map { object in
            let stringValue = (object as? AVMetadataMachineReadableCodeObject)?.stringValue
            return MetadataObjectPayload(
                objectType: object.type.rawValue,
                stringValue: stringValue,
                bounds: CaptureRectPayload(object.bounds)
            )
        })
        callbackBox?.emit(payload)
    }
}

@_cdecl("av_capture_metadata_output_create")
public func av_capture_metadata_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    if #available(macOS 13.0, *) {
        return avcRetain(MetadataOutputBox())
    }
    outErrorMessage?.pointee = ffiString("AVCaptureMetadataOutput requires macOS 13.0 or newer")
    return nil
}

@_cdecl("av_capture_metadata_output_release")
public func av_capture_metadata_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    if #available(macOS 13.0, *) {
        avcRelease(outputPtr, as: MetadataOutputBox.self)
    }
}

@_cdecl("av_capture_metadata_output_info_json")
public func av_capture_metadata_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    if #available(macOS 13.0, *) {
        let output = avcUnretained(outputPtr, as: MetadataOutputBox.self)
        do {
            return ffiString(try avcEncodeJSON(output.infoPayload()))
        } catch {
            outErrorMessage?.pointee = ffiString(error.localizedDescription)
            return nil
        }
    }
    outErrorMessage?.pointee = ffiString("AVCaptureMetadataOutput requires macOS 13.0 or newer")
    return nil
}

@_cdecl("av_capture_metadata_output_set_metadata_object_types_json")
public func av_capture_metadata_output_set_metadata_object_types_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ metadataObjectTypesJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 13.0, *) {
        let output = avcUnretained(outputPtr, as: MetadataOutputBox.self)
        do {
            let rawTypes = try avcDecodeJSON(metadataObjectTypesJson, as: [String].self)
            output.metadataOutput.metadataObjectTypes = rawTypes.map(AVMetadataObject.ObjectType.init(rawValue:))
            return AVC_OK
        } catch {
            outErrorMessage?.pointee = ffiString(error.localizedDescription)
            return AVC_INVALID_ARGUMENT
        }
    }
    outErrorMessage?.pointee = ffiString("AVCaptureMetadataOutput requires macOS 13.0 or newer")
    return AVC_OUTPUT_ERROR
}

@_cdecl("av_capture_metadata_output_set_rect_of_interest_json")
public func av_capture_metadata_output_set_rect_of_interest_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ rectJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    if #available(macOS 13.0, *) {
        let output = avcUnretained(outputPtr, as: MetadataOutputBox.self)
        do {
            let rect = try avcDecodeJSON(rectJson, as: CaptureRectPayload.self)
            output.metadataOutput.rectOfInterest = CGRect(x: rect.x, y: rect.y, width: rect.width, height: rect.height)
            return AVC_OK
        } catch {
            outErrorMessage?.pointee = ffiString(error.localizedDescription)
            return AVC_INVALID_ARGUMENT
        }
    }
    outErrorMessage?.pointee = ffiString("AVCaptureMetadataOutput requires macOS 13.0 or newer")
    return AVC_OUTPUT_ERROR
}

@_cdecl("av_capture_metadata_output_set_metadata_objects_callback")
public func av_capture_metadata_output_set_metadata_objects_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureMetadataOutput requires macOS 13.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing metadata objects callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: MetadataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    output.setCallback(callback: callback, userData: userData, dropUserData: dropUserData, queueLabel: queueLabel)
    return AVC_OK
}

@_cdecl("av_capture_metadata_output_clear_metadata_objects_callback")
public func av_capture_metadata_output_clear_metadata_objects_callback(_ outputPtr: UnsafeMutableRawPointer) {
    if #available(macOS 13.0, *) {
        avcUnretained(outputPtr, as: MetadataOutputBox.self).clearCallback()
    }
}
