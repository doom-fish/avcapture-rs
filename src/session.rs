#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::{CStr, CString};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::device::DeviceInput;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::output::{AudioDataOutput, VideoDataOutput};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSessionInfo {
    pub session_preset: String,
    pub input_count: usize,
    pub output_count: usize,
    pub connection_count: usize,
    pub is_running: bool,
}

/// `AVCaptureSessionPreset` values supported on macOS.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CaptureSessionPreset {
    Photo,
    High,
    Medium,
    Low,
    Qvga320x240,
    Cif352x288,
    Vga640x480,
    Qhd960x540,
    Hd1280x720,
    FullHd1920x1080,
    Uhd3840x2160,
    IFrame960x540,
    IFrame1280x720,
    Unknown(String),
}

impl CaptureSessionPreset {
    #[must_use]
    pub fn as_raw(&self) -> &str {
        match self {
            Self::Photo => "photo",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Qvga320x240 => "320x240",
            Self::Cif352x288 => "352x288",
            Self::Vga640x480 => "640x480",
            Self::Qhd960x540 => "960x540",
            Self::Hd1280x720 => "1280x720",
            Self::FullHd1920x1080 => "1920x1080",
            Self::Uhd3840x2160 => "3840x2160",
            Self::IFrame960x540 => "iframe960x540",
            Self::IFrame1280x720 => "iframe1280x720",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "photo" => Self::Photo,
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            "320x240" => Self::Qvga320x240,
            "352x288" => Self::Cif352x288,
            "640x480" => Self::Vga640x480,
            "960x540" => Self::Qhd960x540,
            "1280x720" => Self::Hd1280x720,
            "1920x1080" => Self::FullHd1920x1080,
            "3840x2160" => Self::Uhd3840x2160,
            "iframe960x540" => Self::IFrame960x540,
            "iframe1280x720" => Self::IFrame1280x720,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// Safe wrapper around `AVCaptureSession`.
pub struct CaptureSession {
    ptr: *mut c_void,
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::av_capture_session_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureSession {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::av_capture_session_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::SESSION_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<CaptureSessionInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::av_capture_session_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::SESSION_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn session_preset(&self) -> Result<CaptureSessionPreset, AVCaptureError> {
        Ok(CaptureSessionPreset::from_raw(&self.info()?.session_preset))
    }

    pub fn is_running(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_running)
    }

    pub fn input_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.input_count)
    }

    pub fn output_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.output_count)
    }

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn begin_configuration(&self) {
        unsafe { ffi::av_capture_session_begin_configuration(self.ptr) };
    }

    pub fn commit_configuration(&self) {
        unsafe { ffi::av_capture_session_commit_configuration(self.ptr) };
    }

    pub fn start_running(&self) {
        unsafe { ffi::av_capture_session_start_running(self.ptr) };
    }

    pub fn stop_running(&self) {
        unsafe { ffi::av_capture_session_stop_running(self.ptr) };
    }

    pub fn can_set_session_preset(&self, preset: &CaptureSessionPreset) -> Result<bool, AVCaptureError> {
        let preset = preset_cstring(preset)?;
        Ok(unsafe { ffi::av_capture_session_can_set_preset(self.ptr, preset.as_ptr()) })
    }

    pub fn set_session_preset(&self, preset: &CaptureSessionPreset) -> Result<(), AVCaptureError> {
        let preset = preset_cstring(preset)?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_capture_session_set_preset(self.ptr, preset.as_ptr(), &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn can_add_device_input(&self, input: &DeviceInput) -> bool {
        unsafe { ffi::av_capture_session_can_add_input(self.ptr, input.ptr) }
    }

    pub fn add_device_input(&self, input: &DeviceInput) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_capture_session_add_input(self.ptr, input.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_device_input(&self, input: &DeviceInput) {
        unsafe { ffi::av_capture_session_remove_input(self.ptr, input.ptr) };
    }

    pub fn can_add_video_data_output(&self, output: &VideoDataOutput) -> bool {
        unsafe { ffi::av_capture_session_can_add_video_output(self.ptr, output.ptr) }
    }

    pub fn add_video_data_output(&self, output: &VideoDataOutput) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_capture_session_add_video_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_video_data_output(&self, output: &VideoDataOutput) {
        unsafe { ffi::av_capture_session_remove_video_output(self.ptr, output.ptr) };
    }

    pub fn can_add_audio_data_output(&self, output: &AudioDataOutput) -> bool {
        unsafe { ffi::av_capture_session_can_add_audio_output(self.ptr, output.ptr) }
    }

    pub fn add_audio_data_output(&self, output: &AudioDataOutput) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { ffi::av_capture_session_add_audio_output(self.ptr, output.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_audio_data_output(&self, output: &AudioDataOutput) {
        unsafe { ffi::av_capture_session_remove_audio_output(self.ptr, output.ptr) };
    }
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
