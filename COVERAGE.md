# Coverage

`avcapture` targets macOS 12 and later and keeps its examples headless-safe by avoiding `startRunning` during validation.

[`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) and [`COVERAGE_AUDIT_V2.md`](COVERAGE_AUDIT_V2.md) count top-level declarations (classes, protocols, enums, constants) from the `AVCapture*.h` headers of the macOS 26.2 SDK. Their 100% figures mean every macOS-available declaration has a Rust counterpart; they do not certify every property or method of those classes. For example, `AVCaptureDevice` was counted as verified while `requestAccessForMediaType:` was missing (added in 0.7.0). The audits were not regenerated against the installed 26.5 and 27.0 SDKs; see the sections below for what is known to be missing.

| Area | Status | Rust surface | Swift bridge | Notes |
| --- | --- | --- | --- | --- |
| Devices | ✅ | `CaptureDevice`, `CaptureDeviceDetails`, `CaptureAutoFocusSystem`, `CaptureFocusMode`, `CaptureWhiteBalanceMode`, `CaptureColorSpace`, `CaptureMicrophoneMode`, `CaptureCenterStageControlMode`, `CaptureReactionType`, `CaptureReactionEffectState`, `CaptureDeviceInputSource`, `CaptureDeviceRotationCoordinator` | `Device.swift` | Enumeration, lookup, authorization status and consent requests (`CaptureDevice::request_access` with a timeout, `async_api::RequestAccessFuture`), active format/frame-duration inspection and configuration, configuration lock, focus / white-balance / torch controls, input sources, rotation coordination, reaction helpers, Center Stage / microphone modes, and notification constants. Setters check the documented preconditions (format of this device, frame duration inside a supported range, supported color space, torch level range) and return `InvalidArgument` instead of letting AVFoundation raise. |
| Device discovery / formats / position | ✅ | `CaptureDeviceDiscoverySession`, `CaptureDeviceFormat`, `CaptureDevicePosition` | `DeviceDiscoverySession.swift`, `DeviceFormat.swift`, `DevicePosition.swift` | Discovery plus format-description and frame-rate inspection. |
| Generic inputs / ports | ✅ | `CaptureInputRef`, `CaptureInputInfo`, `CaptureInputPortInfo` | `Input.swift` | Sealed shared input and port inspection for crate-owned live bridge boxes. |
| Device input | ✅ | `DeviceInput`, `CaptureMultichannelAudioMode` | `DeviceInput.swift` | Safe creation from an `AVCaptureDevice`, multichannel-audio mode access, wind-noise removal, and input-port format-description notification constant. |
| Screen input | ✅ | `ScreenInput`, `ScreenInputInfo` | `ScreenInput.swift` | Main-display / display-ID constructors plus property setters. |
| Sessions / controls | ✅ | `CaptureSession`, `CaptureSessionPreset`, `CaptureControl`, `CaptureIndexPicker`, `CaptureSlider`, `CaptureSystemExposureBiasSlider`, `CaptureSystemZoomSlider`, delegate callback events | `Session.swift`, `SessionControls.swift` | Generic add/remove helpers, session-connection access, notification constants, capture-control wrappers, and deferred-start/session-controls delegate callbacks. Configuration blocks are tracked: start/stop inside one and unmatched commits return `InvalidState`. On the main thread, start/stop are scheduled on the session's serial queue instead of blocking. |
| Connections | ✅ | `CaptureConnection`, `CaptureAudioChannel` | `Connection.swift` | Enable/mirror controls, rotation, frame-duration inspection, and audio-channel inspection/mutation. |
| Generic outputs | ✅ | `CaptureOutputRef`, `CaptureOutputInfo`, `CaptureOutputDataDroppedReason` | `Output.swift` | Sealed connection inspection, media-type lookup, deferred-start info, and typed dropped-sample reasons. |
| Video data output | ✅ | `VideoDataOutput`, `VideoSampleBufferStream`, `VideoDataOutputEventStream`, `VideoDataOutputEvent`, `VideoOutputSettings`, `CaptureTimecode*` helpers | `VideoDataOutput.swift`, `Timecode.swift` | Source-compatible sample-only streaming plus opt-in sample/`didDrop` events, real macOS pixel-format capabilities, counted drop diagnostics, and the `AVCaptureTimecode*` family. |
| Audio data / preview outputs | ✅ | `AudioDataOutput`, `AudioPreviewOutput`, `AudioOutputSettings` | `AudioDataOutput.swift` | Audio settings, closure callbacks, dropped-sample inspection, and audio-preview output control. |
| Photo output | ✅ | `PhotoOutput`, `PhotoOutputReadinessCoordinator`, `PhotoOutputCaptureReadiness`, `PhotoSettings`, `ResolvedPhotoSettings`, `Photo`, `PhotoQualityPrioritization` | `Photo.swift`, `PhotoOutput.swift` | Single-use settings, serialized readiness callbacks, retained pixel buffers, owned encoded bytes, resolved settings, and capture completion ownership. Settings are checked against the output's supported flash modes and maximum quality prioritization before capture and before readiness tracking. |
| Movie / audio file outputs | ✅ | `MovieFileOutput`, `AudioFileOutput`, `RecordingOptions`, recording events, sample-buffer-boundary callbacks | `MovieFileOutput.swift` | Atomic staged finalization, default no-overwrite, regular-file-only explicit replacement, finalization-aware async stop, and identity-checked delegates. |
| Metadata output | ✅ | `MetadataOutput`, `MetadataOutputInfo`, `MetadataObjectsEvent` | `MetadataOutput.swift` | Object-type inspection/configuration, rect-of-interest setters, and delegate-to-closure metadata bridging. |
| Video preview layer / display | ✅ | `VideoPreviewLayer`, `RetainedNativeLayer`, `DeskViewApplication`, `ExternalDisplayConfiguration`, `ExternalDisplayConfigurator` | `VideoPreviewLayer.swift`, `DeskViewApplication.swift`, `ExternalDisplay.swift` | Main-thread frame/bounds/layout, borrowed/retained `CALayer` interop, caller-owned hosting, geometry conversion, Desk View, and external-display helpers. |
| Examples / tests | ✅ | `examples/01`-`14`, `tests/*.rs` | n/a | Logical areas include unit or synthetic contract coverage without requiring active capture. |

## Known gaps in the macOS 26 surface

These macOS-available members are not wrapped yet, although their classes count as verified in the audits: `AVCaptureDevice` `autoVideoFrameRateEnabled`, `focusPointOfInterest` / `exposurePointOfInterest` and the macOS 26 rect-of-interest variants, `centerStageRectOfInterest`, `fallbackPrimaryConstituentDevices`, `connected` / `inUseByAnotherApplication` / `suspended`, `companionDeskViewCamera`, and the portrait-effect / studio-light / background-replacement state; `AVCaptureDeviceInput` locked frame durations, external sync and cinematic video capture; `AVCaptureAudioDataOutput.spatialAudioChannelLayoutTag`; and the photo output's zero-shutter-lag and fast-capture-prioritization switches.

## macOS 27 SDK additions (not wrapped)

The macOS 27.0 SDK adds capture APIs that this crate does not wrap yet:

- `AVCaptureBroadcastVideoOutput` and its dropped-frame replacement policy.
- `AVCaptureAncillaryDataEncoder` and the `AVCaptureAncillaryDataUserKey` constants.
- Low-light video noise reduction on sessions and connections (`lowLightVideoNoiseReductionSupported`, `automaticallyEnablesLowLightVideoNoiseReduction`, `lowLightVideoNoiseReductionEnabled`).
- `AVCaptureDevice` continuous autofocus tracking (`continuousAutoFocusTrackingEnabled`, `continuousAutoFocusTrackingLensPositionBias`, `continuousAutoFocusTrackingSubjectAcquired`, and the format's `continuousAutoFocusTrackingSupported`), `setPrimaryConstituentDeviceSwitchingBehaviorLockedWithDevice:` with its support flag, and `adjustingSignalCompensationDelayWhileRunningSupported`.
- `AVCaptureFileOutput` `usesProVideoStorage` / `proVideoStorageSupported` and cinematic video metadata capture.

## Deferred / skipped Apple SDK rows

The remaining Apple SDK rows are exempt because they are unavailable or deprecated on macOS rather than genuinely uncovered. The largest exempt groups are:

1. ⏭️ iOS / tvOS-only synchronizer, depth-data, aspect-ratio, and framing APIs.
2. ⏭️ Deprecated replacement-only symbols such as legacy device-type constants, `AVCaptureStillImageOutput`, and `AVCaptureVideoOrientation`.
3. ⏭️ macOS-unavailable photo extras such as bracketed capture, deferred photo proxies, raw-photo format enumeration, and photo file-data customizers.
4. ⏭️ Other symbols explicitly marked `API_UNAVAILABLE(macos)` or deprecated in the audited SDK headers.
