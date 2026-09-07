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
    let fileURL: String
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
    owner: AVCCallbackContextOwner
) {
    guard let ctx = owner.userData else { return }
    callback(kind, nil, ctx)
}

private func avcEmitStreamEvent<T: Encodable>(
    _ callback: AVCStreamEventCallback,
    kind: Int32,
    payload: T,
    owner: AVCCallbackContextOwner
) {
    guard let ctx = owner.userData else { return }
    do {
        callback(kind, ffiString(try avcEncodeJSON(payload)), ctx)
    } catch {
        callback(AVC_STREAM_BRIDGE_ERROR_KIND, avcBridgeErrorPayload(error), ctx)
    }
}

private func avcAsyncNotificationName(_ rawValue: String) -> Notification.Name {
    Notification.Name(rawValue: rawValue)
}

private final class AsyncFileOutputBoundaryDelegate: NSObject, AVCaptureFileOutputDelegate {
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

private final class SessionRunningStreamBridge: NSObject {
    private let sessionBox: SessionBox
    private let callback: AVCStreamEventCallback
    private let callbackOwner: AVCCallbackContextOwner
    private var observation: NSKeyValueObservation?

    init(
        sessionBox: SessionBox,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) {
        self.sessionBox = sessionBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        observation = sessionBox.session.observe(\.isRunning, options: [.new]) { [weak self] session, _ in
            guard let self else { return }
            let callbackOwner = self.callbackOwner
            avcEmitStreamEvent(
                self.callback,
                kind: session.isRunning ? 0 : 1,
                owner: callbackOwner
            )
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
    private let callbackOwner: AVCCallbackContextOwner
    private var token: NSObjectProtocol?

    init(
        sessionBox: SessionBox,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) {
        self.sessionBox = sessionBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        token = NotificationCenter.default.addObserver(
            forName: avcAsyncNotificationName("AVCaptureSessionRuntimeErrorNotification"),
            object: sessionBox.session,
            queue: nil
        ) { [weak self] notification in
            guard let self else { return }
            let callbackOwner = self.callbackOwner
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
                owner: callbackOwner
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
    private let callbackOwner: AVCCallbackContextOwner
    private var interruptedToken: NSObjectProtocol?
    private var endedToken: NSObjectProtocol?

    init(
        sessionBox: SessionBox,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) {
        self.sessionBox = sessionBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        interruptedToken = NotificationCenter.default.addObserver(
            forName: avcAsyncNotificationName("AVCaptureSessionWasInterruptedNotification"),
            object: sessionBox.session,
            queue: nil
        ) { [weak self] _ in
            guard let self else { return }
            let callbackOwner = self.callbackOwner
            avcEmitStreamEvent(self.callback, kind: 0, owner: callbackOwner)
        }
        endedToken = NotificationCenter.default.addObserver(
            forName: avcAsyncNotificationName("AVCaptureSessionInterruptionEndedNotification"),
            object: sessionBox.session,
            queue: nil
        ) { [weak self] _ in
            guard let self else { return }
            let callbackOwner = self.callbackOwner
            avcEmitStreamEvent(self.callback, kind: 1, owner: callbackOwner)
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

private final class VideoSampleStreamDelegate: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    private weak var owner: VideoSampleStreamBridge?

    init(owner: VideoSampleStreamBridge) {
        self.owner = owner
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitSample(sampleBuffer)
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didDrop sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.noteDropped(sampleBuffer)
    }
}

private final class VideoSampleStreamBridge: NSObject {
    private let outputBox: VideoDataOutputBox
    private let callback: AVCVideoSampleCallback
    private let callbackOwner: AVCCallbackContextOwner
    private let queue: AVCSerialCallbackQueue
    private let lock = NSLock()
    private var delegate: VideoSampleStreamDelegate?
    private var stopped = false

    init(
        outputBox: VideoDataOutputBox,
        queueLabel: String,
        callback: @escaping AVCVideoSampleCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        queue = AVCSerialCallbackQueue(label: queueLabel)
        super.init()
        let delegate = VideoSampleStreamDelegate(owner: self)
        try outputBox.installStreamDelegate(owner: self, delegate: delegate, queue: queue)
        self.delegate = delegate
    }

    deinit {
        stop()
    }

    private func takeDelegateForStop() -> VideoSampleStreamDelegate? {
        lock.lock()
        defer { lock.unlock() }
        guard !stopped else { return nil }
        stopped = true
        let delegate = self.delegate
        self.delegate = nil
        return delegate
    }

    func stop() {
        guard let delegate = takeDelegateForStop() else { return }
        outputBox.removeStreamDelegate(owner: self, delegate: delegate, queue: queue)
    }

    fileprivate func emitSample(_ sampleBuffer: CMSampleBuffer) {
        let callbackOwner = self.callbackOwner
        guard let ctx = callbackOwner.userData else { return }
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        let pixelOpaque = CMSampleBufferGetImageBuffer(sampleBuffer).map { Unmanaged.passRetained($0).toOpaque() }
        callback(ctx, sampleOpaque, pixelOpaque)
    }

    fileprivate func noteDropped(_ sampleBuffer: CMSampleBuffer) {
        _ = outputBox.recordDroppedSample(sampleBuffer)
    }
}

private final class VideoDataOutputEventStreamDelegate: NSObject,
    AVCaptureVideoDataOutputSampleBufferDelegate
{
    private weak var owner: VideoDataOutputEventStreamBridge?

    init(owner: VideoDataOutputEventStreamBridge) {
        self.owner = owner
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitSample(sampleBuffer)
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didDrop sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitDropped(sampleBuffer)
    }
}

private final class VideoDataOutputEventStreamBridge: NSObject {
    private let outputBox: VideoDataOutputBox
    private let callback: AVCVideoDataOutputEventCallback
    private let callbackOwner: AVCCallbackContextOwner
    private let queue: AVCSerialCallbackQueue
    private let lock = NSLock()
    private var delegate: VideoDataOutputEventStreamDelegate?
    private var stopped = false

    init(
        outputBox: VideoDataOutputBox,
        queueLabel: String,
        callback: @escaping AVCVideoDataOutputEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        queue = AVCSerialCallbackQueue(label: queueLabel)
        super.init()
        let delegate = VideoDataOutputEventStreamDelegate(owner: self)
        try outputBox.installStreamDelegate(owner: self, delegate: delegate, queue: queue)
        self.delegate = delegate
    }

    deinit {
        stop()
    }

    func stop() {
        lock.lock()
        guard !stopped, let delegate else {
            lock.unlock()
            return
        }
        stopped = true
        self.delegate = nil
        lock.unlock()
        outputBox.removeStreamDelegate(owner: self, delegate: delegate, queue: queue)
    }

    fileprivate func emitSample(_ sampleBuffer: CMSampleBuffer) {
        let callbackOwner = self.callbackOwner
        guard let ctx = callbackOwner.userData else { return }
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        let pixelOpaque = CMSampleBufferGetImageBuffer(sampleBuffer).map {
            Unmanaged.passRetained($0).toOpaque()
        }
        callback(ctx, 0, sampleOpaque, pixelOpaque, nil, 0)
    }

    fileprivate func emitDropped(_ sampleBuffer: CMSampleBuffer) {
        let callbackOwner = self.callbackOwner
        guard let ctx = callbackOwner.userData else { return }
        let (reason, total) = outputBox.recordDroppedSample(sampleBuffer)
        callback(
            ctx,
            1,
            Unmanaged.passRetained(sampleBuffer).toOpaque(),
            nil,
            reason.flatMap(ffiString),
            total
        )
    }
}

private final class AudioSampleStreamDelegate: NSObject, AVCaptureAudioDataOutputSampleBufferDelegate {
    private weak var owner: AudioSampleStreamBridge?

    init(owner: AudioSampleStreamBridge) {
        self.owner = owner
    }

    func captureOutput(
        _ output: AVCaptureOutput,
        didOutput sampleBuffer: CMSampleBuffer,
        from connection: AVCaptureConnection
    ) {
        owner?.emitSample(sampleBuffer)
    }
}

private final class AudioSampleStreamBridge: NSObject {
    private let outputBox: AudioDataOutputBox
    private let callback: AVCAudioSampleCallback
    private let callbackOwner: AVCCallbackContextOwner
    private let queue: AVCSerialCallbackQueue
    private let lock = NSLock()
    private var delegate: AudioSampleStreamDelegate?
    private var stopped = false

    init(
        outputBox: AudioDataOutputBox,
        queueLabel: String,
        callback: @escaping AVCAudioSampleCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        queue = AVCSerialCallbackQueue(label: queueLabel)
        super.init()
        let delegate = AudioSampleStreamDelegate(owner: self)
        try outputBox.installStreamDelegate(owner: self, delegate: delegate, queue: queue)
        self.delegate = delegate
    }

    deinit {
        stop()
    }

    func stop() {
        lock.lock()
        guard !stopped, let delegate else {
            lock.unlock()
            return
        }
        stopped = true
        self.delegate = nil
        lock.unlock()
        outputBox.removeStreamDelegate(owner: self, delegate: delegate, queue: queue)
    }

    fileprivate func emitSample(_ sampleBuffer: CMSampleBuffer) {
        let callbackOwner = self.callbackOwner
        guard let ctx = callbackOwner.userData else { return }
        let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
        callback(ctx, sampleOpaque)
    }
}

private func avcFileRecordingKind(_ kind: String) -> Int32? {
    switch kind {
    case "started": return 0
    case "paused": return 1
    case "resumed": return 2
    case "willFinish": return 3
    case "finished": return 4
    default: return nil
    }
}

private final class FileRecordingStreamBridge: NSObject {
    private let outputBox: MovieFileOutputBox
    private let callbackOwner: AVCCallbackContextOwner
    fileprivate var operation: AVCFileRecordingOperation?

    init(
        outputBox: MovieFileOutputBox,
        outputPath: String,
        overwritePolicy: AVCRecordingOverwritePolicy,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        let callbackOwner = self.callbackOwner
        operation = try outputBox.startRecording(
            to: outputPath,
            overwritePolicy: overwritePolicy
        ) { kind, fileURL, error in
            guard let kind = avcFileRecordingKind(kind) else { return }
            avcEmitStreamEvent(
                callback,
                kind: kind,
                payload: FileRecordingStreamPayload(
                    fileURL: fileURL.path,
                    error: error?.localizedDescription
                ),
                owner: callbackOwner
            )
        }
    }

    deinit {
        stop()
    }

    func stop() {
        operation?.requestStop()
    }
}

private final class AudioFileRecordingStreamBridge: NSObject {
    private let outputBox: AudioFileOutputBox
    private let callbackOwner: AVCCallbackContextOwner
    fileprivate var operation: AVCFileRecordingOperation?

    init(
        outputBox: AudioFileOutputBox,
        outputPath: String,
        outputFileType: String,
        overwritePolicy: AVCRecordingOverwritePolicy,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        let callbackOwner = self.callbackOwner
        operation = try outputBox.startRecording(
            to: outputPath,
            outputFileType: outputFileType,
            overwritePolicy: overwritePolicy
        ) { kind, fileURL, error in
            guard let kind = avcFileRecordingKind(kind) else { return }
            avcEmitStreamEvent(
                callback,
                kind: kind,
                payload: FileRecordingStreamPayload(
                    fileURL: fileURL.path,
                    error: error?.localizedDescription
                ),
                owner: callbackOwner
            )
        }
    }

    deinit {
        stop()
    }

    func stop() {
        operation?.requestStop()
    }
}

private final class MovieFileBoundaryStreamBridge: NSObject {
    private let outputBox: MovieFileOutputBox
    private let callbackOwner: AVCCallbackContextOwner
    private var delegate: AsyncFileOutputBoundaryDelegate?

    init(
        outputBox: MovieFileOutputBox,
        callback: @escaping AVCAudioSampleCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        let callbackOwner = self.callbackOwner
        let delegate = AsyncFileOutputBoundaryDelegate { sampleBuffer in
            guard let ctx = callbackOwner.userData else { return }
            let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
            callback(ctx, sampleOpaque)
        }
        self.delegate = delegate
        try outputBox.installBoundaryStream(owner: self, delegate: delegate)
    }

    deinit {
        stop()
    }

    func stop() {
        guard let delegate else { return }
        outputBox.removeBoundaryStream(owner: self, delegate: delegate)
        self.delegate = nil
    }
}

private final class AudioFileBoundaryStreamBridge: NSObject {
    private let outputBox: AudioFileOutputBox
    private let callbackOwner: AVCCallbackContextOwner
    private var delegate: AsyncFileOutputBoundaryDelegate?

    init(
        outputBox: AudioFileOutputBox,
        callback: @escaping AVCAudioSampleCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        super.init()
        let callbackOwner = self.callbackOwner
        let delegate = AsyncFileOutputBoundaryDelegate { sampleBuffer in
            guard let ctx = callbackOwner.userData else { return }
            let sampleOpaque = Unmanaged.passRetained(sampleBuffer).toOpaque()
            callback(ctx, sampleOpaque)
        }
        self.delegate = delegate
        try outputBox.installBoundaryStream(owner: self, delegate: delegate)
    }

    deinit {
        stop()
    }

    func stop() {
        guard let delegate else { return }
        outputBox.removeBoundaryStream(owner: self, delegate: delegate)
        self.delegate = nil
    }
}

@available(macOS 13.0, *)
private final class MetadataObjectsStreamDelegate: NSObject, AVCaptureMetadataOutputObjectsDelegate {
    private weak var owner: MetadataObjectsStreamBridge?

    init(owner: MetadataObjectsStreamBridge) {
        self.owner = owner
    }

    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput metadataObjects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        owner?.emit(metadataObjects)
    }
}

@available(macOS 13.0, *)
private final class MetadataObjectsStreamBridge: NSObject {
    private let outputBox: MetadataOutputBox
    private let callback: AVCStreamEventCallback
    private let callbackOwner: AVCCallbackContextOwner
    private let queue: AVCSerialCallbackQueue
    private let lock = NSLock()
    private var delegate: MetadataObjectsStreamDelegate?
    private var stopped = false

    init(
        outputBox: MetadataOutputBox,
        queueLabel: String,
        callback: @escaping AVCStreamEventCallback,
        ctx: UnsafeMutableRawPointer,
        releaseCtx: AVCDropCallback?
    ) throws {
        self.outputBox = outputBox
        self.callback = callback
        callbackOwner = AVCCallbackContextOwner(
            userData: ctx,
            retainUserData: nil,
            releaseUserData: releaseCtx
        )
        queue = AVCSerialCallbackQueue(label: queueLabel)
        super.init()
        let delegate = MetadataObjectsStreamDelegate(owner: self)
        try outputBox.installStreamDelegate(owner: self, delegate: delegate, queue: queue)
        self.delegate = delegate
    }

    deinit {
        stop()
    }

    func stop() {
        lock.lock()
        guard !stopped, let delegate else {
            lock.unlock()
            return
        }
        stopped = true
        self.delegate = nil
        lock.unlock()
        outputBox.removeStreamDelegate(owner: self, delegate: delegate, queue: queue)
    }

    fileprivate func emit(_ metadataObjects: [AVMetadataObject]) {
        let callbackOwner = self.callbackOwner
        let payload = AsyncMetadataObjectsPayload(objects: metadataObjects.map { object in
            AsyncMetadataObjectPayload(
                objectType: object.type.rawValue,
                stringValue: (object as? AVMetadataMachineReadableCodeObject)?.stringValue,
                bounds: CaptureRectPayload(object.bounds)
            )
        })
        avcEmitStreamEvent(callback, kind: 0, payload: payload, owner: callbackOwner)
    }
}

@_cdecl("avcapture_session_running_subscribe")
public func avcapture_session_running_subscribe(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    return avcRetain(
        SessionRunningStreamBridge(
            sessionBox: avcSessionBox(sessionPtr),
            callback: onEvent,
            ctx: ctx,
            releaseCtx: nil
        )
    )
}

@_cdecl("avcapture_session_running_subscribe_owned")
public func avcapture_session_running_subscribe_owned(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        return nil
    }
    return avcRetain(
        SessionRunningStreamBridge(
            sessionBox: avcSessionBox(sessionPtr),
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
    )
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
    return avcRetain(
        SessionErrorStreamBridge(
            sessionBox: avcSessionBox(sessionPtr),
            callback: onEvent,
            ctx: ctx,
            releaseCtx: nil
        )
    )
}

@_cdecl("avcapture_session_error_subscribe_owned")
public func avcapture_session_error_subscribe_owned(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        return nil
    }
    return avcRetain(
        SessionErrorStreamBridge(
            sessionBox: avcSessionBox(sessionPtr),
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
    )
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
    return avcRetain(
        SessionInterruptionStreamBridge(
            sessionBox: avcSessionBox(sessionPtr),
            callback: onEvent,
            ctx: ctx,
            releaseCtx: nil
        )
    )
}

@_cdecl("avcapture_session_interruption_subscribe_owned")
public func avcapture_session_interruption_subscribe_owned(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        return nil
    }
    return avcRetain(
        SessionInterruptionStreamBridge(
            sessionBox: avcSessionBox(sessionPtr),
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
    )
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
    let outputBox = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        return avcRetain(
            try VideoSampleStreamBridge(
                outputBox: outputBox,
                queueLabel: queueLabel,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        return nil
    }
}

@_cdecl("avcapture_video_sample_subscribe_owned")
public func avcapture_video_sample_subscribe_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCVideoSampleCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing video sample callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        let bridge = try VideoSampleStreamBridge(
            outputBox: outputBox,
            queueLabel: queueLabel,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_CALLBACK_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_video_sample_unsubscribe")
public func avcapture_video_sample_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    avcUnretained(handle, as: VideoSampleStreamBridge.self).stop()
    avcRelease(handle, as: VideoSampleStreamBridge.self)
}

@_cdecl("avcapture_video_data_output_event_subscribe")
public func avcapture_video_data_output_event_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCVideoDataOutputEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing video data-output event callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: VideoDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        let bridge = try VideoDataOutputEventStreamBridge(
            outputBox: outputBox,
            queueLabel: queueLabel,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_CALLBACK_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_video_data_output_event_unsubscribe")
public func avcapture_video_data_output_event_unsubscribe(
    _ handle: UnsafeMutableRawPointer?
) {
    guard let handle else { return }
    avcUnretained(handle, as: VideoDataOutputEventStreamBridge.self).stop()
    avcRelease(handle, as: VideoDataOutputEventStreamBridge.self)
}

@_cdecl("avcapture_audio_sample_subscribe")
public func avcapture_audio_sample_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    let outputBox = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        return avcRetain(
            try AudioSampleStreamBridge(
                outputBox: outputBox,
                queueLabel: queueLabel,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        return nil
    }
}

@_cdecl("avcapture_audio_sample_subscribe_owned")
public func avcapture_audio_sample_subscribe_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing audio sample callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: AudioDataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        let bridge = try AudioSampleStreamBridge(
            outputBox: outputBox,
            queueLabel: queueLabel,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_CALLBACK_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_audio_sample_unsubscribe")
public func avcapture_audio_sample_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    avcUnretained(handle, as: AudioSampleStreamBridge.self).stop()
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
        return avcRetain(
            try FileRecordingStreamBridge(
                outputBox: outputBox,
                outputPath: String(cString: pathPtr),
                overwritePolicy: .failIfExists,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_file_recording_stream_start_with_options_owned")
public func avcapture_file_recording_stream_start_with_options_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ pathBytes: UnsafePointer<UInt8>,
    _ pathLength: Int,
    _ overwritePolicyRaw: Int32,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    var releaseContextOnExit = true
    defer {
        if releaseContextOnExit {
            releaseCtx?(ctx)
        }
    }
    guard let onEvent else {
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing file recording callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        guard let overwritePolicy = AVCRecordingOverwritePolicy(rawValue: overwritePolicyRaw) else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "unsupported recording overwrite policy: \(overwritePolicyRaw)"
            )
        }
        let outputPath = try avcRecordingPath(pathBytes, length: pathLength)
        releaseContextOnExit = false
        let bridge = try FileRecordingStreamBridge(
            outputBox: outputBox,
            outputPath: outputPath,
            overwritePolicy: overwritePolicy,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_OUTPUT_ERROR)
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

@_cdecl("avcapture_file_recording_stream_request_stop")
public func avcapture_file_recording_stream_request_stop(
    _ handle: UnsafeMutableRawPointer?
) -> Bool {
    guard let handle else { return false }
    let bridge = avcUnretained(handle, as: FileRecordingStreamBridge.self)
    return bridge.operation?.requestStop() ?? false
}

@_cdecl("avcapture_audio_file_recording_stream_start")
public func avcapture_audio_file_recording_stream_start(
    _ outputPtr: UnsafeMutableRawPointer,
    _ pathPtr: UnsafePointer<CChar>,
    _ outputFileTypePtr: UnsafePointer<CChar>,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        outErrorMessage?.pointee = ffiString("missing audio file recording callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        return avcRetain(
            try AudioFileRecordingStreamBridge(
                outputBox: outputBox,
                outputPath: String(cString: pathPtr),
                outputFileType: String(cString: outputFileTypePtr),
                overwritePolicy: .failIfExists,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_audio_file_recording_stream_start_with_options_owned")
public func avcapture_audio_file_recording_stream_start_with_options_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ pathBytes: UnsafePointer<UInt8>,
    _ pathLength: Int,
    _ outputFileTypePtr: UnsafePointer<CChar>,
    _ overwritePolicyRaw: Int32,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    var releaseContextOnExit = true
    defer {
        if releaseContextOnExit {
            releaseCtx?(ctx)
        }
    }
    guard let onEvent else {
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing audio file recording callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        guard let overwritePolicy = AVCRecordingOverwritePolicy(rawValue: overwritePolicyRaw) else {
            throw BridgeError.status(
                AVC_INVALID_ARGUMENT,
                "unsupported recording overwrite policy: \(overwritePolicyRaw)"
            )
        }
        let outputPath = try avcRecordingPath(pathBytes, length: pathLength)
        let outputFileType = String(cString: outputFileTypePtr)
        releaseContextOnExit = false
        let bridge = try AudioFileRecordingStreamBridge(
            outputBox: outputBox,
            outputPath: outputPath,
            outputFileType: outputFileType,
            overwritePolicy: overwritePolicy,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_OUTPUT_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_audio_file_recording_stream_stop")
public func avcapture_audio_file_recording_stream_stop(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    let bridge = avcUnretained(handle, as: AudioFileRecordingStreamBridge.self)
    bridge.stop()
    avcRelease(handle, as: AudioFileRecordingStreamBridge.self)
}

@_cdecl("avcapture_audio_file_recording_stream_request_stop")
public func avcapture_audio_file_recording_stream_request_stop(
    _ handle: UnsafeMutableRawPointer?
) -> Bool {
    guard let handle else { return false }
    let bridge = avcUnretained(handle, as: AudioFileRecordingStreamBridge.self)
    return bridge.operation?.requestStop() ?? false
}

@_cdecl("avcapture_movie_file_boundary_subscribe")
public func avcapture_movie_file_boundary_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    let outputBox = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        return avcRetain(
            try MovieFileBoundaryStreamBridge(
                outputBox: outputBox,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        return nil
    }
}

@_cdecl("avcapture_movie_file_boundary_subscribe_owned")
public func avcapture_movie_file_boundary_subscribe_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing movie boundary callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: MovieFileOutputBox.self)
    do {
        let bridge = try MovieFileBoundaryStreamBridge(
            outputBox: outputBox,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_CALLBACK_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_movie_file_boundary_unsubscribe")
public func avcapture_movie_file_boundary_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    avcUnretained(handle, as: MovieFileBoundaryStreamBridge.self).stop()
    avcRelease(handle, as: MovieFileBoundaryStreamBridge.self)
}

@_cdecl("avcapture_audio_file_boundary_subscribe")
public func avcapture_audio_file_boundary_subscribe(
    _ outputPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer
) -> UnsafeMutableRawPointer? {
    guard let onEvent else { return nil }
    let outputBox = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        return avcRetain(
            try AudioFileBoundaryStreamBridge(
                outputBox: outputBox,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        return nil
    }
}

@_cdecl("avcapture_audio_file_boundary_subscribe_owned")
public func avcapture_audio_file_boundary_subscribe_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ onEvent: AVCAudioSampleCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing audio boundary callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: AudioFileOutputBox.self)
    do {
        let bridge = try AudioFileBoundaryStreamBridge(
            outputBox: outputBox,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_CALLBACK_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("avcapture_audio_file_boundary_unsubscribe")
public func avcapture_audio_file_boundary_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    avcUnretained(handle, as: AudioFileBoundaryStreamBridge.self).stop()
    avcRelease(handle, as: AudioFileBoundaryStreamBridge.self)
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
    let outputBox = avcUnretained(outputPtr, as: MetadataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        return avcRetain(
            try MetadataObjectsStreamBridge(
                outputBox: outputBox,
                queueLabel: queueLabel,
                callback: onEvent,
                ctx: ctx,
                releaseCtx: nil
            )
        )
    } catch {
        return nil
    }
}

@available(macOS 13.0, *)
@_cdecl("avcapture_metadata_objects_subscribe_owned")
public func avcapture_metadata_objects_subscribe_owned(
    _ outputPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ onEvent: AVCStreamEventCallback?,
    _ ctx: UnsafeMutableRawPointer,
    _ releaseCtx: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard let onEvent else {
        releaseCtx?(ctx)
        outStatus.pointee = AVC_CALLBACK_ERROR
        outErrorMessage?.pointee = ffiString("missing metadata objects callback")
        return nil
    }
    let outputBox = avcUnretained(outputPtr, as: MetadataOutputBox.self)
    let queueLabel = String(cString: queueLabelPtr)
    do {
        let bridge = try MetadataObjectsStreamBridge(
            outputBox: outputBox,
            queueLabel: queueLabel,
            callback: onEvent,
            ctx: ctx,
            releaseCtx: releaseCtx
        )
        outStatus.pointee = AVC_OK
        return avcRetain(bridge)
    } catch {
        outStatus.pointee = avcStatus(for: error, default: AVC_CALLBACK_ERROR)
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@available(macOS 13.0, *)
@_cdecl("avcapture_metadata_objects_unsubscribe")
public func avcapture_metadata_objects_unsubscribe(_ handle: UnsafeMutableRawPointer?) {
    guard let handle else { return }
    avcUnretained(handle, as: MetadataObjectsStreamBridge.self).stop()
    avcRelease(handle, as: MetadataObjectsStreamBridge.self)
}
