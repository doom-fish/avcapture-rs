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
    private let callbackLock = NSLock()
    private let delegateSlot = AVCDelegateSlot("metadata objects delegate")
    private var callbackBox: AVCJsonCallbackBox?
    private var delegate: MetadataObjectsDelegate?
    private var callbackQueue: AVCSerialCallbackQueue?

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
            callbackInstalled: delegateSlot.isOccupied
        )
    }

    func setCallback(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        dropUserData: AVCDropCallback?,
        queueLabel: String
    ) throws {
        let callbackBox = AVCJsonCallbackBox(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        let delegate = MetadataObjectsDelegate(owner: self)
        let queue = AVCSerialCallbackQueue(label: queueLabel)
        try delegateSlot.acquire(callbackBox)
        metadataOutput.setMetadataObjectsDelegate(delegate, queue: queue.queue)
        callbackLock.lock()
        self.callbackBox = callbackBox
        self.delegate = delegate
        callbackQueue = queue
        callbackLock.unlock()
    }

    func clearCallback() {
        callbackLock.lock()
        let callbackBox = self.callbackBox
        let delegate = self.delegate
        let queue = callbackQueue
        if let callbackBox, delegateSlot.release(callbackBox) {
            self.callbackBox = nil
            self.delegate = nil
            callbackQueue = nil
        }
        callbackLock.unlock()
        guard let delegate else {
            return
        }
        if metadataOutput.metadataObjectsDelegate === delegate {
            metadataOutput.setMetadataObjectsDelegate(nil, queue: nil)
        }
        callbackBox?.dispose()
        queue?.drain()
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
        callbackLock.lock()
        let callbackBox = self.callbackBox
        callbackLock.unlock()
        callbackBox?.emit(payload)
    }

    func installStreamDelegate(
        owner: AnyObject,
        delegate: AVCaptureMetadataOutputObjectsDelegate,
        queue: AVCSerialCallbackQueue
    ) throws {
        try delegateSlot.acquire(owner)
        metadataOutput.setMetadataObjectsDelegate(delegate, queue: queue.queue)
    }

    func removeStreamDelegate(
        owner: AnyObject,
        delegate: AVCaptureMetadataOutputObjectsDelegate,
        queue: AVCSerialCallbackQueue
    ) {
        guard delegateSlot.release(owner) else {
            return
        }
        if metadataOutput.metadataObjectsDelegate === delegate {
            metadataOutput.setMetadataObjectsDelegate(nil, queue: nil)
        }
        queue.drain()
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
            let available = Set(output.metadataOutput.availableMetadataObjectTypes.map(\.rawValue))
            let unsupported = rawTypes.filter { !available.contains($0) }
            guard unsupported.isEmpty else {
                throw BridgeError.status(
                    AVC_INVALID_ARGUMENT,
                    "metadata object types are not available in the current configuration: \(unsupported.joined(separator: ", "))"
                )
            }
            output.metadataOutput.metadataObjectTypes = rawTypes.map(AVMetadataObject.ObjectType.init(rawValue:))
            return AVC_OK
        } catch {
            outErrorMessage?.pointee = ffiString(error.localizedDescription)
            return avcStatus(for: error, default: AVC_INVALID_ARGUMENT)
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
    _ retainUserData: AVCRetainCallback?,
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
    do {
        try output.setCallback(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData,
            queueLabel: queueLabel
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_CALLBACK_ERROR)
    }
}

@_cdecl("av_capture_metadata_output_clear_metadata_objects_callback")
public func av_capture_metadata_output_clear_metadata_objects_callback(_ outputPtr: UnsafeMutableRawPointer) {
    if #available(macOS 13.0, *) {
        avcUnretained(outputPtr, as: MetadataOutputBox.self).clearCallback()
    }
}
