import AVFoundation
import Foundation

private struct CaptureDeviceInputSourcePayload: Codable {
    let inputSourceId: String
    let localizedName: String
}

private struct CaptureReactionEffectStatePayload: Codable {
    let reactionType: String
    let startTime: CMTimePayload
    let endTime: CMTimePayload
}

private struct CaptureDeviceRotationCoordinatorInfoPayload: Codable {
    let videoRotationAngleForHorizonLevelPreview: Double
    let videoRotationAngleForHorizonLevelCapture: Double
}

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
    let exposureMode: Int32?
    let formatsCount: Int
    let activeVideoMinFrameDuration: CMTimePayload
    let activeVideoMaxFrameDuration: CMTimePayload
    let focusMode: Int32?
    let whiteBalanceMode: Int32?
    let autoFocusSystem: Int32?
    let activeColorSpace: Int32?
    let supportedColorSpaces: [Int32]
    let transportControlsSupported: Bool
    let transportControlsPlaybackMode: Int32?
    let transportControlsSpeed: Float?
    let inputSources: [CaptureDeviceInputSourcePayload]
    let activeInputSourceId: String?
    let primaryConstituentDeviceSwitchingBehavior: Int32?
    let primaryConstituentDeviceRestrictedSwitchingBehaviorConditions: UInt64?
    let activePrimaryConstituentDeviceSwitchingBehavior: Int32?
    let activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions: UInt64?
    let centerStageControlMode: Int32?
    let centerStageEnabled: Bool?
    let centerStageActive: Bool?
    let preferredMicrophoneMode: Int32?
    let activeMicrophoneMode: Int32?
    let reactionEffectsEnabled: Bool?
    let reactionEffectGesturesEnabled: Bool?
    let canPerformReactionEffects: Bool?
    let availableReactionTypes: [String]
    let reactionEffectsInProgress: [CaptureReactionEffectStatePayload]
    let cameraLensSmudgeDetectionEnabled: Bool?
    let cameraLensSmudgeDetectionInterval: CMTimePayload
    let cameraLensSmudgeDetectionStatus: Int32?
    let cinematicVideoCaptureSceneMonitoringStatuses: [String]
}

final class DeviceInputSourceBox: NSObject {
    let inputSource: AVCaptureDevice.InputSource

    init(_ inputSource: AVCaptureDevice.InputSource) {
        self.inputSource = inputSource
    }
}

final class DeviceRotationCoordinatorBox: NSObject {
    let coordinator: AnyObject

    @available(macOS 14.0, *)
    init(_ coordinator: AVCaptureDevice.RotationCoordinator) {
        self.coordinator = coordinator
    }
}

private func avcDeviceInputSourceBox(_ ptr: UnsafeMutableRawPointer) -> DeviceInputSourceBox {
    avcUnretained(ptr, as: DeviceInputSourceBox.self)
}

@available(macOS 14.0, *)
private func avcDeviceRotationCoordinator(_ ptr: UnsafeMutableRawPointer) -> AVCaptureDevice.RotationCoordinator {
    avcUnretained(ptr, as: DeviceRotationCoordinatorBox.self).coordinator as! AVCaptureDevice.RotationCoordinator
}

private func avcSupportedExposureModes(for device: AVCaptureDevice) -> [AVCaptureDevice.ExposureMode] {
    var modes: [AVCaptureDevice.ExposureMode] = [.locked, .autoExpose, .continuousAutoExposure]
    if #available(macOS 10.15, *) {
        modes.append(.custom)
    }
    return modes.filter { device.isExposureModeSupported($0) }
}

private func avcSupportedFocusModes(for device: AVCaptureDevice) -> [AVCaptureDevice.FocusMode] {
    [.locked, .autoFocus, .continuousAutoFocus].filter { device.isFocusModeSupported($0) }
}

private func avcSupportedWhiteBalanceModes(for device: AVCaptureDevice) -> [AVCaptureDevice.WhiteBalanceMode] {
    [.locked, .autoWhiteBalance, .continuousAutoWhiteBalance].filter { device.isWhiteBalanceModeSupported($0) }
}

private func avcDeviceInfoPayload(from device: AVCaptureDevice) -> CaptureDeviceInfoPayload {
    CaptureDeviceInfoPayload(
        uniqueId: device.uniqueID,
        localizedName: device.localizedName,
        manufacturer: device.manufacturer
    )
}

private func avcDeviceInputSourcePayload(from inputSource: AVCaptureDevice.InputSource) -> CaptureDeviceInputSourcePayload {
    CaptureDeviceInputSourcePayload(
        inputSourceId: inputSource.inputSourceID,
        localizedName: inputSource.localizedName
    )
}

@available(macOS 14.0, *)
private func avcEncodeReactionType(_ reactionType: AVCaptureReactionType) -> String {
    reactionType.rawValue
}

@available(macOS 14.0, *)
private func avcDecodeReactionType(_ raw: String) -> AVCaptureReactionType? {
    switch raw {
    case AVCaptureReactionType.thumbsUp.rawValue:
        return .thumbsUp
    case AVCaptureReactionType.thumbsDown.rawValue:
        return .thumbsDown
    case AVCaptureReactionType.balloons.rawValue:
        return .balloons
    case AVCaptureReactionType.heart.rawValue:
        return .heart
    case AVCaptureReactionType.fireworks.rawValue:
        return .fireworks
    case AVCaptureReactionType.rain.rawValue:
        return .rain
    case AVCaptureReactionType.confetti.rawValue:
        return .confetti
    case AVCaptureReactionType.lasers.rawValue:
        return .lasers
    default:
        return nil
    }
}

@available(macOS 14.0, *)
private func avcReactionEffectStatePayload(from state: AVCaptureReactionEffectState) -> CaptureReactionEffectStatePayload {
    CaptureReactionEffectStatePayload(
        reactionType: avcEncodeReactionType(state.reactionType),
        startTime: CMTimePayload(state.startTime),
        endTime: CMTimePayload(state.endTime)
    )
}

@available(macOS 14.0, *)
private func avcDeviceRotationCoordinatorInfoPayload(
    from coordinator: AVCaptureDevice.RotationCoordinator
) -> CaptureDeviceRotationCoordinatorInfoPayload {
    CaptureDeviceRotationCoordinatorInfoPayload(
        videoRotationAngleForHorizonLevelPreview: Double(coordinator.videoRotationAngleForHorizonLevelPreview),
        videoRotationAngleForHorizonLevelCapture: Double(coordinator.videoRotationAngleForHorizonLevelCapture)
    )
}

private func avcDecodeColorSpace(_ raw: Int32) -> AVCaptureColorSpace? {
    switch raw {
    case 0:
        return .sRGB
    case 1:
        return .P3_D65
    default:
        return nil
    }
}

private func avcNormalizePoint(_ x: Double, _ y: Double) -> CGPoint? {
    guard x.isFinite, y.isFinite, (0.0 ... 1.0).contains(x), (0.0 ... 1.0).contains(y) else {
        return nil
    }
    return CGPoint(x: x, y: y)
}

private func avcClassBool(_ available: Bool, _ value: @autoclosure () -> Bool) -> Bool? {
    available ? value() : nil
}

private func avcClassInt32(_ available: Bool, _ value: @autoclosure () -> Int32) -> Int32? {
    available ? value() : nil
}

private func avcEncodeClassBool(_ available: Bool, _ value: @autoclosure () -> Bool) -> Int32 {
    guard available else { return -1 }
    return value() ? 1 : 0
}

private func avcDeviceDetailsPayload(from device: AVCaptureDevice) -> CaptureDeviceDetailsPayload {
    let knownMediaTypes: [AVMediaType] = [.video, .audio, .muxed, .metadata]
    let mediaTypes = knownMediaTypes
        .filter { device.hasMediaType($0) }
        .map(avcEncodeMediaType)
    let isVideoDevice = device.hasMediaType(.video)
    let supportedExposureModes = avcSupportedExposureModes(for: device)
    let supportedFocusModes = isVideoDevice ? avcSupportedFocusModes(for: device) : []
    let supportedWhiteBalanceModes = isVideoDevice ? avcSupportedWhiteBalanceModes(for: device) : []

    let activeColorSpace: Int32?
    let supportedColorSpaces: [Int32]
    let autoFocusSystem: Int32?
    if isVideoDevice {
        activeColorSpace = Int32(device.activeColorSpace.rawValue)
        supportedColorSpaces = device.activeFormat.supportedColorSpaces.map { Int32($0.rawValue) }
        autoFocusSystem = Int32(device.activeFormat.autoFocusSystem.rawValue)
    } else {
        activeColorSpace = nil
        supportedColorSpaces = []
        autoFocusSystem = nil
    }

    let transportControlsSupported = device.transportControlsSupported
    let transportControlsPlaybackMode = transportControlsSupported
        ? Int32(device.transportControlsPlaybackMode.rawValue)
        : nil
    let transportControlsSpeed = transportControlsSupported
        ? Float(device.transportControlsSpeed)
        : nil

    let inputSources = device.inputSources.map(avcDeviceInputSourcePayload)
    let activeInputSourceId = device.activeInputSource?.inputSourceID

    let primaryConstituentDeviceSwitchingBehavior: Int32?
    let primaryConstituentDeviceRestrictedSwitchingBehaviorConditions: UInt64?
    let activePrimaryConstituentDeviceSwitchingBehavior: Int32?
    let activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions: UInt64?
    if #available(macOS 12.0, *), isVideoDevice {
        primaryConstituentDeviceSwitchingBehavior = Int32(device.primaryConstituentDeviceSwitchingBehavior.rawValue)
        primaryConstituentDeviceRestrictedSwitchingBehaviorConditions = UInt64(device.primaryConstituentDeviceRestrictedSwitchingBehaviorConditions.rawValue)
        activePrimaryConstituentDeviceSwitchingBehavior = Int32(device.activePrimaryConstituentDeviceSwitchingBehavior.rawValue)
        activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions = UInt64(device.activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions.rawValue)
    } else {
        primaryConstituentDeviceSwitchingBehavior = nil
        primaryConstituentDeviceRestrictedSwitchingBehaviorConditions = nil
        activePrimaryConstituentDeviceSwitchingBehavior = nil
        activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions = nil
    }

    let centerStageControlMode: Int32?
    let centerStageEnabled: Bool?
    let centerStageActive: Bool?
    if #available(macOS 12.3, *) {
        centerStageControlMode = Int32(AVCaptureDevice.centerStageControlMode.rawValue)
        centerStageEnabled = AVCaptureDevice.isCenterStageEnabled
        centerStageActive = isVideoDevice ? device.isCenterStageActive : nil
    } else {
        centerStageControlMode = nil
        centerStageEnabled = nil
        centerStageActive = nil
    }

    let preferredMicrophoneMode: Int32?
    let activeMicrophoneMode: Int32?
    if #available(macOS 12.0, *) {
        preferredMicrophoneMode = Int32(AVCaptureDevice.preferredMicrophoneMode.rawValue)
        activeMicrophoneMode = Int32(AVCaptureDevice.activeMicrophoneMode.rawValue)
    } else {
        preferredMicrophoneMode = nil
        activeMicrophoneMode = nil
    }

    let reactionEffectsEnabled: Bool?
    let reactionEffectGesturesEnabled: Bool?
    if #available(macOS 14.0, *) {
        reactionEffectsEnabled = AVCaptureDevice.reactionEffectsEnabled
        reactionEffectGesturesEnabled = AVCaptureDevice.reactionEffectGesturesEnabled
    } else {
        reactionEffectsEnabled = nil
        reactionEffectGesturesEnabled = nil
    }
    let canPerformReactionEffects: Bool?
    let availableReactionTypes: [String]
    let reactionEffectsInProgress: [CaptureReactionEffectStatePayload]
    if #available(macOS 14.0, *), isVideoDevice {
        canPerformReactionEffects = device.canPerformReactionEffects
        availableReactionTypes = device.availableReactionTypes.map(avcEncodeReactionType).sorted()
        reactionEffectsInProgress = device.reactionEffectsInProgress.map(avcReactionEffectStatePayload)
    } else {
        canPerformReactionEffects = nil
        availableReactionTypes = []
        reactionEffectsInProgress = []
    }

    let cameraLensSmudgeDetectionEnabled: Bool?
    let cameraLensSmudgeDetectionInterval: CMTimePayload
    let cameraLensSmudgeDetectionStatus: Int32?
    let cinematicVideoCaptureSceneMonitoringStatuses: [String]
    if #available(macOS 26.0, *), isVideoDevice {
        let supportsLensSmudgeDetection =
            device.responds(to: #selector(getter: AVCaptureDevice.isCameraLensSmudgeDetectionEnabled)) &&
            device.responds(to: #selector(getter: AVCaptureDevice.cameraLensSmudgeDetectionInterval)) &&
            device.responds(to: #selector(getter: AVCaptureDevice.cameraLensSmudgeDetectionStatus))
        if supportsLensSmudgeDetection {
            cameraLensSmudgeDetectionEnabled = device.isCameraLensSmudgeDetectionEnabled
            cameraLensSmudgeDetectionInterval = CMTimePayload(device.cameraLensSmudgeDetectionInterval)
            cameraLensSmudgeDetectionStatus = Int32(device.cameraLensSmudgeDetectionStatus.rawValue)
        } else {
            cameraLensSmudgeDetectionEnabled = nil
            cameraLensSmudgeDetectionInterval = CMTimePayload(CMTime.invalid)
            cameraLensSmudgeDetectionStatus = nil
        }

        cinematicVideoCaptureSceneMonitoringStatuses = device.responds(
            to: #selector(getter: AVCaptureDevice.cinematicVideoCaptureSceneMonitoringStatuses)
        )
            ? device.cinematicVideoCaptureSceneMonitoringStatuses.map(\.rawValue).sorted()
            : []
    } else {
        cameraLensSmudgeDetectionEnabled = nil
        cameraLensSmudgeDetectionInterval = CMTimePayload(CMTime.invalid)
        cameraLensSmudgeDetectionStatus = nil
        cinematicVideoCaptureSceneMonitoringStatuses = []
    }

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
        exposureMode: supportedExposureModes.isEmpty ? nil : Int32(device.exposureMode.rawValue),
        formatsCount: device.formats.count,
        activeVideoMinFrameDuration: CMTimePayload(device.activeVideoMinFrameDuration),
        activeVideoMaxFrameDuration: CMTimePayload(device.activeVideoMaxFrameDuration),
        focusMode: supportedFocusModes.isEmpty ? nil : Int32(device.focusMode.rawValue),
        whiteBalanceMode: supportedWhiteBalanceModes.isEmpty ? nil : Int32(device.whiteBalanceMode.rawValue),
        autoFocusSystem: autoFocusSystem,
        activeColorSpace: activeColorSpace,
        supportedColorSpaces: supportedColorSpaces,
        transportControlsSupported: transportControlsSupported,
        transportControlsPlaybackMode: transportControlsPlaybackMode,
        transportControlsSpeed: transportControlsSpeed,
        inputSources: inputSources,
        activeInputSourceId: activeInputSourceId,
        primaryConstituentDeviceSwitchingBehavior: primaryConstituentDeviceSwitchingBehavior,
        primaryConstituentDeviceRestrictedSwitchingBehaviorConditions: primaryConstituentDeviceRestrictedSwitchingBehaviorConditions,
        activePrimaryConstituentDeviceSwitchingBehavior: activePrimaryConstituentDeviceSwitchingBehavior,
        activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions: activePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions,
        centerStageControlMode: centerStageControlMode,
        centerStageEnabled: centerStageEnabled,
        centerStageActive: centerStageActive,
        preferredMicrophoneMode: preferredMicrophoneMode,
        activeMicrophoneMode: activeMicrophoneMode,
        reactionEffectsEnabled: reactionEffectsEnabled,
        reactionEffectGesturesEnabled: reactionEffectGesturesEnabled,
        canPerformReactionEffects: canPerformReactionEffects,
        availableReactionTypes: availableReactionTypes,
        reactionEffectsInProgress: reactionEffectsInProgress,
        cameraLensSmudgeDetectionEnabled: cameraLensSmudgeDetectionEnabled,
        cameraLensSmudgeDetectionInterval: cameraLensSmudgeDetectionInterval,
        cameraLensSmudgeDetectionStatus: cameraLensSmudgeDetectionStatus,
        cinematicVideoCaptureSceneMonitoringStatuses: cinematicVideoCaptureSceneMonitoringStatuses
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
        let payload = avcDeviceDetailsPayload(from: device)
        let json = try avcEncodeJSON(payload)
        return ffiString(json)
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

@_cdecl("av_capture_device_is_exposure_mode_supported")
public func av_capture_device_is_exposure_mode_supported(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32
) -> Bool {
    guard let mode = AVCaptureDevice.ExposureMode(rawValue: Int(modeRaw)) else {
        return false
    }
    return avcDeviceBox(devicePtr).device.isExposureModeSupported(mode)
}

@_cdecl("av_capture_device_is_focus_mode_supported")
public func av_capture_device_is_focus_mode_supported(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32
) -> Bool {
    guard let mode = AVCaptureDevice.FocusMode(rawValue: Int(modeRaw)) else {
        return false
    }
    return avcDeviceBox(devicePtr).device.isFocusModeSupported(mode)
}

@_cdecl("av_capture_device_is_white_balance_mode_supported")
public func av_capture_device_is_white_balance_mode_supported(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32
) -> Bool {
    guard let mode = AVCaptureDevice.WhiteBalanceMode(rawValue: Int(modeRaw)) else {
        return false
    }
    return avcDeviceBox(devicePtr).device.isWhiteBalanceModeSupported(mode)
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

@_cdecl("av_capture_device_set_exposure_mode")
public func av_capture_device_set_exposure_mode(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard let mode = AVCaptureDevice.ExposureMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported exposure mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.isExposureModeSupported(mode) else {
        outErrorMessage?.pointee = ffiString("device does not support exposure mode \(modeRaw)")
        return AVC_DEVICE_ERROR
    }
    device.exposureMode = mode
    return AVC_OK
}

@_cdecl("av_capture_device_set_focus_mode")
public func av_capture_device_set_focus_mode(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard let mode = AVCaptureDevice.FocusMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported focus mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.isFocusModeSupported(mode) else {
        outErrorMessage?.pointee = ffiString("device does not support focus mode \(modeRaw)")
        return AVC_DEVICE_ERROR
    }
    device.focusMode = mode
    return AVC_OK
}

@_cdecl("av_capture_device_set_white_balance_mode")
public func av_capture_device_set_white_balance_mode(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard let mode = AVCaptureDevice.WhiteBalanceMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported white balance mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.isWhiteBalanceModeSupported(mode) else {
        outErrorMessage?.pointee = ffiString("device does not support white balance mode \(modeRaw)")
        return AVC_DEVICE_ERROR
    }
    device.whiteBalanceMode = mode
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

@_cdecl("av_capture_device_set_torch_level")
public func av_capture_device_set_torch_level(
    _ devicePtr: UnsafeMutableRawPointer,
    _ level: Float,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard device.hasTorch else {
        outErrorMessage?.pointee = ffiString("device does not have a torch")
        return AVC_DEVICE_ERROR
    }
    guard #available(macOS 10.15, *) else {
        outErrorMessage?.pointee = ffiString("torch level requires macOS 10.15 or newer")
        return AVC_OPERATION_FAILED
    }
    do {
        try device.setTorchModeOn(level: level)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_DEVICE_ERROR
    }
}

@_cdecl("av_capture_device_set_active_color_space")
public func av_capture_device_set_active_color_space(
    _ devicePtr: UnsafeMutableRawPointer,
    _ colorSpaceRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard let colorSpace = avcDecodeColorSpace(colorSpaceRaw) else {
        outErrorMessage?.pointee = ffiString("unsupported color space: \(colorSpaceRaw)")
        return AVC_INVALID_ARGUMENT
    }
    device.activeColorSpace = colorSpace
    return AVC_OK
}

@_cdecl("av_capture_device_input_sources_count")
public func av_capture_device_input_sources_count(_ devicePtr: UnsafeMutableRawPointer) -> Int {
    avcDeviceBox(devicePtr).device.inputSources.count
}

@_cdecl("av_capture_device_input_source_at_index")
public func av_capture_device_input_source_at_index(
    _ devicePtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = avcDeviceBox(devicePtr).device
    guard index >= 0, index < device.inputSources.count else {
        outErrorMessage?.pointee = ffiString("device input source index out of range")
        return nil
    }
    return avcRetain(DeviceInputSourceBox(device.inputSources[index]))
}

@_cdecl("av_capture_device_active_input_source")
public func av_capture_device_active_input_source(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = avcDeviceBox(devicePtr).device
    guard let inputSource = device.activeInputSource else {
        return nil
    }
    return avcRetain(DeviceInputSourceBox(inputSource))
}

@_cdecl("av_capture_device_set_active_input_source")
public func av_capture_device_set_active_input_source(
    _ devicePtr: UnsafeMutableRawPointer,
    _ inputSourcePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    let inputSource = avcDeviceInputSourceBox(inputSourcePtr).inputSource
    guard device.inputSources.contains(where: { $0.inputSourceID == inputSource.inputSourceID }) else {
        outErrorMessage?.pointee = ffiString("input source does not belong to device")
        return AVC_INVALID_ARGUMENT
    }
    device.activeInputSource = inputSource
    return AVC_OK
}

@_cdecl("av_capture_device_input_source_release")
public func av_capture_device_input_source_release(_ inputSourcePtr: UnsafeMutableRawPointer?) {
    avcRelease(inputSourcePtr, as: DeviceInputSourceBox.self)
}

@_cdecl("av_capture_device_input_source_info_json")
public func av_capture_device_input_source_info_json(
    _ inputSourcePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let inputSource = avcDeviceInputSourceBox(inputSourcePtr).inputSource
    do {
        return ffiString(try avcEncodeJSON(avcDeviceInputSourcePayload(from: inputSource)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_set_transport_controls_playback_mode")
public func av_capture_device_set_transport_controls_playback_mode(
    _ devicePtr: UnsafeMutableRawPointer,
    _ modeRaw: Int32,
    _ speed: Float,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard let mode = AVCaptureDevice.TransportControlsPlaybackMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported transport controls playback mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.transportControlsSupported else {
        outErrorMessage?.pointee = ffiString("device does not support transport controls")
        return AVC_DEVICE_ERROR
    }
    device.setTransportControlsPlaybackMode(mode, speed: speed)
    return AVC_OK
}

@_cdecl("av_capture_device_set_primary_constituent_device_switching_behavior")
public func av_capture_device_set_primary_constituent_device_switching_behavior(
    _ devicePtr: UnsafeMutableRawPointer,
    _ behaviorRaw: Int32,
    _ conditionsRaw: UInt64,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("constituent device switching requires macOS 12.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let behavior = AVCaptureDevice.PrimaryConstituentDeviceSwitchingBehavior(rawValue: Int(behaviorRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported switching behavior: \(behaviorRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard behavior != .unsupported else {
        outErrorMessage?.pointee = ffiString("unsupported is not a valid requested switching behavior")
        return AVC_INVALID_ARGUMENT
    }
    let conditions = AVCaptureDevice.PrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions(rawValue: UInt(conditionsRaw))
    guard behavior == .restricted || conditions.isEmpty else {
        outErrorMessage?.pointee = ffiString("restricted switching conditions require restricted switching behavior")
        return AVC_INVALID_ARGUMENT
    }
    guard device.activePrimaryConstituentDeviceSwitchingBehavior != .unsupported else {
        outErrorMessage?.pointee = ffiString("device does not support constituent device switching")
        return AVC_DEVICE_ERROR
    }
    device.setPrimaryConstituentDeviceSwitchingBehavior(
        behavior,
        restrictedSwitchingBehaviorConditions: conditions
    )
    return AVC_OK
}

@_cdecl("av_capture_device_center_stage_control_mode")
public func av_capture_device_center_stage_control_mode() -> Int32 {
    guard #available(macOS 12.3, *) else {
        return -1
    }
    return Int32(AVCaptureDevice.centerStageControlMode.rawValue)
}

@_cdecl("av_capture_device_center_stage_enabled")
public func av_capture_device_center_stage_enabled() -> Int32 {
    guard #available(macOS 12.3, *) else {
        return -1
    }
    return AVCaptureDevice.isCenterStageEnabled ? 1 : 0
}

@_cdecl("av_capture_device_set_center_stage_control_mode")
public func av_capture_device_set_center_stage_control_mode(
    _ modeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 12.3, *) else {
        outErrorMessage?.pointee = ffiString("center stage control mode requires macOS 12.3 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let mode = AVCaptureDevice.CenterStageControlMode(rawValue: Int(modeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported center stage control mode: \(modeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    AVCaptureDevice.centerStageControlMode = mode
    return AVC_OK
}

@_cdecl("av_capture_device_set_center_stage_enabled")
public func av_capture_device_set_center_stage_enabled(
    _ enabled: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 12.3, *) else {
        outErrorMessage?.pointee = ffiString("center stage enablement requires macOS 12.3 or newer")
        return AVC_OPERATION_FAILED
    }
    guard AVCaptureDevice.centerStageControlMode != .user else {
        outErrorMessage?.pointee = ffiString("center stage enablement requires app or cooperative control mode")
        return AVC_OPERATION_FAILED
    }
    AVCaptureDevice.isCenterStageEnabled = enabled
    return AVC_OK
}

@_cdecl("av_capture_device_preferred_microphone_mode")
public func av_capture_device_preferred_microphone_mode() -> Int32 {
    guard #available(macOS 12.0, *) else {
        return -1
    }
    return Int32(AVCaptureDevice.preferredMicrophoneMode.rawValue)
}

@_cdecl("av_capture_device_active_microphone_mode")
public func av_capture_device_active_microphone_mode() -> Int32 {
    guard #available(macOS 12.0, *) else {
        return -1
    }
    return Int32(AVCaptureDevice.activeMicrophoneMode.rawValue)
}

@_cdecl("av_capture_device_show_system_user_interface")
public func av_capture_device_show_system_user_interface(
    _ systemUserInterfaceRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 12.0, *) else {
        outErrorMessage?.pointee = ffiString("system user interface requires macOS 12.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let systemUserInterface = AVCaptureDevice.SystemUserInterface(rawValue: Int(systemUserInterfaceRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported system user interface: \(systemUserInterfaceRaw)")
        return AVC_INVALID_ARGUMENT
    }
    AVCaptureDevice.showSystemUserInterface(systemUserInterface)
    return AVC_OK
}

@_cdecl("av_capture_device_reaction_effects_enabled")
public func av_capture_device_reaction_effects_enabled() -> Int32 {
    guard #available(macOS 14.0, *) else {
        return -1
    }
    return AVCaptureDevice.reactionEffectsEnabled ? 1 : 0
}

@_cdecl("av_capture_device_reaction_effect_gestures_enabled")
public func av_capture_device_reaction_effect_gestures_enabled() -> Int32 {
    guard #available(macOS 14.0, *) else {
        return -1
    }
    return AVCaptureDevice.reactionEffectGesturesEnabled ? 1 : 0
}

@_cdecl("av_capture_device_perform_reaction_effect")
public func av_capture_device_perform_reaction_effect(
    _ devicePtr: UnsafeMutableRawPointer,
    _ reactionTypePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("reaction effects require macOS 14.0 or newer")
        return AVC_OPERATION_FAILED
    }
    let raw = String(cString: reactionTypePtr)
    guard let reactionType = avcDecodeReactionType(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported reaction type: \(raw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.availableReactionTypes.contains(reactionType) else {
        outErrorMessage?.pointee = ffiString("reaction type is not available on the current device configuration")
        return AVC_DEVICE_ERROR
    }
    guard device.canPerformReactionEffects else {
        outErrorMessage?.pointee = ffiString("device cannot perform reaction effects in the current configuration")
        return AVC_OPERATION_FAILED
    }
    device.performEffect(for: reactionType)
    return AVC_OK
}

@_cdecl("av_capture_reaction_system_image_name_for_type")
public func av_capture_reaction_system_image_name_for_type(
    _ reactionTypePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("reaction system image names require macOS 14.0 or newer")
        return nil
    }
    let raw = String(cString: reactionTypePtr)
    guard let reactionType = avcDecodeReactionType(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported reaction type: \(raw)")
        return nil
    }
    do {
        return ffiString(try avcEncodeJSON(reactionType.systemImageName))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_rotation_coordinator_create")
public func av_capture_device_rotation_coordinator_create(
    _ devicePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let device = avcDeviceBox(devicePtr).device
    guard #available(macOS 14.0, *) else {
        return nil
    }
    guard device.hasMediaType(.video) else {
        return nil
    }
    return avcRetain(DeviceRotationCoordinatorBox(AVCaptureDevice.RotationCoordinator(device: device, previewLayer: nil)))
}

@_cdecl("av_capture_device_rotation_coordinator_release")
public func av_capture_device_rotation_coordinator_release(_ coordinatorPtr: UnsafeMutableRawPointer?) {
    avcRelease(coordinatorPtr, as: DeviceRotationCoordinatorBox.self)
}

@_cdecl("av_capture_device_rotation_coordinator_info_json")
public func av_capture_device_rotation_coordinator_info_json(
    _ coordinatorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("rotation coordinator requires macOS 14.0 or newer")
        return nil
    }
    let coordinator = avcDeviceRotationCoordinator(coordinatorPtr)
    do {
        return ffiString(try avcEncodeJSON(avcDeviceRotationCoordinatorInfoPayload(from: coordinator)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_device_set_cinematic_video_tracking_focus_at_point")
public func av_capture_device_set_cinematic_video_tracking_focus_at_point(
    _ devicePtr: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double,
    _ focusModeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("cinematic video focus requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let point = avcNormalizePoint(x, y) else {
        outErrorMessage?.pointee = ffiString("focus point must be normalized between 0.0 and 1.0")
        return AVC_INVALID_ARGUMENT
    }
    guard let focusMode = AVCaptureDevice.CinematicVideoFocusMode(rawValue: Int(focusModeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported cinematic video focus mode: \(focusModeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.responds(to: #selector(AVCaptureDevice.setCinematicVideoTrackingFocus(at:focusMode:))) else {
        outErrorMessage?.pointee = ffiString("cinematic video tracking focus is not supported by this device")
        return AVC_OPERATION_FAILED
    }
    device.setCinematicVideoTrackingFocus(at: point, focusMode: focusMode)
    return AVC_OK
}

@_cdecl("av_capture_device_set_cinematic_video_fixed_focus_at_point")
public func av_capture_device_set_cinematic_video_fixed_focus_at_point(
    _ devicePtr: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double,
    _ focusModeRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("cinematic video focus requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let point = avcNormalizePoint(x, y) else {
        outErrorMessage?.pointee = ffiString("focus point must be normalized between 0.0 and 1.0")
        return AVC_INVALID_ARGUMENT
    }
    guard let focusMode = AVCaptureDevice.CinematicVideoFocusMode(rawValue: Int(focusModeRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported cinematic video focus mode: \(focusModeRaw)")
        return AVC_INVALID_ARGUMENT
    }
    guard device.responds(to: #selector(AVCaptureDevice.setCinematicVideoFixedFocus(at:focusMode:))) else {
        outErrorMessage?.pointee = ffiString("cinematic video fixed focus is not supported by this device")
        return AVC_OPERATION_FAILED
    }
    device.setCinematicVideoFixedFocus(at: point, focusMode: focusMode)
    return AVC_OK
}

@_cdecl("av_capture_device_set_camera_lens_smudge_detection")
public func av_capture_device_set_camera_lens_smudge_detection(
    _ devicePtr: UnsafeMutableRawPointer,
    _ enabled: Bool,
    _ hasInterval: Bool,
    _ interval: CMTime,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let device = avcDeviceBox(devicePtr).device
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("camera lens smudge detection requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard device.responds(to: #selector(AVCaptureDevice.setCameraLensSmudgeDetectionEnabled(_:detectionInterval:))) else {
        outErrorMessage?.pointee = ffiString("camera lens smudge detection is not supported by this device")
        return AVC_OPERATION_FAILED
    }
    if enabled && !device.activeFormat.isCameraLensSmudgeDetectionSupported {
        outErrorMessage?.pointee = ffiString("camera lens smudge detection is not supported by the active format")
        return AVC_DEVICE_ERROR
    }
    device.setCameraLensSmudgeDetectionEnabled(enabled, detectionInterval: hasInterval ? interval : .invalid)
    return AVC_OK
}

@_cdecl("av_capture_device_max_available_torch_level")
public func av_capture_device_max_available_torch_level() -> Float {
    guard #available(macOS 10.15, *) else {
        return Float.greatestFiniteMagnitude
    }
    return AVCaptureDevice.maxAvailableTorchLevel
}
