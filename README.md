# avcapture-rs

Safe Rust bindings for Apple's `AVCaptureSession`, `AVCaptureDeviceInput`, `AVCaptureVideoDataOutput`, and `AVCaptureAudioDataOutput` on macOS.

## Features

- Build and inspect `AVCaptureSession` graphs from Rust.
- Enumerate capture devices and read camera / microphone authorization status.
- Create `AVCaptureDeviceInput`s from discovered devices.
- Configure `AVCaptureVideoDataOutput` and `AVCaptureAudioDataOutput` settings.
- Receive live sample buffers as `apple-cf` `CMSampleBuffer` / `CVPixelBuffer` wrappers.

## Example

```rust,no_run
use avcapture::prelude::*;

fn main() -> Result<(), AVCaptureError> {
    let session = CaptureSession::new()?;
    session.begin_configuration();
    session.set_session_preset(&CaptureSessionPreset::High)?;

    let video_output = VideoDataOutput::new()?;
    video_output.set_video_settings(Some(&VideoOutputSettings::bgra()))?;
    video_output.set_sample_buffer_handler(Some("capture-video"), |sample, pixel_buffer| {
        println!("video samples: {}", sample.num_samples());
        if let Some(pixel_buffer) = pixel_buffer {
            println!("pixel buffer: {}x{}", pixel_buffer.width(), pixel_buffer.height());
        }
    })?;

    if session.can_add_video_data_output(&video_output) {
        session.add_video_data_output(&video_output)?;
    }

    session.commit_configuration();
    Ok(())
}
```

## Smoke test

```bash
cargo run --all-features --example 01_smoke_surface
```

The smoke example intentionally avoids starting capture or opening device inputs, so it should not trigger camera or microphone permission prompts.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
