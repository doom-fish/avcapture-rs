import AVFoundation
import CoreMedia
import Foundation
import QuartzCore

private struct ExternalDisplaySupportInfoPayload: Codable {
    let shouldMatchFrameRateSupported: Bool
    let bypassColorSpaceConversionSupported: Bool
    let preferredResolutionSupported: Bool
}

private struct ExternalDisplayConfigurationInfoPayload: Codable {
    let shouldMatchFrameRate: Bool
    let bypassColorSpaceConversion: Bool
    let preferredResolution: VideoDimensionsPayload
}

private struct ExternalDisplayConfiguratorInfoPayload: Codable {
    let deviceAvailable: Bool
    let previewLayerAvailable: Bool
    let active: Bool
    let activeExternalDisplayFrameRate: Double
}

@available(macOS 26.0, *)
final class ExternalDisplayConfigurationBox: NSObject {
    let configuration = AVCaptureExternalDisplayConfiguration()
}

@available(macOS 26.0, *)
final class ExternalDisplayConfiguratorBox: NSObject {
    let configurator: AVCaptureExternalDisplayConfigurator

    init(
        device: AVCaptureDevice,
        previewLayer: CALayer,
        configuration: AVCaptureExternalDisplayConfiguration
    ) {
        configurator = AVCaptureExternalDisplayConfigurator(
            device: device,
            previewLayer: previewLayer,
            configuration: configuration
        )
    }
}

@available(macOS 26.0, *)
private func avcExternalDisplaySupportInfoPayload() -> ExternalDisplaySupportInfoPayload {
    ExternalDisplaySupportInfoPayload(
        shouldMatchFrameRateSupported: AVCaptureExternalDisplayConfigurator.isMatchingFrameRateSupported,
        bypassColorSpaceConversionSupported: AVCaptureExternalDisplayConfigurator.isBypassingColorSpaceConversionSupported,
        preferredResolutionSupported: AVCaptureExternalDisplayConfigurator.isPreferredResolutionSupported
    )
}

@available(macOS 26.0, *)
private func avcExternalDisplayConfigurationInfoPayload(
    from configuration: AVCaptureExternalDisplayConfiguration
) -> ExternalDisplayConfigurationInfoPayload {
    ExternalDisplayConfigurationInfoPayload(
        shouldMatchFrameRate: configuration.shouldMatchFrameRate,
        bypassColorSpaceConversion: configuration.bypassColorSpaceConversion,
        preferredResolution: VideoDimensionsPayload(configuration.preferredResolution)
    )
}

@available(macOS 26.0, *)
private func avcExternalDisplayConfiguratorInfoPayload(
    from configurator: AVCaptureExternalDisplayConfigurator
) -> ExternalDisplayConfiguratorInfoPayload {
    ExternalDisplayConfiguratorInfoPayload(
        deviceAvailable: configurator.device != nil,
        previewLayerAvailable: configurator.previewLayer != nil,
        active: configurator.isActive,
        activeExternalDisplayFrameRate: configurator.activeExternalDisplayFrameRate
    )
}

@_cdecl("av_capture_external_display_support_info_json")
public func av_capture_external_display_support_info_json(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureExternalDisplayConfigurator requires macOS 26.0 or newer"
        )
        return nil
    }
    do {
        return ffiString(try avcEncodeJSON(avcExternalDisplaySupportInfoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_external_display_configuration_create")
public func av_capture_external_display_configuration_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureExternalDisplayConfiguration requires macOS 26.0 or newer"
        )
        return nil
    }
    return avcRetain(ExternalDisplayConfigurationBox())
}

@_cdecl("av_capture_external_display_configuration_release")
public func av_capture_external_display_configuration_release(
    _ configurationPtr: UnsafeMutableRawPointer?
) {
    if #available(macOS 26.0, *) {
        avcRelease(configurationPtr, as: ExternalDisplayConfigurationBox.self)
    }
}

@_cdecl("av_capture_external_display_configuration_info_json")
public func av_capture_external_display_configuration_info_json(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureExternalDisplayConfiguration requires macOS 26.0 or newer"
        )
        return nil
    }
    let configuration = avcUnretained(configurationPtr, as: ExternalDisplayConfigurationBox.self).configuration
    do {
        return ffiString(try avcEncodeJSON(avcExternalDisplayConfigurationInfoPayload(from: configuration)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_external_display_configuration_set_should_match_frame_rate")
public func av_capture_external_display_configuration_set_should_match_frame_rate(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ shouldMatchFrameRate: Bool
) {
    if #available(macOS 26.0, *) {
        avcUnretained(
            configurationPtr,
            as: ExternalDisplayConfigurationBox.self
        ).configuration.shouldMatchFrameRate = shouldMatchFrameRate
    }
}

@_cdecl("av_capture_external_display_configuration_set_bypass_color_space_conversion")
public func av_capture_external_display_configuration_set_bypass_color_space_conversion(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ bypassColorSpaceConversion: Bool
) {
    if #available(macOS 26.0, *) {
        avcUnretained(
            configurationPtr,
            as: ExternalDisplayConfigurationBox.self
        ).configuration.bypassColorSpaceConversion = bypassColorSpaceConversion
    }
}

@_cdecl("av_capture_external_display_configuration_set_preferred_resolution_json")
public func av_capture_external_display_configuration_set_preferred_resolution_json(
    _ configurationPtr: UnsafeMutableRawPointer,
    _ preferredResolutionJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureExternalDisplayConfiguration requires macOS 26.0 or newer"
        )
        return AVC_OPERATION_FAILED
    }
    let configuration = avcUnretained(configurationPtr, as: ExternalDisplayConfigurationBox.self).configuration
    do {
        let preferredResolution = try avcDecodeJSON(preferredResolutionJson, as: VideoDimensionsPayload.self)
        configuration.preferredResolution = CMVideoDimensions(
            width: preferredResolution.width,
            height: preferredResolution.height
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_external_display_configurator_create")
public func av_capture_external_display_configurator_create(
    _ devicePtr: UnsafeMutableRawPointer,
    _ previewLayerPtr: UnsafeMutableRawPointer,
    _ configurationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureExternalDisplayConfigurator requires macOS 26.0 or newer"
        )
        return nil
    }
    let configuration = avcUnretained(configurationPtr, as: ExternalDisplayConfigurationBox.self).configuration
    if configuration.shouldMatchFrameRate && !AVCaptureExternalDisplayConfigurator.isMatchingFrameRateSupported {
        outErrorMessage?.pointee = ffiString("external display frame-rate matching is not supported on this runtime")
        return nil
    }
    if configuration.bypassColorSpaceConversion && !AVCaptureExternalDisplayConfigurator.isBypassingColorSpaceConversionSupported {
        outErrorMessage?.pointee = ffiString("external display color-space conversion bypass is not supported on this runtime")
        return nil
    }
    if (configuration.preferredResolution.width != 0 || configuration.preferredResolution.height != 0)
        && !AVCaptureExternalDisplayConfigurator.isPreferredResolutionSupported {
        outErrorMessage?.pointee = ffiString("external display preferred-resolution configuration is not supported on this runtime")
        return nil
    }
    return avcRetain(
        ExternalDisplayConfiguratorBox(
            device: avcDeviceBox(devicePtr).device,
            previewLayer: avcPreviewLayerBox(previewLayerPtr).layer,
            configuration: configuration
        )
    )
}

@_cdecl("av_capture_external_display_configurator_release")
public func av_capture_external_display_configurator_release(_ configuratorPtr: UnsafeMutableRawPointer?) {
    if #available(macOS 26.0, *) {
        avcRelease(configuratorPtr, as: ExternalDisplayConfiguratorBox.self)
    }
}

@_cdecl("av_capture_external_display_configurator_info_json")
public func av_capture_external_display_configurator_info_json(
    _ configuratorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureExternalDisplayConfigurator requires macOS 26.0 or newer"
        )
        return nil
    }
    let configurator = avcUnretained(configuratorPtr, as: ExternalDisplayConfiguratorBox.self).configurator
    do {
        return ffiString(try avcEncodeJSON(avcExternalDisplayConfiguratorInfoPayload(from: configurator)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_external_display_configurator_stop")
public func av_capture_external_display_configurator_stop(_ configuratorPtr: UnsafeMutableRawPointer) {
    if #available(macOS 26.0, *) {
        avcUnretained(configuratorPtr, as: ExternalDisplayConfiguratorBox.self).configurator.stop()
    }
}
