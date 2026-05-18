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
/// Snapshot of `AVCaptureSession` state.
pub struct CaptureSessionInfo {
    /// The session preset reported by `AVCaptureSession`.
    pub session_preset: String,
    /// The input count reported by `AVCaptureSession`.
    pub input_count: usize,
    /// The output count reported by `AVCaptureSession`.
    pub output_count: usize,
    /// The connection count reported by `AVCaptureSession`.
    pub connection_count: usize,
    /// The is running reported by `AVCaptureSession`.
    pub is_running: bool,
    /// The supports controls reported by `AVCaptureSession`.
    pub supports_controls: Option<bool>,
    /// The controls count reported by `AVCaptureSession`.
    pub controls_count: Option<usize>,
    /// The max controls count reported by `AVCaptureSession`.
    pub max_controls_count: Option<usize>,
    /// The controls delegate installed reported by `AVCaptureSession`.
    pub controls_delegate_installed: Option<bool>,
    /// The manual deferred start supported reported by `AVCaptureSession`.
    pub manual_deferred_start_supported: Option<bool>,
    /// The automatically runs deferred start reported by `AVCaptureSession`.
    pub automatically_runs_deferred_start: Option<bool>,
    /// The deferred start delegate installed reported by `AVCaptureSession`.
    pub deferred_start_delegate_installed: Option<bool>,
}

/// `AVCaptureSessionPreset` values supported on macOS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
/// `AVCaptureSessionPreset` values.
pub enum CaptureSessionPreset {
    /// Corresponds to the `Photo` case.
    Photo,
    /// Corresponds to the `High` case.
    High,
    /// Corresponds to the `Medium` case.
    Medium,
    /// Corresponds to the `Low` case.
    Low,
    /// Corresponds to the `Qvga320x240` case.
    Qvga320x240,
    /// Corresponds to the `Cif352x288` case.
    Cif352x288,
    /// Corresponds to the `Vga640x480` case.
    Vga640x480,
    /// Corresponds to the `Qhd960x540` case.
    Qhd960x540,
    /// Corresponds to the `Hd1280x720` case.
    Hd1280x720,
    /// Corresponds to the `FullHd1920x1080` case.
    FullHd1920x1080,
    /// Corresponds to the `Uhd3840x2160` case.
    Uhd3840x2160,
    /// Corresponds to the `IFrame960x540` case.
    IFrame960x540,
    /// Corresponds to the `IFrame1280x720` case.
    IFrame1280x720,
    /// A value not recognized by this crate.
    Unknown(String),
}

impl CaptureSessionPreset {
    #[must_use]
    /// Returns the raw SDK value for `AVCaptureSessionPreset`.
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
    /// Wraps an existing `AVCaptureSessionPreset` pointer.
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
#[derive(Debug)]
/// Wraps `AVCaptureSession`.
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
    /// Notification name published by `AVCaptureSession`.
    pub const RUNTIME_ERROR_NOTIFICATION: &str = "AVCaptureSessionRuntimeErrorNotification";
    /// User-info key used with `AVCaptureSession` notifications.
    pub const ERROR_KEY: &str = "AVCaptureSessionErrorKey";
    /// Notification name published by `AVCaptureSession`.
    pub const DID_START_RUNNING_NOTIFICATION: &str = "AVCaptureSessionDidStartRunningNotification";
    /// Notification name published by `AVCaptureSession`.
    pub const DID_STOP_RUNNING_NOTIFICATION: &str = "AVCaptureSessionDidStopRunningNotification";
    /// Notification name published by `AVCaptureSession`.
    pub const WAS_INTERRUPTED_NOTIFICATION: &str = "AVCaptureSessionWasInterruptedNotification";
    /// Notification name published by `AVCaptureSession`.
    pub const INTERRUPTION_ENDED_NOTIFICATION: &str =
        "AVCaptureSessionInterruptionEndedNotification";

    /// Creates a new `AVCaptureSession` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::session::av_capture_session_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::SESSION_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureSession` state.
    pub fn info(&self) -> Result<CaptureSessionInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::session::av_capture_session_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::SESSION_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns whether `AVCaptureSession` supports controls.
    pub fn supports_controls(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_controls.unwrap_or(false))
    }

    /// Returns the controls count reported by `AVCaptureSession`.
    pub fn controls_count(&self) -> Result<usize, AVCaptureError> {
        if !self.supports_controls()? {
            return Ok(0);
        }
        Ok(session_controls_count(self.ptr))
    }

    /// Returns the max controls count reported by `AVCaptureSession`.
    pub fn max_controls_count(&self) -> Result<Option<usize>, AVCaptureError> {
        Ok(self.info()?.max_controls_count)
    }

    /// Corresponds to `AVCaptureSession.controls_delegate_installed`.
    pub fn controls_delegate_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.controls_delegate_installed.unwrap_or(false))
    }

    /// Corresponds to `AVCaptureSession.controls`.
    pub fn controls(&self) -> Result<Vec<CaptureControl>, AVCaptureError> {
        if !self.supports_controls()? {
            return Ok(Vec::new());
        }
        session_controls(self.ptr)
    }

    /// Returns whether `AVCaptureSession` can add control.
    pub fn can_add_control(&self, control: &CaptureControl) -> Result<bool, AVCaptureError> {
        if !self.supports_controls()? {
            return Ok(false);
        }
        Ok(session_can_add_control(self.ptr, control))
    }

    /// Corresponds to `AVCaptureSession.add_control`.
    pub fn add_control(&self, control: &CaptureControl) -> Result<(), AVCaptureError> {
        session_add_control(self.ptr, control)
    }

    /// Corresponds to `AVCaptureSession.remove_control`.
    pub fn remove_control(&self, control: &CaptureControl) {
        session_remove_control(self.ptr, control);
    }

    /// Sets the controls delegate handler on `AVCaptureSession`.
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

    /// Clears the controls delegate handler on `AVCaptureSession`.
    pub fn clear_controls_delegate_handler(&self) {
        clear_controls_delegate_handler(self.ptr);
    }

    /// Corresponds to `AVCaptureSession.manual_deferred_start_supported`.
    pub fn manual_deferred_start_supported(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.manual_deferred_start_supported)
    }

    /// Corresponds to `AVCaptureSession.automatically_runs_deferred_start`.
    pub fn automatically_runs_deferred_start(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.automatically_runs_deferred_start)
    }

    /// Corresponds to `AVCaptureSession.deferred_start_supported`.
    pub fn deferred_start_supported(&self) -> Result<bool, AVCaptureError> {
        let info = self.info()?;
        Ok(info.manual_deferred_start_supported.unwrap_or(false)
            || info.automatically_runs_deferred_start.unwrap_or(false))
    }

    /// Corresponds to `AVCaptureSession.deferred_start_delegate_installed`.
    pub fn deferred_start_delegate_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self
            .info()?
            .deferred_start_delegate_installed
            .unwrap_or(false))
    }

    /// Sets the deferred-start delegate handler on `AVCaptureSession`.
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

    /// Clears the deferred start delegate handler on `AVCaptureSession`.
    pub fn clear_deferred_start_delegate_handler(&self) {
        clear_deferred_start_delegate_handler(self.ptr);
    }

    /// Corresponds to `AVCaptureSession.index_picker`.
    pub fn index_picker(
        localized_title: &str,
        symbol_name: &str,
        number_of_indexes: usize,
    ) -> Result<CaptureIndexPicker, AVCaptureError> {
        CaptureIndexPicker::new(localized_title, symbol_name, number_of_indexes)
    }

    /// Corresponds to `AVCaptureSession.index_picker_with_titles`.
    pub fn index_picker_with_titles(
        localized_title: &str,
        symbol_name: &str,
        localized_index_titles: &[&str],
    ) -> Result<CaptureIndexPicker, AVCaptureError> {
        CaptureIndexPicker::new_with_titles(localized_title, symbol_name, localized_index_titles)
    }

    /// Corresponds to `AVCaptureSession.slider`.
    pub fn slider(
        localized_title: &str,
        symbol_name: &str,
        min_value: f32,
        max_value: f32,
    ) -> Result<CaptureSlider, AVCaptureError> {
        CaptureSlider::new(localized_title, symbol_name, min_value, max_value)
    }

    /// Corresponds to `AVCaptureSession.slider_with_step`.
    pub fn slider_with_step(
        localized_title: &str,
        symbol_name: &str,
        min_value: f32,
        max_value: f32,
        step: f32,
    ) -> Result<CaptureSlider, AVCaptureError> {
        CaptureSlider::new_with_step(localized_title, symbol_name, min_value, max_value, step)
    }

    /// Corresponds to `AVCaptureSession.slider_with_values`.
    pub fn slider_with_values(
        localized_title: &str,
        symbol_name: &str,
        values: &[f32],
    ) -> Result<CaptureSlider, AVCaptureError> {
        CaptureSlider::new_with_values(localized_title, symbol_name, values)
    }

    /// Corresponds to `AVCaptureSession.system_exposure_bias_slider`.
    pub fn system_exposure_bias_slider(
        device: &CaptureDevice,
    ) -> Result<CaptureSystemExposureBiasSlider, AVCaptureError> {
        CaptureSystemExposureBiasSlider::new(device)
    }

    /// Creates an `AVCaptureSystemExposureBiasSlider` wrapper with an action handler.
    pub fn system_exposure_bias_slider_with_handler<F>(
        device: &CaptureDevice,
        callback: F,
    ) -> Result<CaptureSystemExposureBiasSlider, AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        CaptureSystemExposureBiasSlider::new_with_handler(device, callback)
    }

    /// Corresponds to `AVCaptureSession.system_zoom_slider`.
    pub fn system_zoom_slider(
        device: &CaptureDevice,
    ) -> Result<CaptureSystemZoomSlider, AVCaptureError> {
        CaptureSystemZoomSlider::new(device)
    }

    /// Creates an `AVCaptureSystemZoomSlider` wrapper with an action handler.
    pub fn system_zoom_slider_with_handler<F>(
        device: &CaptureDevice,
        callback: F,
    ) -> Result<CaptureSystemZoomSlider, AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        CaptureSystemZoomSlider::new_with_handler(device, callback)
    }

    /// Corresponds to `AVCaptureSession.session_preset`.
    pub fn session_preset(&self) -> Result<CaptureSessionPreset, AVCaptureError> {
        Ok(CaptureSessionPreset::from_raw(&self.info()?.session_preset))
    }

    /// Returns whether `AVCaptureSession` is running.
    pub fn is_running(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_running)
    }

    /// Returns the input count reported by `AVCaptureSession`.
    pub fn input_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.input_count)
    }

    /// Returns the output count reported by `AVCaptureSession`.
    pub fn output_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.output_count)
    }

    /// Returns the connection count reported by `AVCaptureSession`.
    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    /// Returns the connections reported by the underlying API.
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

    /// Corresponds to `AVCaptureSession.begin_configuration`.
    pub fn begin_configuration(&self) {
        unsafe { ffi::session::av_capture_session_begin_configuration(self.ptr) };
    }

    /// Corresponds to `AVCaptureSession.commit_configuration`.
    pub fn commit_configuration(&self) {
        unsafe { ffi::session::av_capture_session_commit_configuration(self.ptr) };
    }

    /// Corresponds to `AVCaptureSession.start_running`.
    pub fn start_running(&self) {
        unsafe { ffi::session::av_capture_session_start_running(self.ptr) };
    }

    /// Corresponds to `AVCaptureSession.stop_running`.
    pub fn stop_running(&self) {
        unsafe { ffi::session::av_capture_session_stop_running(self.ptr) };
    }

    /// Returns whether `AVCaptureSession` can set session preset.
    pub fn can_set_session_preset(
        &self,
        preset: &CaptureSessionPreset,
    ) -> Result<bool, AVCaptureError> {
        let preset = preset_cstring(preset)?;
        Ok(unsafe { ffi::session::av_capture_session_can_set_preset(self.ptr, preset.as_ptr()) })
    }

    /// Sets the session preset on `AVCaptureSession`.
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

    /// Returns whether `AVCaptureSession` can add the supplied input.
    pub fn can_add_input<I: CaptureInputRef>(&self, input: &I) -> bool {
        unsafe { ffi::session::av_capture_session_can_add_input(self.ptr, input.input_ptr()) }
    }

    /// Adds the supplied input to `AVCaptureSession`.
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

    /// Removes the supplied input from `AVCaptureSession`.
    pub fn remove_input<I: CaptureInputRef>(&self, input: &I) {
        unsafe { ffi::session::av_capture_session_remove_input(self.ptr, input.input_ptr()) };
    }

    /// Returns whether `AVCaptureSession` can add the supplied output.
    pub fn can_add_output<O: CaptureOutputRef>(&self, output: &O) -> bool {
        unsafe { ffi::session::av_capture_session_can_add_output(self.ptr, output.output_ptr()) }
    }

    /// Adds the supplied output to `AVCaptureSession`.
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

    /// Removes the supplied output from `AVCaptureSession`.
    pub fn remove_output<O: CaptureOutputRef>(&self, output: &O) {
        unsafe { ffi::session::av_capture_session_remove_output(self.ptr, output.output_ptr()) };
    }

    /// Returns whether `AVCaptureSession` can add device input.
    pub fn can_add_device_input(&self, input: &DeviceInput) -> bool {
        self.can_add_input(input)
    }

    /// Corresponds to `AVCaptureSession.add_device_input`.
    pub fn add_device_input(&self, input: &DeviceInput) -> Result<(), AVCaptureError> {
        self.add_input(input)
    }

    /// Corresponds to `AVCaptureSession.remove_device_input`.
    pub fn remove_device_input(&self, input: &DeviceInput) {
        self.remove_input(input);
    }

    /// Returns whether `AVCaptureSession` can add screen input.
    pub fn can_add_screen_input(&self, input: &ScreenInput) -> bool {
        self.can_add_input(input)
    }

    /// Corresponds to `AVCaptureSession.add_screen_input`.
    pub fn add_screen_input(&self, input: &ScreenInput) -> Result<(), AVCaptureError> {
        self.add_input(input)
    }

    /// Corresponds to `AVCaptureSession.remove_screen_input`.
    pub fn remove_screen_input(&self, input: &ScreenInput) {
        self.remove_input(input);
    }

    /// Returns whether `AVCaptureSession` can add video data output.
    pub fn can_add_video_data_output(&self, output: &VideoDataOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_video_data_output`.
    pub fn add_video_data_output(&self, output: &VideoDataOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_video_data_output`.
    pub fn remove_video_data_output(&self, output: &VideoDataOutput) {
        self.remove_output(output);
    }

    /// Returns whether `AVCaptureSession` can add audio data output.
    pub fn can_add_audio_data_output(&self, output: &AudioDataOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_audio_data_output`.
    pub fn add_audio_data_output(&self, output: &AudioDataOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_audio_data_output`.
    pub fn remove_audio_data_output(&self, output: &AudioDataOutput) {
        self.remove_output(output);
    }

    /// Returns whether `AVCaptureSession` can add audio preview output.
    pub fn can_add_audio_preview_output(&self, output: &AudioPreviewOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_audio_preview_output`.
    pub fn add_audio_preview_output(
        &self,
        output: &AudioPreviewOutput,
    ) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_audio_preview_output`.
    pub fn remove_audio_preview_output(&self, output: &AudioPreviewOutput) {
        self.remove_output(output);
    }

    /// Returns whether `AVCaptureSession` can add photo output.
    pub fn can_add_photo_output(&self, output: &PhotoOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_photo_output`.
    pub fn add_photo_output(&self, output: &PhotoOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_photo_output`.
    pub fn remove_photo_output(&self, output: &PhotoOutput) {
        self.remove_output(output);
    }

    /// Returns whether `AVCaptureSession` can add movie file output.
    pub fn can_add_movie_file_output(&self, output: &MovieFileOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_movie_file_output`.
    pub fn add_movie_file_output(&self, output: &MovieFileOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_movie_file_output`.
    pub fn remove_movie_file_output(&self, output: &MovieFileOutput) {
        self.remove_output(output);
    }

    /// Returns whether `AVCaptureSession` can add audio file output.
    pub fn can_add_audio_file_output(&self, output: &AudioFileOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_audio_file_output`.
    pub fn add_audio_file_output(&self, output: &AudioFileOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_audio_file_output`.
    pub fn remove_audio_file_output(&self, output: &AudioFileOutput) {
        self.remove_output(output);
    }

    /// Returns whether `AVCaptureSession` can add metadata output.
    pub fn can_add_metadata_output(&self, output: &MetadataOutput) -> bool {
        self.can_add_output(output)
    }

    /// Corresponds to `AVCaptureSession.add_metadata_output`.
    pub fn add_metadata_output(&self, output: &MetadataOutput) -> Result<(), AVCaptureError> {
        self.add_output(output)
    }

    /// Corresponds to `AVCaptureSession.remove_metadata_output`.
    pub fn remove_metadata_output(&self, output: &MetadataOutput) {
        self.remove_output(output);
    }
}

fn preset_cstring(preset: &CaptureSessionPreset) -> Result<CString, AVCaptureError> {
    CString::new(preset.as_raw()).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("preset contains NUL byte: {error}"))
    })
}
