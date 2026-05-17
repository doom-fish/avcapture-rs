import AVFoundation
import CoreMedia
import Foundation

private struct TimecodeSourcePayload: Codable {
    let displayName: String
    let sourceType: String
    let uuid: String
}

private struct CaptureTimecodePayload: Codable {
    let hours: UInt8
    let minutes: UInt8
    let seconds: UInt8
    let frames: UInt8
    let userBits: UInt32
    let frameDuration: CMTimePayload
    let sourceType: String
}

private struct TimecodeGeneratorInfoPayload: Codable {
    let availableSourceCount: Int
    let currentSource: TimecodeSourcePayload?
    let synchronizationTimeout: Double
    let timecodeAlignmentOffset: Double
    let timecodeFrameDuration: CMTimePayload
    let delegateInstalled: Bool
}

private struct TimecodeGeneratorEventPayload: Codable {
    let kind: String
    let timecode: CaptureTimecodePayload?
    let source: TimecodeSourcePayload?
    let synchronizationStatus: String?
    let availableSources: [TimecodeSourcePayload]
}

@available(macOS 26.0, *)
final class TimecodeSourceBox: NSObject {
    let source: AVCaptureTimecode.Source

    init(_ source: AVCaptureTimecode.Source) {
        self.source = source
    }
}

@available(macOS 26.0, *)
private final class TimecodeGeneratorDelegateBridge: NSObject, AVCaptureTimecodeGeneratorDelegate {
    let callbackBox: AVCJsonCallbackBox

    init(callbackBox: AVCJsonCallbackBox) {
        self.callbackBox = callbackBox
    }

    func timecodeGenerator(
        _ generator: AVCaptureTimecodeGenerator,
        didReceiveUpdate timecode: AVCaptureTimecode,
        from source: AVCaptureTimecode.Source
    ) {
        callbackBox.emit(
            TimecodeGeneratorEventPayload(
                kind: "didReceiveUpdate",
                timecode: avcCaptureTimecodePayload(from: timecode),
                source: avcTimecodeSourcePayload(from: source),
                synchronizationStatus: nil,
                availableSources: []
            )
        )
    }

    func timecodeGenerator(
        _ generator: AVCaptureTimecodeGenerator,
        transitionedTo synchronizationStatus: AVCaptureTimecodeGenerator.SynchronizationStatus,
        for source: AVCaptureTimecode.Source
    ) {
        callbackBox.emit(
            TimecodeGeneratorEventPayload(
                kind: "transitionedToSynchronizationStatus",
                timecode: nil,
                source: avcTimecodeSourcePayload(from: source),
                synchronizationStatus: avcEncodeTimecodeSynchronizationStatus(synchronizationStatus),
                availableSources: []
            )
        )
    }

    func timecodeGenerator(
        _ generator: AVCaptureTimecodeGenerator,
        didUpdateAvailableSources availableSources: [AVCaptureTimecode.Source]
    ) {
        callbackBox.emit(
            TimecodeGeneratorEventPayload(
                kind: "didUpdateAvailableSources",
                timecode: nil,
                source: nil,
                synchronizationStatus: nil,
                availableSources: availableSources.map(avcTimecodeSourcePayload)
            )
        )
    }
}

@available(macOS 26.0, *)
final class TimecodeGeneratorBox: NSObject {
    let generator = AVCaptureTimecodeGenerator()
    fileprivate var callbackBox: AVCJsonCallbackBox?
    private var delegateBridge: TimecodeGeneratorDelegateBridge?
    private var callbackQueue: DispatchQueue?

    deinit {
        clearDelegate()
    }

    fileprivate func infoPayload() -> TimecodeGeneratorInfoPayload {
        TimecodeGeneratorInfoPayload(
            availableSourceCount: generator.availableSources.count,
            currentSource: avcTimecodeSourcePayload(from: generator.currentSource),
            synchronizationTimeout: generator.synchronizationTimeout,
            timecodeAlignmentOffset: generator.timecodeAlignmentOffset,
            timecodeFrameDuration: CMTimePayload(generator.timecodeFrameDuration),
            delegateInstalled: callbackBox != nil
        )
    }

    func setDelegate(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?,
        queueLabel: String
    ) {
        clearDelegate()
        let callbackBox = AVCJsonCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegateBridge = TimecodeGeneratorDelegateBridge(callbackBox: callbackBox)
        let queue = DispatchQueue(label: queueLabel)
        generator.setDelegate(delegateBridge, queue: queue)
        self.callbackBox = callbackBox
        self.delegateBridge = delegateBridge
        callbackQueue = queue
    }

    func clearDelegate() {
        generator.setDelegate(nil, queue: nil)
        delegateBridge = nil
        callbackQueue = nil
        callbackBox?.dispose()
        callbackBox = nil
    }
}

@available(macOS 26.0, *)
private func avcEncodeTimecodeSourceType(_ sourceType: AVCaptureTimecode.SourceType) -> String {
    switch sourceType {
    case .frameCount:
        return "frameCount"
    case .realTimeClock:
        return "realTimeClock"
    case .external:
        return "external"
    @unknown default:
        return "unknown"
    }
}

@available(macOS 26.0, *)
private func avcDecodeTimecodeSourceType(_ raw: String) -> AVCaptureTimecode.SourceType? {
    switch raw {
    case "frameCount":
        return .frameCount
    case "realTimeClock":
        return .realTimeClock
    case "external":
        return .external
    default:
        return nil
    }
}

@available(macOS 26.0, *)
private func avcEncodeTimecodeSynchronizationStatus(
    _ synchronizationStatus: AVCaptureTimecodeGenerator.SynchronizationStatus
) -> String {
    switch synchronizationStatus {
    case .unknown:
        return "unknown"
    case .sourceSelected:
        return "sourceSelected"
    case .synchronizing:
        return "synchronizing"
    case .synchronized:
        return "synchronized"
    case .timedOut:
        return "timedOut"
    case .sourceUnavailable:
        return "sourceUnavailable"
    case .sourceUnsupported:
        return "sourceUnsupported"
    case .notRequired:
        return "notRequired"
    @unknown default:
        return "unknown"
    }
}

private func avcCMTime(from payload: CMTimePayload) -> CMTime {
    CMTime(
        value: payload.value,
        timescale: payload.timescale,
        flags: CMTimeFlags(rawValue: payload.flags),
        epoch: payload.epoch
    )
}

@available(macOS 26.0, *)
private func avcTimecodeSourcePayload(from source: AVCaptureTimecode.Source) -> TimecodeSourcePayload {
    TimecodeSourcePayload(
        displayName: source.displayName,
        sourceType: avcEncodeTimecodeSourceType(source.type),
        uuid: source.uuid.uuidString
    )
}

@available(macOS 26.0, *)
private func avcCaptureTimecodePayload(from timecode: AVCaptureTimecode) -> CaptureTimecodePayload {
    CaptureTimecodePayload(
        hours: timecode.hours,
        minutes: timecode.minutes,
        seconds: timecode.seconds,
        frames: timecode.frames,
        userBits: timecode.userBits,
        frameDuration: CMTimePayload(timecode.frameDuration),
        sourceType: avcEncodeTimecodeSourceType(timecode.sourceType)
    )
}

@available(macOS 26.0, *)
private func avcCaptureTimecode(from payload: CaptureTimecodePayload) throws -> AVCaptureTimecode {
    guard let sourceType = avcDecodeTimecodeSourceType(payload.sourceType) else {
        throw BridgeError.message("unsupported timecode source type: \(payload.sourceType)")
    }
    return AVCaptureTimecode(
        hours: payload.hours,
        minutes: payload.minutes,
        seconds: payload.seconds,
        frames: payload.frames,
        userBits: payload.userBits,
        frameDuration: avcCMTime(from: payload.frameDuration),
        sourceType: sourceType
    )
}

@_cdecl("av_capture_timecode_generator_create")
public func av_capture_timecode_generator_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return nil
    }
    return avcRetain(TimecodeGeneratorBox())
}

@_cdecl("av_capture_timecode_generator_release")
public func av_capture_timecode_generator_release(_ generatorPtr: UnsafeMutableRawPointer?) {
    if #available(macOS 26.0, *) {
        avcRelease(generatorPtr, as: TimecodeGeneratorBox.self)
    }
}

@_cdecl("av_capture_timecode_generator_info_json")
public func av_capture_timecode_generator_info_json(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return nil
    }
    let generator = avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self)
    do {
        return ffiString(try avcEncodeJSON(generator.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_timecode_generator_available_sources_count")
public func av_capture_timecode_generator_available_sources_count(
    _ generatorPtr: UnsafeMutableRawPointer
) -> Int {
    if #available(macOS 26.0, *) {
        return avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.availableSources.count
    }
    return 0
}

@_cdecl("av_capture_timecode_generator_available_source_at_index")
public func av_capture_timecode_generator_available_source_at_index(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return nil
    }
    let sources = avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.availableSources
    guard sources.indices.contains(index) else {
        outErrorMessage?.pointee = ffiString("timecode source index out of range")
        return nil
    }
    return avcRetain(TimecodeSourceBox(sources[index]))
}

@_cdecl("av_capture_timecode_generator_current_source")
public func av_capture_timecode_generator_current_source(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return nil
    }
    let source = avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.currentSource
    return avcRetain(TimecodeSourceBox(source))
}

@_cdecl("av_capture_timecode_generator_set_synchronization_timeout")
public func av_capture_timecode_generator_set_synchronization_timeout(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ synchronizationTimeout: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard synchronizationTimeout.isFinite, synchronizationTimeout >= 0 else {
        outErrorMessage?.pointee = ffiString("timecode synchronization timeout must be finite and non-negative")
        return AVC_INVALID_ARGUMENT
    }
    avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.synchronizationTimeout = synchronizationTimeout
    return AVC_OK
}

@_cdecl("av_capture_timecode_generator_set_timecode_alignment_offset")
public func av_capture_timecode_generator_set_timecode_alignment_offset(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ timecodeAlignmentOffset: Double,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard timecodeAlignmentOffset.isFinite else {
        outErrorMessage?.pointee = ffiString("timecode alignment offset must be finite")
        return AVC_INVALID_ARGUMENT
    }
    avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.timecodeAlignmentOffset = timecodeAlignmentOffset
    return AVC_OK
}

@_cdecl("av_capture_timecode_generator_set_timecode_frame_duration_json")
public func av_capture_timecode_generator_set_timecode_frame_duration_json(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ frameDurationJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    do {
        let frameDuration = try avcDecodeJSON(frameDurationJson, as: CMTimePayload.self)
        avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.timecodeFrameDuration = avcCMTime(from: frameDuration)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_timecode_generator_start_synchronization")
public func av_capture_timecode_generator_start_synchronization(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ sourcePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.startSynchronization(
        source: avcUnretained(sourcePtr, as: TimecodeSourceBox.self).source
    )
    return AVC_OK
}

@_cdecl("av_capture_timecode_generator_generate_initial_timecode_json")
public func av_capture_timecode_generator_generate_initial_timecode_json(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return nil
    }
    do {
        let timecode = avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).generator.generateInitialTimecode()
        return ffiString(try avcEncodeJSON(avcCaptureTimecodePayload(from: timecode)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_timecode_generator_set_delegate_callback")
public func av_capture_timecode_generator_set_delegate_callback(
    _ generatorPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeGenerator requires macOS 26.0 or newer")
        return AVC_OPERATION_FAILED
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing timecode generator delegate callback")
        return AVC_CALLBACK_ERROR
    }
    let queueLabel = String(cString: queueLabelPtr)
    avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).setDelegate(
        callback: callback,
        userData: userData,
        dropUserData: dropUserData,
        queueLabel: queueLabel
    )
    return AVC_OK
}

@_cdecl("av_capture_timecode_generator_clear_delegate_callback")
public func av_capture_timecode_generator_clear_delegate_callback(_ generatorPtr: UnsafeMutableRawPointer) {
    if #available(macOS 26.0, *) {
        avcUnretained(generatorPtr, as: TimecodeGeneratorBox.self).clearDelegate()
    }
}

@_cdecl("av_capture_timecode_source_release")
public func av_capture_timecode_source_release(_ sourcePtr: UnsafeMutableRawPointer?) {
    if #available(macOS 26.0, *) {
        avcRelease(sourcePtr, as: TimecodeSourceBox.self)
    }
}

@_cdecl("av_capture_timecode_source_info_json")
public func av_capture_timecode_source_info_json(
    _ sourcePtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeSource requires macOS 26.0 or newer")
        return nil
    }
    do {
        return ffiString(try avcEncodeJSON(avcTimecodeSourcePayload(from: avcUnretained(sourcePtr, as: TimecodeSourceBox.self).source)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_timecode_source_frame_count")
public func av_capture_timecode_source_frame_count(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeSource requires macOS 26.0 or newer")
        return nil
    }
    return avcRetain(TimecodeSourceBox(AVCaptureTimecodeGenerator.frameCountSource))
}

@_cdecl("av_capture_timecode_source_real_time_clock")
public func av_capture_timecode_source_real_time_clock(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecodeSource requires macOS 26.0 or newer")
        return nil
    }
    return avcRetain(TimecodeSourceBox(AVCaptureTimecodeGenerator.realTimeClockSource))
}

@_cdecl("av_capture_timecode_advanced_by_frames_json")
public func av_capture_timecode_advanced_by_frames_json(
    _ timecodeJson: UnsafePointer<CChar>,
    _ framesToAdd: Int64,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecode requires macOS 26.0 or newer")
        return nil
    }
    do {
        let timecode = try avcCaptureTimecode(from: avcDecodeJSON(timecodeJson, as: CaptureTimecodePayload.self))
        let advanced = AVCaptureTimecode.advanced(timecode, by: framesToAdd)
        return ffiString(try avcEncodeJSON(avcCaptureTimecodePayload(from: advanced)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_timecode_create_metadata_sample_buffer_associated_with_presentation_time_stamp")
public func av_capture_timecode_create_metadata_sample_buffer_associated_with_presentation_time_stamp(
    _ timecodeJson: UnsafePointer<CChar>,
    _ presentationTimeStampJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecode metadata helpers require macOS 26.0 or newer")
        return nil
    }
    do {
        let timecode = try avcCaptureTimecode(from: avcDecodeJSON(timecodeJson, as: CaptureTimecodePayload.self))
        let presentationTimeStamp = try avcDecodeJSON(presentationTimeStampJson, as: CMTimePayload.self)
        guard let sampleBuffer = AVCaptureTimecode.createMetadataSampleBuffer(
            from: timecode,
            associatedWithPresentationTimeStamp: avcCMTime(from: presentationTimeStamp)
        ) else {
            outErrorMessage?.pointee = ffiString("failed to create timecode metadata sample buffer")
            return nil
        }
        return sampleBuffer.toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_timecode_create_metadata_sample_buffer_for_duration")
public func av_capture_timecode_create_metadata_sample_buffer_for_duration(
    _ timecodeJson: UnsafePointer<CChar>,
    _ durationJson: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("AVCaptureTimecode metadata helpers require macOS 26.0 or newer")
        return nil
    }
    do {
        let timecode = try avcCaptureTimecode(from: avcDecodeJSON(timecodeJson, as: CaptureTimecodePayload.self))
        let duration = try avcDecodeJSON(durationJson, as: CMTimePayload.self)
        guard let sampleBuffer = AVCaptureTimecode.createMetadataSampleBuffer(
            from: timecode,
            forDuration: avcCMTime(from: duration)
        ) else {
            outErrorMessage?.pointee = ffiString("failed to create timecode metadata sample buffer")
            return nil
        }
        return sampleBuffer.toOpaque()
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_timecode_metadata_sample_buffer_release")
public func av_capture_timecode_metadata_sample_buffer_release(_ sampleBufferPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 26.0, *) else { return }
    guard let sampleBufferPtr else { return }
    Unmanaged<CMSampleBuffer>.fromOpaque(sampleBufferPtr).release()
}
