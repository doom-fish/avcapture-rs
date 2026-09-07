//! Raw FFI declarations matching `swift-bridge/Sources/AVCaptureBridge`.

#![allow(missing_docs)]

use ::core::ffi::{c_char, c_void};

pub use doom_fish_utils::ffi_callbacks::DropCallback;

/// C trampoline handed to Swift to take a +1 reference on a refcounted Rust
/// callback context.
///
/// Keeps the context alive for the lifetime of the holding bridge object so an
/// in-flight callback can never observe a freed context. Signature matches
/// [`DropCallback`].
pub type RetainCallback = unsafe extern "C" fn(user_data: *mut c_void);

pub type VideoSampleCallback = unsafe extern "C" fn(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
);
pub type VideoDataOutputEventCallback = unsafe extern "C" fn(
    userdata: *mut c_void,
    kind: i32,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
    dropped_reason: *mut c_char,
    dropped_total: u64,
);
pub type AudioSampleCallback =
    unsafe extern "C" fn(userdata: *mut c_void, sample_buffer: *mut c_void);
pub type JsonCallback = unsafe extern "C" fn(userdata: *mut c_void, payload: *mut c_char);

pub mod async_stream;
pub mod audio_data_output;
pub mod camera_calibration_data;
pub mod connection;
pub mod core;
pub mod desk_view_application;
pub mod device;
pub mod device_discovery_session;
pub mod device_format;
pub mod device_input;
pub mod external_display;
pub mod input;
pub mod metadata_output;
pub mod movie_file_output;
pub mod output;
pub mod photo;
pub mod photo_output;
pub mod screen_input;
pub mod session;
pub mod timecode;
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
    pub const DELEGATE_SLOT_OCCUPIED: i32 = -8;
    pub const UNSUPPORTED_PLATFORM: i32 = -9;
    pub const MAIN_THREAD_REQUIRED: i32 = -10;
    pub const OUTPUT_FILE_EXISTS: i32 = -11;
    pub const CANCELLED: i32 = -12;
    pub const BRIDGE_PROTOCOL: i32 = -13;
}
