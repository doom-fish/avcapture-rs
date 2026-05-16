use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_photo_settings_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_photo_settings_copy_with_unique_id(
        settings: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_photo_settings_release(settings: *mut c_void);
    pub fn av_capture_photo_settings_info_json(
        settings: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_photo_settings_set_flash_mode(
        settings: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_photo_settings_set_photo_quality_prioritization(
        settings: *mut c_void,
        prioritization: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn av_capture_photo_release(photo: *mut c_void);
    pub fn av_capture_photo_info_json(
        photo: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
}
