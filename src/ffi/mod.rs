//! Raw FFI declarations matching `swift-bridge/Sources/AVCaptureBridge`.

#![allow(missing_docs)]

use ::core::ffi::{c_char, c_void};

pub type VideoSampleCallback = unsafe extern "C" fn(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
);
pub type AudioSampleCallback =
    unsafe extern "C" fn(userdata: *mut c_void, sample_buffer: *mut c_void);
pub type JsonCallback = unsafe extern "C" fn(userdata: *mut c_void, payload: *mut c_char);
pub type DropCallback = unsafe extern "C" fn(userdata: *mut c_void);

pub mod audio_data_output;
pub mod connection;
pub mod core;
pub mod device;
pub mod device_discovery_session;
pub mod device_format;
pub mod device_input;
pub mod input;
pub mod metadata_output;
pub mod movie_file_output;
pub mod output;
pub mod photo;
pub mod photo_output;
pub mod screen_input;
pub mod session;
pub mod video_data_output;
pub mod video_preview_layer;

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
