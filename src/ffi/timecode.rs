use core::ffi::{c_char, c_void};

use super::{DropCallback, JsonCallback};

extern "C" {
    pub fn av_capture_timecode_generator_create(out_error_message: *mut *mut c_char)
        -> *mut c_void;
    pub fn av_capture_timecode_generator_release(generator: *mut c_void);
    pub fn av_capture_timecode_generator_info_json(
        generator: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_timecode_generator_available_sources_count(generator: *mut c_void) -> usize;
    pub fn av_capture_timecode_generator_available_source_at_index(
        generator: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_timecode_generator_current_source(
        generator: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_timecode_generator_set_synchronization_timeout(
        generator: *mut c_void,
        synchronization_timeout: f64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_timecode_generator_set_timecode_alignment_offset(
        generator: *mut c_void,
        timecode_alignment_offset: f64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_timecode_generator_set_timecode_frame_duration_json(
        generator: *mut c_void,
        frame_duration_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_timecode_generator_start_synchronization(
        generator: *mut c_void,
        source: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_timecode_generator_generate_initial_timecode_json(
        generator: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_timecode_generator_set_delegate_callback(
        generator: *mut c_void,
        queue_label: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_timecode_generator_clear_delegate_callback(generator: *mut c_void);

    pub fn av_capture_timecode_source_release(source: *mut c_void);
    pub fn av_capture_timecode_source_info_json(
        source: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_timecode_source_frame_count(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_timecode_source_real_time_clock(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;

    pub fn av_capture_timecode_advanced_by_frames_json(
        timecode_json: *const c_char,
        frames_to_add: i64,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_timecode_create_metadata_sample_buffer_associated_with_presentation_time_stamp(
        timecode_json: *const c_char,
        presentation_time_stamp_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_timecode_create_metadata_sample_buffer_for_duration(
        timecode_json: *const c_char,
        duration_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_timecode_metadata_sample_buffer_release(sample_buffer: *mut c_void);
}
