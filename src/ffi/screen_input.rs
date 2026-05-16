use core::ffi::{c_char, c_void};

use apple_cf::cm::CMTime;

extern "C" {
    pub fn av_capture_screen_input_create_main_display(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_screen_input_create_with_display_id(
        display_id: u32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_screen_input_release(input: *mut c_void);
    pub fn av_capture_screen_input_info_json(
        input: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_screen_input_set_min_frame_duration(input: *mut c_void, duration: CMTime);
    pub fn av_capture_screen_input_set_crop_rect(
        input: *mut c_void,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    );
    pub fn av_capture_screen_input_set_scale_factor(input: *mut c_void, scale_factor: f64);
    pub fn av_capture_screen_input_set_captures_mouse_clicks(input: *mut c_void, enabled: bool);
    pub fn av_capture_screen_input_set_captures_cursor(input: *mut c_void, enabled: bool);
    pub fn av_capture_screen_input_set_removes_duplicate_frames(input: *mut c_void, enabled: bool);
}
