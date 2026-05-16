#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::session::CaptureSessionPreset;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDeviceInfo {
    pub unique_id: String,
    pub localized_name: String,
    pub manufacturer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInputInfo {
    pub device_unique_id: String,
    pub device_localized_name: String,
    pub ports_count: usize,
}

/// `AVFoundation` media types used by capture APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// `AVAuthorizationStatus` for camera / microphone capture access.
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

/// Safe wrapper around `AVCaptureDevice`.
pub struct CaptureDevice {
    pub(crate) ptr: *mut c_void,
}

impl Drop for CaptureDevice {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_capture_device_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureDevice {
    pub fn authorization_status(media_type: &MediaType) -> Result<AuthorizationStatus, AVCaptureError> {
        let media_type = media_type_cstring(media_type)?;
        let mut err: *mut c_char = ptr::null_mut();
        let raw = unsafe { ffi::av_capture_authorization_status(media_type.as_ptr(), &mut err) };
        if raw < 0 {
            return Err(unsafe { from_swift(raw, err) });
        }
        Ok(AuthorizationStatus::from_raw(raw))
    }

    pub fn devices(media_type: &MediaType) -> Result<Vec<CaptureDeviceInfo>, AVCaptureError> {
        let media_type = media_type_cstring(media_type)?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_capture_devices_json(media_type.as_ptr(), &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn default(media_type: &MediaType) -> Result<Option<Self>, AVCaptureError> {
        let media_type = media_type_cstring(media_type)?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_capture_default_device(media_type.as_ptr(), &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    pub fn with_unique_id(unique_id: impl AsRef<str>) -> Result<Option<Self>, AVCaptureError> {
        let unique_id = CString::new(unique_id.as_ref())
            .map_err(|error| AVCaptureError::InvalidArgument(format!("device unique ID contains NUL byte: {error}")))?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_capture_device_with_unique_id(unique_id.as_ptr(), &mut err) };
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
        let json_ptr = unsafe { ffi::av_capture_device_info_json(self.ptr, &mut err) };
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

    pub fn supports_session_preset(&self, preset: &CaptureSessionPreset) -> Result<bool, AVCaptureError> {
        let preset = preset_cstring(preset)?;
        Ok(unsafe { ffi::av_capture_device_supports_session_preset(self.ptr, preset.as_ptr()) })
    }
}

/// Safe wrapper around `AVCaptureDeviceInput`.
pub struct DeviceInput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for DeviceInput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_capture_device_input_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl DeviceInput {
    pub fn new(device: &CaptureDevice) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_capture_device_input_create(device.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<DeviceInputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_capture_device_input_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn device_unique_id(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.device_unique_id)
    }

    pub fn device_localized_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.device_localized_name)
    }

    pub fn ports_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.ports_count)
    }
}

fn media_type_cstring(media_type: &MediaType) -> Result<CString, AVCaptureError> {
    CString::new(media_type.as_raw()).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("media type contains NUL byte: {error}"))
    })
}

fn preset_cstring(preset: &CaptureSessionPreset) -> Result<CString, AVCaptureError> {
    CString::new(preset.as_raw())
        .map_err(|error| AVCaptureError::InvalidArgument(format!("preset contains NUL byte: {error}")))
}

fn parse_json_and_free<T: DeserializeOwned>(json_ptr: *mut c_char) -> Result<T, AVCaptureError> {
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::avc_string_free(json_ptr) };
    serde_json::from_str::<T>(&json)
        .map_err(|error| AVCaptureError::OperationFailed(format!("failed to decode bridge JSON: {error}")))
}
