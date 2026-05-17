mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let session = CaptureSession::new()?;
    let layer = VideoPreviewLayer::new(&session)?;
    println!("preview layer info: {:?}", layer.info()?);

    layer.set_video_gravity("resizeAspectFill")?;
    println!("updated video gravity: {}", layer.video_gravity()?);
    println!(
        "preview layer has connection: {}",
        layer.connection()?.is_some()
    );

    layer.clear_session();
    println!(
        "preview layer attached after clear: {}",
        layer.session_attached()?
    );
    layer.set_session(&session)?;
    println!(
        "preview layer attached after reattach: {}",
        layer.session_attached()?
    );
    layer.set_session_with_no_connection(&session)?;
    println!(
        "preview layer attached after no-connection reattach: {}",
        layer.session_attached()?
    );

    let device_point = layer.capture_device_point_of_interest_for_point((0.0, 0.0))?;
    let layer_point = layer.point_for_capture_device_point_of_interest((0.0, 0.0))?;
    let rect = CaptureRect::new(0.0, 0.0, 0.0, 0.0);
    println!("device point for layer origin: {device_point:?}");
    println!("layer point for device origin: {layer_point:?}");
    println!(
        "metadata rect for layer rect: {:?}",
        layer.metadata_output_rect_of_interest_for_rect(&rect)?
    );
    println!(
        "layer rect for metadata rect: {:?}",
        layer.rect_for_metadata_output_rect_of_interest(&rect)?
    );
    Ok(())
}
