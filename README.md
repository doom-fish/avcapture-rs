# avcapture-rs

Safe Rust bindings for Apple's `AVCapture` stack on macOS.

## Highlights

- Added the `async` feature's photo-capture futures plus executor-agnostic `BoundedAsyncStream<T>` wrappers for session state, sample buffers, movie/audio file-recording lifecycle, file-output sample-buffer boundaries, photo readiness, and metadata objects.
- `AVCaptureDevice` now covers focus / white-balance / autofocus / color-space state, torch-level constants, input sources, rotation coordinators, Center Stage / microphone modes, reactions, and related notification constants.
- Added `AVCaptureAudioPreviewOutput`, `AVCaptureAudioFileOutput`, `AVCaptureAudioChannel`, typed dropped-sample reasons, and base file-output sample-buffer-boundary callbacks.
- Added `AVCapturePhotoOutputReadinessCoordinator`, `ResolvedPhotoSettings`, session controls / deferred-start delegates, Desk View / external-display helpers, and the `AVCaptureTimecode*` family.
- `AVCaptureVideoPreviewLayer` includes geometry conversion, explicit borrowed/retained `CALayer` interop, frame/bounds/layout control, caller-owned host-layer attachment, and Desk View / external-display entry points.
- Recording destinations are never recursively deleted: the default is atomic no-overwrite finalization, with explicit opt-in replacement limited to regular files.
- Callback and stream delegate slots are exclusive and identity-checked. Competing registrations return `AVCaptureError::DelegateSlotOccupied` instead of silently replacing one another.
- Rust/Swift JSON uses a validated versioned envelope, preserves acronym keys such as `fileURL`, `outputFileURL`, and `outputDeviceUniqueID`, and reports callback decode failures through `take_callback_diagnostics()`.
- Photo capture retains completion ownership, treats `PhotoSettings` as single-use, and exposes retained pixel buffers plus the macOS-supported encoded file representation.
- Video output reports the native macOS pixel-format capability and delivers `didDrop` events with an always-incremented count and optional reason.
- Headless-safe numbered examples and per-area tests now cover examples `01` through `14`.

See [`COVERAGE.md`](COVERAGE.md) for the detailed surface map.

## Installation

```bash
cargo add avcapture
```

Enable executor-agnostic async future/stream adapters with:

```bash
cargo add avcapture --features async
```

## Async API

The `async` feature adds `avcapture::async_api`, which exposes runtime-agnostic `Future` wrappers for photo capture plus `BoundedAsyncStream<T>` adapters for session running/error/interruption state, video/audio sample buffers, movie/audio file recording lifecycle events, movie/audio file-output sample-buffer boundary callbacks, photo readiness, and metadata-object delivery. `VideoSampleBufferStream` preserves its sample-only event shape; `VideoDataOutputEventStream` is the opt-in stream for samples plus dropped-frame events. Native delegate-backed subscriptions have typed fallible entry points because each delegate slot has one exclusive owner. Recording streams provide `stop_and_finalize()` to await the native final callback.

## Example

```rust,no_run
use avcapture::prelude::*;

fn main() -> Result<(), AVCaptureError> {
    let session = CaptureSession::new()?;
    let video_output = VideoDataOutput::new()?;
    video_output.set_video_settings(Some(&VideoOutputSettings::bgra().with_dimensions(1280, 720)))?;

    session.begin_configuration();
    if session.can_set_session_preset(&CaptureSessionPreset::High)? {
        session.set_session_preset(&CaptureSessionPreset::High)?;
    }
    if session.can_add_video_data_output(&video_output) {
        session.add_video_data_output(&video_output)?;
    }
    session.commit_configuration();

    println!("session info: {:?}", session.info()?);
    Ok(())
}
```

## Headless-safe examples

These examples intentionally avoid `startRunning`, and only invoke photo/movie capture APIs through guarded paths that return descriptive errors when the outputs are not attached to a running session. They should still exit `0` on machines without camera/microphone permissions or capture hardware.

- `cargo run --example 01_smoke_surface`
- `cargo run --example 02_device_discovery_session`
- `cargo run --example 03_device_formats`
- `cargo run --example 04_device_input_ports`
- `cargo run --example 05_screen_input`
- `cargo run --example 06_session_connections`
- `cargo run --example 07_data_outputs`
- `cargo run --example 08_photo_output`
- `cargo run --example 09_movie_file_output`
- `cargo run --example 10_metadata_output`
- `cargo run --example 11_video_preview_layer`
- `cargo run --example 12_session_controls`
- `cargo run --example 13_display_timecode`
- `cargo run --example 14_async_session_streams --features async`

## Notes

- `MetadataOutput::new()` requires macOS 13.0 or newer at runtime.
- `PhotoOutput` capability arrays are often empty until the output is attached to a session with a video source.
- Raw-photo pixel-format capability is represented as `None` on macOS rather than as a fabricated empty list.
- `PhotoSettings` flash-mode and quality-prioritization controls require macOS 13.0 or newer at runtime.
- A `PhotoSettings` value can be submitted for capture once. Use `copy_with_unique_id()` for another request.
- `Photo::pixel_buffer()` returns a retained `CVPixelBuffer`; `Photo::file_data_representation()` returns owned encoded bytes when the native photo provides them.
- Recording uses `RecordingOptions::default()` for no-overwrite behavior. `RecordingOptions::overwrite_regular_file()` is the explicit opt-in replacement policy.
- `VideoPreviewLayer` operations are main-thread-only. The crate exposes the layer but does not own or create a window/view hierarchy; callers attach it to their own `CALayer`.
- `VideoPreviewLayer` may not expose a connection until its session has an eligible video input, and geometry conversion requires meaningful non-zero bounds.
- Newer surfaces such as session controls, Desk View / external-display helpers, and timecode generation are runtime-gated and return descriptive errors on unsupported macOS releases.
- The bundled examples remain headless-safe and intentionally avoid `startRunning` or writing capture files unless the API itself can report the unsupported/not-attached state safely.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
