mod common;

use avcapture::prelude::*;

#[test]
fn photo_output_smoke() -> common::TestResult {
    let output = PhotoOutput::new()?;
    let info = output.info()?;
    assert_eq!(output.output_info()?.connection_count, info.connection_count);
    assert!(!output.callback_installed()?);

    let err = output
        .capture_photo(|result| eprintln!("unexpected photo capture callback: {result:?}"))
        .expect_err("disconnected photo output should refuse capture requests");
    assert!(matches!(
        err,
        AVCaptureError::OutputError(_) | AVCaptureError::OperationFailed(_)
    ));
    Ok(())
}
