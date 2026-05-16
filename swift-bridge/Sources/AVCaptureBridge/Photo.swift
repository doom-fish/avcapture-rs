import AVFoundation
import Foundation

private struct PhotoSettingsInfoPayload: Codable {
    let uniqueId: Int64
    let processedFileType: String?
    let flashMode: Int32?
    let photoQualityPrioritization: Int32?
}

private struct PhotoInfoPayload: Codable {
    let uniqueId: Int64
    let timestamp: CMTimePayload
    let photoCount: Int
    let pixelBufferAvailable: Bool
    let constantColorConfidenceMapAvailable: Bool?
    let constantColorCenterWeightedMeanConfidenceLevel: Float?
    let constantColorFallbackPhoto: Bool?
}

final class PhotoSettingsBox: NSObject {
    let settings: AVCapturePhotoSettings

    init(_ settings: AVCapturePhotoSettings = AVCapturePhotoSettings()) {
        self.settings = settings
    }
}

final class PhotoBox: NSObject {
    let photo: AVCapturePhoto

    init(_ photo: AVCapturePhoto) {
        self.photo = photo
    }
}

private func photoSettingsInfoPayload(from settings: AVCapturePhotoSettings) -> PhotoSettingsInfoPayload {
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
        uniqueId: settings.uniqueID,
        processedFileType: settings.processedFileType?.rawValue,
        flashMode: flashMode,
        photoQualityPrioritization: photoQualityPrioritization
    )
}

private func photoInfoPayload(from photo: AVCapturePhoto) -> PhotoInfoPayload {
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
        uniqueId: photo.resolvedSettings.uniqueID,
        timestamp: CMTimePayload(photo.timestamp),
        photoCount: photo.photoCount,
        pixelBufferAvailable: photo.pixelBuffer != nil,
        constantColorConfidenceMapAvailable: constantColorConfidenceMapAvailable,
        constantColorCenterWeightedMeanConfidenceLevel: constantColorCenterWeightedMeanConfidenceLevel,
        constantColorFallbackPhoto: constantColorFallbackPhoto
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
    let settings = avcPhotoSettingsBox(settingsPtr).settings
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
    avcPhotoSettingsBox(settingsPtr).settings.flashMode = mode
    return AVC_OK
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
    avcPhotoSettingsBox(settingsPtr).settings.photoQualityPrioritization = prioritization
    return AVC_OK
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
