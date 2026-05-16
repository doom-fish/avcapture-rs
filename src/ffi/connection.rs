use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_connection_release(connection: *mut c_void);
    pub fn av_capture_connection_info_json(
        connection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_connection_set_enabled(connection: *mut c_void, enabled: bool);
    pub fn av_capture_connection_set_automatically_adjusts_video_mirroring(
        connection: *mut c_void,
        enabled: bool,
    );
    pub fn av_capture_connection_set_video_mirrored(
        connection: *mut c_void,
        mirrored: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_connection_set_video_rotation_angle(
        connection: *mut c_void,
        angle: f64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
}
