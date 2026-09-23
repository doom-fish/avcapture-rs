mod common;

use avcapture::prelude::*;

const fn is_invalid_state(result: &Result<(), AVCaptureError>) -> bool {
    matches!(result, Err(AVCaptureError::InvalidState(_)))
}

#[test]
fn commit_without_begin_is_rejected() -> common::TestResult {
    let session = CaptureSession::new()?;

    assert!(is_invalid_state(&session.commit_configuration()));

    session.begin_configuration();
    session.commit_configuration()?;
    assert!(is_invalid_state(&session.commit_configuration()));
    Ok(())
}

#[test]
fn start_and_stop_inside_a_configuration_block_are_rejected() -> common::TestResult {
    let session = CaptureSession::new()?;
    session.begin_configuration();

    assert!(is_invalid_state(&session.start_running()));
    assert!(is_invalid_state(&session.stop_running()));
    assert!(!session.is_running()?);

    session.commit_configuration()?;
    Ok(())
}

#[test]
fn nested_configuration_blocks_need_matching_commits() -> common::TestResult {
    let session = CaptureSession::new()?;
    session.begin_configuration();
    session.begin_configuration();
    session.commit_configuration()?;

    assert!(is_invalid_state(&session.start_running()));

    session.commit_configuration()?;
    assert!(is_invalid_state(&session.commit_configuration()));
    Ok(())
}

#[test]
fn an_empty_session_starts_and_stops_off_the_main_thread() -> common::TestResult {
    let session = CaptureSession::new()?;

    session.start_running()?;
    assert!(session.is_running()?);

    session.begin_configuration();
    session.set_session_preset(&CaptureSessionPreset::Medium)?;
    session.commit_configuration()?;
    assert_eq!(session.session_preset()?, CaptureSessionPreset::Medium);

    session.stop_running()?;
    assert!(!session.is_running()?);
    Ok(())
}

#[test]
fn commit_accepts_audio_data_outputs_fed_by_a_microphone() -> common::TestResult {
    let status = CaptureDevice::authorization_status(&MediaType::Audio)?;
    if status != AuthorizationStatus::Authorized {
        common::skip(
            "microphone session graph",
            format!("microphone access is {status:?}"),
        );
        return Ok(());
    }
    let Some(microphone) = CaptureDevice::default(&MediaType::Audio)? else {
        common::skip_no_device("microphone session graph");
        return Ok(());
    };
    let input = DeviceInput::new(&microphone)?;
    let first = AudioDataOutput::new()?;
    let second = AudioDataOutput::new()?;
    let session = CaptureSession::new()?;

    session.begin_configuration();
    session.add_device_input(&input)?;
    session.add_audio_data_output(&first)?;
    session.add_audio_data_output(&second)?;
    session.commit_configuration()?;

    assert_eq!(session.input_count()?, 1);
    assert_eq!(session.output_count()?, 2);
    assert!(!session.is_running()?);
    Ok(())
}
