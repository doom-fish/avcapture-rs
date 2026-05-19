import AVFoundation
import CoreGraphics
import CoreMedia
import Foundation

private struct FormatDescriptionInfoPayload: Codable {
    let mediaType: String
    let mediaTypeCode: UInt32
    let mediaSubtype: String
    let mediaSubtypeCode: UInt32
}

private struct ExposureBiasRangePayload: Codable {
    let minExposureBias: Float
    let maxExposureBias: Float
}

private struct FrameRateRangePayload: Codable {
    let minFrameRate: Double
    let maxFrameRate: Double
    let minFrameDuration: CMTimePayload
    let maxFrameDuration: CMTimePayload
}

private struct ZoomRangePayload: Codable {
    let minZoomFactor: Double
    let maxZoomFactor: Double
}

private struct CaptureDeviceFormatInfoPayload: Codable {
    let mediaType: String
    let formatDescription: FormatDescriptionInfoPayload
    let videoSupportedFrameRateRanges: [FrameRateRangePayload]
    let videoFrameRateRangeForCenterStage: FrameRateRangePayload?
    let videoFrameRateRangeForPortraitEffect: FrameRateRangePayload?
    let videoFrameRateRangeForStudioLight: FrameRateRangePayload?
    let videoFrameRateRangeForReactionEffectsInProgress: FrameRateRangePayload?
    let videoFrameRateRangeForBackgroundReplacement: FrameRateRangePayload?
    let videoFrameRateRangeForCinematicVideo: FrameRateRangePayload?
    let highResolutionStillImageDimensions: VideoDimensionsPayload?
    let supportedMaxPhotoDimensions: [VideoDimensionsPayload]
    let systemRecommendedVideoZoomRange: ZoomRangePayload?
    let systemRecommendedExposureBiasRange: ExposureBiasRangePayload?
    let supportedVideoZoomRangesForDepthDataDelivery: [ZoomRangePayload]
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

private func avcFrameRateRangePayload(_ range: AVFrameRateRange) -> FrameRateRangePayload {
    FrameRateRangePayload(
        minFrameRate: range.minFrameRate,
        maxFrameRate: range.maxFrameRate,
        minFrameDuration: CMTimePayload(range.minFrameDuration),
        maxFrameDuration: CMTimePayload(range.maxFrameDuration)
    )
}

@available(macOS 15.0, *)
private func avcExposureBiasRangePayload(_ range: ClosedRange<Float>) -> ExposureBiasRangePayload {
    ExposureBiasRangePayload(
        minExposureBias: range.lowerBound,
        maxExposureBias: range.upperBound
    )
}

@available(macOS 14.2, *)
private func avcZoomRangePayload(_ range: ClosedRange<CGFloat>) -> ZoomRangePayload {
    ZoomRangePayload(
        minZoomFactor: Double(range.lowerBound),
        maxZoomFactor: Double(range.upperBound)
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

    let videoFrameRateRangeForCenterStage: FrameRateRangePayload?
    if #available(macOS 12.3, *) {
        videoFrameRateRangeForCenterStage = format.videoFrameRateRangeForCenterStage.map(avcFrameRateRangePayload)
    } else {
        videoFrameRateRangeForCenterStage = nil
    }

    let videoFrameRateRangeForPortraitEffect: FrameRateRangePayload?
    if #available(macOS 12.0, *) {
        videoFrameRateRangeForPortraitEffect = format.videoFrameRateRangeForPortraitEffect.map(avcFrameRateRangePayload)
    } else {
        videoFrameRateRangeForPortraitEffect = nil
    }

    let videoFrameRateRangeForStudioLight: FrameRateRangePayload?
    if #available(macOS 13.0, *) {
        videoFrameRateRangeForStudioLight = format.videoFrameRateRangeForStudioLight.map(avcFrameRateRangePayload)
    } else {
        videoFrameRateRangeForStudioLight = nil
    }

    let videoFrameRateRangeForReactionEffectsInProgress: FrameRateRangePayload?
    if #available(macOS 14.0, *) {
        videoFrameRateRangeForReactionEffectsInProgress = format.videoFrameRateRangeForReactionEffectsInProgress.map(avcFrameRateRangePayload)
    } else {
        videoFrameRateRangeForReactionEffectsInProgress = nil
    }

    let videoFrameRateRangeForBackgroundReplacement: FrameRateRangePayload?
    if #available(macOS 15.0, *) {
        videoFrameRateRangeForBackgroundReplacement = format.videoFrameRateRangeForBackgroundReplacement.map(avcFrameRateRangePayload)
    } else {
        videoFrameRateRangeForBackgroundReplacement = nil
    }

    let videoFrameRateRangeForCinematicVideo: FrameRateRangePayload?
    if #available(macOS 26.0, *) {
        videoFrameRateRangeForCinematicVideo = format.videoFrameRateRangeForCinematicVideo.map(avcFrameRateRangePayload)
    } else {
        videoFrameRateRangeForCinematicVideo = nil
    }

    let systemRecommendedVideoZoomRange: ZoomRangePayload?
    if #available(macOS 15.0, *) {
        systemRecommendedVideoZoomRange = format.systemRecommendedVideoZoomRange.map(avcZoomRangePayload)
    } else {
        systemRecommendedVideoZoomRange = nil
    }

    let systemRecommendedExposureBiasRange: ExposureBiasRangePayload?
    if #available(macOS 15.0, *) {
        systemRecommendedExposureBiasRange = format.systemRecommendedExposureBiasRange.map(avcExposureBiasRangePayload)
    } else {
        systemRecommendedExposureBiasRange = nil
    }

    let supportedVideoZoomRangesForDepthDataDelivery: [ZoomRangePayload]
    if #available(macOS 14.2, *) {
        supportedVideoZoomRangesForDepthDataDelivery = format.supportedVideoZoomRangesForDepthDataDelivery.map(avcZoomRangePayload)
    } else {
        supportedVideoZoomRangesForDepthDataDelivery = []
    }

    return CaptureDeviceFormatInfoPayload(
        mediaType: avcEncodeMediaType(format.mediaType),
        formatDescription: avcFormatDescriptionPayload(format.formatDescription),
        videoSupportedFrameRateRanges: format.videoSupportedFrameRateRanges.map(avcFrameRateRangePayload),
        videoFrameRateRangeForCenterStage: videoFrameRateRangeForCenterStage,
        videoFrameRateRangeForPortraitEffect: videoFrameRateRangeForPortraitEffect,
        videoFrameRateRangeForStudioLight: videoFrameRateRangeForStudioLight,
        videoFrameRateRangeForReactionEffectsInProgress: videoFrameRateRangeForReactionEffectsInProgress,
        videoFrameRateRangeForBackgroundReplacement: videoFrameRateRangeForBackgroundReplacement,
        videoFrameRateRangeForCinematicVideo: videoFrameRateRangeForCinematicVideo,
        highResolutionStillImageDimensions: highResolutionStillImageDimensions,
        supportedMaxPhotoDimensions: supportedMaxPhotoDimensions,
        systemRecommendedVideoZoomRange: systemRecommendedVideoZoomRange,
        systemRecommendedExposureBiasRange: systemRecommendedExposureBiasRange,
        supportedVideoZoomRangesForDepthDataDelivery: supportedVideoZoomRangesForDepthDataDelivery
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
