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
    let maxPhotoQualityPrioritization: Int32?
    let highResolutionCaptureEnabled: Bool
    let responsiveCaptureEnabled: Bool?
    let callbackInstalled: Bool
}

struct PhotoCaptureResultPayload: Codable {
    let uniqueId: Int64
    let error: String?
    let resolvedSettings: ResolvedPhotoSettingsInfoPayload
}

private struct PhotoOutputReadinessPayload: Codable {
    let captureReadiness: Int32
}

private final class PhotoCaptureDelegate: NSObject, AVCapturePhotoCaptureDelegate {
    private weak var owner: PhotoOutputBox?

    init(owner: PhotoOutputBox) {
        self.owner = owner
    }

    func photoOutput(
        _ output: AVCapturePhotoOutput,
        didFinishProcessingPhoto photo: AVCapturePhoto,
        error: Error?
    ) {
        owner?.recordProcessedPhoto(photo, error: error)
    }

    func photoOutput(
        _ output: AVCapturePhotoOutput,
        didFinishCaptureFor resolvedSettings: AVCaptureResolvedPhotoSettings,
        error: Error?
    ) {
        owner?.completeCapture(resolvedSettings: resolvedSettings, error: error)
    }
}

@available(macOS 14.0, *)
private final class PhotoOutputReadinessCoordinatorDelegateBox: NSObject,
    AVCapturePhotoOutputReadinessCoordinatorDelegate
{
    private weak var owner: PhotoOutputReadinessCoordinatorBox?

    init(owner: PhotoOutputReadinessCoordinatorBox) {
        self.owner = owner
    }

    func readinessCoordinator(
        _ coordinator: AVCapturePhotoOutputReadinessCoordinator,
        captureReadinessDidChange captureReadiness: AVCapturePhotoOutput.CaptureReadiness
    ) {
        owner?.emitCaptureReadiness(captureReadiness)
    }
}

final class PhotoOutputBox: CaptureOutputBoxBase {
    let photoOutput = AVCapturePhotoOutput()
    fileprivate var captureDelegate: PhotoCaptureDelegate?
    fileprivate var callbackBox: AVCPhotoCallbackBox?
    fileprivate var processedPhoto: AVCapturePhoto?
    fileprivate var processingError: Error?

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
        let maxPhotoQualityPrioritization: Int32?
        if #available(macOS 13.0, *) {
            maxPhotoQualityPrioritization = Int32(photoOutput.maxPhotoQualityPrioritization.rawValue)
        } else {
            maxPhotoQualityPrioritization = nil
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
            maxPhotoQualityPrioritization: maxPhotoQualityPrioritization,
            highResolutionCaptureEnabled: photoOutput.isHighResolutionCaptureEnabled,
            responsiveCaptureEnabled: responsiveCaptureEnabled,
            callbackInstalled: callbackBox != nil
        )
    }

    fileprivate func capturePhoto(
        settingsPtr: UnsafeMutableRawPointer,
        callback: @escaping AVCPhotoCallback,
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

        processedPhoto = nil
        processingError = nil
        callbackBox = AVCPhotoCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = PhotoCaptureDelegate(owner: self)
        captureDelegate = delegate
        photoOutput.capturePhoto(with: avcPhotoSettingsBox(settingsPtr).settings, delegate: delegate)
    }

    fileprivate func recordProcessedPhoto(_ photo: AVCapturePhoto, error: Error?) {
        processedPhoto = photo
        if let error {
            processingError = error
        }
    }

    fileprivate func completeCapture(
        resolvedSettings: AVCaptureResolvedPhotoSettings,
        error: Error?
    ) {
        callbackBox?.emit(
            processedPhoto,
            payload: PhotoCaptureResultPayload(
                uniqueId: Int64(resolvedSettings.uniqueID),
                error: (error ?? processingError)?.localizedDescription,
                resolvedSettings: resolvedPhotoSettingsInfoPayload(from: resolvedSettings)
            )
        )
        clearCaptureState()
    }

    fileprivate func clearCaptureState() {
        captureDelegate = nil
        processedPhoto = nil
        processingError = nil
        callbackBox?.dispose()
        callbackBox = nil
    }
}

@available(macOS 14.0, *)
final class PhotoOutputReadinessCoordinatorBox: NSObject {
    let coordinator: AVCapturePhotoOutputReadinessCoordinator
    fileprivate var delegateBox: PhotoOutputReadinessCoordinatorDelegateBox?
    fileprivate var callbackBox: AVCJsonCallbackBox?

    init(photoOutput: AVCapturePhotoOutput) {
        coordinator = AVCapturePhotoOutputReadinessCoordinator(photoOutput: photoOutput)
    }

    deinit {
        clearCallback()
    }

    fileprivate func captureReadinessRaw() -> Int32 {
        Int32(coordinator.captureReadiness.rawValue)
    }

    fileprivate func setCallback(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        clearCallback()
        let callbackBox = AVCJsonCallbackBox(
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        let delegate = PhotoOutputReadinessCoordinatorDelegateBox(owner: self)
        self.callbackBox = callbackBox
        delegateBox = delegate
        coordinator.delegate = delegate
    }

    fileprivate func clearCallback() {
        coordinator.delegate = nil
        delegateBox = nil
        callbackBox?.dispose()
        callbackBox = nil
    }

    fileprivate func emitCaptureReadiness(
        _ captureReadiness: AVCapturePhotoOutput.CaptureReadiness
    ) {
        callbackBox?.emit(PhotoOutputReadinessPayload(captureReadiness: Int32(captureReadiness.rawValue)))
    }

    fileprivate func startTrackingCaptureRequest(settingsPtr: UnsafeMutableRawPointer) {
        coordinator.startTrackingCaptureRequest(using: avcPhotoSettingsBox(settingsPtr).settings)
    }

    fileprivate func stopTrackingCaptureRequest(settingsUniqueID: Int64) {
        coordinator.stopTrackingCaptureRequest(using: settingsUniqueID)
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

@_cdecl("av_capture_photo_output_set_max_photo_quality_prioritization")
public func av_capture_photo_output_set_max_photo_quality_prioritization(
    _ outputPtr: UnsafeMutableRawPointer,
    _ prioritizationRaw: Int32,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let photoOutput = avcUnretained(outputPtr, as: PhotoOutputBox.self).photoOutput
    guard #available(macOS 13.0, *) else {
        outErrorMessage?.pointee = ffiString("photo quality prioritization requires macOS 13.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    guard let prioritization = AVCapturePhotoOutput.QualityPrioritization(rawValue: Int(prioritizationRaw)) else {
        outErrorMessage?.pointee = ffiString("unsupported photo quality prioritization: \(prioritizationRaw)")
        return AVC_INVALID_ARGUMENT
    }
    photoOutput.maxPhotoQualityPrioritization = prioritization
    return AVC_OK
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
    _ settingsPtr: UnsafeMutableRawPointer,
    _ callback: AVCPhotoCallback?,
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
        try output.capturePhoto(
            settingsPtr: settingsPtr,
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_OUTPUT_ERROR
    }
}

@_cdecl("av_capture_photo_output_readiness_coordinator_create")
public func av_capture_photo_output_readiness_coordinator_create(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires macOS 14.0 or newer")
        return nil
    }
    let output = avcUnretained(outputPtr, as: PhotoOutputBox.self).photoOutput
    guard !output.connections.isEmpty else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires a session-attached photo output")
        return nil
    }
    guard output.connection(with: .video) != nil || output.connection(with: .muxed) != nil else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires a video-capable connection")
        return nil
    }
    return avcRetain(PhotoOutputReadinessCoordinatorBox(photoOutput: output))
}

@_cdecl("av_capture_photo_output_readiness_coordinator_release")
public func av_capture_photo_output_readiness_coordinator_release(
    _ coordinatorPtr: UnsafeMutableRawPointer?
) {
    guard #available(macOS 14.0, *) else {
        return
    }
    avcRelease(coordinatorPtr, as: PhotoOutputReadinessCoordinatorBox.self)
}

@_cdecl("av_capture_photo_output_readiness_coordinator_capture_readiness")
public func av_capture_photo_output_readiness_coordinator_capture_readiness(
    _ coordinatorPtr: UnsafeMutableRawPointer,
    _ outReadiness: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires macOS 14.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    let coordinator = avcUnretained(coordinatorPtr, as: PhotoOutputReadinessCoordinatorBox.self)
    outReadiness.pointee = coordinator.captureReadinessRaw()
    return AVC_OK
}

@_cdecl("av_capture_photo_output_readiness_coordinator_set_callback")
public func av_capture_photo_output_readiness_coordinator_set_callback(
    _ coordinatorPtr: UnsafeMutableRawPointer,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires macOS 14.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing photo output readiness callback")
        return AVC_CALLBACK_ERROR
    }
    let coordinator = avcUnretained(coordinatorPtr, as: PhotoOutputReadinessCoordinatorBox.self)
    coordinator.setCallback(callback: callback, userData: userData, dropUserData: dropUserData)
    return AVC_OK
}

@_cdecl("av_capture_photo_output_readiness_coordinator_clear_callback")
public func av_capture_photo_output_readiness_coordinator_clear_callback(
    _ coordinatorPtr: UnsafeMutableRawPointer
) {
    guard #available(macOS 14.0, *) else {
        return
    }
    avcUnretained(coordinatorPtr, as: PhotoOutputReadinessCoordinatorBox.self).clearCallback()
}

@_cdecl("av_capture_photo_output_readiness_coordinator_start_tracking_capture_request")
public func av_capture_photo_output_readiness_coordinator_start_tracking_capture_request(
    _ coordinatorPtr: UnsafeMutableRawPointer,
    _ settingsPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires macOS 14.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    let coordinator = avcUnretained(coordinatorPtr, as: PhotoOutputReadinessCoordinatorBox.self)
    coordinator.startTrackingCaptureRequest(settingsPtr: settingsPtr)
    return AVC_OK
}

@_cdecl("av_capture_photo_output_readiness_coordinator_stop_tracking_capture_request")
public func av_capture_photo_output_readiness_coordinator_stop_tracking_capture_request(
    _ coordinatorPtr: UnsafeMutableRawPointer,
    _ settingsUniqueID: Int64,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 14.0, *) else {
        outErrorMessage?.pointee = ffiString("photo output readiness coordinator requires macOS 14.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    let coordinator = avcUnretained(coordinatorPtr, as: PhotoOutputReadinessCoordinatorBox.self)
    coordinator.stopTrackingCaptureRequest(settingsUniqueID: settingsUniqueID)
    return AVC_OK
}
