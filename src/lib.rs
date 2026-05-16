#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod device;
mod error;
pub mod ffi;
mod output;
mod session;

pub use device::{AuthorizationStatus, CaptureDevice, CaptureDeviceInfo, DeviceInput, DeviceInputInfo, MediaType};
pub use error::AVCaptureError;
pub use output::{AudioDataOutput, AudioDataOutputInfo, AudioOutputSettings, VideoDataOutput, VideoDataOutputInfo, VideoOutputSettings};
pub use session::{CaptureSession, CaptureSessionInfo, CaptureSessionPreset};

/// Common imports.
pub mod prelude {
    pub use crate::device::{AuthorizationStatus, CaptureDevice, CaptureDeviceInfo, DeviceInput, DeviceInputInfo, MediaType};
    pub use crate::error::AVCaptureError;
    pub use crate::output::{
        AudioDataOutput, AudioDataOutputInfo, AudioOutputSettings, VideoDataOutput,
        VideoDataOutputInfo, VideoOutputSettings,
    };
    pub use crate::session::{CaptureSession, CaptureSessionInfo, CaptureSessionPreset};
}
