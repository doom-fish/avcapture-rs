import AVFoundation
import Darwin
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
    private let callback: AVCAudioSampleCallback
    private let contextOwner: AVCCallbackContextOwner

    init(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        contextOwner = AVCCallbackContextOwner(
            userData: userData,
            retainUserData: retainUserData,
            releaseUserData: dropUserData
        )
    }

    func emit(sampleBuffer: CMSampleBuffer) {
        let contextOwner = self.contextOwner
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(contextOwner.userData, sampleOpaque)
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

enum AVCRecordingOverwritePolicy: Int32 {
    case failIfExists = 0
    case overwriteRegularFile = 1
}

final class AVCPreparedRecordingDestination {
    let requestedURL: URL
    let stagingURL: URL
    let overwritePolicy: AVCRecordingOverwritePolicy

    init(path: String, overwritePolicy: AVCRecordingOverwritePolicy) throws {
        let requestedURL = URL(fileURLWithPath: path).standardizedFileURL
        guard requestedURL.path.hasPrefix("/") else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "recording output path must be absolute"
            )
        }
        let parentURL = requestedURL.deletingLastPathComponent()
        guard let parentStatus = try Self.fileStatus(at: parentURL, followSymlinks: true),
              (parentStatus.st_mode & S_IFMT) == S_IFDIR
        else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "recording output parent directory does not exist or is not a directory"
            )
        }
        if let destinationStatus = try Self.fileStatus(at: requestedURL) {
            switch overwritePolicy {
            case .failIfExists:
                throw BridgeError.status(
                    AVC_OUTPUT_FILE_EXISTS,
                    requestedURL.path
                )
            case .overwriteRegularFile:
                guard (destinationStatus.st_mode & S_IFMT) == S_IFREG else {
                    throw BridgeError.status(
                        AVC_INVALID_ARGUMENT,
                        "explicit recording overwrite is limited to regular files"
                    )
                }
            }
        }

        var stagingURL: URL
        repeat {
            let name = ".\(requestedURL.lastPathComponent).avcapture-\(UUID().uuidString).partial"
            stagingURL = parentURL.appendingPathComponent(name, isDirectory: false)
        } while try Self.fileStatus(at: stagingURL) != nil

        self.requestedURL = requestedURL
        self.stagingURL = stagingURL
        self.overwritePolicy = overwritePolicy
    }

    func finalize(nativeError: Error?) -> Error? {
        if let nativeError {
            let successfullyFinished =
                (nativeError as NSError)
                .userInfo[AVErrorRecordingSuccessfullyFinishedKey] as? Bool
            guard successfullyFinished == true else {
                discardStagingFile()
                return nativeError
            }
        }
        do {
            try moveStagingFileIntoPlace()
            return nativeError
        } catch let finalizationError {
            discardStagingFile()
            if let nativeError {
                return BridgeError.message(
                    "\(nativeError.localizedDescription); finalization failed: \(finalizationError.localizedDescription)"
                )
            }
            return finalizationError
        }
    }

    private func moveStagingFileIntoPlace() throws {
        switch overwritePolicy {
        case .failIfExists:
            let result = stagingURL.withUnsafeFileSystemRepresentation { source in
                requestedURL.withUnsafeFileSystemRepresentation { destination in
                    renameatx_np(
                        AT_FDCWD,
                        source,
                        AT_FDCWD,
                        destination,
                        UInt32(RENAME_EXCL)
                    )
                }
            }
            guard result == 0 else {
                if errno == EEXIST {
                    throw BridgeError.status(AVC_OUTPUT_FILE_EXISTS, requestedURL.path)
                }
                throw POSIXError(POSIXErrorCode(rawValue: errno) ?? .EIO)
            }
        case .overwriteRegularFile:
            if let destinationStatus = try Self.fileStatus(at: requestedURL),
               (destinationStatus.st_mode & S_IFMT) != S_IFREG
            {
                throw BridgeError.status(
                    AVC_INVALID_ARGUMENT,
                    "recording destination stopped being a regular file before finalization"
                )
            }
            let result = stagingURL.withUnsafeFileSystemRepresentation { source in
                requestedURL.withUnsafeFileSystemRepresentation { destination in
                    rename(source, destination)
                }
            }
            guard result == 0 else {
                throw POSIXError(POSIXErrorCode(rawValue: errno) ?? .EIO)
            }
        }
    }

    private func discardStagingFile() {
        guard let status = try? Self.fileStatus(at: stagingURL),
              (status.st_mode & S_IFMT) == S_IFREG
        else {
            return
        }
        _ = stagingURL.withUnsafeFileSystemRepresentation { unlink($0) }
    }

    private static func fileStatus(
        at url: URL,
        followSymlinks: Bool = false
    ) throws -> stat? {
        var fileInfo = stat()
        let result = url.withUnsafeFileSystemRepresentation { path in
            if followSymlinks {
                return fstatat(AT_FDCWD, path, &fileInfo, 0)
            }
            return lstat(path, &fileInfo)
        }
        if result == 0 {
            return fileInfo
        }
        if errno == ENOENT {
            return nil
        }
        throw POSIXError(POSIXErrorCode(rawValue: errno) ?? .EIO)
    }
}

@_cdecl("av_capture_recording_destination_finalize_for_testing")
public func av_capture_recording_destination_finalize_for_testing(
    _ pathBytes: UnsafePointer<UInt8>,
    _ pathLength: Int,
    _ overwritePolicyRaw: Int32,
    _ nativeErrorMode: Int32,
    _ outHadError: UnsafeMutablePointer<Bool>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    do {
        guard let overwritePolicy = AVCRecordingOverwritePolicy(rawValue: overwritePolicyRaw) else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "unsupported recording overwrite policy: \(overwritePolicyRaw)"
            )
        }
        let destination = try AVCPreparedRecordingDestination(
            path: avcRecordingPath(pathBytes, length: pathLength),
            overwritePolicy: overwritePolicy
        )
        guard FileManager.default.createFile(
            atPath: destination.stagingURL.path,
            contents: Data("staged".utf8)
        ) else {
            throw BridgeError.message("failed to create synthetic staged recording")
        }

        let nativeError: Error?
        switch nativeErrorMode {
        case 0:
            nativeError = nil
        case 1:
            nativeError = NSError(
                domain: "AVCaptureBridgeSyntheticRecording",
                code: 1
            )
        case 2:
            nativeError = NSError(
                domain: "AVCaptureBridgeSyntheticRecording",
                code: 2,
                userInfo: [AVErrorRecordingSuccessfullyFinishedKey: true]
            )
        case 3:
            nativeError = NSError(
                domain: "AVCaptureBridgeSyntheticRecording",
                code: 3,
                userInfo: [AVErrorRecordingSuccessfullyFinishedKey: false]
            )
        default:
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "unsupported synthetic recording error mode: \(nativeErrorMode)"
            )
        }

        outHadError.pointee = destination.finalize(nativeError: nativeError) != nil
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_OPERATION_FAILED)
    }
}

protocol AVCFileRecordingOperationOwner: AnyObject {
    func recordingOperationDidFinish(_ operation: AVCFileRecordingOperation)
}

final class AVCFileRecordingOperation: NSObject, AVCaptureFileOutputRecordingDelegate {
    private weak var owner: AVCFileRecordingOperationOwner?
    private let destination: AVCPreparedRecordingDestination
    private let emitHandler: ((String, URL, Error?) -> Void)?
    private let isRecording: () -> Bool
    private let stopRecording: () -> Void
    private let lock = NSLock()
    private var stopRequested = false
    private var finished = false
    private var keepAlive: AVCFileRecordingOperation?

    init(
        owner: AVCFileRecordingOperationOwner,
        destination: AVCPreparedRecordingDestination,
        emitHandler: ((String, URL, Error?) -> Void)?,
        isRecording: @escaping () -> Bool,
        stopRecording: @escaping () -> Void
    ) {
        self.owner = owner
        self.destination = destination
        self.emitHandler = emitHandler
        self.isRecording = isRecording
        self.stopRecording = stopRecording
        super.init()
    }

    var requestedURL: URL {
        destination.requestedURL
    }

    var hasCallback: Bool {
        emitHandler != nil
    }

    func activate() {
        keepAlive = self
    }

    @discardableResult
    func requestStop() -> Bool {
        lock.lock()
        guard !finished, !stopRequested, isRecording() else {
            lock.unlock()
            return false
        }
        stopRequested = true
        lock.unlock()
        stopRecording()
        return true
    }

    private func emit(_ kind: String, error: Error?) {
        emitHandler?(kind, destination.requestedURL, error)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didStartRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        emit("started", error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didPauseRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        emit("paused", error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didResumeRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        emit("resumed", error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        willFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        emit("willFinish", error: error)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        let finalError = destination.finalize(nativeError: error)
        emit("finished", error: finalError)
        lock.lock()
        finished = true
        lock.unlock()
        owner?.recordingOperationDidFinish(self)
        keepAlive = nil
    }
}

func avcRecordingPath(
    _ bytes: UnsafePointer<UInt8>,
    length: Int
) throws -> String {
    let path = bytes.withMemoryRebound(to: CChar.self, capacity: length) {
        FileManager.default.string(
            withFileSystemRepresentation: $0,
            length: length
        )
    }
    guard !path.isEmpty else {
        throw BridgeError.status(AVC_INVALID_ARGUMENT, "recording output path is empty")
    }
    return path
}

final class MovieFileOutputBox: CaptureOutputBoxBase, AVCFileRecordingOperationOwner {
    let movieOutput = AVCaptureMovieFileOutput()
    private let recordingLock = NSLock()
    private let recordingSlot = AVCDelegateSlot("movie recording delegate")
    private var recordingOperation: AVCFileRecordingOperation?
    private var lastOutputURL: URL?
    private let boundaryLock = NSLock()
    private let boundarySlot = AVCDelegateSlot("movie file-output delegate")
    private var sampleBufferBoundaryDelegate: FileOutputBoundaryDelegate?
    private var sampleBufferBoundaryCallbackBox: FileOutputSampleBufferCallbackBox?

    override var output: AVCaptureOutput {
        movieOutput
    }

    deinit {
        recordingLock.lock()
        let operation = recordingOperation
        recordingOperation = nil
        recordingLock.unlock()
        operation?.requestStop()
        clearSampleBufferBoundaryCallback()
    }

    fileprivate func infoPayload() -> MovieFileOutputInfoPayload {
        let spatialVideoCaptureEnabled: Bool?
        if #available(macOS 15.0, *) {
            spatialVideoCaptureEnabled = movieOutput.isSpatialVideoCaptureEnabled
        } else {
            spatialVideoCaptureEnabled = nil
        }
        recordingLock.lock()
        let operation = recordingOperation
        recordingLock.unlock()
        return MovieFileOutputInfoPayload(
            connectionCount: movieOutput.connections.count,
            isRecording: movieOutput.isRecording,
            isRecordingPaused: movieOutput.isRecordingPaused,
            outputFileURL: operation?.requestedURL.path ?? lastOutputURL?.path,
            recordedDuration: CMTimePayload(movieOutput.recordedDuration),
            recordedFileSize: movieOutput.recordedFileSize,
            maxRecordedDuration: CMTimePayload(movieOutput.maxRecordedDuration),
            maxRecordedFileSize: movieOutput.maxRecordedFileSize,
            minFreeDiskSpaceLimit: movieOutput.minFreeDiskSpaceLimit,
            movieFragmentInterval: CMTimePayload(movieOutput.movieFragmentInterval),
            metadataCount: movieOutput.metadata?.count ?? 0,
            spatialVideoCaptureEnabled: spatialVideoCaptureEnabled,
            callbackInstalled: operation?.hasCallback ?? false,
            sampleBufferBoundaryCallbackInstalled: boundarySlot.isOccupied
        )
    }

    func startRecording(
        to outputPath: String,
        overwritePolicy: AVCRecordingOverwritePolicy,
        callback: AVCJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        dropUserData: AVCDropCallback?
    ) throws {
        let callbackBox = callback.map {
            AVCJsonCallbackBox(
                callback: $0,
                userData: userData,
                retainUserData: retainUserData,
                dropUserData: dropUserData
            )
        }
        _ = try startRecording(
            to: outputPath,
            overwritePolicy: overwritePolicy,
            emitHandler: callbackBox.map { callbackBox in
                { kind, fileURL, error in
                    callbackBox.emit(
                        FileRecordingEventPayload(
                            kind: kind,
                            fileURL: fileURL.path,
                            error: error?.localizedDescription
                        )
                    )
                }
            }
        )
    }

    func startRecording(
        to outputPath: String,
        overwritePolicy: AVCRecordingOverwritePolicy,
        emitHandler: ((String, URL, Error?) -> Void)?
    ) throws -> AVCFileRecordingOperation {
        let destination = try AVCPreparedRecordingDestination(
            path: outputPath,
            overwritePolicy: overwritePolicy
        )
        guard !movieOutput.isRecording else {
            throw BridgeError.message("movie file output is already recording")
        }
        guard !movieOutput.connections.isEmpty else {
            throw BridgeError.message("movie file output is not attached to a session")
        }
        let operation = AVCFileRecordingOperation(
            owner: self,
            destination: destination,
            emitHandler: emitHandler,
            isRecording: { [movieOutput] in movieOutput.isRecording },
            stopRecording: { [movieOutput] in movieOutput.stopRecording() }
        )
        try recordingSlot.acquire(operation)
        recordingLock.lock()
        recordingOperation = operation
        lastOutputURL = destination.requestedURL
        recordingLock.unlock()
        operation.activate()
        movieOutput.startRecording(
            to: destination.stagingURL,
            recordingDelegate: operation
        )
        return operation
    }

    func recordingOperationDidFinish(_ operation: AVCFileRecordingOperation) {
        guard recordingSlot.release(operation) else { return }
        recordingLock.lock()
        if recordingOperation === operation {
            recordingOperation = nil
        }
        recordingLock.unlock()
    }

    @discardableResult
    func stopRecording() -> Bool {
        recordingLock.lock()
        let operation = recordingOperation
        recordingLock.unlock()
        return operation?.requestStop() ?? false
    }

    func setSampleBufferBoundaryCallback(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        dropUserData: AVCDropCallback?
    ) throws {
        let box = FileOutputSampleBufferCallbackBox(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        let delegate = FileOutputBoundaryDelegate { sampleBuffer in
            box.emit(sampleBuffer: sampleBuffer)
        }
        try boundarySlot.acquire(box)
        movieOutput.delegate = delegate
        boundaryLock.lock()
        sampleBufferBoundaryDelegate = delegate
        sampleBufferBoundaryCallbackBox = box
        boundaryLock.unlock()
    }

    func clearSampleBufferBoundaryCallback() {
        boundaryLock.lock()
        let box = sampleBufferBoundaryCallbackBox
        let delegate = sampleBufferBoundaryDelegate
        if let box, boundarySlot.release(box) {
            sampleBufferBoundaryDelegate = nil
            sampleBufferBoundaryCallbackBox = nil
        }
        boundaryLock.unlock()
        if let delegate, movieOutput.delegate === delegate {
            movieOutput.delegate = nil
        }
    }

    func installBoundaryStream(owner: AnyObject, delegate: AVCaptureFileOutputDelegate) throws {
        try boundarySlot.acquire(owner)
        movieOutput.delegate = delegate
    }

    func removeBoundaryStream(owner: AnyObject, delegate: AVCaptureFileOutputDelegate) {
        guard boundarySlot.release(owner) else { return }
        if movieOutput.delegate === delegate {
            movieOutput.delegate = nil
        }
    }
}

final class AudioFileOutputBox: CaptureOutputBoxBase, AVCFileRecordingOperationOwner {
    let audioOutput = AVCaptureAudioFileOutput()
    private let recordingLock = NSLock()
    private let recordingSlot = AVCDelegateSlot("audio file recording delegate")
    private var recordingOperation: AVCFileRecordingOperation?
    private var lastOutputURL: URL?
    private let boundaryLock = NSLock()
    private let boundarySlot = AVCDelegateSlot("audio file-output delegate")
    private var sampleBufferBoundaryDelegate: FileOutputBoundaryDelegate?
    private var sampleBufferBoundaryCallbackBox: FileOutputSampleBufferCallbackBox?

    override var output: AVCaptureOutput {
        audioOutput
    }

    deinit {
        recordingLock.lock()
        let operation = recordingOperation
        recordingOperation = nil
        recordingLock.unlock()
        operation?.requestStop()
        clearSampleBufferBoundaryCallback()
    }

    fileprivate func infoPayload() -> AudioFileOutputInfoPayload {
        recordingLock.lock()
        let operation = recordingOperation
        recordingLock.unlock()
        return AudioFileOutputInfoPayload(
            connectionCount: audioOutput.connections.count,
            isRecording: audioOutput.isRecording,
            isRecordingPaused: audioOutput.isRecordingPaused,
            outputFileURL: operation?.requestedURL.path ?? lastOutputURL?.path,
            recordedDuration: CMTimePayload(audioOutput.recordedDuration),
            recordedFileSize: audioOutput.recordedFileSize,
            maxRecordedDuration: CMTimePayload(audioOutput.maxRecordedDuration),
            maxRecordedFileSize: audioOutput.maxRecordedFileSize,
            minFreeDiskSpaceLimit: audioOutput.minFreeDiskSpaceLimit,
            metadataCount: audioOutput.metadata.count,
            availableOutputFileTypes: AVCaptureAudioFileOutput.availableOutputFileTypes().map(\.rawValue),
            audioSettings: avcEncodeAudioSettings(audioOutput.audioSettings),
            callbackInstalled: operation?.hasCallback ?? false,
            sampleBufferBoundaryCallbackInstalled: boundarySlot.isOccupied
        )
    }

    func startRecording(
        to outputPath: String,
        outputFileType rawOutputFileType: String,
        overwritePolicy: AVCRecordingOverwritePolicy,
        callback: AVCJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        dropUserData: AVCDropCallback?
    ) throws {
        let callbackBox = callback.map {
            AVCJsonCallbackBox(
                callback: $0,
                userData: userData,
                retainUserData: retainUserData,
                dropUserData: dropUserData
            )
        }
        _ = try startRecording(
            to: outputPath,
            outputFileType: rawOutputFileType,
            overwritePolicy: overwritePolicy,
            emitHandler: callbackBox.map { callbackBox in
                { kind, fileURL, error in
                    callbackBox.emit(
                        FileRecordingEventPayload(
                            kind: kind,
                            fileURL: fileURL.path,
                            error: error?.localizedDescription
                        )
                    )
                }
            }
        )
    }

    func startRecording(
        to outputPath: String,
        outputFileType rawOutputFileType: String,
        overwritePolicy: AVCRecordingOverwritePolicy,
        emitHandler: ((String, URL, Error?) -> Void)?
    ) throws -> AVCFileRecordingOperation {
        let destination = try AVCPreparedRecordingDestination(
            path: outputPath,
            overwritePolicy: overwritePolicy
        )
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
        let operation = AVCFileRecordingOperation(
            owner: self,
            destination: destination,
            emitHandler: emitHandler,
            isRecording: { [audioOutput] in audioOutput.isRecording },
            stopRecording: { [audioOutput] in audioOutput.stopRecording() }
        )
        try recordingSlot.acquire(operation)
        recordingLock.lock()
        recordingOperation = operation
        lastOutputURL = destination.requestedURL
        recordingLock.unlock()
        operation.activate()
        audioOutput.startRecording(
            to: destination.stagingURL,
            outputFileType: outputFileType,
            recordingDelegate: operation
        )
        return operation
    }

    func recordingOperationDidFinish(_ operation: AVCFileRecordingOperation) {
        guard recordingSlot.release(operation) else { return }
        recordingLock.lock()
        if recordingOperation === operation {
            recordingOperation = nil
        }
        recordingLock.unlock()
    }

    @discardableResult
    func stopRecording() -> Bool {
        recordingLock.lock()
        let operation = recordingOperation
        recordingLock.unlock()
        return operation?.requestStop() ?? false
    }

    func setSampleBufferBoundaryCallback(
        callback: @escaping AVCAudioSampleCallback,
        userData: UnsafeMutableRawPointer?,
        retainUserData: AVCRetainCallback?,
        dropUserData: AVCDropCallback?
    ) throws {
        let box = FileOutputSampleBufferCallbackBox(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        let delegate = FileOutputBoundaryDelegate { sampleBuffer in
            box.emit(sampleBuffer: sampleBuffer)
        }
        try boundarySlot.acquire(box)
        audioOutput.delegate = delegate
        boundaryLock.lock()
        sampleBufferBoundaryDelegate = delegate
        sampleBufferBoundaryCallbackBox = box
        boundaryLock.unlock()
    }

    func clearSampleBufferBoundaryCallback() {
        boundaryLock.lock()
        let box = sampleBufferBoundaryCallbackBox
        let delegate = sampleBufferBoundaryDelegate
        if let box, boundarySlot.release(box) {
            sampleBufferBoundaryDelegate = nil
            sampleBufferBoundaryCallbackBox = nil
        }
        boundaryLock.unlock()
        if let delegate, audioOutput.delegate === delegate {
            audioOutput.delegate = nil
        }
    }

    func installBoundaryStream(owner: AnyObject, delegate: AVCaptureFileOutputDelegate) throws {
        try boundarySlot.acquire(owner)
        audioOutput.delegate = delegate
    }

    func removeBoundaryStream(owner: AnyObject, delegate: AVCaptureFileOutputDelegate) {
        guard boundarySlot.release(owner) else { return }
        if audioOutput.delegate === delegate {
            audioOutput.delegate = nil
        }
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
    _ outputPathBytes: UnsafePointer<UInt8>,
    _ outputPathLength: Int,
    _ overwritePolicyRaw: Int32,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        guard let overwritePolicy = AVCRecordingOverwritePolicy(rawValue: overwritePolicyRaw) else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "unsupported recording overwrite policy: \(overwritePolicyRaw)"
            )
        }
        try output.startRecording(
            to: avcRecordingPath(outputPathBytes, length: outputPathLength),
            overwritePolicy: overwritePolicy,
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_OUTPUT_ERROR)
    }
}

@_cdecl("av_capture_movie_file_output_set_sample_buffer_boundary_callback")
public func av_capture_movie_file_output_set_sample_buffer_boundary_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ callback: AVCAudioSampleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing movie file output sample-buffer callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        try output.setSampleBufferBoundaryCallback(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_CALLBACK_ERROR)
    }
}

@_cdecl("av_capture_movie_file_output_clear_sample_buffer_boundary_callback")
public func av_capture_movie_file_output_clear_sample_buffer_boundary_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).clearSampleBufferBoundaryCallback()
}

@_cdecl("av_capture_movie_file_output_stop_recording")
public func av_capture_movie_file_output_stop_recording(
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    avcUnretained(outputPtr, as: MovieFileOutputBox.self).stopRecording()
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
    _ outputPathBytes: UnsafePointer<UInt8>,
    _ outputPathLength: Int,
    _ outputFileTypePtr: UnsafePointer<CChar>,
    _ overwritePolicyRaw: Int32,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    let outputFileType = String(cString: outputFileTypePtr)
    do {
        guard let overwritePolicy = AVCRecordingOverwritePolicy(rawValue: overwritePolicyRaw) else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "unsupported recording overwrite policy: \(overwritePolicyRaw)"
            )
        }
        try output.startRecording(
            to: avcRecordingPath(outputPathBytes, length: outputPathLength),
            outputFileType: outputFileType,
            overwritePolicy: overwritePolicy,
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_OUTPUT_ERROR)
    }
}

@_cdecl("av_capture_audio_file_output_set_sample_buffer_boundary_callback")
public func av_capture_audio_file_output_set_sample_buffer_boundary_callback(
    _ outputPtr: UnsafeMutableRawPointer,
    _ callback: AVCAudioSampleCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ retainUserData: AVCRetainCallback?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing audio file output sample-buffer callback")
        return AVC_CALLBACK_ERROR
    }
    let output = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        try output.setSampleBufferBoundaryCallback(
            callback: callback,
            userData: userData,
            retainUserData: retainUserData,
            dropUserData: dropUserData
        )
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return avcStatus(for: error, default: AVC_CALLBACK_ERROR)
    }
}

@_cdecl("av_capture_audio_file_output_clear_sample_buffer_boundary_callback")
public func av_capture_audio_file_output_clear_sample_buffer_boundary_callback(_ outputPtr: UnsafeMutableRawPointer) {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).clearSampleBufferBoundaryCallback()
}

@_cdecl("av_capture_audio_file_output_stop_recording")
public func av_capture_audio_file_output_stop_recording(
    _ outputPtr: UnsafeMutableRawPointer
) -> Bool {
    avcUnretained(outputPtr, as: AudioFileOutputBox.self).stopRecording()
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
