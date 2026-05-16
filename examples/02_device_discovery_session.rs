mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let discovery = CaptureDeviceDiscoverySession::new(
        &[
            CaptureDeviceType::BuiltInWideAngleCamera,
            CaptureDeviceType::External,
            CaptureDeviceType::ContinuityCamera,
            CaptureDeviceType::DeskViewCamera,
        ],
        Some(&MediaType::Video),
        CaptureDevicePosition::Unspecified,
    )?;
    let devices = discovery.devices()?;
    println!("video devices from discovery session: {}", devices.len());
    for device in devices {
        let info = device.info()?;
        println!("- {} ({})", info.localized_name, info.unique_id);
    }
    Ok(())
}
