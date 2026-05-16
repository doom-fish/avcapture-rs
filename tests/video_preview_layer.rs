mod common;

use avcapture::prelude::*;

#[test]
fn video_preview_layer_smoke() -> common::TestResult {
    let session = CaptureSession::new()?;
    let layer = VideoPreviewLayer::new(&session)?;
    let info = layer.info()?;
    assert!(info.session_attached);
    assert_eq!(layer.video_gravity()?, info.video_gravity);
    assert_eq!(layer.connection()?.is_some(), info.connection_present);
    layer.set_video_gravity("resizeAspectFill")?;
    assert_eq!(layer.video_gravity()?, "resizeAspectFill");
    Ok(())
}
