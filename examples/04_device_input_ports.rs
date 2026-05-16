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

    println!("device input info: {:?}", input.info()?);
    println!("generic input info: {:?}", input.input_info()?);
    Ok(())
}
