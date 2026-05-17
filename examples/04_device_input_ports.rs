mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let Some(device) = support::default_video_or_audio_device()? else {
        support::print_no_device("device input ports");
        return Ok(());
    };

    let input = match DeviceInput::new(&device) {
        Ok(input) => input,
        Err(err) => {
            support::print_skip("device input", err);
            return Ok(());
        }
    };

    println!(
        "input port format notification: {}",
        DeviceInput::INPUT_PORT_FORMAT_DESCRIPTION_DID_CHANGE_NOTIFICATION
    );
    println!("device input info: {:?}", input.info()?);
    println!("generic input info: {:?}", input.input_info()?);
    println!(
        "multichannel audio mode supported (none): {}",
        input.is_multichannel_audio_mode_supported(0_i32)
    );
    println!(
        "wind noise removal supported: {}",
        input.wind_noise_removal_supported()?
    );
    println!(
        "wind noise removal enabled: {}",
        input.wind_noise_removal_enabled()?
    );
    Ok(())
}
