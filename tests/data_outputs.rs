mod common;

use avcapture::prelude::*;

#[test]
fn data_outputs_smoke() -> common::TestResult {
    let video_output = VideoDataOutput::new()?;
    video_output
        .set_video_settings(Some(&VideoOutputSettings::bgra().with_dimensions(640, 480)))?;
    video_output
        .set_sample_buffer_handler(Some("avcapture-test-video"), |_sample, _pixel_buffer| {})?;
    assert!(video_output.callback_installed()?);
    video_output.clear_sample_buffer_handler();
    assert!(!video_output.callback_installed()?);

    let audio_output = AudioDataOutput::new()?;
    audio_output.set_audio_settings(Some(&AudioOutputSettings::pcm_i16(48_000.0, 2)))?;
    audio_output.set_sample_buffer_handler(Some("avcapture-test-audio"), |_sample| {})?;
    assert!(audio_output.callback_installed()?);
    audio_output.clear_sample_buffer_handler();
    assert!(!audio_output.callback_installed()?);
    Ok(())
}
