mod common;

use avcapture::prelude::*;

#[test]
fn device_formats_smoke() -> common::TestResult {
    let Some(device) = CaptureDevice::default(&MediaType::Video)? else {
        common::skip_no_device("device formats");
        return Ok(());
    };

    let formats = device.formats()?;
    assert_eq!(device.formats_count()?, formats.len());
    let details = device.details()?;
    assert_eq!(device.exposure_mode()?, details.exposure_mode);
    if let Some(mode) = details.exposure_mode {
        assert!(device.is_exposure_mode_supported(mode));
    }
    if let Some(active_format) = device.active_format()? {
        let _ = active_format.info()?;
    }
    if let Some(first_format) = formats.first() {
        let _ = first_format.info()?;
    }
    Ok(())
}
