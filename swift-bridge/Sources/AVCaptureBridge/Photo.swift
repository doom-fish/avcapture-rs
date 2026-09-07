import AVFoundation
import Foundation

private struct PhotoSettingsInfoPayload: Codable {
    let uniqueID: Int64
    let processedFileType: String?
    let flashMode: Int32?
    let photoQualityPrioritization: Int32?
    let usedForCapture: Bool
}

struct ResolvedPhotoSettingsInfoPayload: Codable {
    let uniqueID: Int64
    let photoDimensions: VideoDimensionsPayload
    let expectedPhotoCount: Int
    let fastCapturePrioritizationEnabled: Bool?
}

private struct PhotoInfoPayload: Codable {
    let uniqueID: Int64
    let timestamp: CMTimePayload
    let photoCount: Int
    let pixelBufferAvailable: Bool
    let pixelBufferPixelFormat: UInt32?
    let pixelBufferDimensions: VideoDimensionsPayload?
    let constantColorConfidenceMapAvailable: Bool?
    let constantColorCenterWeightedMeanConfidenceLevel: Float?
    let constantColorFallbackPhoto: Bool?
    let resolvedSettings: ResolvedPhotoSettingsInfoPayload
}

final class PhotoSettingsBox: NSObject {
    let settings: AVCapturePhotoSettings
    private let lock = NSLock()
    private var usedForCapture = false

    init(_ settings: AVCapturePhotoSettings = AVCapturePhotoSettings()) {
        self.settings = settings
    }

    func consumeForCapture() throws {
        lock.lock()
        defer { lock.unlock() }
        guard !usedForCapture else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "photo settings have already been used for capture; create a copy with a new unique ID"
            )
        }
        usedForCapture = true
    }

    func ensureMutable() throws {
        lock.lock()
        defer { lock.unlock() }
        guard !usedForCapture else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "photo settings cannot be changed after they have been used for capture"
            )
        }
    }

    var hasBeenUsedForCapture: Bool {
        lock.lock()
        defer { lock.unlock() }
        return usedForCapture
    }
}

final class ResolvedPhotoSettingsBox: NSObject {
    let resolvedSettings: AVCaptureResolvedPhotoSettings

    init(_ resolvedSettings: AVCaptureResolvedPhotoSettings) {
        self.resolvedSettings = resolvedSettings
    }
}

final class PhotoBox: NSObject {
    let photo: AVCapturePhoto

    init(_ photo: AVCapturePhoto) {
        self.photo = photo
    }
}

private func avcResolvedPhotoSettingsBox(_ ptr: UnsafeMutableRawPointer) -> ResolvedPhotoSettingsBox {
    avcUnretained(ptr, as: ResolvedPhotoSettingsBox.self)
}

private func photoSettingsInfoPayload(from box: PhotoSettingsBox) -> PhotoSettingsInfoPayload {
    let settings = box.settings
    let flashMode: Int32?
    if #available(macOS 13.0, *) {
        flashMode = Int32(settings.flashMode.rawValue)
    } else {
        flashMode = nil
    }
    let photoQualityPrioritization: Int32?
    if #available(macOS 13.0, *) {
        photoQualityPrioritization = Int32(settings.photoQualityPrioritization.rawValue)
    } else {
        photoQualityPrioritization = nil
    }
    return PhotoSettingsInfoPayload(
        uniqueID: settings.uniqueID,
        processedFileType: settings.processedFileType?.rawValue,
        flashMode: flashMode,
        photoQualityPrioritization: photoQualityPrioritization,
        usedForCapture: box.hasBeenUsedForCapture
    )
}

func resolvedPhotoSettingsInfoPayload(
    from resolvedSettings: AVCaptureResolvedPhotoSettings
) -> ResolvedPhotoSettingsInfoPayload {
    let fastCapturePrioritizationEnabled: Bool?
    if #available(macOS 14.0, *) {
        fastCapturePrioritizationEnabled = resolvedSettings.isFastCapturePrioritizationEnabled
    } else {
        fastCapturePrioritizationEnabled = nil
    }
    return ResolvedPhotoSettingsInfoPayload(
        uniqueID: resolvedSettings.uniqueID,
        photoDimensions: VideoDimensionsPayload(resolvedSettings.photoDimensions),
        expectedPhotoCount: resolvedSettings.expectedPhotoCount,
        fastCapturePrioritizationEnabled: fastCapturePrioritizationEnabled
    )
}

private func photoInfoPayload(from photo: AVCapturePhoto) -> PhotoInfoPayload {
    let pixelBuffer = photo.pixelBuffer
    let constantColorConfidenceMapAvailable: Bool?
    let constantColorCenterWeightedMeanConfidenceLevel: Float?
    let constantColorFallbackPhoto: Bool?
    if #available(macOS 15.0, *) {
        constantColorConfidenceMapAvailable = photo.constantColorConfidenceMap != nil
        constantColorCenterWeightedMeanConfidenceLevel = photo.constantColorCenterWeightedMeanConfidenceLevel
        constantColorFallbackPhoto = photo.isConstantColorFallbackPhoto
    } else {
        constantColorConfidenceMapAvailable = nil
        constantColorCenterWeightedMeanConfidenceLevel = nil
        constantColorFallbackPhoto = nil
    }
    return PhotoInfoPayload(
        uniqueID: photo.resolvedSettings.uniqueID,
        timestamp: CMTimePayload(photo.timestamp),
        photoCount: photo.photoCount,
        pixelBufferAvailable: pixelBuffer != nil,
        pixelBufferPixelFormat: pixelBuffer.map(CVPixelBufferGetPixelFormatType),
        pixelBufferDimensions: pixelBuffer.map {
            VideoDimensionsPayload(
                CMVideoDimensions(
                    width: Int32(CVPixelBufferGetWidth($0)),
                    height: Int32(CVPixelBufferGetHeight($0))
                )
            )
        },
        constantColorConfidenceMapAvailable: constantColorConfidenceMapAvailable,
        constantColorCenterWeightedMeanConfidenceLevel: constantColorCenterWeightedMeanConfidenceLevel,
        constantColorFallbackPhoto: constantColorFallbackPhoto,
        resolvedSettings: resolvedPhotoSettingsInfoPayload(from: photo.resolvedSettings)
    )
}

@_cdecl("av_capture_photo_settings_create")
public func av_capture_photo_settings_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(PhotoSettingsBox())
}

@_cdecl("av_capture_photo_settings_copy_with_unique_id")
public func av_capture_photo_settings_copy_with_unique_id(
    _ settingsPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let settings = avcPhotoSettingsBox(settingsPtr).settings
    return avcRetain(PhotoSettingsBox(AVCapturePhotoSettings(from: settings)))
}

@_cdecl("av_capture_photo_settings_release")
public func av_capture_photo_settings_release(_ settingsPtr: UnsafeMutableRawPointer?) {
    avcRelease(settingsPtr, as: PhotoSettingsBox.self)
}

@_cdecl("av_capture_photo_settings_info_json")
public func av_capture_photo_settings_info_json(
    _ settingsPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let settings = avcPhotoSettingsBox(settingsPtr)
    do {
        return ffiString(try avcEncodeJSON(photoSettingsInfoPayload(from: settings)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_photo_settings_set_flash_mode")
public func av_capture_photo_settings_set_flash_mode(
    _ settingsPtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("photo flash mode requires macOS 13.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let mode = AVCaptureDevice.FlashMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported flash mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    let box = avcPhotoSettingsBox(settingsPtr)
    do {
        try box.ensureMutable()
        box.settings.flashMode = mode
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_INVALID_ARGUMENT)
    }
}

@_cdecl("av_capture_photo_settings_set_photo_quality_prioritization")
public func av_capture_photo_settings_set_photo_quality_prioritization(
    _ settingsPtr: UnsafeMutableRawPointer,
    _ prioritizationRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("photo quality prioritization requires macOS 13.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let prioritization = AVCapturePhotoOutput.QualityPrioritization(rawValue: Int(prioritizationRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported photo quality prioritization: \(prioritizationRaw)")
        return AVC_INVALID_ARGUMENT
    }
    let box = avcPhotoSettingsBox(settingsPtr)
    do {
        try box.ensureMutable()
        box.settings.photoQualityPrioritization = prioritization
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_INVALID_ARGUMENT)
    }
}

@_cdecl("av_capture_resolved_photo_settings_release")
public func av_capture_resolved_photo_settings_release(
    _ resolvedSettingsPtr: UnsafeMutableRawPointer?
) {
    avcRelease(resolvedSettingsPtr, as: ResolvedPhotoSettingsBox.self)
}

@_cdecl("av_capture_resolved_photo_settings_info_json")
public func av_capture_resolved_photo_settings_info_json(
    _ resolvedSettingsPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let resolvedSettings = avcResolvedPhotoSettingsBox(resolvedSettingsPtr).resolvedSettings
    do {
        return ffiString(try avcEncodeJSON(resolvedPhotoSettingsInfoPayload(from: resolvedSettings)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_photo_release")
public func av_capture_photo_release(_ photoPtr: UnsafeMutableRawPointer?) {
    avcRelease(photoPtr, as: PhotoBox.self)
}

@_cdecl("av_capture_photo_info_json")
public func av_capture_photo_info_json(
    _ photoPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let photo = avcPhotoBox(photoPtr).photo
    do {
        return ffiString(try avcEncodeJSON(photoInfoPayload(from: photo)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_photo_resolved_settings")
public func av_capture_photo_resolved_settings(
    _ photoPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(ResolvedPhotoSettingsBox(avcPhotoBox(photoPtr).photo.resolvedSettings))
}

@_cdecl("av_capture_photo_pixel_buffer")
public func av_capture_photo_pixel_buffer(
    _ photoPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let pixelBuffer = avcPhotoBox(photoPtr).photo.pixelBuffer else {
        return nil
    }
    return Unmanaged.passRetained(pixelBuffer).toOpaque()
}

@_cdecl("av_capture_photo_file_data_representation")
public func av_capture_photo_file_data_representation(
    _ photoPtr: UnsafeMutableRawPointer,
    _ outLength: UnsafeMutablePointer<Int>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<UInt8>? {
    guard let data = avcPhotoBox(photoPtr).photo.fileDataRepresentation() else {
        outLength.pointee = 0
        return nil
    }
    guard !data.isEmpty else {
        outLength.pointee = 0
        return nil
    }
    let bytes = UnsafeMutablePointer<UInt8>.allocate(capacity: data.count)
    data.copyBytes(to: bytes, count: data.count)
    outLength.pointee = data.count
    return bytes
}

@_cdecl("av_capture_photo_file_data_free")
public func av_capture_photo_file_data_free(_ bytes: UnsafeMutablePointer<UInt8>?) {
    bytes?.deallocate()
}
