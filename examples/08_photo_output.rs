mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let output = PhotoOutput::new()?;
    println!("photo output generic info: {:?}", output.output_info()?);
    println!("photo output specific info: {:?}", output.info()?);

    let settings = PhotoSettings::new()?;
    let copied_settings = settings.copy_with_unique_id()?;
    println!("photo settings: {:?}", settings.info()?);
    println!("copied photo settings: {:?}", copied_settings.info()?);

    if let Some(priority) = output.max_photo_quality_prioritization()? {
        output.set_max_photo_quality_prioritization(priority)?;
    }
    if let Some(priority) = settings.photo_quality_prioritization()? {
        settings.set_photo_quality_prioritization(priority)?;
    }
    if let Some(flash_mode) = settings.flash_mode()? {
        settings.set_flash_mode(flash_mode)?;
    }

    if let Err(err) = output.capture_photo_with_settings(&settings, |event| {
        let photo_info = event.photo.as_ref().and_then(|photo| photo.info().ok());
        println!(
            "photo capture event: unique_id={}, error={:?}, photo_info={photo_info:?}",
            event.unique_id, event.error,
        );
    }) {
        support::print_skip(
            "photo capture (output not attached to a running session)",
            err,
        );
    }

    Ok(())
}
