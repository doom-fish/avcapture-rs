import AVFoundation
import CoreMedia
import CoreVideo
import Foundation

public typealias AVCStreamEventCallback = @convention(c) (
    Int32,
    UnsafeMutablePointer<CChar>?,
    UnsafeMutableRawPointer
) -> Void

private struct SessionErrorStreamPayload: Encodable {
    let errorDescription: String
}

private struct FileRecordingStreamPayload: Encodable {
    let fileUrl: String
    let error: String?
}

private struct AsyncMetadataObjectPayload: Encodable {
    let objectType: String
    let stringValue: String?
    let bounds: CaptureRectPayload
}

private struct AsyncMetadataObjectsPayload: Encodable {
    let objects: [AsyncMetadataObjectPayload]
}

private func avcEmitStreamEvent(
    _ callback: AVCStreamEventCallback,
    kind: Int32,
    ctx: UnsafeMutableRawPointer
) {
    callback(kind, nil, ctx)
}

private func avcEmitStreamEvent<T: Encodable>(
    _ callback: AVCStreamEventCallback,
    kind: Int32,
    payload: T,
    ctx: UnsafeMutableRawPointer
) {
    guard let json = try? avcEncodeJSON(payload) else {
        return
    }
    callback(kind, ffiString(json)!, ctx)
}

private func avcAsyncNotificationName(_ rawValue: String) -> Notification.Name {
    Notification.Name(rawValue: rawValue)
}

private func avcPrepareAsyncRecordingURL(_ outputPath: String) throws -> URL {
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

private final class SessionRunningStreamBridge: NSObject {
    private let sessionBox: SessionBox
    private let callback: AVCStreamEventCallback
    private let ctx: UnsafeMutableRawPointer
    private var observation: NSKeyValueObservation?

    init(sessionBox: SessionBox, callback: @escaping AVCStreamEventCallback, ctx: UnsafeMutableRawPointer) {
        self.sessionBox = sessionBox
        self.callback = callback
        self.ctx = ctx
        super.init()
        observation = sessionBox.session.observe(\.isRunning, options: [.new]) { [weak self] session, _ in
            guard let self else { return }
            avcEmitStreamEvent(self.callback, kind: session.isRunning ? 0 : 1, ctx: self.ctx)
        }
    }

    deinit {
        observation?.invalidate()
        observation = nil
    }
}

private final class SessionErrorStreamBridge: NSObject {
    private let sessionBox: SessionBox
    private let callback: AVCStreamEventCallback
    private let ctx: UnsafeMutableRawPointer
    private var token: NSObjectProtocol?

    init(sessionBox: SessionBox, callback: @escaping AVCStreamEventCallback, ctx: UnsafeMutableRawPointer) {
        self.sessionBox = sessionBox
        self.callback = callback
        self.ctx = ctx
        super.init()
        token = NotificationCenter.default.addObserver(
            forName: avcAsyncNotificationName("AVCaptureSessionRuntimeErrorNotification"),
            object: sessionBox.session,
            queue: nil
        ) { [weak self] notification in
            guard let self else { return }
            let errorDescription: String
            if let error = notification.userInfo?["AVCaptureSessionErrorKey"] as? Error {
                errorDescription = error.localizedDescription
            } else if let error = notification.userInfo?["AVCaptureSessionErrorKey"] as? NSError {
                errorDescription = error.localizedDescription
            } else {
                errorDescription = "Unknown AVCapture session runtime error"
            }
            avcEmitStreamEvent(
                self.callback,
                kind: 0,
                payload: SessionErrorStreamPayload(errorDescription: errorDescription),
                ctx: self.ctx
            )
        }
    }

    deinit {
        if let token {
            NotificationCenter.default.removeObserver(token)
        }
        token = nil
    }
}

private final class SessionInterruptionStreamBridge: NSObject {
    private let sessionBox: SessionBox
    private let callback: AVCStreamEventCallback
    private let ctx: UnsafeMutableRawPointer
    private var interruptedToken: NSObjectProtocol?
    private var endedToken: NSObjectProtocol?

    init(sessionBox: SessionBox, callback: @escaping AVCStreamEventCallback, ctx: UnsafeMutableRawPointer) {
        self.sessionBox = sessionBox
        self.callback = callback
        self.ctx = ctx
        super.init()
        interruptedToken = NotificationCenter.default.addObserver(
            forName: avcAsyncNotificationName("AVCaptureSessionWasInterruptedNotification"),
            object: sessionBox.session,
            queue: nil
        ) { [weak self] _ in
            guard let self else { return }
            avcEmitStreamEvent(self.callback, kind: 0, ctx: self.ctx)
        }
        endedToken = NotificationCenter.default.addObserver(
            forName: avcAsyncNotificationName("AVCaptureSessionInterruptionEndedNotification"),
            object: sessionBox.session,
            queue: nil
        ) { [weak self] _ in
            guard let self else { return }
            avcEmitStreamEvent(self.callback, kind: 1, ctx: self.ctx)
        }
    }

    deinit {
        if let interruptedToken {
            NotificationCenter.default.removeObserver(interruptedToken)
        }
        if let endedToken {
            NotificationCenter.default.removeObserver(endedToken)
        }
        interruptedToken = nil
        endedToken = nil
    }
}

private final class VideoSampleStreamBridge: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    private let outputBox: VideoDataOutputBox
    private let callback: AVCVideoSampleCallback
    private let ctx: UnsafeMutableRawPointer
    private let queue: DispatchQueue

    init(
        outputBox: VideoDataOutputBox,
        queueLabel: String,
        callback: @escaping AVCVideoSampleCallback,
        ctx: UnsafeMutableRawPointer
    ) {
        self.outputBox = outputBox
        self.callback = callback
        self.ctx = ctx
        queue = DispatchQueue(label: queueLabel)
        super.init()
        outputBox.videoOutput.setSampleBufferDelegate(self, queue: queue)
    }

    deinit {
        outputBox.videoOutput.setSampleBufferDelegate(nil, queue: nil)
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        let pixelOpaque = CMSampleBufferGetImageBuffer(sampleBuffer).map { Unmanaged.passRetained($0).toOpaque() }
        callback(ctx, sampleOpaque, pixelOpaque)
    }
}

private final class AudioSampleStreamBridge: NSObject, AVCaptureAudioDataOutputSampleBufferDelegate {
    private let outputBox: AudioDataOutputBox
    private let callback: AVCAudioSampleCallback
    private let ctx: UnsafeMutableRawPointer
    private let queue: DispatchQueue

    init(
        outputBox: AudioDataOutputBox,
        queueLabel: String,
        callback: @escaping AVCAudioSampleCallback,
        ctx: UnsafeMutableRawPointer
    ) {
        self.outputBox = outputBox
        self.callback = callback
        self.ctx = ctx
        queue = DispatchQueue(label: queueLabel)
        super.init()
        outputBox.audioOutput.setSampleBufferDelegate(self, queue: queue)
    }

    deinit {
        outputBox.audioOutput.setSampleBufferDelegate(nil, queue: nil)
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(ctx, sampleOpaque)
    }
}

private final class FileRecordingStreamBridge: NSObject, AVCaptureFileOutputRecordingDelegate {
    private let outputBox: MovieFileOutputBox
    private let callback: AVCStreamEventCallback
    private let ctx: UnsafeMutableRawPointer
    private var stopped = false

    init(
        outputBox: MovieFileOutputBox,
        outputPath: String,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer
    ) throws {
        self.outputBox = outputBox
        self.callback = callback
        self.ctx = ctx
        super.init()

        guard !outputBox.movieOutput.isRecording else {
            throw BridgeError.message("movie file output is already recording")
        }
        guard !outputBox.movieOutput.connections.isEmpty else {
            throw BridgeError.message("movie file output is not attached to a session")
        }

        let url = try avcPrepareAsyncRecordingURL(outputPath)
        outputBox.movieOutput.startRecording(to: url, recordingDelegate: self)
    }

    deinit {
        stop()
    }

    func stop() {
        guard !stopped else { return }
        stopped = true
        if outputBox.movieOutput.isRecording {
            outputBox.movieOutput.stopRecording()
        }
    }

    private func emit(kind: Int32, fileURL: URL, error: Error?) {
        avcEmitStreamEvent(
            callback,
            kind: kind,
            payload: FileRecordingStreamPayload(fileUrl: fileURL.path, error: error?.localizedDescription),
            ctx: ctx
        )
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didStartRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        emit(kind: 0, fileURL: fileURL, error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didPauseRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        emit(kind: 1, fileURL: fileURL, error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didResumeRecordingTo fileURL: URL,
        from connections: [AVCaptureConnection]
    ) {
        emit(kind: 2, fileURL: fileURL, error: nil)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        willFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        emit(kind: 3, fileURL: outputFileURL, error: error)
    }

    func fileOutput(
        _ output: AVCaptureFileOutput,
        didFinishRecordingTo outputFileURL: URL,
        from connections: [AVCaptureConnection],
        error: Error?
    ) {
        emit(kind: 4, fileURL: outputFileURL, error: error)
        stopped = true
    }
}

@available(macOS 13.0, *)
private final class MetadataObjectsStreamBridge: NSObject, AVCaptureMetadataOutputObjectsDelegate {
    private let outputBox: MetadataOutputBox
    private let callback: AVCStreamEventCallback
    private let ctx: UnsafeMutableRawPointer
    private let queue: DispatchQueue

    init(
        outputBox: MetadataOutputBox,
        queueLabel: String,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer
    ) {
        self.outputBox = outputBox
        self.callback = callback
        self.ctx = ctx
        queue = DispatchQueue(label: queueLabel)
        super.init()
        outputBox.metadataOutput.setMetadataObjectsDelegate(self, queue: queue)
    }

    deinit {
        outputBox.metadataOutput.setMetadataObjectsDelegate(nil, queue: nil)
    }

    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput metadataObjects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        let payload = AsyncMetadataObjectsPayload(objects: metadataObjects.map { object in
            AsyncMetadataObjectPayload(
                objectType: object.type.rawValue,
                stringValue: (object as? AVMetadataMachineReadableCodeObject)?.stringValue,
                bounds: CaptureRectPayload(object.bounds)
            )
        })
        avcEmitStreamEvent(callback, kind: 0, payload: payload, ctx: ctx)
    }
}

@_cdecl("avcapture_session_running_subscribe")
public func avcapture_session_running_subscribe(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    return avcRetain(SessionRunningStreamBridge(sessionBox: avcSessionBox(sessionPtr), callback: onEvent, ctx: ctx))
}

@_cdecl("avcapture_session_running_unsubscribe")
public func avcapture_session_running_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    avcRelease(handle, as: SessionRunningStreamBridge.self)
}

@_cdecl("avcapture_session_error_subscribe")
public func avcapture_session_error_subscribe(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    return avcRetain(SessionErrorStreamBridge(sessionBox: avcSessionBox(sessionPtr), callback: onEvent, ctx: ctx))
}

@_cdecl("avcapture_session_error_unsubscribe")
public func avcapture_session_error_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    avcRelease(handle, as: SessionErrorStreamBridge.self)
}

@_cdecl("avcapture_session_interruption_subscribe")
public func avcapture_session_interruption_subscribe(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    return avcRetain(SessionInterruptionStreamBridge(sessionBox: avcSessionBox(sessionPtr), callback: onEvent, ctx: ctx))
}

@_cdecl("avcapture_session_interruption_unsubscribe")
public func avcapture_session_interruption_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    avcRelease(handle, as: SessionInterruptionStreamBridge.self)
}

@_cdecl("avcapture_video_sample_subscribe")
public func avcapture_video_sample_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCVideoSampleCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    av_capture_video_output_clear_sample_buffer_callback(outputPtr)
    let outputBox = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    return avcRetain(VideoSampleStreamBridge(outputBox: outputBox, queueLabel: queueLabel, callback: onEvent, ctx: ctx))
}

@_cdecl("avcapture_video_sample_unsubscribe")
public func avcapture_video_sample_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    avcRelease(handle, as: VideoSampleStreamBridge.self)
}

@_cdecl("avcapture_audio_sample_subscribe")
public func avcapture_audio_sample_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    av_capture_audio_output_clear_sample_buffer_callback(outputPtr)
    let outputBox = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    return avcRetain(AudioSampleStreamBridge(outputBox: outputBox, queueLabel: queueLabel, callback: onEvent, ctx: ctx))
}

@_cdecl("avcapture_audio_sample_unsubscribe")
public func avcapture_audio_sample_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    avcRelease(handle, as: AudioSampleStreamBridge.self)
}

@_cdecl("avcapture_file_recording_stream_start")
public func avcapture_file_recording_stream_start(
    _ outputPtr: UnsafeMutableRawPointer,
    _ pathPtr: UnsafePointer<CChar>,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        outErrorMessage?.pointee = ffiString("missing file recording callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        let bridge = try FileRecordingStreamBridge(
            outputBox: outputBox,
            outputPath: String(cString: pathPtr),
            callback: onEvent,
            ctx: ctx
        )
        return avcRetain(bridge)
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_file_recording_stream_stop")
public func avcapture_file_recording_stream_stop(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    let bridge = avcUnretained(handle, as: FileRecordingStreamBridge.self)
    bridge.stop()
    avcRelease(handle, as: FileRecordingStreamBridge.self)
}

@available(macOS 13.0, *)
@_cdecl("avcapture_metadata_objects_subscribe")
public func avcapture_metadata_objects_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    av_capture_metadata_output_clear_metadata_objects_callback(outputPtr)
    let outputBox = avcUnretained(outputPtr, as: MetadataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    return avcRetain(MetadataObjectsStreamBridge(outputBox: outputBox, queueLabel: queueLabel, callback: onEvent, ctx: ctx))
}

@available(macOS 13.0, *)
@_cdecl("avcapture_metadata_objects_unsubscribe")
public func avcapture_metadata_objects_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    avcRelease(handle, as: MetadataObjectsStreamBridge.self)
}
