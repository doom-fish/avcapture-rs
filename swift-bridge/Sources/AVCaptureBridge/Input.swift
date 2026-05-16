import AVFoundation
import Foundation

private func avcInputInfoPayload(from input: AVCaptureInput) -> CaptureInputInfoPayload {
    CaptureInputInfoPayload(
        ports: input.ports.map { port in
            CaptureInputPortInfoPayload(
                mediaType: avcEncodeMediaType(port.mediaType),
                enabled: port.isEnabled,
                hasFormatDescription: port.formatDescription != nil
            )
        }
    )
}

@_cdecl("av_capture_input_info_json")
public func av_capture_input_info_json(
    _ inputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let input = avcInputBox(inputPtr).input
    do {
        return ffiString(try avcEncodeJSON(avcInputInfoPayload(from: input)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
