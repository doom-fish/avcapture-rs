# avcapture-rs

Safe Rust bindings for Apple's `AVCapture` stack on macOS.

## 0.2.1 highlights

- `AVCaptureDevice` enumeration, authorization, lookup, format inspection, configuration locking, and typed exposure / flash / torch modes.
- `AVCaptureDeviceDiscoverySession` wrappers for type/media/position-based discovery.
- `AVCaptureDeviceInput`, `AVCaptureScreenInput`, generic input/port inspection, and `AVCaptureVideoPreviewLayer` preview inspection.
- `AVCaptureSession` presets plus session-level `AVCaptureConnection` inspection.
- `AVCaptureVideoDataOutput` and `AVCaptureAudioDataOutput` with Rust closure callbacks.
- `AVCapturePhotoOutput`, `PhotoSettings`, `Photo`, `PhotoQualityPrioritization`, `AVCaptureMovieFileOutput`, and `AVCaptureMetadataOutput` wrappers plus delegate-to-closure callback bridges.
- Headless-safe numbered examples and per-area tests.

See [`COVERAGE.md`](COVERAGE.md) for the detailed surface map.

## Installation

```bash
cargo add avcapture
```

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

## Notes

- `MetadataOutput::new()` requires macOS 13.0 or newer at runtime.
- `PhotoOutput` capability arrays are often empty until the output is attached to a session with a video source.
- `PhotoSettings` flash-mode and quality-prioritization controls require macOS 13.0 or newer at runtime.
- `VideoPreviewLayer` may not expose a connection until its session has an eligible video input.
- `MovieFileOutput` recording controls and callbacks are exposed, but the bundled examples intentionally stop short of running a session and writing files.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
