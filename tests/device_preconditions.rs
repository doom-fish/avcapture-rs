mod common;

use apple_cf::cm::CMTime;
use avcapture::prelude::*;

fn authorized_video_device(context: &str) -> Result<Option<CaptureDevice>, AVCaptureError> {
    let status = CaptureDevice::authorization_status(&MediaType::Video)?;
    if status != AuthorizationStatus::Authorized {
        common::skip(context, format!("camera access is {status:?}"));
        return Ok(None);
    }
    let device = CaptureDevice::default(&MediaType::Video)?;
    if device.is_none() {
        common::skip_no_device(context);
    }
    Ok(device)
}

const fn is_invalid_argument(result: &Result<(), AVCaptureError>) -> bool {
    matches!(result, Err(AVCaptureError::InvalidArgument(_)))
}

#[test]
fn frame_durations_outside_the_active_format_are_rejected() -> common::TestResult {
    let Some(device) = authorized_video_device("frame duration validation")? else {
        return Ok(());
    };
    let lock = match device.lock_for_configuration() {
        Ok(lock) => lock,
        Err(err) => {
            common::skip("frame duration validation", err);
            return Ok(());
        }
    };

    let far_too_fast = CMTime::new(1, 1_000_000);
    assert!(is_invalid_argument(
        &lock.set_active_video_min_frame_duration(far_too_fast)
    ));
    assert!(is_invalid_argument(
        &lock.set_active_video_max_frame_duration(far_too_fast)
    ));
    assert!(is_invalid_argument(
        &lock.set_active_video_min_frame_duration(CMTime::indefinite())
    ));
    Ok(())
}

#[test]
fn a_format_from_another_device_is_rejected() -> common::TestResult {
    let Some(video) = authorized_video_device("foreign active format")? else {
        return Ok(());
    };
    let Some(audio) = CaptureDevice::default(&MediaType::Audio)? else {
        common::skip_no_device("foreign active format");
        return Ok(());
    };
    let Some(foreign_format) = audio.formats()?.into_iter().next() else {
        common::skip("foreign active format", "the audio device lists no formats");
        return Ok(());
    };
    let lock = match video.lock_for_configuration() {
        Ok(lock) => lock,
        Err(err) => {
            common::skip("foreign active format", err);
            return Ok(());
        }
    };

    assert!(is_invalid_argument(
        &lock.set_active_format(&foreign_format)
    ));
    Ok(())
}

#[test]
fn unlocking_through_another_wrapper_keeps_the_outer_lock() -> common::TestResult {
    let Some(first) = authorized_video_device("nested configuration locks")? else {
        return Ok(());
    };
    let Some(second) = CaptureDevice::default(&MediaType::Video)? else {
        return Ok(());
    };
    let outer = match first.lock_for_configuration() {
        Ok(lock) => lock,
        Err(err) => {
            common::skip("nested configuration locks", err);
            return Ok(());
        }
    };
    let current = first.active_video_min_frame_duration();

    drop(second.lock_for_configuration()?);

    outer.set_active_video_min_frame_duration(current)?;
    assert!(first
        .active_video_min_frame_duration()
        .compare(current)
        .is_eq());
    Ok(())
}
