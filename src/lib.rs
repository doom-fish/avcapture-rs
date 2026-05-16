#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod audio_data_output;
mod connection;
mod device;
mod device_discovery_session;
mod device_format;
mod device_input;
mod device_position;
mod error;
pub mod ffi;
mod helpers;
mod input;
mod metadata_output;
mod movie_file_output;
mod output;
mod photo;
mod photo_output;
mod screen_input;
mod session;
mod video_data_output;
mod video_preview_layer;

pub use audio_data_output::{AudioDataOutput, AudioDataOutputInfo, AudioOutputSettings};
pub use connection::{CaptureConnection, CaptureConnectionInfo};
pub use device::{
    AuthorizationStatus, CaptureDevice, CaptureDeviceConfigurationLock, CaptureDeviceDetails,
    CaptureDeviceInfo, CaptureDeviceType, CaptureExposureMode, CaptureFlashMode, CaptureTorchMode,
    MediaType,
};
pub use device_discovery_session::CaptureDeviceDiscoverySession;
pub use device_format::{
    CaptureDeviceFormat, CaptureDeviceFormatInfo, FormatDescriptionInfo, FrameRateRange,
};
pub use device_input::{DeviceInput, DeviceInputInfo};
pub use device_position::CaptureDevicePosition;
pub use error::AVCaptureError;
pub use helpers::{CaptureRect, VideoDimensions};
pub use input::{CaptureInputInfo, CaptureInputPortInfo, CaptureInputRef};
pub use metadata_output::{
    MetadataObject, MetadataObjectsEvent, MetadataOutput, MetadataOutputInfo,
};
pub use movie_file_output::{
    MovieFileOutput, MovieFileOutputInfo, MovieRecordingEvent, MovieRecordingEventKind,
};
pub use output::{CaptureOutputInfo, CaptureOutputRef};
pub use photo::{Photo, PhotoInfo, PhotoQualityPrioritization, PhotoSettings, PhotoSettingsInfo};
pub use photo_output::{PhotoCaptureEvent, PhotoCaptureResult, PhotoOutput, PhotoOutputInfo};
pub use screen_input::{ScreenInput, ScreenInputInfo};
pub use session::{CaptureSession, CaptureSessionInfo, CaptureSessionPreset};
pub use video_data_output::{VideoDataOutput, VideoDataOutputInfo, VideoOutputSettings};
pub use video_preview_layer::{VideoPreviewLayer, VideoPreviewLayerInfo};

/// Common imports.
pub mod prelude {
    pub use crate::{
        AVCaptureError, AudioDataOutput, AudioDataOutputInfo, AudioOutputSettings,
        AuthorizationStatus, CaptureConnection, CaptureConnectionInfo, CaptureDevice,
        CaptureDeviceConfigurationLock, CaptureDeviceDetails, CaptureDeviceDiscoverySession,
        CaptureDeviceFormat, CaptureDeviceFormatInfo, CaptureDeviceInfo, CaptureDevicePosition,
        CaptureDeviceType, CaptureExposureMode, CaptureFlashMode, CaptureInputInfo,
        CaptureInputPortInfo, CaptureInputRef, CaptureOutputInfo, CaptureOutputRef, CaptureRect,
        CaptureSession, CaptureSessionInfo, CaptureSessionPreset, CaptureTorchMode, DeviceInput,
        DeviceInputInfo, FormatDescriptionInfo, FrameRateRange, MediaType, MetadataObject,
        MetadataObjectsEvent, MetadataOutput, MetadataOutputInfo, MovieFileOutput,
        MovieFileOutputInfo, MovieRecordingEvent, MovieRecordingEventKind, Photo,
        PhotoCaptureEvent, PhotoCaptureResult, PhotoInfo, PhotoOutput, PhotoOutputInfo,
        PhotoQualityPrioritization, PhotoSettings, PhotoSettingsInfo, ScreenInput, ScreenInputInfo,
        VideoDataOutput, VideoDataOutputInfo, VideoDimensions, VideoOutputSettings,
        VideoPreviewLayer, VideoPreviewLayerInfo,
    };
}
