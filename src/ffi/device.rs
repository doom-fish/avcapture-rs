use core::ffi::{c_char, c_void};

use apple_cf::cm::CMTime;

extern "C" {
    pub fn av_capture_authorization_status(
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_devices_json(
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_default_device(
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_default_device_for_type(
        device_type: *const c_char,
        media_type: *const c_char,
        position: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_with_unique_id(
        unique_id: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_release(device: *mut c_void);
    pub fn av_capture_device_info_json(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_details_json(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_supports_session_preset(
        device: *mut c_void,
        preset: *const c_char,
    ) -> bool;
    pub fn av_capture_device_formats_count(device: *mut c_void) -> usize;
    pub fn av_capture_device_format_at_index(
        device: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_active_format(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_active_video_min_frame_duration(device: *mut c_void) -> CMTime;
    pub fn av_capture_device_active_video_max_frame_duration(device: *mut c_void) -> CMTime;
    pub fn av_capture_device_lock_for_configuration(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_unlock_for_configuration(device: *mut c_void);
    pub fn av_capture_device_set_active_format(
        device: *mut c_void,
        format: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_active_video_min_frame_duration(
        device: *mut c_void,
        duration: CMTime,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_active_video_max_frame_duration(
        device: *mut c_void,
        duration: CMTime,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_torch_mode(
        device: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
}
