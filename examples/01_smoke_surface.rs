mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    println!(
        "video authorization: {:?}",
        CaptureDevice::authorization_status(&MediaType::Video)?
    );
    println!(
        "audio authorization: {:?}",
        CaptureDevice::authorization_status(&MediaType::Audio)?
    );
    println!(
        "video devices: {}",
        CaptureDevice::devices(&MediaType::Video)?.len()
    );
    println!(
        "audio devices: {}",
        CaptureDevice::devices(&MediaType::Audio)?.len()
    );

    let session = CaptureSession::new()?;
    session.begin_configuration();
    if session.can_set_session_preset(&CaptureSessionPreset::High)? {
        session.set_session_preset(&CaptureSessionPreset::High)?;
    }

    let video_output = VideoDataOutput::new()?;
    video_output
        .set_video_settings(Some(&VideoOutputSettings::bgra().with_dimensions(640, 480)))?;
    video_output
        .set_sample_buffer_handler(Some("avcapture-smoke-video"), |_sample, _pixel_buffer| {})?;
    if session.can_add_video_data_output(&video_output) {
        session.add_video_data_output(&video_output)?;
    }

    let audio_output = AudioDataOutput::new()?;
    audio_output.set_audio_settings(Some(&AudioOutputSettings::pcm_i16(48_000.0, 2)))?;
    audio_output.set_sample_buffer_handler(Some("avcapture-smoke-audio"), |_sample| {})?;
    if session.can_add_audio_data_output(&audio_output) {
        session.add_audio_data_output(&audio_output)?;
    }

    let photo_output = PhotoOutput::new()?;

    let movie_output = MovieFileOutput::new()?;
    movie_output.set_max_recorded_file_size(16 * 1024 * 1024);
    if session.can_add_movie_file_output(&movie_output) {
        session.add_movie_file_output(&movie_output)?;
    }

    match MetadataOutput::new() {
        Ok(metadata_output) => {
            metadata_output.set_metadata_object_types(Vec::<String>::new())?;
            if session.can_add_metadata_output(&metadata_output) {
                session.add_metadata_output(&metadata_output)?;
            }
            println!("metadata output info: {:?}", metadata_output.info()?);
        }
        Err(err) => support::print_skip("metadata output", err),
    }

    match ScreenInput::new() {
        Ok(screen_input) => {
            if session.can_add_screen_input(&screen_input) {
                session.add_screen_input(&screen_input)?;
                println!("screen input info: {:?}", screen_input.info()?);
            }
        }
        Err(err) => support::print_skip("screen input", err),
    }

    session.commit_configuration();

    println!("session info: {:?}", session.info()?);
    println!("session connections: {}", session.connection_count()?);
    if let Some(connection) = session.connections()?.first() {
        println!("first connection info: {:?}", connection.info()?);
    }
    println!("video output info: {:?}", video_output.info()?);
    println!("audio output info: {:?}", audio_output.info()?);
    println!("photo output info: {:?}", photo_output.info()?);
    println!("movie output info: {:?}", movie_output.info()?);
    println!("✅ avcapture 0.2 surface OK (no permission prompts)");
    Ok(())
}
