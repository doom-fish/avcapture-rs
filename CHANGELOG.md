# Changelog

## [0.6.0] - 2026-09-07

### Added

- Retained `Photo::pixel_buffer()` and owned `Photo::file_data_representation()` accessors for captured macOS photo data.
- `VideoDataOutputEvent` delivery for native samples and `didDrop` callbacks, including a total dropped-frame count and optional native reason.
- Additive `VideoDataOutputEventStream` support while preserving the existing sample-only `VideoSampleBufferEvent` fields and `VideoSampleBufferStream::subscribe` API.
- Restored the original three-argument `VideoSampleCallback` ABI and legacy video callback/subscription symbols; sample-and-drop delivery now uses separately named additive callback types and exports.
- Restored every pre-existing async bridge export to its HEAD parameter count, ordering, types, and caller-owned context ABI. Safe Rust now uses additive `_owned` or `_with_options_owned` exports for retained contexts, typed status, byte paths, and recording policies.
- `RecordingOptions` / `RecordingOverwritePolicy`, async `stop_and_finalize()`, callback diagnostics, and preview-layer native handle, frame, bounds, layout, and host-layer APIs.

### Changed (breaking)

- Sealed `CaptureInputRef` and `CaptureOutputRef`; all retained Swift bridge-box adoption constructors are now crate-private and unsafe with exact +1 box contracts.
- Migrated retained `CMSampleBuffer` and `CVPixelBuffer` adoption to the final
  explicit unsafe `apple-cf` ownership contract while preserving the existing
  retained-object behavior for callers.
- Rust/Swift JSON now uses a versioned envelope and explicit acronym keys/legacy aliases.
- Delegate-backed handlers and streams now use exclusive identity-checked registration and return `DelegateSlotOccupied` instead of replacing an existing owner.
- Photo capture retains its bridge completion owner; dropping a future cancels the waiter while native capture continues to final cleanup. `PhotoSettings` is single-use for capture.
- Recording now stages output in the destination directory and atomically finalizes it. Existing paths are preserved by default; explicit overwrite is limited to regular files.
- Preview-layer APIs are main-thread-only and operate on caller-owned layer hierarchies.
- Raised in-family requirements to `apple-cf >=0.10, <0.11` and
  `doom-fish-utils >=0.4, <0.5`.

### Fixed

- Prevented async callback context use-after-free and callback-queue self-deadlocks during stream teardown or callback-triggered drop.
- Ensured owned movie/audio recording exports release transferred callback contexts exactly once when callback, overwrite-policy, or path validation fails before bridge construction.
- Serialized synchronously reentrant photo-readiness callbacks without recursive mutable closure aliasing.
- Corrected `fileURL`, `outputFileURL`, `outputDeviceUniqueID`, `uniqueID`, `displayID`, and `availableVideoCVPixelFormatTypes` payload keys and stopped discarding callback decode failures.
- Validated manual mirroring, metadata type subsets, and single-use photo settings before native exception paths.
- Replaced the macOS video pixel-format empty stub with `availableVideoPixelFormatTypes`; raw-photo formats are explicitly unsupported on macOS.

## [0.5.0] - 2026-05-20

### Added

- `async_api` photo capture futures, photo-readiness streaming, audio file recording lifecycle streaming, and movie/audio file-output sample-buffer boundary streams.
- Async API coverage for the new photo and file-output wrappers.

### Notes

- Phase 32 completeness + async sweep.

## [0.4.10] - 2026-05-20

- Added in-`src/` unit tests across `camera_calibration_data`, `device`, `error`, and `video_data_output_timecode` (Tier 2 quality polish), providing fast `cargo test --lib` fail-fast signal alongside the existing integration tests under `tests/`.

## [0.4.9] - 2026-05-20

- Clippy hygiene sweep: cleared all `-D warnings` lints across the crate. No public API change.

## [0.4.8] - 2026-05-20

- Widen `doom-fish-utils` dependency bound to `<0.4` so the 0.3.x SPSC-ring release resolves cleanly. No source changes.

## [0.4.7] - 2026-05-19

- Bump MSRV from 1.70 to 1.76 to match fleet baseline.

## [0.4.6] - 2026-05-19

### Added
- Added `CameraCalibrationData` support with serde snapshots for camera intrinsics, extrinsics, pixel size, and lens-distortion lookup tables sourced from `AVCameraCalibrationData`.

## [0.4.5] - 2026-05-19

### Added
- Wrapped `AVExposureBiasRange`, `AVFrameRateRange`, and `AVZoomRange` through `CaptureDeviceFormat`, including recommended zoom / exposure-bias ranges and effect-specific frame-rate range inspection.

## [0.4.4] - 2026-05-18

### Changed
- Added rustdoc coverage across the safe `src/` surface, documenting public wrappers, enums, info snapshots, fields, and callback or stream helpers against their `AVCapture*` counterparts.

## [0.4.3] - 2026-05-18

- Widen apple-cf version bound to `<0.10` so 0.9.x resolves.

## [0.4.2] - 2026-05-18

### Changed
- Derived `Debug` for every public wrapper and async stream struct in the crate that can support it.

## [0.4.1] - 2026-05-18

### Changed
- Re-exported `DropCallback` from `doom-fish-utils::ffi_callbacks` and removed the duplicate local FFI typedef.

## [0.4.0] - 2026-05-20

### Changed
- Widened `apple-cf` to `>=0.4, <0.9` and aligned `CaptureRect` field access with the nested `origin`/`size` layout used by `apple-cf` 0.8.
- Kept `CaptureRect`'s bridge JSON shape flat (`x`, `y`, `width`, `height`) so the Swift bridge payload stays unchanged.

## [0.3.1] - 2026-05-20

### Fixed
- **Swift deinit race (use-after-free prevention)**: `VideoSampleStreamBridge` and
  `AudioSampleStreamBridge` now call `queue.sync {}` after
  `setSampleBufferDelegate(nil, nil)` in `deinit`. This drains any capture-queue
  callbacks that were enqueued before the delegate was cleared, ensuring the Rust
  `SenderBox` (`ctx` pointer) is never accessed after it has been freed.
- **Panic safety across FFI**: `video_sample_trampoline` and `audio_sample_trampoline`
  now wrap user closure invocations in
  `doom_fish_utils::panic_safe::catch_user_panic`. A panic in a user-supplied sample
  buffer handler previously had undefined behaviour (unwind across `extern "C"`).
- **SAFETY comments**: Added `// SAFETY:` documentation to all `unsafe` blocks and
  `unsafe impl` declarations in `async_api.rs`, `video_data_output.rs`, and
  `audio_data_output.rs`.
- **`Clone` doc on sample-buffer events**: `VideoSampleBufferEvent` and
  `AudioSampleBufferEvent` now document that `Clone` is a cheap `CFRetain`
  (reference-count increment), not a copy of pixel/audio data.
- **Cargo.toml**: Widened `doom-fish-utils` version range from `"0.1"` to
  `">=0.1, <0.3"` per workspace version-range convention.

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
