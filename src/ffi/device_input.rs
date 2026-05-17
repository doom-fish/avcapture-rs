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
    pub fn av_capture_device_input_is_multichannel_audio_mode_supported(
        input: *mut c_void,
        mode: i32,
    ) -> bool;
    pub fn av_capture_device_input_set_multichannel_audio_mode(
        input: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_input_set_wind_noise_removal_enabled(
        input: *mut c_void,
        enabled: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
}
