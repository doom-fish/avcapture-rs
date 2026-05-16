use avcapture::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let video_auth = CaptureDevice::authorization_status(&MediaType::Video)?;
    let audio_auth = CaptureDevice::authorization_status(&MediaType::Audio)?;
    println!("video authorization: {video_auth:?}");
    println!("audio authorization: {audio_auth:?}");

    let video_devices = CaptureDevice::devices(&MediaType::Video)?;
    let audio_devices = CaptureDevice::devices(&MediaType::Audio)?;
    println!("video devices discovered: {}", video_devices.len());
    println!("audio devices discovered: {}", audio_devices.len());

    if let Some(default_video) = CaptureDevice::default(&MediaType::Video)? {
        println!("default video device: {}", default_video.localized_name()?);
    }
    if let Some(default_audio) = CaptureDevice::default(&MediaType::Audio)? {
        println!("default audio device: {}", default_audio.localized_name()?);
    }

    let session = CaptureSession::new()?;
    println!("initial session preset: {:?}", session.session_preset()?);
    println!("initial session running: {}", session.is_running()?);

    session.begin_configuration();
    assert!(session.can_set_session_preset(&CaptureSessionPreset::High)?);
    session.set_session_preset(&CaptureSessionPreset::High)?;

    let video_output = VideoDataOutput::new()?;
    video_output.set_video_settings(Some(&VideoOutputSettings::bgra().with_dimensions(640, 480)))?;
    video_output.set_always_discards_late_video_frames(true);
    video_output.set_sample_buffer_handler(Some("avcapture-smoke-video"), |_sample, _pixel_buffer| {
        println!("unexpected video sample callback in no-input smoke test");
    })?;

    let audio_output = AudioDataOutput::new()?;
    audio_output.set_audio_settings(Some(&AudioOutputSettings::pcm_i16(48_000.0, 2)))?;
    audio_output.set_sample_buffer_handler(Some("avcapture-smoke-audio"), |_sample| {
        println!("unexpected audio sample callback in no-input smoke test");
    })?;

    assert!(session.can_add_video_data_output(&video_output));
    assert!(session.can_add_audio_data_output(&audio_output));
    session.add_video_data_output(&video_output)?;
    session.add_audio_data_output(&audio_output)?;
    session.commit_configuration();

    println!("configured session preset: {:?}", session.session_preset()?);
    println!("session outputs: {}", session.output_count()?);
    println!("session connections: {}", session.connection_count()?);
    println!("video output info: {:?}", video_output.info()?);
    println!("audio output info: {:?}", audio_output.info()?);
    println!("✅ avcapture surface OK (no permission prompts)");
    Ok(())
}
