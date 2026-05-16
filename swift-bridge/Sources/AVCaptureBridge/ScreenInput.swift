import AVFoundation
import CoreGraphics
import Foundation

private struct ScreenInputInfoPayload: Codable {
    let displayId: UInt32
    let minFrameDuration: CMTimePayload
    let cropRect: CaptureRectPayload
    let scaleFactor: Double
    let capturesMouseClicks: Bool
    let capturesCursor: Bool
    let removesDuplicateFrames: Bool
}

final class ScreenInputBox: CaptureInputBoxBase {
    let screenInput: AVCaptureScreenInput
    let displayID: CGDirectDisplayID

    init?(displayID: CGDirectDisplayID) {
        guard let screenInput = AVCaptureScreenInput(displayID: displayID) else {
            return nil
        }
        self.screenInput = screenInput
        self.displayID = displayID
    }

    override var input: AVCaptureInput {
        screenInput
    }
}

private func avcScreenInputInfoPayload(from box: ScreenInputBox) -> ScreenInputInfoPayload {
    let input = box.screenInput
    return ScreenInputInfoPayload(
        displayId: box.displayID,
        minFrameDuration: CMTimePayload(input.minFrameDuration),
        cropRect: CaptureRectPayload(input.cropRect),
        scaleFactor: input.scaleFactor,
        capturesMouseClicks: input.capturesMouseClicks,
        capturesCursor: input.capturesCursor,
        removesDuplicateFrames: input.removesDuplicateFrames
    )
}

@_cdecl("av_capture_screen_input_create_main_display")
public func av_capture_screen_input_create_main_display(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let box = ScreenInputBox(displayID: CGMainDisplayID()) else {
        outErrorMessage?.pointee = ffiString("failed to create screen input for the main display")
        return nil
    }
    return avcRetain(box)
}

@_cdecl("av_capture_screen_input_create_with_display_id")
public func av_capture_screen_input_create_with_display_id(
    _ displayID: UInt32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let box = ScreenInputBox(displayID: displayID) else {
        outErrorMessage?.pointee = ffiString("failed to create screen input for display ID \(displayID)")
        return nil
    }
    return avcRetain(box)
}

@_cdecl("av_capture_screen_input_release")
public func av_capture_screen_input_release(_ inputPtr: UnsafeMutableRawPointer?) {
    avcRelease(inputPtr, as: ScreenInputBox.self)
}

@_cdecl("av_capture_screen_input_info_json")
public func av_capture_screen_input_info_json(
    _ inputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let input = avcUnretained(inputPtr, as: ScreenInputBox.self)
    do {
        return ffiString(try avcEncodeJSON(avcScreenInputInfoPayload(from: input)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_screen_input_set_min_frame_duration")
public func av_capture_screen_input_set_min_frame_duration(_ inputPtr: UnsafeMutableRawPointer, _ duration: CMTime) {
    avcUnretained(inputPtr, as: ScreenInputBox.self).screenInput.minFrameDuration = duration
}

@_cdecl("av_capture_screen_input_set_crop_rect")
public func av_capture_screen_input_set_crop_rect(
    _ inputPtr: UnsafeMutableRawPointer,
    _ x: Double,
    _ y: Double,
    _ width: Double,
    _ height: Double
) {
    avcUnretained(inputPtr, as: ScreenInputBox.self).screenInput.cropRect = CGRect(
        x: x,
        y: y,
        width: width,
        height: height
    )
}

@_cdecl("av_capture_screen_input_set_scale_factor")
public func av_capture_screen_input_set_scale_factor(_ inputPtr: UnsafeMutableRawPointer, _ scaleFactor: Double) {
    avcUnretained(inputPtr, as: ScreenInputBox.self).screenInput.scaleFactor = scaleFactor
}

@_cdecl("av_capture_screen_input_set_captures_mouse_clicks")
public func av_capture_screen_input_set_captures_mouse_clicks(_ inputPtr: UnsafeMutableRawPointer, _ enabled: Bool) {
    avcUnretained(inputPtr, as: ScreenInputBox.self).screenInput.capturesMouseClicks = enabled
}

@_cdecl("av_capture_screen_input_set_captures_cursor")
public func av_capture_screen_input_set_captures_cursor(_ inputPtr: UnsafeMutableRawPointer, _ enabled: Bool) {
    avcUnretained(inputPtr, as: ScreenInputBox.self).screenInput.capturesCursor = enabled
}

@_cdecl("av_capture_screen_input_set_removes_duplicate_frames")
public func av_capture_screen_input_set_removes_duplicate_frames(_ inputPtr: UnsafeMutableRawPointer, _ enabled: Bool) {
    avcUnretained(inputPtr, as: ScreenInputBox.self).screenInput.removesDuplicateFrames = enabled
}
