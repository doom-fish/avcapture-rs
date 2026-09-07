use core::ffi::{c_char, c_void};

use apple_cf::cm::CMTime;

use super::{AudioSampleCallback, DropCallback, JsonCallback, RetainCallback};

extern "C" {
    pub fn av_capture_movie_file_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_movie_file_output_release(output: *mut c_void);
    pub fn av_capture_movie_file_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_movie_file_output_start_recording(
        output: *mut c_void,
        output_path_bytes: *const u8,
        output_path_length: usize,
        overwrite_policy: i32,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        retain_userdata: Option<RetainCallback>,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_movie_file_output_set_sample_buffer_boundary_callback(
        output: *mut c_void,
        callback: Option<AudioSampleCallback>,
        userdata: *mut c_void,
        retain_userdata: Option<RetainCallback>,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_movie_file_output_clear_sample_buffer_boundary_callback(output: *mut c_void);
    pub fn av_capture_movie_file_output_stop_recording(output: *mut c_void) -> bool;
    pub fn av_capture_movie_file_output_pause_recording(output: *mut c_void);
    pub fn av_capture_movie_file_output_resume_recording(output: *mut c_void);
    pub fn av_capture_movie_file_output_set_max_recorded_duration(
        output: *mut c_void,
        duration: CMTime,
    );
    pub fn av_capture_movie_file_output_set_max_recorded_file_size(output: *mut c_void, bytes: i64);
    pub fn av_capture_movie_file_output_set_min_free_disk_space_limit(
        output: *mut c_void,
        bytes: i64,
    );
    pub fn av_capture_movie_file_output_set_movie_fragment_interval(
        output: *mut c_void,
        interval: CMTime,
    );
    pub fn av_capture_movie_file_output_set_spatial_video_capture_enabled(
        output: *mut c_void,
        enabled: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn av_capture_audio_file_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_audio_file_output_release(output: *mut c_void);
    pub fn av_capture_audio_file_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_audio_file_output_set_audio_settings_json(
        output: *mut c_void,
        settings_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_audio_file_output_start_recording(
        output: *mut c_void,
        output_path_bytes: *const u8,
        output_path_length: usize,
        output_file_type: *const c_char,
        overwrite_policy: i32,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        retain_userdata: Option<RetainCallback>,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_audio_file_output_set_sample_buffer_boundary_callback(
        output: *mut c_void,
        callback: Option<AudioSampleCallback>,
        userdata: *mut c_void,
        retain_userdata: Option<RetainCallback>,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_audio_file_output_clear_sample_buffer_boundary_callback(output: *mut c_void);
    pub fn av_capture_audio_file_output_stop_recording(output: *mut c_void) -> bool;
    pub fn av_capture_audio_file_output_pause_recording(output: *mut c_void);
    pub fn av_capture_audio_file_output_resume_recording(output: *mut c_void);
    pub fn av_capture_audio_file_output_set_max_recorded_duration(
        output: *mut c_void,
        duration: CMTime,
    );
    pub fn av_capture_audio_file_output_set_max_recorded_file_size(output: *mut c_void, bytes: i64);
    pub fn av_capture_audio_file_output_set_min_free_disk_space_limit(
        output: *mut c_void,
        bytes: i64,
    );

    #[cfg(test)]
    pub fn av_capture_recording_destination_finalize_for_testing(
        path_bytes: *const u8,
        path_length: usize,
        overwrite_policy: i32,
        native_error_mode: i32,
        out_had_error: *mut bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
}
