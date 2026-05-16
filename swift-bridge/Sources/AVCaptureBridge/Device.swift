import AVFoundation
import Foundation

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
    return Unmanaged.passRetained(device).toOpaque()
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
    return Unmanaged.passRetained(device).toOpaque()
}

@_cdecl("av_capture_device_release")
public func av_capture_device_release(_ devicePtr: UnsafeMutableRawPointer?) {
    guard let devicePtr else { return }
    Unmanaged<AVCaptureDevice>.fromOpaque(devicePtr).release()
}

@_cdecl("av_capture_device_info_json")
public func av_capture_device_info_json(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let device = Unmanaged<AVCaptureDevice>.fromOpaque(devicePtr).takeUnretainedValue()
    do {
        return ffiString(try avcEncodeJSON(avcDeviceInfoPayload(from: device)))
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
    let device = Unmanaged<AVCaptureDevice>.fromOpaque(devicePtr).takeUnretainedValue()
    let raw = String(cString: presetPtr)
    guard let preset = avcDecodeSessionPreset(raw) else {
        return false
    }
    return device.supportsSessionPreset(preset)
}

@_cdecl("av_capture_device_input_create")
public func av_capture_device_input_create(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = Unmanaged<AVCaptureDevice>.fromOpaque(devicePtr).takeUnretainedValue()
    do {
        let input = try AVCaptureDeviceInput(device: device)
        return Unmanaged.passRetained(input).toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_input_release")
public func av_capture_device_input_release(_ inputPtr: UnsafeMutableRawPointer?) {
    guard let inputPtr else { return }
    Unmanaged<AVCaptureDeviceInput>.fromOpaque(inputPtr).release()
}

@_cdecl("av_capture_device_input_info_json")
public func av_capture_device_input_info_json(
    _ inputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let input = Unmanaged<AVCaptureDeviceInput>.fromOpaque(inputPtr).takeUnretainedValue()
    let payload = DeviceInputInfoPayload(
        deviceUniqueId: input.device.uniqueID,
        deviceLocalizedName: input.device.localizedName,
        portsCount: input.ports.count
    )
    do {
        return ffiString(try avcEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

private func avcDeviceInfoPayload(from device: AVCaptureDevice) -> CaptureDeviceInfoPayload {
    CaptureDeviceInfoPayload(
        uniqueId: device.uniqueID,
        localizedName: device.localizedName,
        manufacturer: device.manufacturer
    )
}
