mod common;

use apple_cf::cm::CMTime;
use avcapture::prelude::*;

#[test]
fn screen_input_settings_round_trip() -> common::TestResult {
    let input = match ScreenInput::new() {
        Ok(input) => input,
        Err(err) => {
            common::skip("screen input", err);
            return Ok(());
        }
    };

    let info = input.info()?;
    assert!(info.scale_factor > 0.0);

    input.set_scale_factor(0.5);
    assert!((input.scale_factor()? - 0.5).abs() < f64::EPSILON);

    input.set_captures_cursor(!info.captures_cursor);
    assert_eq!(input.captures_cursor()?, !info.captures_cursor);

    input.set_captures_mouse_clicks(!info.captures_mouse_clicks);
    assert_eq!(input.captures_mouse_clicks()?, !info.captures_mouse_clicks);

    let frame_duration = CMTime::new(1, 15);
    input.set_min_frame_duration(frame_duration);
    assert!(input.min_frame_duration()?.compare(frame_duration).is_eq());

    let crop = CaptureRect::new(0.0, 0.0, 320.0, 240.0);
    input.set_crop_rect(crop);
    assert_eq!(input.crop_rect()?, crop);

    input.set_removes_duplicate_frames(info.removes_duplicate_frames);
    assert_eq!(input.display_id()?, info.display_id);
    let ports = input.input_info()?;
    assert!(ports.ports_count() >= 1);
    Ok(())
}
