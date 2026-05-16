#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::audio_data_output::AudioDataOutput;
use crate::connection::CaptureConnection;
use crate::device_input::DeviceInput;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::parse_json_and_free;
use crate::input::CaptureInputRef;
use crate::metadata_output::MetadataOutput;
use crate::movie_file_output::MovieFileOutput;
use crate::output::CaptureOutputRef;
use crate::photo_output::PhotoOutput;
use crate::screen_input::ScreenInput;
use crate::video_data_output::VideoDataOutput;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
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

impl From<String> for CaptureSessionPreset {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<CaptureSessionPreset> for String {
    fn from(value: CaptureSessionPreset) -> Self {
        value.as_raw().to_owned()
    }
}

/// Safe wrapper around `AVCaptureSession`.
pub struct CaptureSession {
    pub(crate) ptr: *mut c_void,
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::session::av_capture_session_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureSession {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::session::av_capture_session_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::SESSION_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<CaptureSessionInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::session::av_capture_session_info_json(self.ptr, &mut err) };
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

    pub fn connections(&self) -> Result<Vec<CaptureConnection>, AVCaptureError> {
        let count = unsafe { ffi::session::av_capture_session_connections_count(self.ptr) };
        let mut connections = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::session::av_capture_session_connection_at_index(self.ptr, index, &mut err)
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::SESSION_ERROR, err) });
            }
            connections.push(CaptureConnection::from_raw(ptr));
        }
        Ok(connections)
    }

    pub fn begin_configuration(&self) {
        unsafe { ffi::session::av_capture_session_begin_configuration(self.ptr) };
    }

    pub fn commit_configuration(&self) {
        unsafe { ffi::session::av_capture_session_commit_configuration(self.ptr) };
    }

    pub fn start_running(&self) {
        unsafe { ffi::session::av_capture_session_start_running(self.ptr) };
    }

    pub fn stop_running(&self) {
        unsafe { ffi::session::av_capture_session_stop_running(self.ptr) };
    }

    pub fn can_set_session_preset(
        &self,
        preset: &CaptureSessionPreset,
    ) -> Result<bool, AVCaptureError> {
        let preset = preset_cstring(preset)?;
        Ok(unsafe { ffi::session::av_capture_session_can_set_preset(self.ptr, preset.as_ptr()) })
    }

    pub fn set_session_preset(&self, preset: &CaptureSessionPreset) -> Result<(), AVCaptureError> {
        let preset = preset_cstring(preset)?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::session::av_capture_session_set_preset(self.ptr, preset.as_ptr(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn can_add_input<I: CaptureInputRef>(&self, input: &I) -> bool {
        unsafe { ffi::session::av_capture_session_can_add_input(self.ptr, input.input_ptr()) }
    }

    pub fn add_input<I: CaptureInputRef>(&self, input: &I) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::session::av_capture_session_add_input(self.ptr, input.input_ptr(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_input<I: CaptureInputRef>(&self, input: &I) {
        unsafe { ffi::session::av_capture_session_remove_input(self.ptr, input.input_ptr()) };
    }

    pub fn can_add_output<O: CaptureOutputRef>(&self, output: &O) -> bool {
        unsafe { ffi::session::av_capture_session_can_add_output(self.ptr, output.output_ptr()) }
    }

    pub fn add_output<O: CaptureOutputRef>(&self, output: &O) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::session::av_capture_session_add_output(self.ptr, output.output_ptr(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn remove_output<O: CaptureOutputRef>(&self, output: &O) {
        unsafe { ffi::session::av_capture_session_remove_output(self.ptr, output.output_ptr()) };
    }

    pub fn can_add_device_input(&self, input: &DeviceInput) -> bool {
        self.can_add_input(input)
    }

    pub fn add_device_input(&self, input: &DeviceInput) -> Result<(), AVCaptureError> {
        self.add_input(input)
    }

    pub fn remove_device_input(&self, input: &DeviceInput) {
        self.remove_input(input);
    }

    pub fn can_add_screen_input(&self, input: &ScreenInput) -> bool {
        self.can_add_input(input)
    }

    pub fn add_screen_input(&self, input: &ScreenInput) -> Result<(), AVCaptureError> {
        self.add_input(input)
    }

    pub fn remove_screen_input(&self, input: &ScreenInput) {
        self.remove_input(input);
    }

    pub fn can_add_video_data_output(&self, output: &VideoDataOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_video_data_output(&self, output: &VideoDataOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_video_data_output(&self, output: &VideoDataOutput) {
        self.remove_output(output);
    }

    pub fn can_add_audio_data_output(&self, output: &AudioDataOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_audio_data_output(&self, output: &AudioDataOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_audio_data_output(&self, output: &AudioDataOutput) {
        self.remove_output(output);
    }

    pub fn can_add_photo_output(&self, output: &PhotoOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_photo_output(&self, output: &PhotoOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_photo_output(&self, output: &PhotoOutput) {
        self.remove_output(output);
    }

    pub fn can_add_movie_file_output(&self, output: &MovieFileOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_movie_file_output(&self, output: &MovieFileOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_movie_file_output(&self, output: &MovieFileOutput) {
        self.remove_output(output);
    }

    pub fn can_add_metadata_output(&self, output: &MetadataOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_metadata_output(&self, output: &MetadataOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_metadata_output(&self, output: &MetadataOutput) {
        self.remove_output(output);
    }
}

fn preset_cstring(preset: &CaptureSessionPreset) -> Result<CString, AVCaptureError> {
    CString::new(preset.as_raw()).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("preset contains NUL byte: {error}"))
    })
}
