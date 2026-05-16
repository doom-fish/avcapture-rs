import AVFoundation
import Foundation

struct PhotoOutputInfoPayload: Codable {
    let connectionCount: Int
    let availablePhotoCodecTypes: [String]
    let availablePhotoFileTypes: [String]
    let availablePhotoPixelFormatTypes: [UInt32]
    let availableRawPhotoPixelFormatTypes: [UInt32]
    let supportedFlashModes: [Int32]
    let maxPhotoDimensions: VideoDimensionsPayload?
    let captureReadiness: Int32?
    let highResolutionCaptureEnabled: Bool
    let responsiveCaptureEnabled: Bool?
    let callbackInstalled: Bool
}

struct PhotoCaptureResultPayload: Codable {
    let uniqueId: Int64
    let error: String?
}

private final class PhotoCaptureDelegate: NSObject, AVCapturePhotoCaptureDelegate {
    private weak var owner: PhotoOutputBox?

    init(owner: PhotoOutputBox) {
        self.owner = owner
    }

    func photoOutput(
        _ output: AVCapturePhotoOutput,
        didFinishCaptureFor resolvedSettings: AVCaptureResolvedPhotoSettings,
        error: Error?
    ) {
        owner?.completeCapture(uniqueId: Int64(resolvedSettings.uniqueID), error: error)
    }
}

final class PhotoOutputBox: CaptureOutputBoxBase {
    let photoOutput = AVCapturePhotoOutput()
    fileprivate var captureDelegate: PhotoCaptureDelegate?
    fileprivate var callbackBox: AVCJsonCallbackBox?

    override var output: AVCaptureOutput {
        photoOutput
    }

    deinit {
        clearCaptureState()
    }

    fileprivate func infoPayload() -> PhotoOutputInfoPayload {
        let supportedFlashModes: [Int32]
        if #available(macOS 13.0, *) {
            supportedFlashModes = photoOutput.supportedFlashModes.map { Int32($0.rawValue) }
        } else {
            supportedFlashModes = []
        }
        let maxPhotoDimensions: VideoDimensionsPayload?
        if #available(macOS 13.0, *) {
            let dimensions = photoOutput.maxPhotoDimensions
            if dimensions.width == 0 && dimensions.height == 0 {
                maxPhotoDimensions = nil
            } else {
                maxPhotoDimensions = VideoDimensionsPayload(dimensions)
            }
        } else {
            maxPhotoDimensions = nil
        }
        let captureReadiness: Int32?
        if #available(macOS 14.0, *) {
            captureReadiness = Int32(photoOutput.captureReadiness.rawValue)
        } else {
            captureReadiness = nil
        }
        let responsiveCaptureEnabled: Bool?
        if #available(macOS 14.0, *) {
            responsiveCaptureEnabled = photoOutput.isResponsiveCaptureEnabled
        } else {
            responsiveCaptureEnabled = nil
        }
        let availablePhotoCodecTypes = photoOutput.availablePhotoCodecTypes.map { $0.rawValue }
        let availablePhotoFileTypes = photoOutput.availablePhotoFileTypes.map { $0.rawValue }
        let availablePhotoPixelFormatTypes = photoOutput.availablePhotoPixelFormatTypes.map { UInt32($0) }
        let availableRawPhotoPixelFormatTypes = photoOutput.availableRawPhotoPixelFormatTypes.map { UInt32($0) }
        return PhotoOutputInfoPayload(
            connectionCount: photoOutput.connections.count,
            availablePhotoCodecTypes: availablePhotoCodecTypes,
            availablePhotoFileTypes: availablePhotoFileTypes,
            availablePhotoPixelFormatTypes: availablePhotoPixelFormatTypes,
            availableRawPhotoPixelFormatTypes: availableRawPhotoPixelFormatTypes,
            supportedFlashModes: supportedFlashModes,
            maxPhotoDimensions: maxPhotoDimensions,
            captureReadiness: captureReadiness,
            highResolutionCaptureEnabled: photoOutput.isHighResolutionCaptureEnabled,
            responsiveCaptureEnabled: responsiveCaptureEnabled,
            callbackInstalled: callbackBox != nil
        )
    }

    fileprivate func capturePhoto(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) throws {
        guard callbackBox == nil else {
            throw BridgeError.message("photo capture is already in progress")
        }
        guard !photoOutput.connections.isEmpty else {
            throw BridgeError.message("photo output is not attached to a session")
        }
        guard photoOutput.connection(with: .video) != nil || photoOutput.connection(with: .muxed) != nil else {
            throw BridgeError.message("photo output has no video-capable connection")
        }

        callbackBox = AVCJsonCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = PhotoCaptureDelegate(owner: self)
        captureDelegate = delegate
        photoOutput.capturePhoto(with: AVCapturePhotoSettings(), delegate: delegate)
    }

    fileprivate func completeCapture(uniqueId: Int64, error: Error?) {
        callbackBox?.emit(PhotoCaptureResultPayload(uniqueId: uniqueId, error: error?.localizedDescription))
        clearCaptureState()
    }

    fileprivate func clearCaptureState() {
        captureDelegate = nil
        callbackBox?.dispose()
        callbackBox = nil
    }
}

@_cdecl("av_capture_photo_output_create")
public func av_capture_photo_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(PhotoOutputBox())
}

@_cdecl("av_capture_photo_output_release")
public func av_capture_photo_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    avcRelease(outputPtr, as: PhotoOutputBox.self)
}

@_cdecl("av_capture_photo_output_info_json")
public func av_capture_photo_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcUnretained(outputPtr, as: PhotoOutputBox.self)
    do {
        return ffiString(try avcEncodeJSON(output.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_photo_output_set_high_resolution_capture_enabled")
public func av_capture_photo_output_set_high_resolution_capture_enabled(
    _ outputPtr: UnsafeMutableRawPointer,
    _ enabled: Bool
) {
    avcUnretained(outputPtr, as: PhotoOutputBox.self).photoOutput.isHighResolutionCaptureEnabled = enabled
}

@_cdecl("av_capture_photo_output_set_responsive_capture_enabled")
public func av_capture_photo_output_set_responsive_capture_enabled(
    _ outputPtr: UnsafeMutableRawPointer,
    _ enabled: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let photoOutput = avcUnretained(outputPtr, as: PhotoOutputBox.self).photoOutput
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("responsive capture requires macOS 14.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    if enabled && !photoOutput.isResponsiveCaptureSupported {
        outErrorMessage?.pointee = ffiString("responsive capture is not supported for the current session configuration")
        return AVC_OUTPUT_ERROR
    }
    photoOutput.isResponsiveCaptureEnabled = enabled
    return AVC_OK
}

@_cdecl("av_capture_photo_output_capture_photo")
public func av_capture_photo_output_capture_photo(
    _ outputPtr: UnsafeMutableRawPointer,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing photo capture callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: PhotoOutputBox.self)
    do {
        try output.capturePhoto(callback: callback, userData: userData, dropUserData: dropUserData)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_OUTPUT_ERROR
    }
}
