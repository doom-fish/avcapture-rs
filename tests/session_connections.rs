mod common;

use avcapture::prelude::*;

#[test]
fn session_connections_smoke() -> common::TestResult {
    let screen_input = match ScreenInput::new() {
        Ok(input) => input,
        Err(err) => {
            common::skip("session connections", err);
            return Ok(());
        }
    };

    let video_output = VideoDataOutput::new()?;
    let session = CaptureSession::new()?;
    session.begin_configuration();
    if !session.can_add_screen_input(&screen_input)
        || !session.can_add_video_data_output(&video_output)
    {
        session.commit_configuration();
        common::skip(
            "session connections",
            "session cannot add screen input + video output",
        );
        return Ok(());
    }
    session.add_screen_input(&screen_input)?;
    session.add_video_data_output(&video_output)?;
    session.commit_configuration();

    let info = session.info()?;
    let connections = session.connections()?;
    assert_eq!(info.connection_count, connections.len());
    Ok(())
}
