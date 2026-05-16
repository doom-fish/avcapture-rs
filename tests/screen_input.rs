mod common;

use avcapture::prelude::*;

#[test]
fn screen_input_smoke() -> common::TestResult {
    let input = match ScreenInput::new() {
        Ok(input) => input,
        Err(err) => {
            common::skip("screen input", err);
            return Ok(());
        }
    };

    let info = input.info()?;
    input.set_scale_factor(info.scale_factor);
    input.set_captures_cursor(info.captures_cursor);
    input.set_captures_mouse_clicks(info.captures_mouse_clicks);
    input.set_removes_duplicate_frames(info.removes_duplicate_frames);
    let _ = input.input_info()?;
    Ok(())
}
