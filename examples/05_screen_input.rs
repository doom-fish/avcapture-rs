mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let input = match ScreenInput::new() {
        Ok(input) => input,
        Err(err) => {
            support::print_skip("screen input", err);
            return Ok(());
        }
    };

    let info = input.info()?;
    println!("screen input info: {info:?}");
    println!("screen input ports: {:?}", input.input_info()?);
    input.set_scale_factor(info.scale_factor);
    input.set_captures_cursor(info.captures_cursor);
    input.set_captures_mouse_clicks(info.captures_mouse_clicks);
    input.set_removes_duplicate_frames(info.removes_duplicate_frames);
    Ok(())
}
