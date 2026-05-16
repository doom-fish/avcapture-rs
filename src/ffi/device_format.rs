use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_device_format_release(format: *mut c_void);
    pub fn av_capture_device_format_info_json(
        format: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
}
