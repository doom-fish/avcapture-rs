mod common;

use avcapture::prelude::*;

#[test]
fn device_inventory_surface_smoke() -> common::TestResult {
    let _ = CaptureDevice::authorization_status(&MediaType::Video)?;
    let _ = CaptureDevice::authorization_status(&MediaType::Audio)?;
    let _ = CaptureDevice::devices(&MediaType::Video)?;
    let _ = CaptureDevice::devices(&MediaType::Audio)?;
    Ok(())
}
