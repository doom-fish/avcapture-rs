import AVFoundation
import CoreGraphics
import Foundation

private struct DeskViewApplicationInfoPayload: Codable {
    let runtimeSupported: Bool
}

private struct DeskViewApplicationLaunchConfigurationInfoPayload: Codable {
    let mainWindowFrame: CaptureRectPayload
    let requiresSetUpModeCompletion: Bool
}

private struct DeskViewApplicationCompletionPayload: Codable {
    let error: String?
}

@available(macOS 13.0, *)
final class DeskViewApplicationBox: NSObject {
    let application = AVCaptureDeskViewApplication()
}

@available(macOS 13.0, *)
final class DeskViewApplicationLaunchConfigurationBox: NSObject {
    let configuration = AVCaptureDeskViewApplication.LaunchConfiguration()
}

@_cdecl("av_capture_desk_view_application_create")
public func av_capture_desk_view_application_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureDeskViewApplication requires macOS 13.0 or newer")
        return nil
    }
    return avcRetain(DeskViewApplicationBox())
}

@_cdecl("av_capture_desk_view_application_release")
public func av_capture_desk_view_application_release(_ applicationPtr: UnsafeMutableRawPointer?) {
    if #available(macOS 13.0, *) {
        avcRelease(applicationPtr, as: DeskViewApplicationBox.self)
    }
}

@_cdecl("av_capture_desk_view_application_info_json")
public func av_capture_desk_view_application_info_json(
    _ applicationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureDeskViewApplication requires macOS 13.0 or newer")
        return nil
    }
    _ = avcUnretained(applicationPtr, as: DeskViewApplicationBox.self)
    do {
        return ffiString(try avcEncodeJSON(DeskViewApplicationInfoPayload(runtimeSupported: true)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_desk_view_application_present")
public func av_capture_desk_view_application_present(
    _ applicationPtr: UnsafeMutableRawPointer,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureDeskViewApplication requires macOS 13.0 or newer")
        return AVC_OPERATION_FAILED
    }
    let application = avcUnretained(applicationPtr, as: DeskViewApplicationBox.self).application
    let callbackBox = callback.map { AVCJsonCallbackBox(callback: $0, userData: userData, dropUserData: dropUserData) }
    application.present { error in
        guard let callbackBox else { return }
        callbackBox.emit(DeskViewApplicationCompletionPayload(error: error?.localizedDescription))
        callbackBox.dispose()
    }
    return AVC_OK
}

@_cdecl("av_capture_desk_view_application_present_with_launch_configuration")
public func av_capture_desk_view_application_present_with_launch_configuration(
    _ applicationPtr: UnsafeMutableRawPointer,
    _ launchConfigurationPtr: UnsafeMutableRawPointer,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureDeskViewApplication requires macOS 13.0 or newer")
        return AVC_OPERATION_FAILED
    }
    let application = avcUnretained(applicationPtr, as: DeskViewApplicationBox.self).application
    let launchConfiguration = avcUnretained(
        launchConfigurationPtr,
        as: DeskViewApplicationLaunchConfigurationBox.self
    ).configuration
    let callbackBox = callback.map { AVCJsonCallbackBox(callback: $0, userData: userData, dropUserData: dropUserData) }
    application.present(launchConfiguration: launchConfiguration) { error in
        guard let callbackBox else { return }
        callbackBox.emit(DeskViewApplicationCompletionPayload(error: error?.localizedDescription))
        callbackBox.dispose()
    }
    return AVC_OK
}

@_cdecl("av_capture_desk_view_application_launch_configuration_create")
public func av_capture_desk_view_application_launch_configuration_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureDeskViewApplicationLaunchConfiguration requires macOS 13.0 or newer"
        )
        return nil
    }
    return avcRetain(DeskViewApplicationLaunchConfigurationBox())
}

@_cdecl("av_capture_desk_view_application_launch_configuration_release")
public func av_capture_desk_view_application_launch_configuration_release(
    _ launchConfigurationPtr: UnsafeMutableRawPointer?
) {
    if #available(macOS 13.0, *) {
        avcRelease(launchConfigurationPtr, as: DeskViewApplicationLaunchConfigurationBox.self)
    }
}

@_cdecl("av_capture_desk_view_application_launch_configuration_info_json")
public func av_capture_desk_view_application_launch_configuration_info_json(
    _ launchConfigurationPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureDeskViewApplicationLaunchConfiguration requires macOS 13.0 or newer"
        )
        return nil
    }
    let configuration = avcUnretained(
        launchConfigurationPtr,
        as: DeskViewApplicationLaunchConfigurationBox.self
    ).configuration
    do {
        return ffiString(
            try avcEncodeJSON(
                DeskViewApplicationLaunchConfigurationInfoPayload(
                    mainWindowFrame: CaptureRectPayload(configuration.mainWindowFrame),
                    requiresSetUpModeCompletion: configuration.requiresSetUpModeCompletion
                )
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_desk_view_application_launch_configuration_set_main_window_frame_json")
public func av_capture_desk_view_application_launch_configuration_set_main_window_frame_json(
    _ launchConfigurationPtr: UnsafeMutableRawPointer,
    _ frameJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString(
            "AVCaptureDeskViewApplicationLaunchConfiguration requires macOS 13.0 or newer"
        )
        return AVC_OPERATION_FAILED
    }
    let configuration = avcUnretained(
        launchConfigurationPtr,
        as: DeskViewApplicationLaunchConfigurationBox.self
    ).configuration
    do {
        let frame = try avcDecodeJSON(frameJson, as: CaptureRectPayload.self)
        configuration.mainWindowFrame = CGRect(
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_desk_view_application_launch_configuration_set_requires_setup_mode_completion")
public func av_capture_desk_view_application_launch_configuration_set_requires_setup_mode_completion(
    _ launchConfigurationPtr: UnsafeMutableRawPointer,
    _ requiresSetUpModeCompletion: Bool
) {
    if #available(macOS 13.0, *) {
        avcUnretained(
            launchConfigurationPtr,
            as: DeskViewApplicationLaunchConfigurationBox.self
        ).configuration.requiresSetUpModeCompletion = requiresSetUpModeCompletion
    }
}
