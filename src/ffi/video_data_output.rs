use core::ffi::{c_char, c_void};

use super::{DropCallback, RetainCallback, VideoDataOutputEventCallback, VideoSampleCallback};

extern "C" {
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
    pub fn av_capture_video_output_set_always_discards_late_video_frames(
        output: *mut c_void,
        enabled: bool,
    );
    pub fn av_capture_video_output_set_sample_buffer_callback(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<VideoSampleCallback>,
        userdata: *mut c_void,
        retain_userdata: Option<RetainCallback>,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_output_clear_sample_buffer_callback(output: *mut c_void);
    pub fn av_capture_video_output_set_sample_buffer_event_callback(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<VideoDataOutputEventCallback>,
        userdata: *mut c_void,
        retain_userdata: Option<RetainCallback>,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_video_output_clear_sample_buffer_event_callback(output: *mut c_void);
    #[cfg(test)]
    pub fn av_capture_video_output_record_drop_for_testing(
        output: *mut c_void,
        reason: *const c_char,
    ) -> u64;
}
