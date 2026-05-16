import AVFoundation
import Foundation

private struct CaptureDeviceDetailsPayload: Codable {
    let uniqueId: String
    let localizedName: String
    let manufacturer: String
    let transportType: Int?
    let mediaTypes: [String]
    let position: Int32
    let deviceType: String
    let hasFlash: Bool
    let flashAvailable: Bool
    let hasTorch: Bool
    let torchAvailable: Bool
    let torchLevel: Float?
    let formatsCount: Int
    let activeVideoMinFrameDuration: CMTimePayload
    let activeVideoMaxFrameDuration: CMTimePayload
}

private func avcDeviceInfoPayload(from device: AVCaptureDevice) -> CaptureDeviceInfoPayload {
    CaptureDeviceInfoPayload(
        uniqueId: device.uniqueID,
        localizedName: device.localizedName,
        manufacturer: device.manufacturer
    )
}

private func avcDeviceDetailsPayload(from device: AVCaptureDevice) -> CaptureDeviceDetailsPayload {
    let knownMediaTypes: [AVMediaType] = [.video, .audio, .muxed, .metadata]
    let mediaTypes = knownMediaTypes
        .filter { device.hasMediaType($0) }
        .map(avcEncodeMediaType)
    return CaptureDeviceDetailsPayload(
        uniqueId: device.uniqueID,
        localizedName: device.localizedName,
        manufacturer: device.manufacturer,
        transportType: Int(device.transportType),
        mediaTypes: mediaTypes,
        position: avcEncodeDevicePosition(device.position),
        deviceType: device.deviceType.rawValue,
        hasFlash: device.hasFlash,
        flashAvailable: device.isFlashAvailable,
        hasTorch: device.hasTorch,
        torchAvailable: device.isTorchAvailable,
        torchLevel: device.hasTorch ? Float(device.torchLevel) : nil,
        formatsCount: device.formats.count,
        activeVideoMinFrameDuration: CMTimePayload(device.activeVideoMinFrameDuration),
        activeVideoMaxFrameDuration: CMTimePayload(device.activeVideoMaxFrameDuration)
    )
}

@_cdecl("av_capture_authorization_status")
public func av_capture_authorization_status(
    _ mediaTypePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let raw = String(cString: mediaTypePtr)
    guard let mediaType = avcDecodeMediaType(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported media type: \(raw)")
        return AVC_INVALID_ARGUMENT
    }
    guard mediaType == .audio || mediaType == .video else {
        outErrorMessage?.pointee = ffiString("authorization status is only defined for audio/video capture")
        return AVC_INVALID_ARGUMENT
    }
    return Int32(AVCaptureDevice.authorizationStatus(for: mediaType).rawValue)
}

@_cdecl("av_capture_devices_json")
public func av_capture_devices_json(
    _ mediaTypePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let raw = String(cString: mediaTypePtr)
    guard let mediaType = avcDecodeMediaType(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported media type: \(raw)")
        return nil
    }
    let payload = AVCaptureDevice.devices(for: mediaType).map(avcDeviceInfoPayload)
    do {
        return ffiString(try avcEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_default_device")
public func av_capture_default_device(
    _ mediaTypePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let raw = String(cString: mediaTypePtr)
    guard let mediaType = avcDecodeMediaType(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported media type: \(raw)")
        return nil
    }
    guard let device = AVCaptureDevice.default(for: mediaType) else {
        return nil
    }
    return avcRetain(DeviceBox(device))
}

@_cdecl("av_capture_default_device_for_type")
public func av_capture_default_device_for_type(
    _ deviceTypePtr: UnsafePointer<CChar>,
    _ mediaTypePtr: UnsafePointer<CChar>?,
    _ positionRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let deviceTypeRaw = String(cString: deviceTypePtr)
    let deviceType = AVCaptureDevice.DeviceType(rawValue: deviceTypeRaw)
    let mediaType: AVMediaType?
    if let mediaTypePtr {
        let raw = String(cString: mediaTypePtr)
        guard let decoded = avcDecodeMediaType(raw) else {
            outErrorMessage?.pointee = ffiString("unsupported media type: \(raw)")
            return nil
        }
        mediaType = decoded
    } else {
        mediaType = nil
    }
    let position = avcDecodeDevicePosition(positionRaw)
    guard let device = AVCaptureDevice.default(deviceType, for: mediaType, position: position) else {
        return nil
    }
    return avcRetain(DeviceBox(device))
}

@_cdecl("av_capture_device_with_unique_id")
public func av_capture_device_with_unique_id(
    _ uniqueIdPtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let uniqueId = String(cString: uniqueIdPtr)
    guard let device = AVCaptureDevice.devices().first(where: { $0.uniqueID == uniqueId }) else {
        return nil
    }
    return avcRetain(DeviceBox(device))
}

@_cdecl("av_capture_device_release")
public func av_capture_device_release(_ devicePtr: UnsafeMutableRawPointer?) {
    avcRelease(devicePtr, as: DeviceBox.self)
}

@_cdecl("av_capture_device_info_json")
public func av_capture_device_info_json(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let device = avcDeviceBox(devicePtr).device
    do {
        return ffiString(try avcEncodeJSON(avcDeviceInfoPayload(from: device)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_details_json")
public func av_capture_device_details_json(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let device = avcDeviceBox(devicePtr).device
    do {
        return ffiString(try avcEncodeJSON(avcDeviceDetailsPayload(from: device)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_supports_session_preset")
public func av_capture_device_supports_session_preset(
    _ devicePtr: UnsafeMutableRawPointer,
    _ presetPtr: UnsafePointer<CChar>
) -> Bool {
    let device = avcDeviceBox(devicePtr).device
    let raw = String(cString: presetPtr)
    guard let preset = avcDecodeSessionPreset(raw) else {
        return false
    }
    return device.supportsSessionPreset(preset)
}

@_cdecl("av_capture_device_formats_count")
public func av_capture_device_formats_count(_ devicePtr: UnsafeMutableRawPointer) -> Int {
    avcDeviceBox(devicePtr).device.formats.count
}

@_cdecl("av_capture_device_format_at_index")
public func av_capture_device_format_at_index(
    _ devicePtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = avcDeviceBox(devicePtr).device
    guard index >= 0, index < device.formats.count else {
        outErrorMessage?.pointee = ffiString("device format index out of range")
        return nil
    }
    return avcRetain(DeviceFormatBox(device.formats[index]))
}

@_cdecl("av_capture_device_active_format")
public func av_capture_device_active_format(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = avcDeviceBox(devicePtr).device
    return avcRetain(DeviceFormatBox(device.activeFormat))
}

@_cdecl("av_capture_device_active_video_min_frame_duration")
public func av_capture_device_active_video_min_frame_duration(_ devicePtr: UnsafeMutableRawPointer) -> CMTime {
    avcDeviceBox(devicePtr).device.activeVideoMinFrameDuration
}

@_cdecl("av_capture_device_active_video_max_frame_duration")
public func av_capture_device_active_video_max_frame_duration(_ devicePtr: UnsafeMutableRawPointer) -> CMTime {
    avcDeviceBox(devicePtr).device.activeVideoMaxFrameDuration
}

@_cdecl("av_capture_device_lock_for_configuration")
public func av_capture_device_lock_for_configuration(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    do {
        try device.lockForConfiguration()
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_DEVICE_ERROR
    }
}

@_cdecl("av_capture_device_unlock_for_configuration")
public func av_capture_device_unlock_for_configuration(_ devicePtr: UnsafeMutableRawPointer) {
    avcDeviceBox(devicePtr).device.unlockForConfiguration()
}

@_cdecl("av_capture_device_set_active_format")
public func av_capture_device_set_active_format(
    _ devicePtr: UnsafeMutableRawPointer,
    _ formatPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    let format = avcDeviceFormatBox(formatPtr).format
    device.activeFormat = format
    return AVC_OK
}

@_cdecl("av_capture_device_set_active_video_min_frame_duration")
public func av_capture_device_set_active_video_min_frame_duration(
    _ devicePtr: UnsafeMutableRawPointer,
    _ duration: CMTime,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    device.activeVideoMinFrameDuration = duration
    return AVC_OK
}

@_cdecl("av_capture_device_set_active_video_max_frame_duration")
public func av_capture_device_set_active_video_max_frame_duration(
    _ devicePtr: UnsafeMutableRawPointer,
    _ duration: CMTime,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    device.activeVideoMaxFrameDuration = duration
    return AVC_OK
}

@_cdecl("av_capture_device_set_torch_mode")
public func av_capture_device_set_torch_mode(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard let mode = AVCaptureDevice.TorchMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported torch mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.hasTorch else {
        outErrorMessage?.pointee = ffiString("device does not have a torch")
        return AVC_DEVICE_ERROR
    }
    guard device.isTorchModeSupported(mode) else {
        outErrorMessage?.pointee = ffiString("device does not support torch mode \(modeRaw)")
        return AVC_DEVICE_ERROR
    }
    device.torchMode = mode
    return AVC_OK
}
