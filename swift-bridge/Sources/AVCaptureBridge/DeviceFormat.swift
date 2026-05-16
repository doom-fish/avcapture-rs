import AVFoundation
import CoreMedia
import Foundation

private struct FormatDescriptionInfoPayload: Codable {
    let mediaType: String
    let mediaTypeCode: UInt32
    let mediaSubtype: String
    let mediaSubtypeCode: UInt32
}

private struct FrameRateRangePayload: Codable {
    let minFrameRate: Double
    let maxFrameRate: Double
    let minFrameDuration: CMTimePayload
    let maxFrameDuration: CMTimePayload
}

private struct CaptureDeviceFormatInfoPayload: Codable {
    let mediaType: String
    let formatDescription: FormatDescriptionInfoPayload
    let videoSupportedFrameRateRanges: [FrameRateRangePayload]
    let highResolutionStillImageDimensions: VideoDimensionsPayload?
    let supportedMaxPhotoDimensions: [VideoDimensionsPayload]
}

private func avcFormatDescriptionPayload(_ description: CMFormatDescription) -> FormatDescriptionInfoPayload {
    let mediaTypeCode = UInt32(CMFormatDescriptionGetMediaType(description))
    let mediaSubtypeCode = UInt32(CMFormatDescriptionGetMediaSubType(description))
    return FormatDescriptionInfoPayload(
        mediaType: avcFourCCString(mediaTypeCode),
        mediaTypeCode: mediaTypeCode,
        mediaSubtype: avcFourCCString(mediaSubtypeCode),
        mediaSubtypeCode: mediaSubtypeCode
    )
}

private func avcCaptureDeviceFormatInfoPayload(from format: AVCaptureDevice.Format) -> CaptureDeviceFormatInfoPayload {
    let highResolutionStillImageDimensions: VideoDimensionsPayload? = nil
    let supportedMaxPhotoDimensions: [VideoDimensionsPayload]
    if #available(macOS 13.0, *) {
        supportedMaxPhotoDimensions = format.supportedMaxPhotoDimensions.map(VideoDimensionsPayload.init)
    } else {
        supportedMaxPhotoDimensions = []
    }
    return CaptureDeviceFormatInfoPayload(
        mediaType: avcEncodeMediaType(format.mediaType),
        formatDescription: avcFormatDescriptionPayload(format.formatDescription),
        videoSupportedFrameRateRanges: format.videoSupportedFrameRateRanges.map { range in
            FrameRateRangePayload(
                minFrameRate: range.minFrameRate,
                maxFrameRate: range.maxFrameRate,
                minFrameDuration: CMTimePayload(range.minFrameDuration),
                maxFrameDuration: CMTimePayload(range.maxFrameDuration)
            )
        },
        highResolutionStillImageDimensions: highResolutionStillImageDimensions,
        supportedMaxPhotoDimensions: supportedMaxPhotoDimensions
    )
}

@_cdecl("av_capture_device_format_release")
public func av_capture_device_format_release(_ formatPtr: UnsafeMutableRawPointer?) {
    avcRelease(formatPtr, as: DeviceFormatBox.self)
}

@_cdecl("av_capture_device_format_info_json")
public func av_capture_device_format_info_json(
    _ formatPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let format = avcDeviceFormatBox(formatPtr).format
    do {
        return ffiString(try avcEncodeJSON(avcCaptureDeviceFormatInfoPayload(from: format)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
