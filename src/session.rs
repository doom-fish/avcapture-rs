#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use serde::{Deserialize, Serialize};

#[path = "session_controls.rs"]
mod controls;

use self::controls::{
    clear_controls_delegate_handler, clear_deferred_start_delegate_handler,
    install_controls_delegate_handler, install_deferred_start_delegate_handler,
    session_add_control, session_can_add_control, session_controls, session_controls_count,
    session_remove_control,
};
pub use self::controls::{
    CaptureControl, CaptureControlInfo, CaptureIndexPicker, CaptureIndexPickerInfo,
    CaptureSessionControlsEvent, CaptureSessionDeferredStartEvent, CaptureSlider,
    CaptureSliderInfo, CaptureSystemExposureBiasSlider, CaptureSystemZoomSlider,
};
use crate::audio_data_output::{AudioDataOutput, AudioPreviewOutput};
use crate::connection::CaptureConnection;
use crate::device::CaptureDevice;
use crate::device_input::DeviceInput;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::parse_json_and_free;
use crate::input::CaptureInputRef;
use crate::metadata_output::MetadataOutput;
use crate::movie_file_output::{AudioFileOutput, MovieFileOutput};
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
    pub supports_controls: Option<bool>,
    pub controls_count: Option<usize>,
    pub max_controls_count: Option<usize>,
    pub controls_delegate_installed: Option<bool>,
    pub manual_deferred_start_supported: Option<bool>,
    pub automatically_runs_deferred_start: Option<bool>,
    pub deferred_start_delegate_installed: Option<bool>,
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
    pub const RUNTIME_ERROR_NOTIFICATION: &str = "AVCaptureSessionRuntimeErrorNotification";
    pub const ERROR_KEY: &str = "AVCaptureSessionErrorKey";
    pub const DID_START_RUNNING_NOTIFICATION: &str = "AVCaptureSessionDidStartRunningNotification";
    pub const DID_STOP_RUNNING_NOTIFICATION: &str = "AVCaptureSessionDidStopRunningNotification";
    pub const WAS_INTERRUPTED_NOTIFICATION: &str = "AVCaptureSessionWasInterruptedNotification";
    pub const INTERRUPTION_ENDED_NOTIFICATION: &str =
        "AVCaptureSessionInterruptionEndedNotification";

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

    pub fn supports_controls(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_controls.unwrap_or(false))
    }

    pub fn controls_count(&self) -> Result<usize, AVCaptureError> {
        if !self.supports_controls()? {
            return Ok(0);
        }
        Ok(session_controls_count(self.ptr))
    }

    pub fn max_controls_count(&self) -> Result<Option<usize>, AVCaptureError> {
        Ok(self.info()?.max_controls_count)
    }

    pub fn controls_delegate_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.controls_delegate_installed.unwrap_or(false))
    }

    pub fn controls(&self) -> Result<Vec<CaptureControl>, AVCaptureError> {
        if !self.supports_controls()? {
            return Ok(Vec::new());
        }
        session_controls(self.ptr)
    }

    pub fn can_add_control(&self, control: &CaptureControl) -> Result<bool, AVCaptureError> {
        if !self.supports_controls()? {
            return Ok(false);
        }
        Ok(session_can_add_control(self.ptr, control))
    }

    pub fn add_control(&self, control: &CaptureControl) -> Result<(), AVCaptureError> {
        session_add_control(self.ptr, control)
    }

    pub fn remove_control(&self, control: &CaptureControl) {
        session_remove_control(self.ptr, control);
    }

    pub fn set_controls_delegate_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(CaptureSessionControlsEvent) + Send + 'static,
    {
        install_controls_delegate_handler(self.ptr, queue_label, callback)
    }

    pub fn clear_controls_delegate_handler(&self) {
        clear_controls_delegate_handler(self.ptr);
    }

    pub fn manual_deferred_start_supported(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.manual_deferred_start_supported)
    }

    pub fn automatically_runs_deferred_start(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.automatically_runs_deferred_start)
    }

    pub fn deferred_start_supported(&self) -> Result<bool, AVCaptureError> {
        let info = self.info()?;
        Ok(info.manual_deferred_start_supported.unwrap_or(false)
            || info.automatically_runs_deferred_start.unwrap_or(false))
    }

    pub fn deferred_start_delegate_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self
            .info()?
            .deferred_start_delegate_installed
            .unwrap_or(false))
    }

    pub fn set_deferred_start_delegate_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(CaptureSessionDeferredStartEvent) + Send + 'static,
    {
        install_deferred_start_delegate_handler(self.ptr, queue_label, callback)
    }

    pub fn clear_deferred_start_delegate_handler(&self) {
        clear_deferred_start_delegate_handler(self.ptr);
    }

    pub fn index_picker(
        localized_title: &str,
        symbol_name: &str,
        number_of_indexes: usize,
    ) -> Result<CaptureIndexPicker, AVCaptureError> {
        CaptureIndexPicker::new(localized_title, symbol_name, number_of_indexes)
    }

    pub fn index_picker_with_titles(
        localized_title: &str,
        symbol_name: &str,
        localized_index_titles: &[&str],
    ) -> Result<CaptureIndexPicker, AVCaptureError> {
        CaptureIndexPicker::new_with_titles(localized_title, symbol_name, localized_index_titles)
    }

    pub fn slider(
        localized_title: &str,
        symbol_name: &str,
        min_value: f32,
        max_value: f32,
    ) -> Result<CaptureSlider, AVCaptureError> {
        CaptureSlider::new(localized_title, symbol_name, min_value, max_value)
    }

    pub fn slider_with_step(
        localized_title: &str,
        symbol_name: &str,
        min_value: f32,
        max_value: f32,
        step: f32,
    ) -> Result<CaptureSlider, AVCaptureError> {
        CaptureSlider::new_with_step(localized_title, symbol_name, min_value, max_value, step)
    }

    pub fn slider_with_values(
        localized_title: &str,
        symbol_name: &str,
        values: &[f32],
    ) -> Result<CaptureSlider, AVCaptureError> {
        CaptureSlider::new_with_values(localized_title, symbol_name, values)
    }

    pub fn system_exposure_bias_slider(
        device: &CaptureDevice,
    ) -> Result<CaptureSystemExposureBiasSlider, AVCaptureError> {
        CaptureSystemExposureBiasSlider::new(device)
    }

    pub fn system_exposure_bias_slider_with_handler<F>(
        device: &CaptureDevice,
        callback: F,
    ) -> Result<CaptureSystemExposureBiasSlider, AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        CaptureSystemExposureBiasSlider::new_with_handler(device, callback)
    }

    pub fn system_zoom_slider(
        device: &CaptureDevice,
    ) -> Result<CaptureSystemZoomSlider, AVCaptureError> {
        CaptureSystemZoomSlider::new(device)
    }

    pub fn system_zoom_slider_with_handler<F>(
        device: &CaptureDevice,
        callback: F,
    ) -> Result<CaptureSystemZoomSlider, AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        CaptureSystemZoomSlider::new_with_handler(device, callback)
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

    pub fn can_add_audio_preview_output(&self, output: &AudioPreviewOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_audio_preview_output(
        &self,
        output: &AudioPreviewOutput,
    ) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_audio_preview_output(&self, output: &AudioPreviewOutput) {
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

    pub fn can_add_audio_file_output(&self, output: &AudioFileOutput) -> bool {
        self.can_add_output(output)
    }

    pub fn add_audio_file_output(&self, output: &AudioFileOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    pub fn remove_audio_file_output(&self, output: &AudioFileOutput) {
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
