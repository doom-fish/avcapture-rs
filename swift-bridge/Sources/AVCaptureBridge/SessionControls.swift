import AVFoundation
import Foundation
import ObjectiveC

private var sessionControlsDelegateRegistrationKey: UInt8 = 0
private var sessionDeferredStartDelegateRegistrationKey: UInt8 = 0

struct CaptureSessionExtendedInfoPayload: Codable {
    let sessionPreset: String
    let inputCount: Int
    let outputCount: Int
    let connectionCount: Int
    let isRunning: Bool
    let supportsControls: Bool?
    let controlsCount: Int?
    let maxControlsCount: Int?
    let controlsDelegateInstalled: Bool?
    let manualDeferredStartSupported: Bool?
    let automaticallyRunsDeferredStart: Bool?
    let deferredStartDelegateInstalled: Bool?
}

private struct CaptureControlInfoPayload: Codable {
    let kind: String
    let enabled: Bool
    let localizedTitle: String?
    let symbolName: String?
    let accessibilityIdentifier: String?
}

private struct CaptureSliderInfoPayload: Codable {
    let kind: String
    let enabled: Bool
    let localizedTitle: String?
    let symbolName: String?
    let accessibilityIdentifier: String?
    let value: Float
    let minValue: Float?
    let maxValue: Float?
    let step: Float?
    let values: [Float]
    let prominentValues: [Float]
    let localizedValueFormat: String?
    let hasActionHandler: Bool
}

private struct CaptureIndexPickerInfoPayload: Codable {
    let kind: String
    let enabled: Bool
    let localizedTitle: String?
    let symbolName: String?
    let accessibilityIdentifier: String?
    let selectedIndex: Int
    let numberOfIndexes: Int
    let localizedIndexTitles: [String]
    let hasActionHandler: Bool
}

private struct CaptureSessionEventPayload: Codable {
    let kind: String
}

private struct SliderActionPayload: Codable {
    let value: Float
}

private struct IndexPickerActionPayload: Codable {
    let selectedIndex: Int
}

private final class SessionJsonCallbackBox {
    let callback: AVCJsonCallback
    let userData: UnsafeMutableRawPointer?
    let dropUserData: AVCDropCallback?
    private var disposed = false

    init(
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        self.callback = callback
        self.userData = userData
        self.dropUserData = dropUserData
    }

    deinit {
        dispose()
    }

    func emit<T: Encodable>(_ payload: T) {
        guard !disposed,
              let json = try? avcEncodeJSON(payload),
              let payloadPtr = ffiString(json)
        else {
            return
        }
        callback(userData, payloadPtr)
    }

    func dispose() {
        guard !disposed else { return }
        disposed = true
        if let userData, let dropUserData {
            dropUserData(userData)
        }
    }
}

private final class SessionActionQueue {
    let queue: DispatchQueue
    private let key = DispatchSpecificKey<UInt8>()

    init(label: String) {
        queue = DispatchQueue(label: label)
        queue.setSpecific(key: key, value: 1)
    }

    func sync(_ action: () -> Void) {
        if DispatchQueue.getSpecific(key: key) != nil {
            action()
        } else {
            queue.sync(execute: action)
        }
    }
}

@available(macOS 15.0, *)
private final class SessionControlsDelegateRegistration: NSObject {
    let delegate: SessionControlsDelegateBridge
    let callbackBox: SessionJsonCallbackBox
    let queue: DispatchQueue

    init(delegate: SessionControlsDelegateBridge, callbackBox: SessionJsonCallbackBox, queue: DispatchQueue) {
        self.delegate = delegate
        self.callbackBox = callbackBox
        self.queue = queue
    }
}

@available(macOS 26.0, *)
private final class SessionDeferredStartDelegateRegistration: NSObject {
    let delegate: SessionDeferredStartDelegateBridge
    let callbackBox: SessionJsonCallbackBox
    let queue: DispatchQueue

    init(
        delegate: SessionDeferredStartDelegateBridge,
        callbackBox: SessionJsonCallbackBox,
        queue: DispatchQueue
    ) {
        self.delegate = delegate
        self.callbackBox = callbackBox
        self.queue = queue
    }
}

@available(macOS 15.0, *)
private final class SessionControlsDelegateBridge: NSObject, AVCaptureSessionControlsDelegate {
    let callbackBox: SessionJsonCallbackBox

    init(callbackBox: SessionJsonCallbackBox) {
        self.callbackBox = callbackBox
    }

    func sessionControlsDidBecomeActive(_ session: AVCaptureSession) {
        callbackBox.emit(CaptureSessionEventPayload(kind: "didBecomeActive"))
    }

    func sessionControlsWillEnterFullscreenAppearance(_ session: AVCaptureSession) {
        callbackBox.emit(CaptureSessionEventPayload(kind: "willEnterFullscreenAppearance"))
    }

    func sessionControlsWillExitFullscreenAppearance(_ session: AVCaptureSession) {
        callbackBox.emit(CaptureSessionEventPayload(kind: "willExitFullscreenAppearance"))
    }

    func sessionControlsDidBecomeInactive(_ session: AVCaptureSession) {
        callbackBox.emit(CaptureSessionEventPayload(kind: "didBecomeInactive"))
    }
}

@available(macOS 26.0, *)
private final class SessionDeferredStartDelegateBridge: NSObject, AVCaptureSessionDeferredStartDelegate {
    let callbackBox: SessionJsonCallbackBox

    init(callbackBox: SessionJsonCallbackBox) {
        self.callbackBox = callbackBox
    }

    func sessionWillRunDeferredStart(_ session: AVCaptureSession) {
        callbackBox.emit(CaptureSessionEventPayload(kind: "willRunDeferredStart"))
    }

    func sessionDidRunDeferredStart(_ session: AVCaptureSession) {
        callbackBox.emit(CaptureSessionEventPayload(kind: "didRunDeferredStart"))
    }
}

@available(macOS 15.0, *)
private func avcControlsDelegateRegistration(
    for sessionBox: SessionBox
) -> SessionControlsDelegateRegistration? {
    objc_getAssociatedObject(sessionBox, &sessionControlsDelegateRegistrationKey)
        as? SessionControlsDelegateRegistration
}

@available(macOS 15.0, *)
private func avcSetControlsDelegateRegistration(
    _ registration: SessionControlsDelegateRegistration?,
    for sessionBox: SessionBox
) {
    objc_setAssociatedObject(
        sessionBox,
        &sessionControlsDelegateRegistrationKey,
        registration,
        .OBJC_ASSOCIATION_RETAIN_NONATOMIC
    )
}

@available(macOS 26.0, *)
private func avcDeferredStartDelegateRegistration(
    for sessionBox: SessionBox
) -> SessionDeferredStartDelegateRegistration? {
    objc_getAssociatedObject(sessionBox, &sessionDeferredStartDelegateRegistrationKey)
        as? SessionDeferredStartDelegateRegistration
}

@available(macOS 26.0, *)
private func avcSetDeferredStartDelegateRegistration(
    _ registration: SessionDeferredStartDelegateRegistration?,
    for sessionBox: SessionBox
) {
    objc_setAssociatedObject(
        sessionBox,
        &sessionDeferredStartDelegateRegistrationKey,
        registration,
        .OBJC_ASSOCIATION_RETAIN_NONATOMIC
    )
}

@available(macOS 26.0, *)
private func avcSessionSupportsDeferredStart(_ session: AVCaptureSession) -> Bool {
    session.isManualDeferredStartSupported || session.automaticallyRunsDeferredStart
}

func avcSessionInfoPayload(from sessionBox: SessionBox) -> CaptureSessionExtendedInfoPayload {
    let session = sessionBox.session

    let supportsControls: Bool?
    let controlsCount: Int?
    let maxControlsCount: Int?
    let controlsDelegateInstalled: Bool?
    if #available(macOS 15.0, *) {
        let supported = session.supportsControls
        supportsControls = supported
        if supported {
            controlsCount = session.controls.count
            maxControlsCount = Int(session.maxControlsCount)
            controlsDelegateInstalled = avcControlsDelegateRegistration(for: sessionBox) != nil
        } else {
            controlsCount = 0
            maxControlsCount = 0
            controlsDelegateInstalled = false
        }
    } else {
        supportsControls = nil
        controlsCount = nil
        maxControlsCount = nil
        controlsDelegateInstalled = nil
    }

    let manualDeferredStartSupported: Bool?
    let automaticallyRunsDeferredStart: Bool?
    let deferredStartDelegateInstalled: Bool?
    if #available(macOS 26.0, *) {
        manualDeferredStartSupported = session.isManualDeferredStartSupported
        automaticallyRunsDeferredStart = session.automaticallyRunsDeferredStart
        deferredStartDelegateInstalled = avcDeferredStartDelegateRegistration(for: sessionBox) != nil
    } else {
        manualDeferredStartSupported = nil
        automaticallyRunsDeferredStart = nil
        deferredStartDelegateInstalled = nil
    }

    return CaptureSessionExtendedInfoPayload(
        sessionPreset: avcEncodeSessionPreset(session.sessionPreset),
        inputCount: session.inputs.count,
        outputCount: session.outputs.count,
        connectionCount: session.connections.count,
        isRunning: session.isRunning,
        supportsControls: supportsControls,
        controlsCount: controlsCount,
        maxControlsCount: maxControlsCount,
        controlsDelegateInstalled: controlsDelegateInstalled,
        manualDeferredStartSupported: manualDeferredStartSupported,
        automaticallyRunsDeferredStart: automaticallyRunsDeferredStart,
        deferredStartDelegateInstalled: deferredStartDelegateInstalled
    )
}

@available(macOS 15.0, *)
private func avcControlKind(_ control: AVCaptureControl) -> String {
    if control is AVCaptureSystemZoomSlider {
        return "systemZoomSlider"
    }
    if control is AVCaptureSystemExposureBiasSlider {
        return "systemExposureBiasSlider"
    }
    if control is AVCaptureIndexPicker {
        return "indexPicker"
    }
    if control is AVCaptureSlider {
        return "slider"
    }
    return "control"
}

@available(macOS 15.0, *)
private func avcControlInfoPayload(from control: AVCaptureControl) -> CaptureControlInfoPayload {
    let localizedTitle: String?
    let symbolName: String?
    let accessibilityIdentifier: String?

    if let picker = control as? AVCaptureIndexPicker {
        localizedTitle = picker.localizedTitle
        symbolName = picker.symbolName
        accessibilityIdentifier = picker.accessibilityIdentifier
    } else if let slider = control as? AVCaptureSlider {
        localizedTitle = slider.localizedTitle
        symbolName = slider.symbolName
        accessibilityIdentifier = slider.accessibilityIdentifier
    } else {
        localizedTitle = nil
        symbolName = nil
        accessibilityIdentifier = nil
    }

    return CaptureControlInfoPayload(
        kind: avcControlKind(control),
        enabled: control.isEnabled,
        localizedTitle: localizedTitle,
        symbolName: symbolName,
        accessibilityIdentifier: accessibilityIdentifier
    )
}

@available(macOS 15.0, *)
private enum SliderConfiguration {
    case continuous(min: Float, max: Float)
    case stepped(min: Float, max: Float, step: Float)
    case values([Float])

    var minValue: Float? {
        switch self {
        case .continuous(let min, _), .stepped(let min, _, _):
            return min
        case .values(let values):
            return values.min()
        }
    }

    var maxValue: Float? {
        switch self {
        case .continuous(_, let max), .stepped(_, let max, _):
            return max
        case .values(let values):
            return values.max()
        }
    }

    var step: Float? {
        switch self {
        case .continuous:
            return nil
        case .stepped(_, _, let step):
            return step
        case .values:
            return nil
        }
    }

    var values: [Float] {
        switch self {
        case .continuous, .stepped:
            return []
        case .values(let values):
            return values
        }
    }

    func contains(_ value: Float) -> Bool {
        let tolerance: Float = 0.000_1
        switch self {
        case .continuous(let min, let max):
            return value.isFinite && value >= min && value <= max
        case .stepped(let min, let max, let step):
            guard value.isFinite, value >= min, value <= max else { return false }
            let multiple = (value - min) / step
            return abs(multiple.rounded() - multiple) <= tolerance
        case .values(let values):
            return values.contains { abs($0 - value) <= tolerance }
        }
    }
}

@available(macOS 15.0, *)
private class CaptureControlBoxBase: NSObject {
    let control: AVCaptureControl

    init(control: AVCaptureControl) {
        self.control = control
    }

    func canAddToSession() -> Bool {
        true
    }
}

@available(macOS 15.0, *)
private final class GenericCaptureControlBox: CaptureControlBoxBase {}

@available(macOS 15.0, *)
private final class CaptureIndexPickerBox: CaptureControlBoxBase {
    let picker: AVCaptureIndexPicker
    fileprivate var callbackBox: SessionJsonCallbackBox?
    fileprivate var actionQueue: SessionActionQueue?

    init(picker: AVCaptureIndexPicker) {
        self.picker = picker
        super.init(control: picker)
    }

    func infoPayload() -> CaptureIndexPickerInfoPayload {
        let controlInfo = avcControlInfoPayload(from: picker)
        return CaptureIndexPickerInfoPayload(
            kind: controlInfo.kind,
            enabled: controlInfo.enabled,
            localizedTitle: controlInfo.localizedTitle,
            symbolName: controlInfo.symbolName,
            accessibilityIdentifier: controlInfo.accessibilityIdentifier,
            selectedIndex: picker.selectedIndex,
            numberOfIndexes: picker.numberOfIndexes,
            localizedIndexTitles: picker.localizedIndexTitles,
            hasActionHandler: callbackBox != nil
        )
    }

    func setSelectedIndex(_ selectedIndex: Int) throws {
        guard selectedIndex >= 0, selectedIndex < picker.numberOfIndexes else {
            throw BridgeError.message("selected index \(selectedIndex) is outside 0..<\(picker.numberOfIndexes)")
        }
        if let actionQueue {
            actionQueue.sync {
                picker.selectedIndex = selectedIndex
            }
        } else {
            picker.selectedIndex = selectedIndex
        }
    }

    func setAction(
        queueLabel: String,
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        let actionQueue = SessionActionQueue(label: queueLabel)
        let callbackBox = SessionJsonCallbackBox(
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        picker.setActionQueue(actionQueue.queue) { selectedIndex in
            callbackBox.emit(IndexPickerActionPayload(selectedIndex: selectedIndex))
        }
        self.actionQueue = actionQueue
        self.callbackBox = callbackBox
    }

    func clearAction() {
        guard callbackBox != nil || actionQueue != nil else { return }
        let noopQueue = DispatchQueue(label: "avcapture-index-picker-noop")
        picker.setActionQueue(noopQueue) { _ in }
        callbackBox = nil
        actionQueue = nil
    }

    override func canAddToSession() -> Bool {
        callbackBox != nil
    }
}

@available(macOS 15.0, *)
private final class CaptureSliderBox: CaptureControlBoxBase {
    let slider: AVCaptureSlider
    let configuration: SliderConfiguration
    fileprivate var callbackBox: SessionJsonCallbackBox?
    fileprivate var actionQueue: SessionActionQueue?

    init(slider: AVCaptureSlider, configuration: SliderConfiguration) {
        self.slider = slider
        self.configuration = configuration
        super.init(control: slider)
    }

    func infoPayload() -> CaptureSliderInfoPayload {
        let controlInfo = avcControlInfoPayload(from: slider)
        return CaptureSliderInfoPayload(
            kind: controlInfo.kind,
            enabled: controlInfo.enabled,
            localizedTitle: controlInfo.localizedTitle,
            symbolName: controlInfo.symbolName,
            accessibilityIdentifier: controlInfo.accessibilityIdentifier,
            value: slider.value,
            minValue: configuration.minValue,
            maxValue: configuration.maxValue,
            step: configuration.step,
            values: configuration.values,
            prominentValues: slider.prominentValues,
            localizedValueFormat: slider.localizedValueFormat,
            hasActionHandler: callbackBox != nil
        )
    }

    func setValue(_ value: Float) throws {
        guard configuration.contains(value) else {
            throw BridgeError.message("slider value \(value) is not supported by this slider")
        }
        if let actionQueue {
            actionQueue.sync {
                slider.value = value
            }
        } else {
            slider.value = value
        }
    }

    func setAction(
        queueLabel: String,
        callback: @escaping AVCJsonCallback,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) {
        let actionQueue = SessionActionQueue(label: queueLabel)
        let callbackBox = SessionJsonCallbackBox(
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        slider.setActionQueue(actionQueue.queue) { newValue in
            callbackBox.emit(SliderActionPayload(value: newValue))
        }
        self.actionQueue = actionQueue
        self.callbackBox = callbackBox
    }

    func clearAction() {
        guard callbackBox != nil || actionQueue != nil else { return }
        let noopQueue = DispatchQueue(label: "avcapture-slider-noop")
        slider.setActionQueue(noopQueue) { _ in }
        callbackBox = nil
        actionQueue = nil
    }

    override func canAddToSession() -> Bool {
        callbackBox != nil
    }
}

@available(macOS 15.0, *)
private final class CaptureSystemExposureBiasSliderBox: CaptureControlBoxBase {
    fileprivate let callbackBox: SessionJsonCallbackBox?

    init(
        device: AVCaptureDevice,
        callback: AVCJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) throws {
        guard device.hasMediaType(.video) else {
            throw BridgeError.message("system exposure bias slider requires a video capture device")
        }
        let range = (device.activeFormat as AnyObject).value(forKey: "systemRecommendedExposureBiasRange")
        guard range != nil else {
            throw BridgeError.message("device \(device.localizedName) does not support system exposure bias controls")
        }

        let callbackBox = callback.map {
            SessionJsonCallbackBox(callback: $0, userData: userData, dropUserData: dropUserData)
        }
        let control: AVCaptureControl
        if let callbackBox {
            control = AVCaptureSystemExposureBiasSlider(device: device) { exposureTargetBias in
                callbackBox.emit(SliderActionPayload(value: exposureTargetBias))
            }
        } else {
            control = AVCaptureSystemExposureBiasSlider(device: device)
        }
        self.callbackBox = callbackBox
        super.init(control: control)
    }
}

@available(macOS 15.0, *)
private final class CaptureSystemZoomSliderBox: CaptureControlBoxBase {
    fileprivate let callbackBox: SessionJsonCallbackBox?

    init(
        device: AVCaptureDevice,
        callback: AVCJsonCallback?,
        userData: UnsafeMutableRawPointer?,
        dropUserData: AVCDropCallback?
    ) throws {
        guard device.hasMediaType(.video) else {
            throw BridgeError.message("system zoom slider requires a video capture device")
        }

        let callbackBox = callback.map {
            SessionJsonCallbackBox(callback: $0, userData: userData, dropUserData: dropUserData)
        }
        let control: AVCaptureControl
        if let callbackBox {
            control = AVCaptureSystemZoomSlider(device: device) { videoZoomFactor in
                callbackBox.emit(SliderActionPayload(value: Float(videoZoomFactor)))
            }
        } else {
            control = AVCaptureSystemZoomSlider(device: device)
        }
        self.callbackBox = callbackBox
        super.init(control: control)
    }
}

@available(macOS 15.0, *)
private func avcControlBox(_ ptr: UnsafeMutableRawPointer) -> CaptureControlBoxBase {
    avcUnretained(ptr, as: CaptureControlBoxBase.self)
}

@_cdecl("av_capture_session_controls_count")
public func av_capture_session_controls_count(_ sessionPtr: UnsafeMutableRawPointer) -> Int {
    guard #available(macOS 15.0, *) else { return 0 }
    let session = avcSessionBox(sessionPtr).session
    guard session.supportsControls else { return 0 }
    return session.controls.count
}

@_cdecl("av_capture_session_control_at_index")
public func av_capture_session_control_at_index(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    let session = avcSessionBox(sessionPtr).session
    guard session.supportsControls else {
        outErrorMessage?.pointee = ffiString("capture session controls are not supported on this system")
        return nil
    }
    guard index >= 0, index < session.controls.count else {
        outErrorMessage?.pointee = ffiString("session control index out of range")
        return nil
    }
    return avcRetain(GenericCaptureControlBox(control: session.controls[index]))
}

@_cdecl("av_capture_session_can_add_control")
public func av_capture_session_can_add_control(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ controlPtr: UnsafeMutableRawPointer
) -> Bool {
    guard #available(macOS 15.0, *) else { return false }
    let session = avcSessionBox(sessionPtr).session
    guard session.supportsControls else { return false }
    let controlBox = avcControlBox(controlPtr)
    guard controlBox.canAddToSession() else { return false }
    return session.canAddControl(controlBox.control)
}

@_cdecl("av_capture_session_add_control")
public func av_capture_session_add_control(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ controlPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return AVC_SESSION_ERROR
    }
    let session = avcSessionBox(sessionPtr).session
    guard session.supportsControls else {
        outErrorMessage?.pointee = ffiString("capture session controls are not supported on this system")
        return AVC_SESSION_ERROR
    }
    let controlBox = avcControlBox(controlPtr)
    guard controlBox.canAddToSession() else {
        outErrorMessage?.pointee = ffiString(
            "capture sliders and index pickers must have an action handler before being added to a session"
        )
        return AVC_CALLBACK_ERROR
    }
    guard session.canAddControl(controlBox.control) else {
        outErrorMessage?.pointee = ffiString("session cannot add capture control")
        return AVC_SESSION_ERROR
    }
    session.addControl(controlBox.control)
    return AVC_OK
}

@_cdecl("av_capture_session_remove_control")
public func av_capture_session_remove_control(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ controlPtr: UnsafeMutableRawPointer
) {
    guard #available(macOS 15.0, *) else { return }
    let session = avcSessionBox(sessionPtr).session
    guard session.supportsControls else { return }
    session.removeControl(avcControlBox(controlPtr).control)
}

@_cdecl("av_capture_session_set_controls_delegate_callback")
public func av_capture_session_set_controls_delegate_callback(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture session controls delegate requires macOS 15.0 or newer")
        return AVC_SESSION_ERROR
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing session controls delegate callback")
        return AVC_CALLBACK_ERROR
    }
    let sessionBox = avcSessionBox(sessionPtr)
    let session = sessionBox.session
    guard session.supportsControls else {
        outErrorMessage?.pointee = ffiString("capture session controls are not supported on this system")
        return AVC_SESSION_ERROR
    }

    av_capture_session_clear_controls_delegate_callback(sessionPtr)

    let queue = DispatchQueue(label: String(cString: queueLabelPtr))
    let callbackBox = SessionJsonCallbackBox(
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    let delegate = SessionControlsDelegateBridge(callbackBox: callbackBox)
    session.setControlsDelegate(delegate, queue: queue)
    avcSetControlsDelegateRegistration(
        SessionControlsDelegateRegistration(delegate: delegate, callbackBox: callbackBox, queue: queue),
        for: sessionBox
    )
    return AVC_OK
}

@_cdecl("av_capture_session_clear_controls_delegate_callback")
public func av_capture_session_clear_controls_delegate_callback(_ sessionPtr: UnsafeMutableRawPointer) {
    guard #available(macOS 15.0, *) else { return }
    let sessionBox = avcSessionBox(sessionPtr)
    let session = sessionBox.session
    if session.supportsControls {
        session.setControlsDelegate(nil, queue: nil)
    }
    avcSetControlsDelegateRegistration(nil, for: sessionBox)
}

@_cdecl("av_capture_session_set_deferred_start_delegate_callback")
public func av_capture_session_set_deferred_start_delegate_callback(
    _ sessionPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 26.0, *) else {
        outErrorMessage?.pointee = ffiString("capture session deferred start delegate requires macOS 26.0 or newer")
        return AVC_SESSION_ERROR
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing session deferred start delegate callback")
        return AVC_CALLBACK_ERROR
    }
    let sessionBox = avcSessionBox(sessionPtr)
    let session = sessionBox.session
    guard avcSessionSupportsDeferredStart(session) else {
        outErrorMessage?.pointee = ffiString(
            "capture session deferred start is not supported by the current session"
        )
        return AVC_SESSION_ERROR
    }

    av_capture_session_clear_deferred_start_delegate_callback(sessionPtr)

    let queue = DispatchQueue(label: String(cString: queueLabelPtr))
    let callbackBox = SessionJsonCallbackBox(
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    let delegate = SessionDeferredStartDelegateBridge(callbackBox: callbackBox)
    session.setDeferredStartDelegate(delegate, deferredStartDelegateCallbackQueue: queue)
    avcSetDeferredStartDelegateRegistration(
        SessionDeferredStartDelegateRegistration(delegate: delegate, callbackBox: callbackBox, queue: queue),
        for: sessionBox
    )
    return AVC_OK
}

@_cdecl("av_capture_session_clear_deferred_start_delegate_callback")
public func av_capture_session_clear_deferred_start_delegate_callback(_ sessionPtr: UnsafeMutableRawPointer) {
    guard #available(macOS 26.0, *) else { return }
    let sessionBox = avcSessionBox(sessionPtr)
    if avcSessionSupportsDeferredStart(sessionBox.session) {
        sessionBox.session.setDeferredStartDelegate(nil, deferredStartDelegateCallbackQueue: nil)
    }
    avcSetDeferredStartDelegateRegistration(nil, for: sessionBox)
}

@_cdecl("av_capture_control_release")
public func av_capture_control_release(_ controlPtr: UnsafeMutableRawPointer?) {
    guard #available(macOS 15.0, *), let controlPtr else { return }
    avcRelease(controlPtr, as: CaptureControlBoxBase.self)
}

@_cdecl("av_capture_control_info_json")
public func av_capture_control_info_json(
    _ controlPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    do {
        return ffiString(try avcEncodeJSON(avcControlInfoPayload(from: avcControlBox(controlPtr).control)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_control_set_enabled")
public func av_capture_control_set_enabled(_ controlPtr: UnsafeMutableRawPointer, _ enabled: Bool) {
    guard #available(macOS 15.0, *) else { return }
    avcControlBox(controlPtr).control.isEnabled = enabled
}

@_cdecl("av_capture_index_picker_create")
public func av_capture_index_picker_create(
    _ localizedTitlePtr: UnsafePointer<CChar>,
    _ symbolNamePtr: UnsafePointer<CChar>,
    _ numberOfIndexes: Int,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    guard numberOfIndexes > 0 else {
        outStatus?.pointee = AVC_INVALID_ARGUMENT
        outErrorMessage?.pointee = ffiString("numberOfIndexes must be greater than 0")
        return nil
    }
    let picker = AVCaptureIndexPicker(
        String(cString: localizedTitlePtr),
        symbolName: String(cString: symbolNamePtr),
        numberOfIndexes: numberOfIndexes
    )
    outStatus?.pointee = AVC_OK
    return avcRetain(CaptureIndexPickerBox(picker: picker))
}

@_cdecl("av_capture_index_picker_create_with_titles_json")
public func av_capture_index_picker_create_with_titles_json(
    _ localizedTitlePtr: UnsafePointer<CChar>,
    _ symbolNamePtr: UnsafePointer<CChar>,
    _ localizedIndexTitlesJson: UnsafePointer<CChar>,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    do {
        let titles = try avcDecodeJSON(localizedIndexTitlesJson, as: [String].self)
        guard !titles.isEmpty else {
            outStatus?.pointee = AVC_INVALID_ARGUMENT
            outErrorMessage?.pointee = ffiString("localizedIndexTitles must not be empty")
            return nil
        }
        let picker = AVCaptureIndexPicker(
            String(cString: localizedTitlePtr),
            symbolName: String(cString: symbolNamePtr),
            localizedIndexTitles: titles
        )
        outStatus?.pointee = AVC_OK
        return avcRetain(CaptureIndexPickerBox(picker: picker))
    } catch {
        outStatus?.pointee = AVC_INVALID_ARGUMENT
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_index_picker_info_json")
public func av_capture_index_picker_info_json(
    _ controlPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureIndexPickerBox else {
        outErrorMessage?.pointee = ffiString("control is not an AVCaptureIndexPicker")
        return nil
    }
    do {
        return ffiString(try avcEncodeJSON(controlBox.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_index_picker_set_selected_index")
public func av_capture_index_picker_set_selected_index(
    _ controlPtr: UnsafeMutableRawPointer,
    _ selectedIndex: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return AVC_SESSION_ERROR
    }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureIndexPickerBox else {
        outErrorMessage?.pointee = ffiString("control is not an AVCaptureIndexPicker")
        return AVC_INVALID_ARGUMENT
    }
    do {
        try controlBox.setSelectedIndex(selectedIndex)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_index_picker_set_action_callback")
public func av_capture_index_picker_set_action_callback(
    _ controlPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return AVC_SESSION_ERROR
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing index picker action callback")
        return AVC_CALLBACK_ERROR
    }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureIndexPickerBox else {
        outErrorMessage?.pointee = ffiString("control is not an AVCaptureIndexPicker")
        return AVC_INVALID_ARGUMENT
    }
    controlBox.setAction(
        queueLabel: String(cString: queueLabelPtr),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return AVC_OK
}

@_cdecl("av_capture_index_picker_clear_action_callback")
public func av_capture_index_picker_clear_action_callback(_ controlPtr: UnsafeMutableRawPointer) {
    guard #available(macOS 15.0, *) else { return }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureIndexPickerBox else { return }
    controlBox.clearAction()
}

@_cdecl("av_capture_slider_create")
public func av_capture_slider_create(
    _ localizedTitlePtr: UnsafePointer<CChar>,
    _ symbolNamePtr: UnsafePointer<CChar>,
    _ minValue: Float,
    _ maxValue: Float,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    guard minValue < maxValue else {
        outStatus?.pointee = AVC_INVALID_ARGUMENT
        outErrorMessage?.pointee = ffiString("minValue must be less than maxValue")
        return nil
    }
    let slider = AVCaptureSlider(
        __localizedTitle: String(cString: localizedTitlePtr),
        symbolName: String(cString: symbolNamePtr),
        minValue: minValue,
        maxValue: maxValue
    )
    outStatus?.pointee = AVC_OK
    return avcRetain(CaptureSliderBox(slider: slider, configuration: .continuous(min: minValue, max: maxValue)))
}

@_cdecl("av_capture_slider_create_with_step")
public func av_capture_slider_create_with_step(
    _ localizedTitlePtr: UnsafePointer<CChar>,
    _ symbolNamePtr: UnsafePointer<CChar>,
    _ minValue: Float,
    _ maxValue: Float,
    _ step: Float,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    guard minValue < maxValue else {
        outStatus?.pointee = AVC_INVALID_ARGUMENT
        outErrorMessage?.pointee = ffiString("minValue must be less than maxValue")
        return nil
    }
    guard step > 0 else {
        outStatus?.pointee = AVC_INVALID_ARGUMENT
        outErrorMessage?.pointee = ffiString("step must be greater than 0")
        return nil
    }
    let slider = AVCaptureSlider(
        __localizedTitle: String(cString: localizedTitlePtr),
        symbolName: String(cString: symbolNamePtr),
        minValue: minValue,
        maxValue: maxValue,
        step: step
    )
    outStatus?.pointee = AVC_OK
    return avcRetain(
        CaptureSliderBox(slider: slider, configuration: .stepped(min: minValue, max: maxValue, step: step))
    )
}

@_cdecl("av_capture_slider_create_with_values_json")
public func av_capture_slider_create_with_values_json(
    _ localizedTitlePtr: UnsafePointer<CChar>,
    _ symbolNamePtr: UnsafePointer<CChar>,
    _ valuesJson: UnsafePointer<CChar>,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    do {
        let values = try avcDecodeJSON(valuesJson, as: [Float].self)
        guard !values.isEmpty else {
            outStatus?.pointee = AVC_INVALID_ARGUMENT
            outErrorMessage?.pointee = ffiString("values must not be empty")
            return nil
        }
        guard values.allSatisfy(\.isFinite) else {
            outStatus?.pointee = AVC_INVALID_ARGUMENT
            outErrorMessage?.pointee = ffiString("values must all be finite")
            return nil
        }
        let slider = AVCaptureSlider(
            __localizedTitle: String(cString: localizedTitlePtr),
            symbolName: String(cString: symbolNamePtr),
            values: values.map { NSNumber(value: $0) }
        )
        outStatus?.pointee = AVC_OK
        return avcRetain(CaptureSliderBox(slider: slider, configuration: .values(values)))
    } catch {
        outStatus?.pointee = AVC_INVALID_ARGUMENT
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_slider_info_json")
public func av_capture_slider_info_json(
    _ controlPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureSliderBox else {
        outErrorMessage?.pointee = ffiString("control is not an AVCaptureSlider")
        return nil
    }
    do {
        return ffiString(try avcEncodeJSON(controlBox.infoPayload()))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_slider_set_value")
public func av_capture_slider_set_value(
    _ controlPtr: UnsafeMutableRawPointer,
    _ value: Float,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return AVC_SESSION_ERROR
    }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureSliderBox else {
        outErrorMessage?.pointee = ffiString("control is not an AVCaptureSlider")
        return AVC_INVALID_ARGUMENT
    }
    do {
        try controlBox.setValue(value)
        return AVC_OK
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return AVC_INVALID_ARGUMENT
    }
}

@_cdecl("av_capture_slider_set_action_callback")
public func av_capture_slider_set_action_callback(
    _ controlPtr: UnsafeMutableRawPointer,
    _ queueLabelPtr: UnsafePointer<CChar>,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    guard #available(macOS 15.0, *) else {
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return AVC_SESSION_ERROR
    }
    guard let callback else {
        outErrorMessage?.pointee = ffiString("missing slider action callback")
        return AVC_CALLBACK_ERROR
    }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureSliderBox else {
        outErrorMessage?.pointee = ffiString("control is not an AVCaptureSlider")
        return AVC_INVALID_ARGUMENT
    }
    controlBox.setAction(
        queueLabel: String(cString: queueLabelPtr),
        callback: callback,
        userData: userData,
        dropUserData: dropUserData
    )
    return AVC_OK
}

@_cdecl("av_capture_slider_clear_action_callback")
public func av_capture_slider_clear_action_callback(_ controlPtr: UnsafeMutableRawPointer) {
    guard #available(macOS 15.0, *) else { return }
    guard let controlBox = avcControlBox(controlPtr) as? CaptureSliderBox else { return }
    controlBox.clearAction()
}

@_cdecl("av_capture_system_exposure_bias_slider_create")
public func av_capture_system_exposure_bias_slider_create(
    _ devicePtr: UnsafeMutableRawPointer,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    do {
        let box = try CaptureSystemExposureBiasSliderBox(
            device: avcDeviceBox(devicePtr).device,
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        outStatus?.pointee = AVC_OK
        return avcRetain(box)
    } catch {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_system_zoom_slider_create")
public func av_capture_system_zoom_slider_create(
    _ devicePtr: UnsafeMutableRawPointer,
    _ callback: AVCJsonCallback?,
    _ userData: UnsafeMutableRawPointer?,
    _ dropUserData: AVCDropCallback?,
    _ outStatus: UnsafeMutablePointer<Int32>?,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    guard #available(macOS 15.0, *) else {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString("capture controls require macOS 15.0 or newer")
        return nil
    }
    do {
        let box = try CaptureSystemZoomSliderBox(
            device: avcDeviceBox(devicePtr).device,
            callback: callback,
            userData: userData,
            dropUserData: dropUserData
        )
        outStatus?.pointee = AVC_OK
        return avcRetain(box)
    } catch {
        outStatus?.pointee = AVC_SESSION_ERROR
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
