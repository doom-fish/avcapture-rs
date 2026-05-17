# Coverage

`avcapture` targets macOS and keeps its examples/tests headless-safe by avoiding `startRunning` during validation.

As of `avcapture` 0.2.2, every audited macOS-available top-level symbol from `AVCapture*.h` is covered by the crate's Rust surface and Swift bridge. See [`COVERAGE_AUDIT.md`](COVERAGE_AUDIT.md) for the symbol-by-symbol audit.

| Area | Status | Rust surface | Swift bridge | Notes |
| --- | --- | --- | --- | --- |
| Devices | ✅ | `CaptureDevice`, `CaptureDeviceDetails`, `CaptureAutoFocusSystem`, `CaptureFocusMode`, `CaptureWhiteBalanceMode`, `CaptureColorSpace`, `CaptureMicrophoneMode`, `CaptureCenterStageControlMode`, `CaptureReactionType`, `CaptureReactionEffectState`, `CaptureDeviceInputSource`, `CaptureDeviceRotationCoordinator` | `Device.swift` | Enumeration, lookup, active format/frame-duration inspection, configuration lock, focus / white-balance / torch controls, input sources, rotation coordination, reaction helpers, Center Stage / microphone modes, and notification constants. |
| Device discovery / formats / position | ✅ | `CaptureDeviceDiscoverySession`, `CaptureDeviceFormat`, `CaptureDevicePosition` | `DeviceDiscoverySession.swift`, `DeviceFormat.swift`, `DevicePosition.swift` | Discovery plus format-description and frame-rate inspection. |
| Generic inputs / ports | ✅ | `CaptureInputRef`, `CaptureInputInfo`, `CaptureInputPortInfo` | `Input.swift` | Shared input and port inspection for capture graphs. |
| Device input | ✅ | `DeviceInput`, `CaptureMultichannelAudioMode` | `DeviceInput.swift` | Safe creation from an `AVCaptureDevice`, multichannel-audio mode access, wind-noise removal, and input-port format-description notification constant. |
| Screen input | ✅ | `ScreenInput`, `ScreenInputInfo` | `ScreenInput.swift` | Main-display / display-ID constructors plus property setters. |
| Sessions / controls | ✅ | `CaptureSession`, `CaptureSessionPreset`, `CaptureControl`, `CaptureIndexPicker`, `CaptureSlider`, `CaptureSystemExposureBiasSlider`, `CaptureSystemZoomSlider`, delegate callback events | `Session.swift`, `SessionControls.swift` | Generic add/remove helpers, session-connection access, notification constants, capture-control wrappers, and deferred-start/session-controls delegate callbacks. |
| Connections | ✅ | `CaptureConnection`, `CaptureAudioChannel` | `Connection.swift` | Enable/mirror controls, rotation, frame-duration inspection, and audio-channel inspection/mutation. |
| Generic outputs | ✅ | `CaptureOutputRef`, `CaptureOutputInfo`, `CaptureOutputDataDroppedReason` | `Output.swift` | Shared connection inspection, media-type lookup, deferred-start info, and typed dropped-sample reasons. |
| Video data output | ✅ | `VideoDataOutput`, `VideoOutputSettings`, `CaptureTimecode*` helpers | `VideoDataOutput.swift`, `Timecode.swift` | Video settings, closure callbacks, dropped-sample inspection, and the `AVCaptureTimecode*` family. |
| Audio data / preview outputs | ✅ | `AudioDataOutput`, `AudioPreviewOutput`, `AudioOutputSettings` | `AudioDataOutput.swift` | Audio settings, closure callbacks, dropped-sample inspection, and audio-preview output control. |
| Photo output | ✅ | `PhotoOutput`, `PhotoOutputReadinessCoordinator`, `PhotoOutputCaptureReadiness`, `PhotoSettings`, `ResolvedPhotoSettings`, `Photo`, `PhotoQualityPrioritization` | `Photo.swift`, `PhotoOutput.swift` | Capability inspection, typed settings, readiness callbacks, resolved-settings inspection, and delegate-to-closure photo capture. |
| Movie / audio file outputs | ✅ | `MovieFileOutput`, `AudioFileOutput`, recording events, sample-buffer-boundary callbacks | `MovieFileOutput.swift` | Recording controls, recording-event callbacks, base file-output delegate bridging, fragment-interval / spatial-video setters, and audio-file output support. |
| Metadata output | ✅ | `MetadataOutput`, `MetadataOutputInfo`, `MetadataObjectsEvent` | `MetadataOutput.swift` | Object-type inspection/configuration, rect-of-interest setters, and delegate-to-closure metadata bridging. |
| Video preview layer / display | ✅ | `VideoPreviewLayer`, `DeskViewApplication`, `ExternalDisplayConfiguration`, `ExternalDisplayConfigurator` | `VideoPreviewLayer.swift`, `DeskViewApplication.swift`, `ExternalDisplay.swift` | Session-backed preview-layer creation, geometry conversion helpers, Desk View integration, and external-display configuration/runtime helpers. |
| Examples / tests | ✅ | `examples/01`-`13`, `tests/*.rs` | n/a | Every logical area has at least one numbered example and at least one integration test. |

## Deferred / skipped Apple SDK rows

The remaining Apple SDK rows are exempt because they are unavailable or deprecated on macOS rather than genuinely uncovered. The largest exempt groups are:

1. ⏭️ iOS / tvOS-only synchronizer, depth-data, aspect-ratio, and framing APIs.
2. ⏭️ Deprecated replacement-only symbols such as legacy device-type constants, `AVCaptureStillImageOutput`, and `AVCaptureVideoOrientation`.
3. ⏭️ macOS-unavailable photo extras such as bracketed capture, deferred photo proxies, and photo file-data customizers.
4. ⏭️ Other symbols explicitly marked `API_UNAVAILABLE(macos)` or deprecated in the audited SDK headers.
