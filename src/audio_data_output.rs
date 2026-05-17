#![allow(clippy::missing_errors_doc, clippy::must_use_candidate, dead_code)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use apple_cf::cm::CMSampleBuffer;
use serde::{Deserialize, Serialize};

use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, optional_json_cstring, parse_json_and_free};
use crate::output::{AVCaptureOutputDataDroppedReason, CaptureOutputRef};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioOutputSettings {
    pub sample_rate: Option<f64>,
    pub channel_count: Option<u32>,
    pub bits_per_channel: u32,
    pub is_float: bool,
    pub is_non_interleaved: bool,
}

impl AudioOutputSettings {
    #[must_use]
    pub const fn pcm_i16(sample_rate: f64, channel_count: u32) -> Self {
        Self {
            sample_rate: Some(sample_rate),
            channel_count: Some(channel_count),
            bits_per_channel: 16,
            is_float: false,
            is_non_interleaved: false,
        }
    }

    #[must_use]
    pub const fn pcm_i32(sample_rate: f64, channel_count: u32) -> Self {
        Self {
            sample_rate: Some(sample_rate),
            channel_count: Some(channel_count),
            bits_per_channel: 32,
            is_float: false,
            is_non_interleaved: false,
        }
    }

    #[must_use]
    pub const fn pcm_f32(sample_rate: f64, channel_count: u32) -> Self {
        Self {
            sample_rate: Some(sample_rate),
            channel_count: Some(channel_count),
            bits_per_channel: 32,
            is_float: true,
            is_non_interleaved: false,
        }
    }

    #[must_use]
    pub const fn non_interleaved(mut self, non_interleaved: bool) -> Self {
        self.is_non_interleaved = non_interleaved;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDataOutputInfo {
    pub connection_count: usize,
    pub callback_installed: bool,
    pub audio_settings: Option<AudioOutputSettings>,
    pub dropped_sample_count: usize,
    pub last_dropped_sample_reason: Option<AVCaptureOutputDataDroppedReason>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioPreviewOutputInfo {
    pub connection_count: usize,
    pub output_device_unique_id: Option<String>,
    pub volume: f32,
}

struct AudioCallbackState {
    callback: Box<dyn FnMut(CMSampleBuffer) + Send + 'static>,
}

/// Safe wrapper around `AVCaptureAudioDataOutput`.
pub struct AudioDataOutput {
    pub(crate) ptr: *mut c_void,
}

/// Safe wrapper around `AVCaptureAudioPreviewOutput`.
pub struct AudioPreviewOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for AudioDataOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::audio_data_output::av_capture_audio_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl Drop for AudioPreviewOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::audio_data_output::av_capture_audio_preview_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureOutputRef for AudioDataOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl CaptureOutputRef for AudioPreviewOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl AudioDataOutput {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::audio_data_output::av_capture_audio_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<AudioDataOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::audio_data_output::av_capture_audio_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn audio_settings(&self) -> Result<Option<AudioOutputSettings>, AVCaptureError> {
        Ok(self.info()?.audio_settings)
    }

    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    pub fn dropped_sample_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.dropped_sample_count)
    }

    pub fn last_dropped_sample_reason(
        &self,
    ) -> Result<Option<AVCaptureOutputDataDroppedReason>, AVCaptureError> {
        Ok(self.info()?.last_dropped_sample_reason)
    }

    pub fn set_audio_settings(
        &self,
        settings: Option<&AudioOutputSettings>,
    ) -> Result<(), AVCaptureError> {
        let settings = optional_json_cstring(settings, "audio output settings")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::audio_data_output::av_capture_audio_output_set_audio_settings_json(
                self.ptr,
                settings.as_ref().map_or(ptr::null(), |json| json.as_ptr()),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_sample_buffer_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(CMSampleBuffer) + Send + 'static,
    {
        let queue_label = queue_label.unwrap_or("avcapture-audio-output");
        let queue_label = CString::new(queue_label).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
        })?;
        let state = Box::new(AudioCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::audio_data_output::av_capture_audio_output_set_sample_buffer_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(audio_sample_trampoline),
                userdata,
                Some(audio_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { audio_callback_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn clear_sample_buffer_handler(&self) {
        unsafe {
            ffi::audio_data_output::av_capture_audio_output_clear_sample_buffer_callback(self.ptr);
        }
    }
}

impl AudioPreviewOutput {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::audio_data_output::av_capture_audio_preview_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<AudioPreviewOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::audio_data_output::av_capture_audio_preview_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn output_device_unique_id(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.output_device_unique_id)
    }

    pub fn volume(&self) -> Result<f32, AVCaptureError> {
        Ok(self.info()?.volume)
    }

    pub fn set_output_device_unique_id(
        &self,
        output_device_unique_id: Option<&str>,
    ) -> Result<(), AVCaptureError> {
        let output_device_unique_id = output_device_unique_id
            .map(|value| cstring(value, "audio preview output device unique id"))
            .transpose()?;
        unsafe {
            ffi::audio_data_output::av_capture_audio_preview_output_set_output_device_unique_id(
                self.ptr,
                output_device_unique_id
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
            );
        }
        Ok(())
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), AVCaptureError> {
        if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
            return Err(AVCaptureError::InvalidArgument(
                "audio preview output volume must be a finite value between 0.0 and 1.0".to_owned(),
            ));
        }
        unsafe {
            ffi::audio_data_output::av_capture_audio_preview_output_set_volume(self.ptr, volume);
        }
        Ok(())
    }
}

unsafe extern "C" fn audio_sample_trampoline(userdata: *mut c_void, sample_buffer: *mut c_void) {
    // SAFETY: `userdata` is the `Box<AudioCallbackState>` cast to `*mut c_void`
    // in `set_sample_buffer_handler`. It is non-null and properly aligned for
    // the entire lifetime of this callback registration.
    let Some(state) = userdata.cast::<AudioCallbackState>().as_mut() else {
        return;
    };
    // SAFETY: `sample_buffer` is a `CMSampleBufferRef` at +1 retain passed from
    // the Swift bridge via `Unmanaged.passRetained(...).toOpaque()`.
    let Some(sample_buffer) = CMSampleBuffer::from_raw(sample_buffer) else {
        return;
    };
    // User closures can panic; catch them here so the panic doesn't unwind
    // across the `extern "C"` boundary (which is UB).
    doom_fish_utils::panic_safe::catch_user_panic("audio_sample_trampoline", || {
        (state.callback)(sample_buffer);
    });
}

unsafe extern "C" fn audio_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: `userdata` was created by `Box::into_raw(Box::new(AudioCallbackState { .. }))`
    // in `set_sample_buffer_handler` and is only freed here, exactly once.
    drop(Box::from_raw(userdata.cast::<AudioCallbackState>()));
}
