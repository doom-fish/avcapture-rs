use core::ffi::{c_char, c_void};

use super::{DropCallback, JsonCallback};

extern "C" {
    pub fn av_capture_desk_view_application_create(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_desk_view_application_release(application: *mut c_void);
    pub fn av_capture_desk_view_application_info_json(
        application: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_desk_view_application_present(
        application: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_desk_view_application_present_with_launch_configuration(
        application: *mut c_void,
        launch_configuration: *mut c_void,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn av_capture_desk_view_application_launch_configuration_create(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_desk_view_application_launch_configuration_release(
        launch_configuration: *mut c_void,
    );
    pub fn av_capture_desk_view_application_launch_configuration_info_json(
        launch_configuration: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_desk_view_application_launch_configuration_set_main_window_frame_json(
        launch_configuration: *mut c_void,
        frame_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_desk_view_application_launch_configuration_set_requires_setup_mode_completion(
        launch_configuration: *mut c_void,
        requires_setup_mode_completion: bool,
    );
}
