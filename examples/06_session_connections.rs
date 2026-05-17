mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let screen_input = match ScreenInput::new() {
        Ok(input) => input,
        Err(err) => {
            support::print_skip("session connections", err);
            return Ok(());
        }
    };

    let video_output = VideoDataOutput::new()?;
    video_output.set_video_settings(Some(
        &VideoOutputSettings::bgra().with_dimensions(1280, 720),
    ))?;

    let session = CaptureSession::new()?;
    session.begin_configuration();
    if session.can_set_session_preset(&CaptureSessionPreset::High)? {
        session.set_session_preset(&CaptureSessionPreset::High)?;
    }
    if !session.can_add_screen_input(&screen_input)
        || !session.can_add_video_data_output(&video_output)
    {
        session.commit_configuration();
        println!("skipping session connections: session cannot add screen input + video output without prompting");
        return Ok(());
    }
    session.add_screen_input(&screen_input)?;
    session.add_video_data_output(&video_output)?;
    session.commit_configuration();

    println!("session info: {:?}", session.info()?);
    println!("video output info: {:?}", video_output.output_info()?);
    for connection in session.connections()? {
        println!("connection: {:?}", connection.info()?);
        println!(
            "connection audio channels: {:?}",
            connection.audio_channels_info()?
        );
    }
    Ok(())
}
