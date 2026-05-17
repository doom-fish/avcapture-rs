# Changelog

## [0.3.0] - 2026-05-17

### Added
- `async` feature gate with `src/async_api.rs` module
- `SessionRunningStream` — KVO `AVCaptureSession.isRunning` as async stream
- `SessionErrorStream` — `runtimeErrorNotification` as async stream
- `SessionInterruptionStream` — `wasInterruptedNotification` / `interruptionEndedNotification` as async stream
- `VideoSampleBufferStream` — `AVCaptureVideoDataOutputSampleBufferDelegate` as async stream
- `AudioSampleBufferStream` — `AVCaptureAudioDataOutputSampleBufferDelegate` as async stream
- `FileRecordingStream` — `AVCaptureFileOutputRecordingDelegate` lifecycle as async stream
- `MetadataObjectsStream` — `AVCaptureMetadataOutputObjectsDelegate` as async stream
- `doom-fish-utils` dependency (executor-agnostic `BoundedAsyncStream<T>`)
- Example `14_async_session_streams`

## 0.2.2

- Closed the remaining macOS audit gaps across `AVCapturePhoto*`, `AVCaptureAudioPreviewOutput`, `AVCaptureAudioFileOutput`, `AVCaptureAudioChannel`, session controls, Desk View / external-display, and `AVCaptureTimecode*`.
- Added typed dropped-sample reasons, base file-output sample-buffer-boundary callbacks, `ResolvedPhotoSettings`, and `PhotoOutputReadinessCoordinator` callback support.
- Expanded the public crate-root/prelude exports, refreshed numbered examples through `13_display_timecode`, and updated coverage documentation for full audited macOS top-level symbol coverage.

## 0.2.1

- Added `CaptureExposureMode` plus `AVCapturePhotoSettings` / `AVCapturePhoto` wrappers and settings-based photo capture.
- Added `PhotoQualityPrioritization` support on `PhotoOutput` and `PhotoSettings`.
- Added a safe `VideoPreviewLayer` wrapper with preview-layer inspection and video-gravity control.
- Added headless-safe preview-layer example/test coverage and refreshed audit/coverage documentation.

## 0.2.0

- Split the Swift bridge and Rust FFI into per-area modules.
- Added safe wrappers for device discovery, device formats, screen input, generic input/output inspection, session connections, and photo/movie/metadata outputs.
- Added numbered headless-safe examples plus per-area integration tests.
- Added `COVERAGE.md` and refreshed crate documentation.

## 0.1.0

- Initial `AVCaptureSession` / `AVCaptureDeviceInput` / data-output bindings.
- Device enumeration and authorization-status helpers.
- Video / audio data-output configuration with Rust closures for sample-buffer callbacks.
- No-permission-prompt smoke example for surface validation.
