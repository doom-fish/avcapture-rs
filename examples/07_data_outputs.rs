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
    println!("video output specific info: {:?}", video_output.info()?);
    println!(
        "audio output generic info: {:?}",
        audio_output.output_info()?
    );
    println!("audio output specific info: {:?}", audio_output.info()?);

    video_output.clear_sample_buffer_handler();
    audio_output.clear_sample_buffer_handler();
    Ok(())
}
