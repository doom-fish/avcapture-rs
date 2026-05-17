mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let video_output = VideoDataOutput::new()?;
    video_output.set_video_settings(Some(
        &VideoOutputSettings::bgra().with_dimensions(1920, 1080),
    ))?;
    video_output.set_always_discards_late_video_frames(true);
    video_output
        .set_sample_buffer_handler(Some("avcapture-example-video"), |_sample, _pixel_buffer| {})?;

    let audio_output = AudioDataOutput::new()?;
    audio_output.set_audio_settings(Some(&AudioOutputSettings::pcm_i16(48_000.0, 2)))?;
    audio_output.set_sample_buffer_handler(Some("avcapture-example-audio"), |_sample| {})?;

    println!(
        "video output generic info: {:?}",
        video_output.output_info()?
    );
    println!(
        "video output deferred start: supported={:?}, enabled={:?}",
        video_output.deferred_start_supported()?,
        video_output.deferred_start_enabled()?
    );
    println!("video output specific info: {:?}", video_output.info()?);
    println!(
        "video dropped-sample surface: count={}, last_reason={:?}",
        video_output.dropped_sample_count()?,
        video_output
            .last_dropped_sample_reason()?
            .as_ref()
            .map(|reason| reason.as_raw())
    );
    println!(
        "audio output generic info: {:?}",
        audio_output.output_info()?
    );
    println!(
        "audio output deferred start: supported={:?}, enabled={:?}",
        audio_output.deferred_start_supported()?,
        audio_output.deferred_start_enabled()?
    );
    println!("audio output specific info: {:?}", audio_output.info()?);
    println!(
        "audio dropped-sample surface: count={}, last_reason={:?}",
        audio_output.dropped_sample_count()?,
        audio_output
            .last_dropped_sample_reason()?
            .as_ref()
            .map(|reason| reason.as_raw())
    );

    video_output.clear_sample_buffer_handler();
    audio_output.clear_sample_buffer_handler();
    Ok(())
}
