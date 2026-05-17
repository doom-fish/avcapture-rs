mod common;

use avcapture::prelude::*;

#[test]
fn device_inventory_surface_smoke() -> common::TestResult {
    let _ = CaptureDevice::authorization_status(&MediaType::Video)?;
    let _ = CaptureDevice::authorization_status(&MediaType::Audio)?;
    let _ = CaptureDevice::devices(&MediaType::Video)?;
    let _ = CaptureDevice::devices(&MediaType::Audio)?;

    assert_eq!(
        CaptureDevice::WAS_CONNECTED_NOTIFICATION,
        "AVCaptureDeviceWasConnectedNotification"
    );
    assert_eq!(
        CaptureDevice::WAS_DISCONNECTED_NOTIFICATION,
        "AVCaptureDeviceWasDisconnectedNotification"
    );
    let _ = CaptureDevice::max_available_torch_level();
    let _ = CaptureDevice::center_stage_control_mode();
    let _ = CaptureDevice::center_stage_enabled();
    let _ = CaptureDevice::preferred_microphone_mode();
    let _ = CaptureDevice::active_microphone_mode();
    let _ = CaptureDevice::reaction_effects_enabled();
    let _ = CaptureDevice::reaction_effect_gestures_enabled();
    let scene_status_raw: String = CaptureDevice::scene_monitoring_status_not_enough_light().into();
    assert_eq!(
        scene_status_raw,
        "AVCaptureSceneMonitoringStatusNotEnoughLight"
    );

    let Some(device) = common::default_video_or_audio_device()? else {
        return Ok(());
    };

    let details = device.details()?;
    let _ = details.focus_mode.map(i32::from);
    let _ = details.white_balance_mode.map(i32::from);
    let _ = details.auto_focus_system.map(i32::from);
    let _ = details.active_color_space.map(i32::from);
    let _ = details.transport_controls_playback_mode.map(i32::from);
    let _ = details
        .primary_constituent_device_switching_behavior
        .map(i32::from);
    let _ = details
        .primary_constituent_device_restricted_switching_behavior_conditions
        .map(u64::from);
    let _ = details
        .active_primary_constituent_device_switching_behavior
        .map(i32::from);
    let _ = details
        .active_primary_constituent_device_restricted_switching_behavior_conditions
        .map(u64::from);
    let _ = details.preferred_microphone_mode.map(i32::from);
    let _ = details.active_microphone_mode.map(i32::from);
    let _ = &details.available_reaction_types;
    let _ = &details.reaction_effects_in_progress;
    let _ = details.camera_lens_smudge_detection_status.map(i32::from);
    let _ = &details.cinematic_video_capture_scene_monitoring_statuses;

    let _ = device.input_source_infos()?;
    let input_sources = device.input_sources()?;
    for input_source in &input_sources {
        let _ = input_source.info()?;
    }
    let _ = device.active_input_source()?;
    let available_reaction_types = device.available_reaction_types()?;
    for reaction_type in &available_reaction_types {
        let _ = reaction_type.system_image_name()?;
    }
    if device.media_types()?.contains(&MediaType::Video) {
        if let Some(rotation_coordinator) = device.rotation_coordinator()? {
            let _ = rotation_coordinator.info()?;
            let _ = rotation_coordinator.video_rotation_angle_for_horizon_level_preview()?;
            let _ = rotation_coordinator.video_rotation_angle_for_horizon_level_capture()?;
        }
    }
    Ok(())
}
