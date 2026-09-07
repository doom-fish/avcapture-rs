import AVFoundation
import CoreGraphics
import CoreMedia
import CoreVideo
import Foundation

let AVC_OK: Int32 = 0
let AVC_INVALID_ARGUMENT: Int32 = -1
let AVC_DEVICE_ERROR: Int32 = -2
let AVC_INPUT_ERROR: Int32 = -3
let AVC_SESSION_ERROR: Int32 = -4
let AVC_OUTPUT_ERROR: Int32 = -5
let AVC_CALLBACK_ERROR: Int32 = -6
let AVC_OPERATION_FAILED: Int32 = -7
let AVC_DELEGATE_SLOT_OCCUPIED: Int32 = -8
let AVC_UNSUPPORTED_PLATFORM: Int32 = -9
let AVC_MAIN_THREAD_REQUIRED: Int32 = -10
let AVC_OUTPUT_FILE_EXISTS: Int32 = -11
let AVC_CANCELLED: Int32 = -12
let AVC_BRIDGE_PROTOCOL: Int32 = -13
let AVC_BRIDGE_SCHEMA_VERSION: UInt64 = 1
let AVC_STREAM_BRIDGE_ERROR_KIND: Int32 = -1

public typealias AVCVideoSampleCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?
) -> Void
public typealias AVCVideoDataOutputEventCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    Int32,
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<CChar>?,
    UInt64
) -> Void
public typealias AVCAudioSampleCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?
) -> Void
public typealias AVCJsonCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<CChar>?
) -> Void
public typealias AVCPhotoCallback = @convention(c) (
    UnsafeMutableRawPointer?,
    UnsafeMutableRawPointer?,
    UnsafeMutablePointer<CChar>?
) -> Void
public typealias AVCDropCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void
public typealias AVCRetainCallback = @convention(c) (UnsafeMutableRawPointer?) -> Void

final class AVCCallbackContextOwner {
    let userData: UnsafeMutableRawPointer?
    private let releaseUserData: AVCDropCallback?

    init(
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        releaseUserData: AVCDropCallback?
    ) {
        self.userData = userData
        self.releaseUserData = releaseUserData
        if let userData, let retainUserData {
            retainUserData(userData)
        }
    }

    deinit {
        if let userData, let releaseUserData {
            releaseUserData(userData)
        }
    }
}

final class AVCDelegateSlot {
    private let name: String
    private let lock = NSLock()
    private weak var owner: AnyObject?

    init(_ name: String) {
        self.name = name
    }

    func acquire(_ newOwner: AnyObject) throws {
        lock.lock()
        defer { lock.unlock() }
        guard owner == nil else {
            throw BridgeError.status(
                AVC_DELEGATE_SLOT_OCCUPIED,
                "\(name) is already registered"
            )
        }
        owner = newOwner
    }

    func release(_ releasingOwner: AnyObject) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        guard owner === releasingOwner else {
            return false
        }
        owner = nil
        return true
    }

    func isOwned(by candidate: AnyObject) -> Bool {
        lock.lock()
        defer { lock.unlock() }
        return owner === candidate
    }

    var isOccupied: Bool {
        lock.lock()
        defer { lock.unlock() }
        return owner != nil
    }
}

final class AVCSerialCallbackQueue {
    let queue: DispatchQueue
    private let key = DispatchSpecificKey<UInt8>()

    init(label: String) {
        queue = DispatchQueue(label: label)
        queue.setSpecific(key: key, value: 1)
    }

    func drain() {
        if DispatchQueue.getSpecific(key: key) == nil {
            queue.sync {}
        }
    }
}

private struct AVCBridgeErrorPayload: Encodable {
    let bridgeError: String
}

func avcBridgeErrorPayload(_ error: Error) -> UnsafeMutablePointer<CChar>? {
    if let json = try? avcEncodeJSON(AVCBridgeErrorPayload(bridgeError: error.localizedDescription)) {
        return ffiString(json)
    }
    return ffiString(
        "{\"schemaVersion\":1,\"payload\":{\"bridgeError\":\"bridge payload encoding failed\"}}"
    )
}

final class AVCJsonCallbackBox {
    private let callback: AVCJsonCallback
    private let contextOwner: AVCCallbackContextOwner
    private let lock = NSLock()
    private var disposed = false

    init(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback? = nil,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        contextOwner = AVCCallbackContextOwner(
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: dropUserData
        )
    }

    func emit<T: Encodable>(_ payload: T) {
        lock.lock()
        let active = !disposed
        let contextOwner = self.contextOwner
        lock.unlock()
        guard active else {
            return
        }
        do {
            callback(contextOwner.userData, ffiString(try avcEncodeJSON(payload)))
        } catch {
            callback(contextOwner.userData, avcBridgeErrorPayload(error))
        }
    }

    func dispose() {
        lock.lock()
        disposed = true
        lock.unlock()
    }
}

final class AVCPhotoCallbackBox {
    private let callback: AVCPhotoCallback
    private let contextOwner: AVCCallbackContextOwner
    private let lock = NSLock()
    private var disposed = false

    init(
        callback: @escaping AVCPhotoCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback? = nil,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        contextOwner = AVCCallbackContextOwner(
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: dropUserData
        )
    }

    func emit<T: Encodable>(_ photo: AVCapturePhoto?, payload: T) {
        lock.lock()
        let active = !disposed
        let contextOwner = self.contextOwner
        lock.unlock()
        guard active else { return }
        let photoPtr = photo.map { avcRetain(PhotoBox($0)) }
        do {
            callback(
                contextOwner.userData,
                photoPtr,
                ffiString(try avcEncodeJSON(payload))
            )
        } catch {
            if let photoPtr {
                avcRelease(photoPtr, as: PhotoBox.self)
            }
            callback(contextOwner.userData, nil, avcBridgeErrorPayload(error))
        }
    }

    func dispose() {
        lock.lock()
        disposed = true
        lock.unlock()
    }
}

@_cdecl("avc_string_free")
public func avc_string_free(_ str: UnsafeMutablePointer<CChar>?) {
    guard let str else { return }
    free(str)
}

func ffiString(_ string: String) -> UnsafeMutablePointer<CChar>? {
    string.withCString { strdup($0) }
}

enum BridgeError: LocalizedError {
    case message(String)
    case status(Int32, String)

    var errorDescription: String? {
        switch self {
        case .message(let message):
            return message
        case .status(_, let message):
            return message
        }
    }

    var statusCode: Int32? {
        switch self {
        case .message:
            return nil
        case .status(let status, _):
            return status
        }
    }
}

func avcStatus(for error: Error, default defaultStatus: Int32) -> Int32 {
    (error as? BridgeError)?.statusCode ?? defaultStatus
}

final class SessionBox: NSObject {
    let session = AVCaptureSession()
    let controlsDelegateSlot = AVCDelegateSlot("session controls delegate")
    let deferredStartDelegateSlot = AVCDelegateSlot("session deferred-start delegate")
}

final class DeviceBox: NSObject {
    let device: AVCaptureDevice

    init(_ device: AVCaptureDevice) {
        self.device = device
    }
}

final class DeviceFormatBox: NSObject {
    let format: AVCaptureDevice.Format

    init(_ format: AVCaptureDevice.Format) {
        self.format = format
    }
}

final class ConnectionBox: NSObject {
    let connection: AVCaptureConnection

    init(_ connection: AVCaptureConnection) {
        self.connection = connection
    }
}

final class DiscoverySessionBox: NSObject {
    let session: AVCaptureDevice.DiscoverySession

    init(_ session: AVCaptureDevice.DiscoverySession) {
        self.session = session
    }
}

class CaptureInputBoxBase: NSObject {
    var input: AVCaptureInput {
        fatalError("CaptureInputBoxBase.input must be overridden")
    }
}

class CaptureOutputBoxBase: NSObject {
    var output: AVCaptureOutput {
        fatalError("CaptureOutputBoxBase.output must be overridden")
    }
}

func avcRetain<T: AnyObject>(_ value: T) -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(value).toOpaque()
}

func avcRelease<T: AnyObject>(_ ptr: UnsafeMutableRawPointer?, as type: T.Type) {
    guard let ptr else { return }
    Unmanaged<T>.fromOpaque(ptr).release()
}

func avcUnretained<T: AnyObject>(_ ptr: UnsafeMutableRawPointer, as type: T.Type) -> T {
    Unmanaged<T>.fromOpaque(ptr).takeUnretainedValue()
}

func avcSessionBox(_ ptr: UnsafeMutableRawPointer) -> SessionBox {
    avcUnretained(ptr, as: SessionBox.self)
}

func avcDeviceBox(_ ptr: UnsafeMutableRawPointer) -> DeviceBox {
    avcUnretained(ptr, as: DeviceBox.self)
}

func avcDeviceFormatBox(_ ptr: UnsafeMutableRawPointer) -> DeviceFormatBox {
    avcUnretained(ptr, as: DeviceFormatBox.self)
}

func avcPhotoSettingsBox(_ ptr: UnsafeMutableRawPointer) -> PhotoSettingsBox {
    avcUnretained(ptr, as: PhotoSettingsBox.self)
}

func avcPhotoBox(_ ptr: UnsafeMutableRawPointer) -> PhotoBox {
    avcUnretained(ptr, as: PhotoBox.self)
}

func avcConnectionBox(_ ptr: UnsafeMutableRawPointer) -> ConnectionBox {
    avcUnretained(ptr, as: ConnectionBox.self)
}

func avcDiscoverySessionBox(_ ptr: UnsafeMutableRawPointer) -> DiscoverySessionBox {
    avcUnretained(ptr, as: DiscoverySessionBox.self)
}

func avcInputBox(_ ptr: UnsafeMutableRawPointer) -> CaptureInputBoxBase {
    avcUnretained(ptr, as: CaptureInputBoxBase.self)
}

func avcOutputBox(_ ptr: UnsafeMutableRawPointer) -> CaptureOutputBoxBase {
    avcUnretained(ptr, as: CaptureOutputBoxBase.self)
}

func avcPreviewLayerBox(_ ptr: UnsafeMutableRawPointer) -> PreviewLayerBox {
    avcUnretained(ptr, as: PreviewLayerBox.self)
}

struct CMTimePayload: Codable {
    let value: Int64
    let timescale: Int32
    let flags: UInt32
    let epoch: Int64

    init(_ time: CMTime) {
        value = time.value
        timescale = time.timescale
        flags = time.flags.rawValue
        epoch = time.epoch
    }
}

struct CaptureRectPayload: Codable {
    let x: Double
    let y: Double
    let width: Double
    let height: Double

    init(_ rect: CGRect) {
        x = rect.origin.x
        y = rect.origin.y
        width = rect.size.width
        height = rect.size.height
    }
}

struct VideoDimensionsPayload: Codable {
    let width: Int32
    let height: Int32

    init(_ dimensions: CMVideoDimensions) {
        width = dimensions.width
        height = dimensions.height
    }
}

struct CaptureDeviceInfoPayload: Codable {
    let uniqueId: String
    let localizedName: String
    let manufacturer: String

    enum CodingKeys: String, CodingKey {
        case uniqueId = "uniqueID"
        case localizedName
        case manufacturer
    }
}

struct DeviceInputInfoPayload: Codable {
    let deviceUniqueId: String
    let deviceLocalizedName: String
    let portsCount: Int

    enum CodingKeys: String, CodingKey {
        case deviceUniqueId = "deviceUniqueID"
        case deviceLocalizedName
        case portsCount
    }
}

struct CaptureSessionInfoPayload: Codable {
    let sessionPreset: String
    let inputCount: Int
    let outputCount: Int
    let connectionCount: Int
    let isRunning: Bool
}

struct CaptureInputPortInfoPayload: Codable {
    let mediaType: String
    let enabled: Bool
    let hasFormatDescription: Bool
}

struct CaptureInputInfoPayload: Codable {
    let ports: [CaptureInputPortInfoPayload]
}

struct CaptureOutputInfoPayload: Codable {
    let connectionCount: Int
}

struct VideoOutputSettingsPayload: Codable {
    let pixelFormat: UInt32
    let width: Int?
    let height: Int?
}

struct AudioOutputSettingsPayload: Codable {
    let sampleRate: Double?
    let channelCount: UInt32?
    let bitsPerChannel: UInt32
    let isFloat: Bool
    let isNonInterleaved: Bool
}

struct VideoDataOutputInfoPayload: Codable {
    let connectionCount: Int
    let alwaysDiscardsLateVideoFrames: Bool
    let availableVideoCvPixelFormatTypes: [UInt32]
    let callbackInstalled: Bool
    let videoSettings: VideoOutputSettingsPayload?

    enum CodingKeys: String, CodingKey {
        case connectionCount
        case alwaysDiscardsLateVideoFrames
        case availableVideoCvPixelFormatTypes = "availableVideoCVPixelFormatTypes"
        case callbackInstalled
        case videoSettings
    }
}

struct AudioDataOutputInfoPayload: Codable {
    let connectionCount: Int
    let callbackInstalled: Bool
    let audioSettings: AudioOutputSettingsPayload?
}

func avcEncodeJSON<T: Encodable>(_ value: T) throws -> String {
    let encoded = try JSONEncoder().encode(value)
    var data = Data(
        "{\"schemaVersion\":\(AVC_BRIDGE_SCHEMA_VERSION),\"payload\":".utf8
    )
    data.append(encoded)
    data.append(UInt8(ascii: "}"))
    guard let string = String(data: data, encoding: .utf8) else {
        throw BridgeError.message("failed to UTF-8 encode JSON payload")
    }
    return string
}

func avcDecodeJSON<T: Decodable>(_ ptr: UnsafePointer<CChar>?, as type: T.Type) throws -> T {
    guard let ptr else {
        throw BridgeError.message("missing JSON payload")
    }
    let string = String(cString: ptr)
    guard let data = string.data(using: .utf8) else {
        throw BridgeError.message("payload was not valid UTF-8")
    }
    guard let object = try JSONSerialization.jsonObject(with: data) as? [String: Any],
          let version = object["schemaVersion"] as? NSNumber,
          let payload = object["payload"]
    else {
        throw BridgeError.message("bridge JSON payload is not a versioned envelope")
    }
    guard version.uint64Value == AVC_BRIDGE_SCHEMA_VERSION else {
        throw BridgeError.message(
            "unsupported bridge JSON schema version \(version); expected \(AVC_BRIDGE_SCHEMA_VERSION)"
        )
    }
    let payloadData = try JSONSerialization.data(withJSONObject: payload, options: [.fragmentsAllowed])
    return try JSONDecoder().decode(T.self, from: payloadData)
}

func avcDecodeMediaType(_ raw: String) -> AVMediaType? {
    switch raw {
    case "audio":
        return .audio
    case "video":
        return .video
    case "muxed":
        return .muxed
    case "metadata":
        return .metadata
    default:
        return nil
    }
}

func avcEncodeMediaType(_ mediaType: AVMediaType) -> String {
    switch mediaType {
    case .audio:
        return "audio"
    case .video:
        return "video"
    case .muxed:
        return "muxed"
    case .metadata:
        return "metadata"
    default:
        return mediaType.rawValue
    }
}

func avcDecodeSessionPreset(_ raw: String) -> AVCaptureSession.Preset? {
    switch raw {
    case "photo":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPresetPhoto")
    case "high":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPresetHigh")
    case "medium":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPresetMedium")
    case "low":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPresetLow")
    case "320x240":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset320x240")
    case "352x288":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset352x288")
    case "640x480":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset640x480")
    case "960x540":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset960x540")
    case "1280x720":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset1280x720")
    case "1920x1080":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset1920x1080")
    case "3840x2160":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPreset3840x2160")
    case "iframe960x540":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPresetiFrame960x540")
    case "iframe1280x720":
        return AVCaptureSession.Preset(rawValue: "AVCaptureSessionPresetiFrame1280x720")
    default:
        return nil
    }
}

func avcEncodeSessionPreset(_ preset: AVCaptureSession.Preset) -> String {
    switch preset.rawValue {
    case "AVCaptureSessionPresetPhoto":
        return "photo"
    case "AVCaptureSessionPresetHigh":
        return "high"
    case "AVCaptureSessionPresetMedium":
        return "medium"
    case "AVCaptureSessionPresetLow":
        return "low"
    case "AVCaptureSessionPreset320x240":
        return "320x240"
    case "AVCaptureSessionPreset352x288":
        return "352x288"
    case "AVCaptureSessionPreset640x480":
        return "640x480"
    case "AVCaptureSessionPreset960x540":
        return "960x540"
    case "AVCaptureSessionPreset1280x720":
        return "1280x720"
    case "AVCaptureSessionPreset1920x1080":
        return "1920x1080"
    case "AVCaptureSessionPreset3840x2160":
        return "3840x2160"
    case "AVCaptureSessionPresetiFrame960x540":
        return "iframe960x540"
    case "AVCaptureSessionPresetiFrame1280x720":
        return "iframe1280x720"
    default:
        return preset.rawValue
    }
}

func avcVideoSettingsDictionary(from payload: VideoOutputSettingsPayload) -> [String: Any] {
    var settings: [String: Any] = [
        kCVPixelBufferPixelFormatTypeKey as String: NSNumber(value: payload.pixelFormat)
    ]
    if let width = payload.width {
        settings[kCVPixelBufferWidthKey as String] = NSNumber(value: width)
    }
    if let height = payload.height {
        settings[kCVPixelBufferHeightKey as String] = NSNumber(value: height)
    }
    return settings
}

func avcEncodeVideoSettings(_ settings: [String: Any]?) -> VideoOutputSettingsPayload? {
    guard let settings else { return nil }
    let pixelFormat = (settings[kCVPixelBufferPixelFormatTypeKey as String] as? NSNumber)?.uint32Value ?? 0
    let width = (settings[kCVPixelBufferWidthKey as String] as? NSNumber)?.intValue
    let height = (settings[kCVPixelBufferHeightKey as String] as? NSNumber)?.intValue
    if pixelFormat == 0, width == nil, height == nil {
        return nil
    }
    return VideoOutputSettingsPayload(pixelFormat: pixelFormat, width: width, height: height)
}

func avcAudioSettingsDictionary(from payload: AudioOutputSettingsPayload) -> [String: Any] {
    var settings: [String: Any] = [
        AVFormatIDKey: NSNumber(value: 0x6c70636d as UInt32),
        AVLinearPCMBitDepthKey: NSNumber(value: payload.bitsPerChannel),
        AVLinearPCMIsFloatKey: payload.isFloat,
        AVLinearPCMIsNonInterleaved: payload.isNonInterleaved,
    ]
    if let sampleRate = payload.sampleRate {
        settings[AVSampleRateKey] = NSNumber(value: sampleRate)
    }
    if let channelCount = payload.channelCount {
        settings[AVNumberOfChannelsKey] = NSNumber(value: channelCount)
    }
    return settings
}

func avcEncodeAudioSettings(_ settings: [String: Any]?) -> AudioOutputSettingsPayload? {
    guard let settings else { return nil }
    let bitsPerChannel = (settings[AVLinearPCMBitDepthKey] as? NSNumber)?.uint32Value ?? 16
    let isFloat = (settings[AVLinearPCMIsFloatKey] as? NSNumber)?.boolValue
        ?? (settings[AVLinearPCMIsFloatKey] as? Bool)
        ?? false
    let isNonInterleaved = (settings[AVLinearPCMIsNonInterleaved] as? NSNumber)?.boolValue
        ?? (settings[AVLinearPCMIsNonInterleaved] as? Bool)
        ?? false
    let sampleRate = (settings[AVSampleRateKey] as? NSNumber)?.doubleValue
    let channelCount = (settings[AVNumberOfChannelsKey] as? NSNumber)?.uint32Value
    return AudioOutputSettingsPayload(
        sampleRate: sampleRate,
        channelCount: channelCount,
        bitsPerChannel: bitsPerChannel,
        isFloat: isFloat,
        isNonInterleaved: isNonInterleaved
    )
}

func avcFourCCString(_ code: FourCharCode) -> String {
    let bytes = [
        UInt8((code >> 24) & 0xFF),
        UInt8((code >> 16) & 0xFF),
        UInt8((code >> 8) & 0xFF),
        UInt8(code & 0xFF),
    ]
    let printable = bytes.allSatisfy { $0 >= 32 && $0 < 127 }
    if printable {
        let scalarValues = bytes.map { Character(UnicodeScalar($0)) }
        return String(scalarValues)
    }
    return String(format: "0x%08X", code)
}
