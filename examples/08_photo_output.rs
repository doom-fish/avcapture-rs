mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let output = PhotoOutput::new()?;
    println!("photo output generic info: {:?}", output.output_info()?);
    println!("photo output specific info: {:?}", output.info()?);
    println!(
        "photo output capture readiness: {:?}",
        output.capture_readiness()?
    );

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

    if output.connection_count()? > 0 {
        match output.readiness_coordinator() {
            Ok(coordinator) => {
                println!(
                    "photo output readiness coordinator capture readiness: {:?}",
                    coordinator.capture_readiness()?
                );
                coordinator.set_capture_readiness_handler(|readiness| {
                    println!("photo output readiness changed: {readiness:?}");
                })?;
                coordinator.start_tracking_capture_request(&settings)?;
                coordinator.stop_tracking_capture_request_for_settings(&settings)?;
                coordinator.clear_capture_readiness_handler();
            }
            Err(err) => support::print_skip(
                "photo output readiness coordinator (requires macOS 14.0 or newer)",
                err,
            ),
        }
    } else {
        support::print_skip(
            "photo output readiness coordinator",
            "photo output is not attached to a session",
        );
    }

    if let Err(err) = output.capture_photo_with_settings(&settings, |event| {
        let photo_info = event.photo.as_ref().and_then(|photo| photo.info().ok());
        let resolved_settings = event
            .photo
            .as_ref()
            .and_then(|photo| photo.resolved_settings_info().ok())
            .unwrap_or(event.resolved_settings);
        println!(
            "photo capture event: unique_id={}, error={:?}, resolved_settings={resolved_settings:?}, photo_info={photo_info:?}",
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
