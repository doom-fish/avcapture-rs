//! Raw FFI declarations matching `swift-bridge/Sources/AVCaptureBridge`.

#![allow(missing_docs)]

use core::ffi::{c_char, c_void};

pub type VideoSampleCallback =
    unsafe extern "C" fn(userdata: *mut c_void, sample_buffer: *mut c_void, pixel_buffer: *mut c_void);
pub type AudioSampleCallback = unsafe extern "C" fn(userdata: *mut c_void, sample_buffer: *mut c_void);
pub type DropCallback = unsafe extern "C" fn(userdata: *mut c_void);

extern "C" {
    pub fn avc_string_free(s: *mut c_char);

    pub fn av_capture_authorization_status(media_type: *const c_char, out_error_message: *mut *mut c_char) -> i32;
    pub fn av_capture_devices_json(media_type: *const c_char, out_error_message: *mut *mut c_char) -> *mut c_char;
    pub fn av_capture_default_device(media_type: *const c_char, out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_device_with_unique_id(
        unique_id: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_release(device: *mut c_void);
    pub fn av_capture_device_info_json(device: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;
    pub fn av_capture_device_supports_session_preset(device: *mut c_void, preset: *const c_char) -> bool;

    pub fn av_capture_device_input_create(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_input_release(input: *mut c_void);
    pub fn av_capture_device_input_info_json(input: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;

    pub fn av_capture_session_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_session_release(session: *mut c_void);
    pub fn av_capture_session_info_json(session: *mut c_void, out_error_message: *mut *mut c_char) -> *mut c_char;
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
    pub fn av_capture_session_can_add_video_output(session: *mut c_void, output: *mut c_void) -> bool;
    pub fn av_capture_session_add_video_output(
        session: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_remove_video_output(session: *mut c_void, output: *mut c_void);
    pub fn av_capture_session_can_add_audio_output(session: *mut c_void, output: *mut c_void) -> bool;
    pub fn av_capture_session_add_audio_output(
        session: *mut c_void,
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_session_remove_audio_output(session: *mut c_void, output: *mut c_void);

    pub fn av_capture_video_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_video_output_release(output: *mut c_void);
    pub fn av_capture_video_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_video_output_set_video_settings_json(
        output: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_output_set_always_discards_late_video_frames(output: *mut c_void, enabled: bool);
    pub fn av_capture_video_output_set_sample_buffer_callback(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<VideoSampleCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_output_clear_sample_buffer_callback(output: *mut c_void);

    pub fn av_capture_audio_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_audio_output_release(output: *mut c_void);
    pub fn av_capture_audio_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_audio_output_set_audio_settings_json(
        output: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_audio_output_set_sample_buffer_callback(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<AudioSampleCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_audio_output_clear_sample_buffer_callback(output: *mut c_void);
}

pub mod status {
    pub const OK: i32 = 0;
    pub const INVALID_ARGUMENT: i32 = -1;
    pub const DEVICE_ERROR: i32 = -2;
    pub const INPUT_ERROR: i32 = -3;
    pub const SESSION_ERROR: i32 = -4;
    pub const OUTPUT_ERROR: i32 = -5;
    pub const CALLBACK_ERROR: i32 = -6;
    pub const OPERATION_FAILED: i32 = -7;
}
