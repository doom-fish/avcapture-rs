use core::ffi::{c_char, c_void};

use super::{DropCallback, JsonCallback};

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
    pub fn av_capture_session_controls_count(session: *mut c_void) -> usize;
    pub fn av_capture_session_control_at_index(
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
    pub fn av_capture_session_can_add_control(session: *mut c_void, control: *mut c_void) -> bool;
    pub fn av_capture_session_add_control(
        session: *mut c_void,
        control: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_remove_control(session: *mut c_void, control: *mut c_void);
    pub fn av_capture_session_set_controls_delegate_callback(
        session: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_clear_controls_delegate_callback(session: *mut c_void);
    pub fn av_capture_session_set_deferred_start_delegate_callback(
        session: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_clear_deferred_start_delegate_callback(session: *mut c_void);

    pub fn av_capture_control_release(control: *mut c_void);
    pub fn av_capture_control_info_json(
        control: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_control_set_enabled(control: *mut c_void, enabled: bool);

    pub fn av_capture_index_picker_create(
        localized_title: *const c_char,
        symbol_name: *const c_char,
        number_of_indexes: usize,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_index_picker_create_with_titles_json(
        localized_title: *const c_char,
        symbol_name: *const c_char,
        localized_index_titles_json: *const c_char,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_index_picker_info_json(
        control: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_index_picker_set_selected_index(
        control: *mut c_void,
        selected_index: usize,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_index_picker_set_action_callback(
        control: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_index_picker_clear_action_callback(control: *mut c_void);

    pub fn av_capture_slider_create(
        localized_title: *const c_char,
        symbol_name: *const c_char,
        min_value: f32,
        max_value: f32,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_slider_create_with_step(
        localized_title: *const c_char,
        symbol_name: *const c_char,
        min_value: f32,
        max_value: f32,
        step: f32,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_slider_create_with_values_json(
        localized_title: *const c_char,
        symbol_name: *const c_char,
        values_json: *const c_char,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_slider_info_json(
        control: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_slider_set_value(
        control: *mut c_void,
        value: f32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_slider_set_action_callback(
        control: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_slider_clear_action_callback(control: *mut c_void);

    pub fn av_capture_system_exposure_bias_slider_create(
        device: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_system_zoom_slider_create(
        device: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_status: *mut i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
}
