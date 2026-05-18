#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
/// Async stream adapters for `AVCapture*` notifications and delegate callbacks.
pub mod async_api;
mod audio_data_output;
mod connection;
mod device;
mod device_discovery_session;
mod device_format;
mod device_input;
mod device_position;
mod error;
/// Raw FFI declarations backing the safe `AVCapture*` wrappers.
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

pub use audio_data_output::{
    AudioDataOutput, AudioDataOutputInfo, AudioOutputSettings, AudioPreviewOutput,
    AudioPreviewOutputInfo,
};
pub use connection::{
    CaptureAudioChannel, CaptureAudioChannelInfo, CaptureConnection, CaptureConnectionInfo,
};
pub use device::{
    AuthorizationStatus, CaptureAutoFocusSystem, CaptureCameraLensSmudgeDetectionStatus,
    CaptureCenterStageControlMode, CaptureCinematicVideoFocusMode, CaptureColorSpace,
    CaptureDevice, CaptureDeviceConfigurationLock, CaptureDeviceDetails, CaptureDeviceInfo,
    CaptureDeviceInputSource, CaptureDeviceInputSourceInfo, CaptureDeviceRotationCoordinator,
    CaptureDeviceRotationCoordinatorInfo, CaptureDeviceTransportControlsPlaybackMode,
    CaptureDeviceType, CaptureExposureMode, CaptureFlashMode, CaptureFocusMode,
    CaptureMicrophoneMode, CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions,
    CapturePrimaryConstituentDeviceSwitchingBehavior, CaptureReactionEffectState,
    CaptureReactionType, CaptureSceneMonitoringStatus, CaptureSystemUserInterface,
    CaptureTorchMode, CaptureWhiteBalanceMode, MediaType,
};
pub use device_discovery_session::CaptureDeviceDiscoverySession;
pub use device_format::{
    CaptureDeviceFormat, CaptureDeviceFormatInfo, FormatDescriptionInfo, FrameRateRange,
};
pub use device_input::{CaptureMultichannelAudioMode, DeviceInput, DeviceInputInfo};
pub use device_position::CaptureDevicePosition;
pub use error::AVCaptureError;
pub use helpers::{CapturePoint, CaptureRect, CaptureSize, VideoDimensions};
pub use input::{CaptureInputInfo, CaptureInputPortInfo, CaptureInputRef};
pub use metadata_output::{
    MetadataObject, MetadataObjectsEvent, MetadataOutput, MetadataOutputInfo,
};
pub use movie_file_output::{
    AudioFileOutput, AudioFileOutputInfo, AudioFileRecordingEvent, AudioFileRecordingEventKind,
    MovieFileOutput, MovieFileOutputInfo, MovieRecordingEvent, MovieRecordingEventKind,
};
pub use output::{
    AVCaptureOutputDataDroppedReason, CaptureOutputDataDroppedReason, CaptureOutputInfo,
    CaptureOutputRef,
};
pub use photo::{
    Photo, PhotoInfo, PhotoQualityPrioritization, PhotoSettings, PhotoSettingsInfo,
    ResolvedPhotoSettings, ResolvedPhotoSettingsInfo,
};
pub use photo_output::{
    PhotoCaptureEvent, PhotoCaptureResult, PhotoOutput, PhotoOutputCaptureReadiness,
    PhotoOutputInfo, PhotoOutputReadinessCoordinator,
};
pub use screen_input::{ScreenInput, ScreenInputInfo};
pub use session::{
    CaptureControl, CaptureControlInfo, CaptureIndexPicker, CaptureIndexPickerInfo, CaptureSession,
    CaptureSessionControlsEvent, CaptureSessionDeferredStartEvent, CaptureSessionInfo,
    CaptureSessionPreset, CaptureSlider, CaptureSliderInfo, CaptureSystemExposureBiasSlider,
    CaptureSystemZoomSlider,
};
pub use video_data_output::{
    CaptureTimecode, CaptureTimecodeGenerator, CaptureTimecodeGeneratorEvent,
    CaptureTimecodeGeneratorInfo, CaptureTimecodeGeneratorSynchronizationStatus,
    CaptureTimecodeSource, CaptureTimecodeSourceInfo, CaptureTimecodeSourceType,
    TimecodeMetadataSampleBuffer, VideoDataOutput, VideoDataOutputInfo, VideoOutputSettings,
};
pub use video_preview_layer::{
    DeskViewApplication, DeskViewApplicationInfo, DeskViewApplicationLaunchConfiguration,
    DeskViewApplicationLaunchConfigurationInfo, ExternalDisplayConfiguration,
    ExternalDisplayConfigurationInfo, ExternalDisplayConfigurator, ExternalDisplayConfiguratorInfo,
    ExternalDisplaySupportInfo, VideoPreviewLayer, VideoPreviewLayerInfo,
};

/// Common imports.
pub mod prelude {
    pub use crate::{
        AVCaptureError, AVCaptureOutputDataDroppedReason, AudioDataOutput, AudioDataOutputInfo,
        AudioFileOutput, AudioFileOutputInfo, AudioFileRecordingEvent, AudioFileRecordingEventKind,
        AudioOutputSettings, AudioPreviewOutput, AudioPreviewOutputInfo, AuthorizationStatus,
        CaptureAudioChannel, CaptureAudioChannelInfo, CaptureAutoFocusSystem,
        CaptureCameraLensSmudgeDetectionStatus, CaptureCenterStageControlMode,
        CaptureCinematicVideoFocusMode, CaptureColorSpace, CaptureConnection,
        CaptureConnectionInfo, CaptureControl, CaptureControlInfo, CaptureDevice,
        CaptureDeviceConfigurationLock, CaptureDeviceDetails, CaptureDeviceDiscoverySession,
        CaptureDeviceFormat, CaptureDeviceFormatInfo, CaptureDeviceInfo, CaptureDeviceInputSource,
        CaptureDeviceInputSourceInfo, CaptureDevicePosition, CaptureDeviceRotationCoordinator,
        CaptureDeviceRotationCoordinatorInfo, CaptureDeviceTransportControlsPlaybackMode,
        CaptureDeviceType, CaptureExposureMode, CaptureFlashMode, CaptureFocusMode,
        CaptureIndexPicker, CaptureIndexPickerInfo, CaptureInputInfo, CaptureInputPortInfo,
        CaptureInputRef, CaptureMicrophoneMode, CaptureMultichannelAudioMode,
        CaptureOutputDataDroppedReason, CaptureOutputInfo, CaptureOutputRef, CapturePoint,
        CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions,
        CapturePrimaryConstituentDeviceSwitchingBehavior, CaptureReactionEffectState,
        CaptureReactionType, CaptureRect, CaptureSceneMonitoringStatus, CaptureSession,
        CaptureSessionControlsEvent, CaptureSessionDeferredStartEvent, CaptureSessionInfo,
        CaptureSessionPreset, CaptureSize, CaptureSlider, CaptureSliderInfo,
        CaptureSystemExposureBiasSlider, CaptureSystemUserInterface, CaptureSystemZoomSlider,
        CaptureTimecode, CaptureTimecodeGenerator, CaptureTimecodeGeneratorEvent,
        CaptureTimecodeGeneratorInfo, CaptureTimecodeGeneratorSynchronizationStatus,
        CaptureTimecodeSource, CaptureTimecodeSourceInfo, CaptureTimecodeSourceType,
        CaptureTorchMode, CaptureWhiteBalanceMode, DeskViewApplication, DeskViewApplicationInfo,
        DeskViewApplicationLaunchConfiguration, DeskViewApplicationLaunchConfigurationInfo,
        DeviceInput, DeviceInputInfo, ExternalDisplayConfiguration,
        ExternalDisplayConfigurationInfo, ExternalDisplayConfigurator,
        ExternalDisplayConfiguratorInfo, ExternalDisplaySupportInfo, FormatDescriptionInfo,
        FrameRateRange, MediaType, MetadataObject, MetadataObjectsEvent, MetadataOutput,
        MetadataOutputInfo, MovieFileOutput, MovieFileOutputInfo, MovieRecordingEvent,
        MovieRecordingEventKind, Photo, PhotoCaptureEvent, PhotoCaptureResult, PhotoInfo,
        PhotoOutput, PhotoOutputCaptureReadiness, PhotoOutputInfo, PhotoOutputReadinessCoordinator,
        PhotoQualityPrioritization, PhotoSettings, PhotoSettingsInfo, ResolvedPhotoSettings,
        ResolvedPhotoSettingsInfo, ScreenInput, ScreenInputInfo, TimecodeMetadataSampleBuffer,
        VideoDataOutput, VideoDataOutputInfo, VideoDimensions, VideoOutputSettings,
        VideoPreviewLayer, VideoPreviewLayerInfo,
    };
}
