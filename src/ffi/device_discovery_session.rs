use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_device_discovery_session_create(
        criteria_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_discovery_session_release(session: *mut c_void);
    pub fn av_capture_device_discovery_session_devices_count(session: *mut c_void) -> usize;
    pub fn av_capture_device_discovery_session_device_at_index(
        session: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
}
