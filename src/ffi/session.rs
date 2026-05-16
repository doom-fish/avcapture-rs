use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_session_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_session_release(session: *mut c_void);
    pub fn av_capture_session_info_json(
        session: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_session_connections_count(session: *mut c_void) -> usize;
    pub fn av_capture_session_connection_at_index(
        session: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_session_begin_configuration(session: *mut c_void);
    pub fn av_capture_session_commit_configuration(session: *mut c_void);
    pub fn av_capture_session_start_running(session: *mut c_void);
    pub fn av_capture_session_stop_running(session: *mut c_void);
    pub fn av_capture_session_can_set_preset(session: *mut c_void, preset: *const c_char) -> bool;
    pub fn av_capture_session_set_preset(
        session: *mut c_void,
        preset: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_can_add_input(session: *mut c_void, input: *mut c_void) -> bool;
    pub fn av_capture_session_add_input(
        session: *mut c_void,
        input: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_remove_input(session: *mut c_void, input: *mut c_void);
    pub fn av_capture_session_can_add_output(session: *mut c_void, output: *mut c_void) -> bool;
    pub fn av_capture_session_add_output(
        session: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_remove_output(session: *mut c_void, output: *mut c_void);
}
