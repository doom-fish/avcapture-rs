mod support;

use avcapture::prelude::*;

#[allow(clippy::too_many_lines)]
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
    println!(
        "device notifications: {}, {}",
        CaptureDevice::WAS_CONNECTED_NOTIFICATION,
        CaptureDevice::WAS_DISCONNECTED_NOTIFICATION
    );
    println!(
        "input port format notification: {}",
        DeviceInput::INPUT_PORT_FORMAT_DESCRIPTION_DID_CHANGE_NOTIFICATION
    );
    println!(
        "scene monitoring status constant: {}",
        String::from(CaptureDevice::scene_monitoring_status_not_enough_light())
    );
    println!(
        "max available torch level sentinel: {}",
        CaptureDevice::max_available_torch_level()
    );
    println!(
        "center stage control mode: {:?}",
        CaptureDevice::center_stage_control_mode()
    );
    println!(
        "center stage enabled: {:?}",
        CaptureDevice::center_stage_enabled()
    );
    println!(
        "preferred microphone mode: {:?}",
        CaptureDevice::preferred_microphone_mode()
    );
    println!(
        "active microphone mode: {:?}",
        CaptureDevice::active_microphone_mode()
    );
    println!(
        "reaction effects enabled: {:?}",
        CaptureDevice::reaction_effects_enabled()
    );
    println!(
        "reaction gestures enabled: {:?}",
        CaptureDevice::reaction_effect_gestures_enabled()
    );

    if let Some(device) = support::default_video_or_audio_device()? {
        println!("default device info: {:?}", device.info()?);
        println!("default device details: {:?}", device.details()?);
        println!(
            "default device input sources: {:?}",
            device.input_source_infos()?
        );
        if let Some(rotation_coordinator) = device.rotation_coordinator()? {
            println!(
                "rotation coordinator info: {:?}",
                rotation_coordinator.info()?
            );
        }
        for reaction_type in device.available_reaction_types()? {
            println!(
                "reaction type {:?} system image: {}",
                reaction_type,
                reaction_type.system_image_name()?
            );
        }
    }

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
        println!(
            "first connection audio channels: {:?}",
            connection.audio_channels_info()?
        );
    }
    println!("video output info: {:?}", video_output.info()?);
    println!("audio output info: {:?}", audio_output.info()?);
    println!("photo output info: {:?}", photo_output.info()?);
    println!("movie output info: {:?}", movie_output.info()?);
    println!("✅ avcapture 0.2 surface OK (no permission prompts)");
    Ok(())
}
