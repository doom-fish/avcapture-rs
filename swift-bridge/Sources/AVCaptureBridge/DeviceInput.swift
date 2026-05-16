import AVFoundation
import Foundation

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
