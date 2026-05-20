use core::ffi::{c_char, c_void};

use super::{AudioSampleCallback, VideoSampleCallback};

pub type StreamEventCallback =
    unsafe extern "C" fn(kind: i32, payload: *mut c_char, ctx: *mut c_void);

extern "C" {
    pub fn avcapture_session_running_subscribe(
        session: *mut c_void,
        callback: Option<StreamEventCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_session_running_unsubscribe(handle: *mut c_void);

    pub fn avcapture_session_error_subscribe(
        session: *mut c_void,
        callback: Option<StreamEventCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_session_error_unsubscribe(handle: *mut c_void);

    pub fn avcapture_session_interruption_subscribe(
        session: *mut c_void,
        callback: Option<StreamEventCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_session_interruption_unsubscribe(handle: *mut c_void);

    pub fn avcapture_video_sample_subscribe(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<VideoSampleCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_video_sample_unsubscribe(handle: *mut c_void);

    pub fn avcapture_audio_sample_subscribe(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<AudioSampleCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_audio_sample_unsubscribe(handle: *mut c_void);

    pub fn avcapture_file_recording_stream_start(
        output: *mut c_void,
        path: *const c_char,
        callback: Option<StreamEventCallback>,
        ctx: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn avcapture_file_recording_stream_stop(handle: *mut c_void);

    pub fn avcapture_audio_file_recording_stream_start(
        output: *mut c_void,
        path: *const c_char,
        output_file_type: *const c_char,
        callback: Option<StreamEventCallback>,
        ctx: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn avcapture_audio_file_recording_stream_stop(handle: *mut c_void);

    pub fn avcapture_movie_file_boundary_subscribe(
        output: *mut c_void,
        callback: Option<AudioSampleCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_movie_file_boundary_unsubscribe(handle: *mut c_void);

    pub fn avcapture_audio_file_boundary_subscribe(
        output: *mut c_void,
        callback: Option<AudioSampleCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_audio_file_boundary_unsubscribe(handle: *mut c_void);

    pub fn avcapture_metadata_objects_subscribe(
        output: *mut c_void,
        queue_label: *const c_char,
        callback: Option<StreamEventCallback>,
        ctx: *mut c_void,
    ) -> *mut c_void;
    pub fn avcapture_metadata_objects_unsubscribe(handle: *mut c_void);
}
