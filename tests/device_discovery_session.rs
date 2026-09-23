mod common;

use std::collections::HashSet;

use avcapture::prelude::*;

#[test]
fn discovery_session_returns_requested_video_devices() -> common::TestResult {
    let requested = [
        CaptureDeviceType::BuiltInWideAngleCamera,
        CaptureDeviceType::External,
        CaptureDeviceType::ContinuityCamera,
    ];
    let discovery = CaptureDeviceDiscoverySession::new(
        &requested,
        Some(&MediaType::Video),
        CaptureDevicePosition::Unspecified,
    )?;
    let devices = discovery.devices()?;

    let mut unique_ids = HashSet::new();
    for device in &devices {
        let info = device.info()?;
        assert!(!info.unique_id.is_empty());
        assert!(!info.localized_name.is_empty());
        assert!(
            unique_ids.insert(info.unique_id.clone()),
            "{} was discovered twice",
            info.unique_id
        );
        assert!(device.media_types()?.contains(&MediaType::Video));
        let device_type = device.device_type()?;
        assert!(
            requested.contains(&device_type),
            "{device_type:?} was not requested"
        );
    }

    if let Some(default_device) = CaptureDevice::default(&MediaType::Video)? {
        if requested.contains(&default_device.device_type()?) {
            assert!(unique_ids.contains(&default_device.unique_id()?));
        }
    }
    Ok(())
}
