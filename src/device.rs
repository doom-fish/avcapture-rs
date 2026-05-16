#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use apple_cf::cm::CMTime;
use serde::{Deserialize, Serialize};

use crate::device_format::CaptureDeviceFormat;
use crate::device_position::CaptureDevicePosition;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, cstring, parse_json_and_free};
use crate::session::CaptureSessionPreset;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
pub enum MediaType {
    Audio,
    Video,
    Muxed,
    Metadata,
    Unknown(String),
}

impl MediaType {
    #[must_use]
    pub fn as_raw(&self) -> &str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Muxed => "muxed",
            Self::Metadata => "metadata",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "audio" => Self::Audio,
            "video" => Self::Video,
            "muxed" => Self::Muxed,
            "metadata" => Self::Metadata,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl From<String> for MediaType {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<MediaType> for String {
    fn from(value: MediaType) -> Self {
        value.as_raw().to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AuthorizationStatus {
    NotDetermined,
    Restricted,
    Denied,
    Authorized,
    Limited,
    Unknown,
}

impl AuthorizationStatus {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::NotDetermined,
            1 => Self::Restricted,
            2 => Self::Denied,
            3 => Self::Authorized,
            4 => Self::Limited,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
pub enum CaptureDeviceType {
    External,
    Microphone,
    BuiltInWideAngleCamera,
    ContinuityCamera,
    DeskViewCamera,
    Unknown(String),
}

impl CaptureDeviceType {
    #[must_use]
    pub fn as_raw(&self) -> &str {
        match self {
            Self::External => "AVCaptureDeviceTypeExternal",
            Self::Microphone => "AVCaptureDeviceTypeMicrophone",
            Self::BuiltInWideAngleCamera => "AVCaptureDeviceTypeBuiltInWideAngleCamera",
            Self::ContinuityCamera => "AVCaptureDeviceTypeContinuityCamera",
            Self::DeskViewCamera => "AVCaptureDeviceTypeDeskViewCamera",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "AVCaptureDeviceTypeExternal" | "AVCaptureDeviceTypeExternalUnknown" => Self::External,
            "AVCaptureDeviceTypeMicrophone" | "AVCaptureDeviceTypeBuiltInMicrophone" => {
                Self::Microphone
            }
            "AVCaptureDeviceTypeBuiltInWideAngleCamera" => Self::BuiltInWideAngleCamera,
            "AVCaptureDeviceTypeContinuityCamera" => Self::ContinuityCamera,
            "AVCaptureDeviceTypeDeskViewCamera" => Self::DeskViewCamera,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl From<String> for CaptureDeviceType {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<CaptureDeviceType> for String {
    fn from(value: CaptureDeviceType) -> Self {
        value.as_raw().to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
pub enum CaptureFlashMode {
    Off,
    On,
    Auto,
    Unknown(i32),
}

impl CaptureFlashMode {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Off,
            1 => Self::On,
            2 => Self::Auto,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::On => 1,
            Self::Auto => 2,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for CaptureFlashMode {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<CaptureFlashMode> for i32 {
    fn from(value: CaptureFlashMode) -> Self {
        value.as_raw()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
pub enum CaptureTorchMode {
    Off,
    On,
    Auto,
    Unknown(i32),
}

impl CaptureTorchMode {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Off,
            1 => Self::On,
            2 => Self::Auto,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::On => 1,
            Self::Auto => 2,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for CaptureTorchMode {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<CaptureTorchMode> for i32 {
    fn from(value: CaptureTorchMode) -> Self {
        value.as_raw()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
pub enum CaptureExposureMode {
    Locked,
    AutoExpose,
    ContinuousAutoExposure,
    Custom,
    Unknown(i32),
}

impl CaptureExposureMode {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Locked,
            1 => Self::AutoExpose,
            2 => Self::ContinuousAutoExposure,
            3 => Self::Custom,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Locked => 0,
            Self::AutoExpose => 1,
            Self::ContinuousAutoExposure => 2,
            Self::Custom => 3,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for CaptureExposureMode {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<CaptureExposureMode> for i32 {
    fn from(value: CaptureExposureMode) -> Self {
        value.as_raw()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDeviceInfo {
    pub unique_id: String,
    pub localized_name: String,
    pub manufacturer: String,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDeviceDetails {
    pub unique_id: String,
    pub localized_name: String,
    pub manufacturer: String,
    pub transport_type: Option<i32>,
    pub media_types: Vec<MediaType>,
    pub position: CaptureDevicePosition,
    pub device_type: CaptureDeviceType,
    pub has_flash: bool,
    pub flash_available: bool,
    pub has_torch: bool,
    pub torch_available: bool,
    pub torch_level: Option<f32>,
    pub exposure_mode: Option<CaptureExposureMode>,
    pub formats_count: usize,
    #[serde(with = "cm_time_serde")]
    pub active_video_min_frame_duration: CMTime,
    #[serde(with = "cm_time_serde")]
    pub active_video_max_frame_duration: CMTime,
}

/// Safe wrapper around `AVCaptureDevice`.
pub struct CaptureDevice {
    pub(crate) ptr: *mut c_void,
}

impl Drop for CaptureDevice {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device::av_capture_device_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureDevice {
    pub fn authorization_status(
        media_type: &MediaType,
    ) -> Result<AuthorizationStatus, AVCaptureError> {
        let media_type = cstring(media_type.as_raw(), "media type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let raw =
            unsafe { ffi::device::av_capture_authorization_status(media_type.as_ptr(), &mut err) };
        if raw < 0 {
            return Err(unsafe { from_swift(raw, err) });
        }
        Ok(AuthorizationStatus::from_raw(raw))
    }

    pub fn devices(media_type: &MediaType) -> Result<Vec<CaptureDeviceInfo>, AVCaptureError> {
        let media_type = cstring(media_type.as_raw(), "media type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::device::av_capture_devices_json(media_type.as_ptr(), &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn default(media_type: &MediaType) -> Result<Option<Self>, AVCaptureError> {
        let media_type = cstring(media_type.as_raw(), "media type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::device::av_capture_default_device(media_type.as_ptr(), &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    pub fn default_with_device_type(
        device_type: &CaptureDeviceType,
        media_type: Option<&MediaType>,
        position: CaptureDevicePosition,
    ) -> Result<Option<Self>, AVCaptureError> {
        let device_type = cstring(device_type.as_raw(), "device type")?;
        let media_type = media_type
            .map(|value| cstring(value.as_raw(), "media type"))
            .transpose()?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::device::av_capture_default_device_for_type(
                device_type.as_ptr(),
                media_type
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                position.as_raw(),
                &mut err,
            )
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    pub fn with_unique_id(unique_id: impl AsRef<str>) -> Result<Option<Self>, AVCaptureError> {
        let unique_id = CString::new(unique_id.as_ref()).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("device unique ID contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::device::av_capture_device_with_unique_id(unique_id.as_ptr(), &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    pub fn info(&self) -> Result<CaptureDeviceInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::device::av_capture_device_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn details(&self) -> Result<CaptureDeviceDetails, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::device::av_capture_device_details_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn unique_id(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    pub fn localized_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.localized_name)
    }

    pub fn manufacturer(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.manufacturer)
    }

    pub fn position(&self) -> Result<CaptureDevicePosition, AVCaptureError> {
        Ok(self.details()?.position)
    }

    pub fn device_type(&self) -> Result<CaptureDeviceType, AVCaptureError> {
        Ok(self.details()?.device_type)
    }

    pub fn media_types(&self) -> Result<Vec<MediaType>, AVCaptureError> {
        Ok(self.details()?.media_types)
    }

    pub fn transport_type(&self) -> Result<Option<i32>, AVCaptureError> {
        Ok(self.details()?.transport_type)
    }

    pub fn has_flash(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.has_flash)
    }

    pub fn flash_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.flash_available)
    }

    pub fn has_torch(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.has_torch)
    }

    pub fn torch_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.torch_available)
    }

    pub fn torch_level(&self) -> Result<Option<f32>, AVCaptureError> {
        Ok(self.details()?.torch_level)
    }

    pub fn exposure_mode(&self) -> Result<Option<CaptureExposureMode>, AVCaptureError> {
        Ok(self.details()?.exposure_mode)
    }

    pub fn is_exposure_mode_supported(&self, mode: CaptureExposureMode) -> bool {
        unsafe {
            ffi::device::av_capture_device_is_exposure_mode_supported(self.ptr, mode.as_raw())
        }
    }

    pub fn formats_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.details()?.formats_count)
    }

    pub fn supports_session_preset(
        &self,
        preset: &CaptureSessionPreset,
    ) -> Result<bool, AVCaptureError> {
        let preset = preset_cstring(preset)?;
        Ok(unsafe {
            ffi::device::av_capture_device_supports_session_preset(self.ptr, preset.as_ptr())
        })
    }

    pub fn formats(&self) -> Result<Vec<CaptureDeviceFormat>, AVCaptureError> {
        let count = unsafe { ffi::device::av_capture_device_formats_count(self.ptr) };
        let mut formats = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::device::av_capture_device_format_at_index(self.ptr, index, &mut err)
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
            }
            formats.push(CaptureDeviceFormat::from_raw(ptr));
        }
        Ok(formats)
    }

    pub fn active_format(&self) -> Result<Option<CaptureDeviceFormat>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::device::av_capture_device_active_format(self.ptr, &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(CaptureDeviceFormat::from_raw(ptr)))
    }

    pub fn active_video_min_frame_duration(&self) -> CMTime {
        unsafe { ffi::device::av_capture_device_active_video_min_frame_duration(self.ptr) }
    }

    pub fn active_video_max_frame_duration(&self) -> CMTime {
        unsafe { ffi::device::av_capture_device_active_video_max_frame_duration(self.ptr) }
    }

    pub fn lock_for_configuration(
        &self,
    ) -> Result<CaptureDeviceConfigurationLock<'_>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::device::av_capture_device_lock_for_configuration(self.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(CaptureDeviceConfigurationLock { device: self })
    }
}

pub struct CaptureDeviceConfigurationLock<'a> {
    device: &'a CaptureDevice,
}

impl CaptureDeviceConfigurationLock<'_> {
    pub fn set_active_format(&self, format: &CaptureDeviceFormat) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_format(self.device.ptr, format.ptr, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_active_video_min_frame_duration(
        &self,
        duration: CMTime,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_video_min_frame_duration(
                self.device.ptr,
                duration,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_active_video_max_frame_duration(
        &self,
        duration: CMTime,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_video_max_frame_duration(
                self.device.ptr,
                duration,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_exposure_mode(&self, mode: CaptureExposureMode) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_exposure_mode(
                self.device.ptr,
                mode.as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_torch_mode(&self, mode: CaptureTorchMode) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_torch_mode(self.device.ptr, mode.as_raw(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl Drop for CaptureDeviceConfigurationLock<'_> {
    fn drop(&mut self) {
        unsafe { ffi::device::av_capture_device_unlock_for_configuration(self.device.ptr) };
    }
}

fn preset_cstring(preset: &CaptureSessionPreset) -> Result<CString, AVCaptureError> {
    CString::new(preset.as_raw()).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("preset contains NUL byte: {error}"))
    })
}
