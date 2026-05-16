#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;
use serde::{Deserialize, Serialize};

use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{optional_json_cstring, parse_json_and_free};
use crate::output::CaptureOutputRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOutputSettings {
    pub pixel_format: u32,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

impl VideoOutputSettings {
    #[must_use]
    pub const fn new(pixel_format: u32) -> Self {
        Self {
            pixel_format,
            width: None,
            height: None,
        }
    }

    #[must_use]
    pub const fn bgra() -> Self {
        Self::new(u32::from_be_bytes(*b"BGRA"))
    }

    #[must_use]
    pub const fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDataOutputInfo {
    pub connection_count: usize,
    pub always_discards_late_video_frames: bool,
    pub available_video_cv_pixel_format_types: Vec<u32>,
    pub callback_installed: bool,
    pub video_settings: Option<VideoOutputSettings>,
}

struct VideoCallbackState {
    callback: Box<dyn FnMut(CMSampleBuffer, Option<CVPixelBuffer>) + Send + 'static>,
}

/// Safe wrapper around `AVCaptureVideoDataOutput`.
pub struct VideoDataOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for VideoDataOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::video_data_output::av_capture_video_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureOutputRef for VideoDataOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl VideoDataOutput {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::video_data_output::av_capture_video_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<VideoDataOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_data_output::av_capture_video_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn available_video_cv_pixel_format_types(&self) -> Result<Vec<u32>, AVCaptureError> {
        Ok(self.info()?.available_video_cv_pixel_format_types)
    }

    pub fn video_settings(&self) -> Result<Option<VideoOutputSettings>, AVCaptureError> {
        Ok(self.info()?.video_settings)
    }

    pub fn always_discards_late_video_frames(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.always_discards_late_video_frames)
    }

    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    pub fn set_video_settings(
        &self,
        settings: Option<&VideoOutputSettings>,
    ) -> Result<(), AVCaptureError> {
        let settings = optional_json_cstring(settings, "video output settings")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_data_output::av_capture_video_output_set_video_settings_json(
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

    pub fn set_always_discards_late_video_frames(&self, enabled: bool) {
        unsafe {
            ffi::video_data_output::av_capture_video_output_set_always_discards_late_video_frames(
                self.ptr, enabled,
            );
        }
    }

    pub fn set_sample_buffer_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(CMSampleBuffer, Option<CVPixelBuffer>) + Send + 'static,
    {
        let queue_label = queue_label.unwrap_or("avcapture-video-output");
        let queue_label = CString::new(queue_label).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
        })?;
        let state = Box::new(VideoCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_data_output::av_capture_video_output_set_sample_buffer_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(video_sample_trampoline),
                userdata,
                Some(video_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { video_callback_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn clear_sample_buffer_handler(&self) {
        unsafe {
            ffi::video_data_output::av_capture_video_output_clear_sample_buffer_callback(self.ptr);
        }
    }
}

unsafe extern "C" fn video_sample_trampoline(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
) {
    let Some(state) = userdata.cast::<VideoCallbackState>().as_mut() else {
        return;
    };
    let Some(sample_buffer) = CMSampleBuffer::from_raw(sample_buffer) else {
        return;
    };
    let pixel_buffer = CVPixelBuffer::from_raw(pixel_buffer);
    (state.callback)(sample_buffer, pixel_buffer);
}

unsafe extern "C" fn video_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(userdata.cast::<VideoCallbackState>()));
}
