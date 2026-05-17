use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_video_preview_layer_create(
        session: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_video_preview_layer_release(layer: *mut c_void);
    pub fn av_capture_video_preview_layer_info_json(
        layer: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_video_preview_layer_connection(
        layer: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_video_preview_layer_set_video_gravity(
        layer: *mut c_void,
        video_gravity: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_preview_layer_set_session(
        layer: *mut c_void,
        session: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_preview_layer_clear_session(layer: *mut c_void);
    pub fn av_capture_video_preview_layer_set_session_with_no_connection(
        layer: *mut c_void,
        session: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json(
        layer: *mut c_void,
        point_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_video_preview_layer_point_for_capture_device_point_of_interest_json(
        layer: *mut c_void,
        point_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_video_preview_layer_metadata_output_rect_of_interest_for_rect_json(
        layer: *mut c_void,
        rect_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_video_preview_layer_rect_for_metadata_output_rect_of_interest_json(
        layer: *mut c_void,
        rect_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
}
