import AVFoundation
import Foundation

private struct DiscoveryCriteriaPayload: Codable {
    let deviceTypes: [String]
    let mediaType: String?
    let position: Int32
}

@_cdecl("av_capture_device_discovery_session_create")
public func av_capture_device_discovery_session_create(
    _ criteriaJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    do {
        let criteria = try avcDecodeJSON(criteriaJson, as: DiscoveryCriteriaPayload.self)
        let deviceTypes = criteria.deviceTypes.map(AVCaptureDevice.DeviceType.init(rawValue:))
        let mediaType: AVMediaType?
        if let mediaTypeRaw = criteria.mediaType {
            guard let decoded = avcDecodeMediaType(mediaTypeRaw) else {
                throw BridgeError.message("unsupported media type: \(mediaTypeRaw)")
            }
            mediaType = decoded
        } else {
            mediaType = nil
        }
        let position = avcDecodeDevicePosition(criteria.position)
        let session = AVCaptureDevice.DiscoverySession(
            deviceTypes: deviceTypes,
            mediaType: mediaType,
            position: position
        )
        return avcRetain(DiscoverySessionBox(session))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_discovery_session_release")
public func av_capture_device_discovery_session_release(_ sessionPtr: UnsafeMutableRawPointer?) {
    avcRelease(sessionPtr, as: DiscoverySessionBox.self)
}

@_cdecl("av_capture_device_discovery_session_devices_count")
public func av_capture_device_discovery_session_devices_count(_ sessionPtr: UnsafeMutableRawPointer) -> Int {
    avcDiscoverySessionBox(sessionPtr).session.devices.count
}

@_cdecl("av_capture_device_discovery_session_device_at_index")
public func av_capture_device_discovery_session_device_at_index(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let session = avcDiscoverySessionBox(sessionPtr).session
    guard index >= 0, index < session.devices.count else {
        outErrorMessage?.pointee = ffiString("discovery session device index out of range")
        return nil
    }
    return avcRetain(DeviceBox(session.devices[index]))
}
