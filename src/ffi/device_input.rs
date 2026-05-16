use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_device_input_create(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_input_release(input: *mut c_void);
    pub fn av_capture_device_input_info_json(
        input: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
}
