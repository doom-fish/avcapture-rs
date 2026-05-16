mod common;

use avcapture::prelude::*;

#[test]
fn photo_output_smoke() -> common::TestResult {
    let output = PhotoOutput::new()?;
    let info = output.info()?;
    assert_eq!(
        output.output_info()?.connection_count,
        info.connection_count
    );
    assert_eq!(
        output.max_photo_quality_prioritization()?,
        info.max_photo_quality_prioritization
    );
    assert!(!output.callback_installed()?);

    let settings = PhotoSettings::new()?;
    let copied_settings = settings.copy_with_unique_id()?;
    assert_ne!(settings.unique_id()?, copied_settings.unique_id()?);
    assert_eq!(
        settings.photo_quality_prioritization()?,
        settings.info()?.photo_quality_prioritization
    );
    if let Some(priority) = settings.photo_quality_prioritization()? {
        settings.set_photo_quality_prioritization(priority)?;
    }
    if let Some(flash_mode) = settings.flash_mode()? {
        settings.set_flash_mode(flash_mode)?;
    }
    if let Some(priority) = output.max_photo_quality_prioritization()? {
        output.set_max_photo_quality_prioritization(priority)?;
    }

    let err = output
        .capture_photo_with_settings(&settings, |event| {
            if let Some(photo) = event.photo.as_ref() {
                let _ = photo.info();
            }
            eprintln!(
                "unexpected photo capture event: unique_id={}, error={:?}",
                event.unique_id, event.error,
            );
        })
        .expect_err("disconnected photo output should refuse capture requests");
    assert!(matches!(
        err,
        AVCaptureError::OutputError(_) | AVCaptureError::OperationFailed(_)
    ));
    Ok(())
}
