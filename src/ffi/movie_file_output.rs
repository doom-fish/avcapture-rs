use core::ffi::{c_char, c_void};

use apple_cf::cm::CMTime;

use super::{DropCallback, JsonCallback};

extern "C" {
    pub fn av_capture_movie_file_output_create(out_error_message: *mut *mut c_char) -> *mut c_void;
    pub fn av_capture_movie_file_output_release(output: *mut c_void);
    pub fn av_capture_movie_file_output_info_json(
        output: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_movie_file_output_start_recording(
        output: *mut c_void,
        output_path: *const c_char,
        callback: Option<JsonCallback>,
        userdata: *mut c_void,
        drop_userdata: Option<DropCallback>,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_movie_file_output_stop_recording(output: *mut c_void);
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
}
