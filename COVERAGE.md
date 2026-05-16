# Coverage

`avcapture` targets macOS and keeps its examples/tests headless-safe by avoiding `startRunning` during validation.

| Area | Status | Rust surface | Swift bridge | Notes |
| --- | --- | --- | --- | --- |
| Devices | ✅ | `CaptureDevice`, `MediaType`, `AuthorizationStatus`, `CaptureDeviceDetails`, `CaptureExposureMode` | `Device.swift` | Enumeration, lookup, active format/frame-duration inspection, configuration lock, and typed exposure / flash / torch mode access. |
| Device discovery | ✅ | `CaptureDeviceDiscoverySession` | `DeviceDiscoverySession.swift` | Discovery by device type, media type, and position. |
| Device formats | ✅ | `CaptureDeviceFormat`, `CaptureDeviceFormatInfo` | `DeviceFormat.swift` | Frame-rate ranges and format-description inspection; `highResolutionStillImageDimensions` is unavailable on macOS and is reported as `None`. |
| Device position | ✅ | `CaptureDevicePosition` | `DevicePosition.swift` | Typed conversion helpers for the `AVCaptureDevicePosition` enum. |
| Generic inputs / ports | ✅ | `CaptureInputRef`, `CaptureInputInfo`, `CaptureInputPortInfo` | `Input.swift` | Shared port/media-type inspection for capture inputs. |
| Device input | ✅ | `DeviceInput`, `DeviceInputInfo` | `DeviceInput.swift` | Safe creation from an `AVCaptureDevice`. |
| Screen input | ✅ | `ScreenInput`, `ScreenInputInfo` | `ScreenInput.swift` | Main-display / display-ID constructors plus property setters. |
| Sessions | ✅ | `CaptureSession`, `CaptureSessionPreset`, `CaptureSessionInfo` | `Session.swift` | Generic add/remove input/output helpers and session-connection access. |
| Connections | ✅ | `CaptureConnection`, `CaptureConnectionInfo` | `Connection.swift` | Enable/mirror controls, rotation, and frame-duration inspection. |
| Generic outputs | ✅ | `CaptureOutputRef`, `CaptureOutputInfo` | `Output.swift` | Shared connection inspection and media-type lookup. |
| Video data output | ✅ | `VideoDataOutput`, `VideoOutputSettings` | `VideoDataOutput.swift` | Settings plus Rust closure sample-buffer callback. |
| Audio data output | ✅ | `AudioDataOutput`, `AudioOutputSettings` | `AudioDataOutput.swift` | Settings plus Rust closure sample-buffer callback. |
| Photo output | ✅ | `PhotoOutput`, `PhotoOutputInfo`, `PhotoCaptureResult`, `PhotoCaptureEvent`, `PhotoSettings`, `PhotoSettingsInfo`, `Photo`, `PhotoInfo`, `PhotoQualityPrioritization` | `Photo.swift` / `PhotoOutput.swift` | Capability inspection, typed settings, quality-prioritization control, and delegate-to-closure photo capture with photo metadata inspection. |
| Movie file output | ✅ | `MovieFileOutput`, `MovieFileOutputInfo`, `MovieRecordingEvent` | `MovieFileOutput.swift` | Recording controls, fragment-interval / spatial-video setters, and recording-event callback bridge. |
| Metadata output | ✅ | `MetadataOutput`, `MetadataOutputInfo`, `MetadataObjectsEvent` | `MetadataOutput.swift` | Object-type inspection/configuration, rect-of-interest setters, and delegate-to-closure metadata bridge. |
| Video preview layer | ✅ | `VideoPreviewLayer`, `VideoPreviewLayerInfo` | `VideoPreviewLayer.swift` | Session-backed preview layer creation, connection inspection, and video-gravity control. |
| Examples / tests | ✅ | `examples/01`-`11`, `tests/*.rs` | n/a | Every logical area has at least one numbered example and one integration test. |

## Deferred / skipped Apple SDK rows

The table above reflects the macOS capture surface covered by `avcapture` 0.2.1. The following SDK slices remain intentionally deferred and are called out here instead of being omitted silently:

1. ⏭️ `AVCaptureSession` multi-cam, interruption-management, and capture-control APIs — iOS / Catalyst oriented or notification-heavy surfaces outside this crate's headless-safe macOS scope.
2. ⏭️ `AVCaptureDevice` focus / white-balance / zoom / depth / reaction-effect tuning APIs — mostly iOS-only camera-control surfaces that are not part of the requested capture-session abstraction.
3. ⏭️ `AVCapturePhotoOutput` Live Photo, depth / portrait / semantic-segmentation, bracketed capture, readiness-coordinator, and resolved-settings types — unavailable on macOS or substantially larger than the current photo-capture subset.
4. ⏭️ `AVCaptureMovieFileOutput` per-connection encoder dictionaries and metadata-collection mutation APIs — safe file-recording controls are wrapped, but low-level encoder tuning is still deferred.
5. ⏭️ `AVCaptureMetadataOutput` transformed-metadata helpers and cinematic-video required-metadata APIs — adjacent newer APIs not needed for the current capture graph.
