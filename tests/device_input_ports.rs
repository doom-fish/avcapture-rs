mod common;

use avcapture::prelude::*;

#[test]
fn device_input_ports_smoke() -> common::TestResult {
    let Some(device) = common::default_video_or_audio_device()? else {
        common::skip_no_device("device input ports");
        return Ok(());
    };

    let input = match DeviceInput::new(&device) {
        Ok(input) => input,
        Err(err) => {
            common::skip("device input", err);
            return Ok(());
        }
    };

    let info = input.info()?;
    let generic_info = input.input_info()?;
    assert_eq!(info.ports_count, generic_info.ports_count());
    assert_eq!(
        DeviceInput::INPUT_PORT_FORMAT_DESCRIPTION_DID_CHANGE_NOTIFICATION,
        "AVCaptureInputPortFormatDescriptionDidChangeNotification"
    );
    let _ = input.multichannel_audio_mode()?;
    let _ = input.wind_noise_removal_supported()?;
    let _ = input.wind_noise_removal_enabled()?;
    let _ = input.is_multichannel_audio_mode_supported(0_i32);
    let _ = info.multichannel_audio_mode.map(i32::from);
    Ok(())
}
