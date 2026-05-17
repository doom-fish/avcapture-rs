import AVFoundation
import Foundation

struct MovieFileOutputInfoPayload: Codable {
    let connectionCount: Int
    let isRecording: Bool
    let isRecordingPaused: Bool
    let outputFileURL: String?
    let recordedDuration: CMTimePayload
    let recordedFileSize: Int64
    let maxRecordedDuration: CMTimePayload
    let maxRecordedFileSize: Int64
    let minFreeDiskSpaceLimit: Int64
    let movieFragmentInterval: CMTimePayload
    let metadataCount: Int
    let spatialVideoCaptureEnabled: Bool?
    let callbackInstalled: Bool
    let sampleBufferBoundaryCallbackInstalled: Bool
}

struct AudioFileOutputInfoPayload: Codable {
    let connectionCount: Int
    let isRecording: Bool
    let isRecordingPaused: Bool
    let outputFileURL: String?
    let recordedDuration: CMTimePayload
    let recordedFileSize: Int64
    let maxRecordedDuration: CMTimePayload
    let maxRecordedFileSize: Int64
    let minFreeDiskSpaceLimit: Int64
    let metadataCount: Int
    let availableOutputFileTypes: [String]
    let audioSettings: AudioOutputSettingsPayload?
    let callbackInstalled: Bool
    let sampleBufferBoundaryCallbackInstalled: Bool
}

private struct FileRecordingEventPayload: Codable {
    let kind: String
    let fileURL: String
    let error: String?
}

private final class FileOutputSampleBufferCallbackBox {
    let callback: AVCAudioSampleCallback
    let userData: UnsafeMutableRawPointer?
    let dropUserData: AVCDropCallback?
    private var disposed = false

    init(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
    }

    func emit(sampleBuffer: CMSampleBuffer) {
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(userData, sampleOpaque)
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }
}

private final class FileOutputBoundaryDelegate: NSObject, AVCaptureFileOutputDelegate {
    private let onSampleBuffer: (CMSampleBuffer) -> Void

    init(onSampleBuffer: @escaping (CMSampleBuffer) -> Void) {
        self.onSampleBuffer = onSampleBuffer
    }

    func fileOutputShouldProvideSampleAccurateRecordingStart(_ output: AVCaptureFileOutput) -> Bool {
        true
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didOutputSampleBuffer sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        onSampleBuffer(sampleBuffer)
    }
}

private protocol FileRecordingEventOwner: AnyObject {
    func emitEvent(kind: String, fileURL: URL, error: Error?)
    func finishRecording()
}

private final class FileRecordingDelegate<Owner: FileRecordingEventOwner>: NSObject, AVCaptureFileOutputRecordingDelegate {
    private weak var owner: Owner?

    init(owner: Owner) {
        self.owner = owner
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didStartRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        owner?.emitEvent(kind: "started", fileURL: fileURL, error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didPauseRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        owner?.emitEvent(kind: "paused", fileURL: fileURL, error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didResumeRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        owner?.emitEvent(kind: "resumed", fileURL: fileURL, error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        willFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        owner?.emitEvent(kind: "willFinish", fileURL: outputFileURL, error: error)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        owner?.emitEvent(kind: "finished", fileURL: outputFileURL, error: error)
        owner?.finishRecording()
    }
}

private func avcRecordingFilePath(from outputFileURL: URL?) -> String? {
    let path = outputFileURL?.path
    guard let path, !path.isEmpty, path != "/" else { return nil }
    return path
}

private func avcPrepareRecordingURL(_ outputPath: String) throws -> URL {
    let url = URL(fileURLWithPath: outputPath)
    let parentDirectory = url.deletingLastPathComponent()
    if !parentDirectory.path.isEmpty {
        try FileManager.default.createDirectory(at: parentDirectory, withIntermediateDirectories: true)
    }
    if FileManager.default.fileExists(atPath: url.path) {
        try FileManager.default.removeItem(at: url)
    }
    return url
}

final class MovieFileOutputBox: CaptureOutputBoxBase, FileRecordingEventOwner {
    let movieOutput = AVCaptureMovieFileOutput()
    fileprivate var recordingDelegate: FileRecordingDelegate<MovieFileOutputBox>?
    fileprivate var callbackBox: AVCJsonCallbackBox?
    private var sampleBufferBoundaryDelegate: FileOutputBoundaryDelegate?
    private var sampleBufferBoundaryCallbackBox: FileOutputSampleBufferCallbackBox?

    override var output: AVCaptureOutput {
        movieOutput
    }

    deinit {
        clearRecordingState()
        clearSampleBufferBoundaryCallback()
    }

    fileprivate func infoPayload() -> MovieFileOutputInfoPayload {
        let spatialVideoCaptureEnabled: Bool?
        if #available(macOS 15.0, *) {
            spatialVideoCaptureEnabled = movieOutput.isSpatialVideoCaptureEnabled
        } else {
            spatialVideoCaptureEnabled = nil
        }
        return MovieFileOutputInfoPayload(
            connectionCount: movieOutput.connections.count,
            isRecording: movieOutput.isRecording,
            isRecordingPaused: movieOutput.isRecordingPaused,
            outputFileURL: avcRecordingFilePath(from: movieOutput.outputFileURL),
            recordedDuration: CMTimePayload(movieOutput.recordedDuration),
            recordedFileSize: movieOutput.recordedFileSize,
            maxRecordedDuration: CMTimePayload(movieOutput.maxRecordedDuration),
            maxRecordedFileSize: movieOutput.maxRecordedFileSize,
            minFreeDiskSpaceLimit: movieOutput.minFreeDiskSpaceLimit,
            movieFragmentInterval: CMTimePayload(movieOutput.movieFragmentInterval),
            metadataCount: movieOutput.metadata?.count ?? 0,
            spatialVideoCaptureEnabled: spatialVideoCaptureEnabled,
            callbackInstalled: callbackBox != nil,
            sampleBufferBoundaryCallbackInstalled: sampleBufferBoundaryCallbackBox != nil
        )
    }

    func startRecording(
        to outputPath: String,
        callback: AVCJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) throws {
        guard !movieOutput.isRecording else {
            throw BridgeError.message("movie file output is already recording")
        }
        guard !movieOutput.connections.isEmpty else {
            throw BridgeError.message("movie file output is not attached to a session")
        }

        let url = try avcPrepareRecordingURL(outputPath)
        clearRecordingState()
        if let callback {
            callbackBox = AVCJsonCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        }
        let delegate = FileRecordingDelegate(owner: self)
        recordingDelegate = delegate
        movieOutput.startRecording(to: url, recordingDelegate: delegate)
    }

    func emitEvent(kind: String, fileURL: URL, error: Error?) {
        callbackBox?.emit(FileRecordingEventPayload(
            kind: kind,
            fileURL: fileURL.path,
            error: error?.localizedDescription
        ))
    }

    func finishRecording() {
        clearRecordingState()
    }

    func clearRecordingState() {
        recordingDelegate = nil
        callbackBox?.dispose()
        callbackBox = nil
    }

    func setSampleBufferBoundaryCallback(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        clearSampleBufferBoundaryCallback()
        let box = FileOutputSampleBufferCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = FileOutputBoundaryDelegate { sampleBuffer in
            box.emit(sampleBuffer: sampleBuffer)
        }
        movieOutput.delegate = delegate
        sampleBufferBoundaryDelegate = delegate
        sampleBufferBoundaryCallbackBox = box
    }

    func clearSampleBufferBoundaryCallback() {
        movieOutput.delegate = nil
        sampleBufferBoundaryDelegate = nil
        sampleBufferBoundaryCallbackBox?.dispose()
        sampleBufferBoundaryCallbackBox = nil
    }
}

final class AudioFileOutputBox: CaptureOutputBoxBase, FileRecordingEventOwner {
    let audioOutput = AVCaptureAudioFileOutput()
    fileprivate var recordingDelegate: FileRecordingDelegate<AudioFileOutputBox>?
    fileprivate var callbackBox: AVCJsonCallbackBox?
    private var sampleBufferBoundaryDelegate: FileOutputBoundaryDelegate?
    private var sampleBufferBoundaryCallbackBox: FileOutputSampleBufferCallbackBox?

    override var output: AVCaptureOutput {
        audioOutput
    }

    deinit {
        clearRecordingState()
        clearSampleBufferBoundaryCallback()
    }

    fileprivate func infoPayload() -> AudioFileOutputInfoPayload {
        AudioFileOutputInfoPayload(
            connectionCount: audioOutput.connections.count,
            isRecording: audioOutput.isRecording,
            isRecordingPaused: audioOutput.isRecordingPaused,
            outputFileURL: avcRecordingFilePath(from: audioOutput.outputFileURL),
            recordedDuration: CMTimePayload(audioOutput.recordedDuration),
            recordedFileSize: audioOutput.recordedFileSize,
            maxRecordedDuration: CMTimePayload(audioOutput.maxRecordedDuration),
            maxRecordedFileSize: audioOutput.maxRecordedFileSize,
            minFreeDiskSpaceLimit: audioOutput.minFreeDiskSpaceLimit,
            metadataCount: audioOutput.metadata.count,
            availableOutputFileTypes: AVCaptureAudioFileOutput.availableOutputFileTypes().map(\.rawValue),
            audioSettings: avcEncodeAudioSettings(audioOutput.audioSettings),
            callbackInstalled: callbackBox != nil,
            sampleBufferBoundaryCallbackInstalled: sampleBufferBoundaryCallbackBox != nil
        )
    }

    func startRecording(
        to outputPath: String,
        outputFileType rawOutputFileType: String,
        callback: AVCJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) throws {
        guard !audioOutput.isRecording else {
            throw BridgeError.message("audio file output is already recording")
        }
        guard !audioOutput.connections.isEmpty else {
            throw BridgeError.message("audio file output is not attached to a session")
        }

        let outputFileType = AVFileType(rawValue: rawOutputFileType)
        guard AVCaptureAudioFileOutput.availableOutputFileTypes().contains(outputFileType) else {
            throw BridgeError.message("unsupported audio file output type: \(rawOutputFileType)")
        }

        let url = try avcPrepareRecordingURL(outputPath)
        clearRecordingState()
        if let callback {
            callbackBox = AVCJsonCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        }
        let delegate = FileRecordingDelegate(owner: self)
        recordingDelegate = delegate
        audioOutput.startRecording(to: url, outputFileType: outputFileType, recordingDelegate: delegate)
    }

    func emitEvent(kind: String, fileURL: URL, error: Error?) {
        callbackBox?.emit(FileRecordingEventPayload(
            kind: kind,
            fileURL: fileURL.path,
            error: error?.localizedDescription
        ))
    }

    func finishRecording() {
        clearRecordingState()
    }

    func clearRecordingState() {
        recordingDelegate = nil
        callbackBox?.dispose()
        callbackBox = nil
    }

    func setSampleBufferBoundaryCallback(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        clearSampleBufferBoundaryCallback()
        let box = FileOutputSampleBufferCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        let delegate = FileOutputBoundaryDelegate { sampleBuffer in
            box.emit(sampleBuffer: sampleBuffer)
        }
        audioOutput.delegate = delegate
        sampleBufferBoundaryDelegate = delegate
        sampleBufferBoundaryCallbackBox = box
    }

    func clearSampleBufferBoundaryCallback() {
        audioOutput.delegate = nil
        sampleBufferBoundaryDelegate = nil
        sampleBufferBoundaryCallbackBox?.dispose()
        sampleBufferBoundaryCallbackBox = nil
    }
}

@_cdecl("av_capture_movie_file_output_create")
public func av_capture_movie_file_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(MovieFileOutputBox())
}

@_cdecl("av_capture_movie_file_output_release")
public func av_capture_movie_file_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    avcRelease(outputPtr, as: MovieFileOutputBox.self)
}

@_cdecl("av_capture_movie_file_output_info_json")
public func av_capture_movie_file_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        return ffiString(try avcEncodeJSON(output.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_movie_file_output_start_recording")
public func av_capture_movie_file_output_start_recording(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outputPathPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    let outputPath = String(cString: outputPathPtr)
    do {
        try output.startRecording(to: outputPath, callback: callback, userData: userData, dropUserData: dropUserData)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_OUTPUT_ERROR
    }
}

@_cdecl("av_capture_movie_file_output_set_sample_buffer_boundary_callback")
public func av_capture_movie_file_output_set_sample_buffer_boundary_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ callback: AVCAudioSampleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing movie file output sample-buffer callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    output.setSampleBufferBoundaryCallback(callback: callback, userData: userData, dropUserData: dropUserData)
    return AVC_OK
}

@_cdecl("av_capture_movie_file_output_clear_sample_buffer_boundary_callback")
public func av_capture_movie_file_output_clear_sample_buffer_boundary_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).clearSampleBufferBoundaryCallback()
}

@_cdecl("av_capture_movie_file_output_stop_recording")
public func av_capture_movie_file_output_stop_recording(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput.stopRecording()
}

@_cdecl("av_capture_movie_file_output_pause_recording")
public func av_capture_movie_file_output_pause_recording(_ outputPtr: UnsafeMutableRawPointer) {
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput
    if output.isRecording {
        output.pauseRecording()
    }
}

@_cdecl("av_capture_movie_file_output_resume_recording")
public func av_capture_movie_file_output_resume_recording(_ outputPtr: UnsafeMutableRawPointer) {
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput
    if output.isRecording {
        output.resumeRecording()
    }
}

@_cdecl("av_capture_movie_file_output_set_max_recorded_duration")
public func av_capture_movie_file_output_set_max_recorded_duration(_ outputPtr: UnsafeMutableRawPointer, _ duration: CMTime) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput.maxRecordedDuration = duration
}

@_cdecl("av_capture_movie_file_output_set_max_recorded_file_size")
public func av_capture_movie_file_output_set_max_recorded_file_size(_ outputPtr: UnsafeMutableRawPointer, _ bytes: Int64) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput.maxRecordedFileSize = bytes
}

@_cdecl("av_capture_movie_file_output_set_min_free_disk_space_limit")
public func av_capture_movie_file_output_set_min_free_disk_space_limit(_ outputPtr: UnsafeMutableRawPointer, _ bytes: Int64) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput.minFreeDiskSpaceLimit = bytes
}

@_cdecl("av_capture_movie_file_output_set_movie_fragment_interval")
public func av_capture_movie_file_output_set_movie_fragment_interval(_ outputPtr: UnsafeMutableRawPointer, _ interval: CMTime) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput.movieFragmentInterval = interval
}

@_cdecl("av_capture_movie_file_output_set_spatial_video_capture_enabled")
public func av_capture_movie_file_output_set_spatial_video_capture_enabled(
    _ outputPtr: UnsafeMutableRawPointer,
    _ enabled: Bool,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let movieOutput = avcUnretained(outputPtr, as: MovieFileOutputBox.self).movieOutput
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("spatial video capture requires macOS 15.0 or newer")
        return AVC_OUTPUT_ERROR
    }
    if enabled && !movieOutput.isSpatialVideoCaptureSupported {
        outErrorMessage?.pointee = ffiString("spatial video capture is not supported for the current session configuration")
        return AVC_OUTPUT_ERROR
    }
    movieOutput.isSpatialVideoCaptureEnabled = enabled
    return AVC_OK
}

@_cdecl("av_capture_audio_file_output_create")
public func av_capture_audio_file_output_create(
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    avcRetain(AudioFileOutputBox())
}

@_cdecl("av_capture_audio_file_output_release")
public func av_capture_audio_file_output_release(_ outputPtr: UnsafeMutableRawPointer?) {
    avcRelease(outputPtr, as: AudioFileOutputBox.self)
}

@_cdecl("av_capture_audio_file_output_info_json")
public func av_capture_audio_file_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        return ffiString(try avcEncodeJSON(output.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_audio_file_output_set_audio_settings_json")
public func av_capture_audio_file_output_set_audio_settings_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ settingsJson: UnsafePointer<CChar>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        guard let settingsJson else {
            output.audioOutput.audioSettings = nil
            return AVC_OK
        }
        let payload = try avcDecodeJSON(settingsJson, as: AudioOutputSettingsPayload.self)
        output.audioOutput.audioSettings = avcAudioSettingsDictionary(from: payload)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_audio_file_output_start_recording")
public func av_capture_audio_file_output_start_recording(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outputPathPtr: UnsafePointer<CChar>,
    _ outputFileTypePtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    let outputPath = String(cString: outputPathPtr)
    let outputFileType = String(cString: outputFileTypePtr)
    do {
        try output.startRecording(
            to: outputPath,
            outputFileType: outputFileType,
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

@_cdecl("av_capture_audio_file_output_set_sample_buffer_boundary_callback")
public func av_capture_audio_file_output_set_sample_buffer_boundary_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ callback: AVCAudioSampleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing audio file output sample-buffer callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    output.setSampleBufferBoundaryCallback(callback: callback, userData: userData, dropUserData: dropUserData)
    return AVC_OK
}

@_cdecl("av_capture_audio_file_output_clear_sample_buffer_boundary_callback")
public func av_capture_audio_file_output_clear_sample_buffer_boundary_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).clearSampleBufferBoundaryCallback()
}

@_cdecl("av_capture_audio_file_output_stop_recording")
public func av_capture_audio_file_output_stop_recording(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).audioOutput.stopRecording()
}

@_cdecl("av_capture_audio_file_output_pause_recording")
public func av_capture_audio_file_output_pause_recording(_ outputPtr: UnsafeMutableRawPointer) {
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self).audioOutput
    if output.isRecording {
        output.pauseRecording()
    }
}

@_cdecl("av_capture_audio_file_output_resume_recording")
public func av_capture_audio_file_output_resume_recording(_ outputPtr: UnsafeMutableRawPointer) {
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self).audioOutput
    if output.isRecording {
        output.resumeRecording()
    }
}

@_cdecl("av_capture_audio_file_output_set_max_recorded_duration")
public func av_capture_audio_file_output_set_max_recorded_duration(_ outputPtr: UnsafeMutableRawPointer, _ duration: CMTime) {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).audioOutput.maxRecordedDuration = duration
}

@_cdecl("av_capture_audio_file_output_set_max_recorded_file_size")
public func av_capture_audio_file_output_set_max_recorded_file_size(_ outputPtr: UnsafeMutableRawPointer, _ bytes: Int64) {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).audioOutput.maxRecordedFileSize = bytes
}

@_cdecl("av_capture_audio_file_output_set_min_free_disk_space_limit")
public func av_capture_audio_file_output_set_min_free_disk_space_limit(_ outputPtr: UnsafeMutableRawPointer, _ bytes: Int64) {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).audioOutput.minFreeDiskSpaceLimit = bytes
}
