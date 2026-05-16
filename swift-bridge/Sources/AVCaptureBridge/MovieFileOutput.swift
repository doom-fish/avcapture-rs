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
}

struct MovieRecordingEventPayload: Codable {
    let kind: String
    let fileURL: String
    let error: String?
}

private final class MovieFileRecordingDelegate: NSObject, AVCaptureFileOutputRecordingDelegate {
    private weak var owner: MovieFileOutputBox?

    init(owner: MovieFileOutputBox) {
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

final class MovieFileOutputBox: CaptureOutputBoxBase {
    let movieOutput = AVCaptureMovieFileOutput()
    fileprivate var recordingDelegate: MovieFileRecordingDelegate?
    fileprivate var callbackBox: AVCJsonCallbackBox?

    override var output: AVCaptureOutput {
        movieOutput
    }

    deinit {
        clearRecordingState()
    }

    fileprivate func infoPayload() -> MovieFileOutputInfoPayload {
        let path = movieOutput.outputFileURL?.path
        let outputFileURL: String? = {
            guard let path, !path.isEmpty, path != "/" else { return nil }
            return path
        }()
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
            outputFileURL: outputFileURL,
            recordedDuration: CMTimePayload(movieOutput.recordedDuration),
            recordedFileSize: movieOutput.recordedFileSize,
            maxRecordedDuration: CMTimePayload(movieOutput.maxRecordedDuration),
            maxRecordedFileSize: movieOutput.maxRecordedFileSize,
            minFreeDiskSpaceLimit: movieOutput.minFreeDiskSpaceLimit,
            movieFragmentInterval: CMTimePayload(movieOutput.movieFragmentInterval),
            metadataCount: movieOutput.metadata?.count ?? 0,
            spatialVideoCaptureEnabled: spatialVideoCaptureEnabled,
            callbackInstalled: callbackBox != nil
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

        let url = URL(fileURLWithPath: outputPath)
        let parentDirectory = url.deletingLastPathComponent()
        if !parentDirectory.path.isEmpty {
            try FileManager.default.createDirectory(at: parentDirectory, withIntermediateDirectories: true)
        }
        if FileManager.default.fileExists(atPath: url.path) {
            try FileManager.default.removeItem(at: url)
        }

        clearRecordingState()
        if let callback {
            callbackBox = AVCJsonCallbackBox(callback: callback, userData: userData, dropUserData: dropUserData)
        }
        let delegate = MovieFileRecordingDelegate(owner: self)
        recordingDelegate = delegate
        movieOutput.startRecording(to: url, recordingDelegate: delegate)
    }

    func emitEvent(kind: String, fileURL: URL, error: Error?) {
        callbackBox?.emit(MovieRecordingEventPayload(
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
