#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::{Deserialize, Serialize};

use crate::device::CaptureDevice;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::parse_json_and_free;
use crate::input::CaptureInputRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
/// `AVCaptureMultichannelAudioMode` values.
pub enum CaptureMultichannelAudioMode {
    /// Corresponds to the `None` case.
    None,
    /// Corresponds to the `Stereo` case.
    Stereo,
    /// Corresponds to the `FirstOrderAmbisonics` case.
    FirstOrderAmbisonics,
    /// A value not recognized by this crate.
    Unknown(i32),
}

impl CaptureMultichannelAudioMode {
    #[must_use]
    /// Wraps an existing `AVCaptureMultichannelAudioMode` pointer.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::None,
            1 => Self::Stereo,
            2 => Self::FirstOrderAmbisonics,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    /// Returns the raw SDK value for `AVCaptureMultichannelAudioMode`.
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::None => 0,
            Self::Stereo => 1,
            Self::FirstOrderAmbisonics => 2,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for CaptureMultichannelAudioMode {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<CaptureMultichannelAudioMode> for i32 {
    fn from(value: CaptureMultichannelAudioMode) -> Self {
        value.as_raw()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureDeviceInput` state.
pub struct DeviceInputInfo {
    /// The device unique id reported by `AVCaptureDeviceInput`.
    pub device_unique_id: String,
    /// The device localized name reported by `AVCaptureDeviceInput`.
    pub device_localized_name: String,
    /// The ports count reported by `AVCaptureDeviceInput`.
    pub ports_count: usize,
    #[serde(default)]
    /// The multichannel audio mode reported by `AVCaptureDeviceInput`.
    pub multichannel_audio_mode: Option<CaptureMultichannelAudioMode>,
    #[serde(default)]
    /// The wind noise removal supported reported by `AVCaptureDeviceInput`.
    pub wind_noise_removal_supported: bool,
    #[serde(default)]
    /// The wind noise removal enabled reported by `AVCaptureDeviceInput`.
    pub wind_noise_removal_enabled: bool,
}

/// Safe wrapper around `AVCaptureDeviceInput`.
#[derive(Debug)]
/// Wraps `AVCaptureDeviceInput`.
pub struct DeviceInput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for DeviceInput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device_input::av_capture_device_input_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl DeviceInput {
    /// Notification name published by `AVCaptureDeviceInput`.
    pub const INPUT_PORT_FORMAT_DESCRIPTION_DID_CHANGE_NOTIFICATION: &str =
        "AVCaptureInputPortFormatDescriptionDidChangeNotification";

    /// Creates a new `AVCaptureDeviceInput` wrapper.
    pub fn new(device: &CaptureDevice) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::device_input::av_capture_device_input_create(device.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureDeviceInput` state.
    pub fn info(&self) -> Result<DeviceInputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::device_input::av_capture_device_input_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureDeviceInput.device_unique_id`.
    pub fn device_unique_id(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.device_unique_id)
    }

    /// Corresponds to `AVCaptureDeviceInput.device_localized_name`.
    pub fn device_localized_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.device_localized_name)
    }

    /// Returns the number of input ports reported by `AVCaptureInput`.
    pub fn ports_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.ports_count)
    }

    /// Corresponds to `AVCaptureDeviceInput.multichannel_audio_mode`.
    pub fn multichannel_audio_mode(
        &self,
    ) -> Result<Option<CaptureMultichannelAudioMode>, AVCaptureError> {
        Ok(self.info()?.multichannel_audio_mode)
    }

    /// Corresponds to `AVCaptureDeviceInput.wind_noise_removal_supported`.
    pub fn wind_noise_removal_supported(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.wind_noise_removal_supported)
    }

    /// Corresponds to `AVCaptureDeviceInput.wind_noise_removal_enabled`.
    pub fn wind_noise_removal_enabled(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.wind_noise_removal_enabled)
    }

    /// Returns whether `AVCaptureDeviceInput` is multichannel audio mode supported.
    pub fn is_multichannel_audio_mode_supported(
        &self,
        mode: impl Into<CaptureMultichannelAudioMode>,
    ) -> bool {
        unsafe {
            ffi::device_input::av_capture_device_input_is_multichannel_audio_mode_supported(
                self.ptr,
                mode.into().as_raw(),
            )
        }
    }

    /// Sets the multichannel audio mode on `AVCaptureDeviceInput`.
    pub fn set_multichannel_audio_mode(
        &self,
        mode: impl Into<CaptureMultichannelAudioMode>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device_input::av_capture_device_input_set_multichannel_audio_mode(
                self.ptr,
                mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the wind noise removal enabled on `AVCaptureDeviceInput`.
    pub fn set_wind_noise_removal_enabled(&self, enabled: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device_input::av_capture_device_input_set_wind_noise_removal_enabled(
                self.ptr, enabled, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl CaptureInputRef for DeviceInput {
    fn input_ptr(&self) -> *mut c_void {
        self.ptr
    }
}
