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

    layer.clear_session();
    assert!(!layer.session_attached()?);
    layer.set_session(&session)?;
    assert!(layer.session_attached()?);
    layer.set_session_with_no_connection(&session)?;
    assert!(layer.session_attached()?);

    let device_point = layer.capture_device_point_of_interest_for_point((0.0, 0.0))?;
    assert!(device_point.0.is_finite() && device_point.1.is_finite());

    let layer_point = layer.point_for_capture_device_point_of_interest((0.0, 0.0))?;
    assert!(layer_point.0.is_finite() && layer_point.1.is_finite());

    let rect = CaptureRect::new(0.0, 0.0, 0.0, 0.0);
    let metadata_rect = layer.metadata_output_rect_of_interest_for_rect(&rect)?;
    assert!(
        metadata_rect.origin.x.is_finite()
            && metadata_rect.origin.y.is_finite()
            && metadata_rect.size.width.is_finite()
            && metadata_rect.size.height.is_finite()
    );

    let layer_rect = layer.rect_for_metadata_output_rect_of_interest(&rect)?;
    assert!(
        layer_rect.origin.x.is_finite()
            && layer_rect.origin.y.is_finite()
            && layer_rect.size.width.is_finite()
            && layer_rect.size.height.is_finite()
    );
    Ok(())
}
