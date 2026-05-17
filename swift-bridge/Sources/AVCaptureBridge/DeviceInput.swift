import AVFoundation
import Foundation

private struct DeviceInputBridgeInfoPayload: Codable {
    let deviceUniqueId: String
    let deviceLocalizedName: String
    let portsCount: Int
    let multichannelAudioMode: Int32?
    let windNoiseRemovalSupported: Bool
    let windNoiseRemovalEnabled: Bool
}

final class DeviceInputBox: CaptureInputBoxBase {
    let deviceInput: AVCaptureDeviceInput

    init(_ deviceInput: AVCaptureDeviceInput) {
        self.deviceInput = deviceInput
    }

    override var input: AVCaptureInput {
        deviceInput
    }
}

@_cdecl("av_capture_device_input_create")
public func av_capture_device_input_create(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = avcDeviceBox(devicePtr).device
    do {
        let input = try AVCaptureDeviceInput(device: device)
        return avcRetain(DeviceInputBox(input))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_input_release")
public func av_capture_device_input_release(_ inputPtr: UnsafeMutableRawPointer?) {
    avcRelease(inputPtr, as: DeviceInputBox.self)
}

@_cdecl("av_capture_device_input_info_json")
public func av_capture_device_input_info_json(
    _ inputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let input = avcUnretained(inputPtr, as: DeviceInputBox.self).deviceInput
    let multichannelAudioMode: Int32?
    let windNoiseRemovalSupported: Bool
    let windNoiseRemovalEnabled: Bool
    if #available(macOS 15.0, *) {
        multichannelAudioMode = Int32(input.multichannelAudioMode.rawValue)
        windNoiseRemovalSupported = input.isWindNoiseRemovalSupported
        windNoiseRemovalEnabled = input.isWindNoiseRemovalEnabled
    } else {
        multichannelAudioMode = nil
        windNoiseRemovalSupported = false
        windNoiseRemovalEnabled = false
    }
    let payload = DeviceInputBridgeInfoPayload(
        deviceUniqueId: input.device.uniqueID,
        deviceLocalizedName: input.device.localizedName,
        portsCount: input.ports.count,
        multichannelAudioMode: multichannelAudioMode,
        windNoiseRemovalSupported: windNoiseRemovalSupported,
        windNoiseRemovalEnabled: windNoiseRemovalEnabled
    )
    do {
        return ffiString(try avcEncodeJSON(payload))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_input_is_multichannel_audio_mode_supported")
public func av_capture_device_input_is_multichannel_audio_mode_supported(
    _ inputPtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32
) -> Bool {
    guard #available(macOS 15.0, *) else {
        return false
    }
    guard let mode = AVCaptureMultichannelAudioMode(rawValue: Int(modeRaw)) else {
        return false
    }
    return avcUnretained(inputPtr, as: DeviceInputBox.self).deviceInput.isMultichannelAudioModeSupported(mode)
}

@_cdecl("av_capture_device_input_set_multichannel_audio_mode")
public func av_capture_device_input_set_multichannel_audio_mode(
    _ inputPtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let input = avcUnretained(inputPtr, as: DeviceInputBox.self).deviceInput
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("multichannel audio mode requires macOS 15.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let mode = AVCaptureMultichannelAudioMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported multichannel audio mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard input.isMultichannelAudioModeSupported(mode) else {
        outErrorMessage?.pointee = ffiString("device input does not support multichannel audio mode \(modeRaw)")
        return AVC_INPUT_ERROR
    }
    input.multichannelAudioMode = mode
    return AVC_OK
}

@_cdecl("av_capture_device_input_set_wind_noise_removal_enabled")
public func av_capture_device_input_set_wind_noise_removal_enabled(
    _ inputPtr: UnsafeMutableRawPointer,
    _ enabled: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let input = avcUnretained(inputPtr, as: DeviceInputBox.self).deviceInput
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("wind noise removal requires macOS 15.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard input.isWindNoiseRemovalSupported else {
        outErrorMessage?.pointee = ffiString("device input does not support wind noise removal")
        return AVC_INPUT_ERROR
    }
    input.isWindNoiseRemovalEnabled = enabled
    return AVC_OK
}
