use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_connection_release(connection: *mut c_void);
    pub fn av_capture_connection_info_json(
        connection: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_connection_audio_channels_count(connection: *mut c_void) -> usize;
    pub fn av_capture_connection_audio_channel_at_index(
        connection: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_audio_channel_release(audio_channel: *mut c_void);
    pub fn av_capture_audio_channel_info_json(
        audio_channel: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_audio_channel_set_volume(
        audio_channel: *mut c_void,
        volume: f32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_audio_channel_set_enabled(
        audio_channel: *mut c_void,
        enabled: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
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
