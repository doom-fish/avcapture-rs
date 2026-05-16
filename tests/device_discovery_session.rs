mod common;

use avcapture::prelude::*;

#[test]
fn discovery_session_smoke() -> common::TestResult {
    let discovery = CaptureDeviceDiscoverySession::new(
        &[
            CaptureDeviceType::BuiltInWideAngleCamera,
            CaptureDeviceType::External,
            CaptureDeviceType::ContinuityCamera,
        ],
        Some(&MediaType::Video),
        CaptureDevicePosition::Unspecified,
    )?;
    let devices = discovery.devices()?;
    if let Some(device) = devices.first() {
        let _ = device.info()?;
    }
    Ok(())
}
