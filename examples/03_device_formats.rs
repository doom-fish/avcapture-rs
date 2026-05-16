mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let Some(device) = CaptureDevice::default(&MediaType::Video)? else {
        support::print_no_device("device formats");
        return Ok(());
    };

    println!("device details: {:?}", device.details()?);
    println!(
        "supports session preset high: {}",
        device.supports_session_preset(&CaptureSessionPreset::High)?
    );

    let formats = device.formats()?;
    println!("format count: {}", formats.len());
    if let Some(active_format) = device.active_format()? {
        println!("active format: {:?}", active_format.info()?);
    }
    if let Some(first_format) = formats.first() {
        println!("first format: {:?}", first_format.info()?);
    }
    Ok(())
}
