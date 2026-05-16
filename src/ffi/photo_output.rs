use core::ffi::{c_char, c_void};

use super::{DropCallback, JsonCallback};

extern "C" {
    pub fn av_capture_photo_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_photo_output_release(output: *mut c_void);
    pub fn av_capture_photo_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_photo_output_set_high_resolution_capture_enabled(
        output: *mut c_void,
        enabled: bool,
    );
    pub fn av_capture_photo_output_set_responsive_capture_enabled(
        output: *mut c_void,
        enabled: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_output_capture_photo(
        output: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
}
