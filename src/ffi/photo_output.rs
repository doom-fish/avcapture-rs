use core::ffi::{c_char, c_void};

use super::{DropCallback, JsonCallback};

pub type PhotoCallback =
    unsafe extern "C" fn(userdata: *mut c_void, photo: *mut c_void, payload: *mut c_char);

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
    pub fn av_capture_photo_output_set_max_photo_quality_prioritization(
        output: *mut c_void,
        prioritization: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_output_set_responsive_capture_enabled(
        output: *mut c_void,
        enabled: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_output_capture_photo(
        output: *mut c_void,
        settings: *mut c_void,
        callback: Option<PhotoCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn av_capture_photo_output_readiness_coordinator_create(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_photo_output_readiness_coordinator_release(coordinator: *mut c_void);
    pub fn av_capture_photo_output_readiness_coordinator_capture_readiness(
        coordinator: *mut c_void,
        out_readiness: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_output_readiness_coordinator_set_callback(
        coordinator: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_output_readiness_coordinator_clear_callback(coordinator: *mut c_void);
    pub fn av_capture_photo_output_readiness_coordinator_start_tracking_capture_request(
        coordinator: *mut c_void,
        settings: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_output_readiness_coordinator_stop_tracking_capture_request(
        coordinator: *mut c_void,
        settings_unique_id: i64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
}
