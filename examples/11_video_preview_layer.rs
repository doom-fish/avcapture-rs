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
    Ok(())
}
