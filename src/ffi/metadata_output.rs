use core::ffi::{c_char, c_void};

use super::{DropCallback, JsonCallback};

extern "C" {
    pub fn av_capture_metadata_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_metadata_output_release(output: *mut c_void);
    pub fn av_capture_metadata_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_metadata_output_set_metadata_object_types_json(
        output: *mut c_void,
        metadata_object_types_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_metadata_output_set_rect_of_interest_json(
        output: *mut c_void,
        rect_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_metadata_output_set_metadata_objects_callback(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_metadata_output_clear_metadata_objects_callback(output: *mut c_void);
}
