mod common;

use avcapture::prelude::*;

#[test]
fn metadata_output_smoke() -> common::TestResult {
    let output = match MetadataOutput::new() {
        Ok(output) => output,
        Err(err) => {
            common::skip("metadata output", err);
            return Ok(());
        }
    };

    output.set_metadata_object_types(Vec::<String>::new())?;
    output.set_rect_of_interest(&CaptureRect::new(0.0, 0.0, 1.0, 1.0))?;
    output.set_metadata_objects_handler(Some("avcapture-tests-metadata"), |event| {
        eprintln!("metadata callback event: {event:?}");
    })?;
    assert!(output.callback_installed()?);
    output.clear_metadata_objects_handler();
    assert!(!output.callback_installed()?);

    let info = output.info()?;
    assert!(info.metadata_object_types.is_empty());
    assert_eq!(info.rect_of_interest, CaptureRect::new(0.0, 0.0, 1.0, 1.0));
    Ok(())
}
