use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_output_connections_count(output: *mut c_void) -> usize;
    pub fn av_capture_output_connection_at_index(
        output: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_output_connection_for_media_type(
        output: *mut c_void,
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
}
