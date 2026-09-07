#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CStr;
use std::ffi::CString;

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;
use serde::{Deserialize, Serialize};

#[path = "video_data_output_timecode.rs"]
mod timecode_support;

pub use self::timecode_support::{
    CaptureTimecode, CaptureTimecodeGenerator, CaptureTimecodeGeneratorEvent,
    CaptureTimecodeGeneratorInfo, CaptureTimecodeGeneratorSynchronizationStatus,
    CaptureTimecodeSource, CaptureTimecodeSourceInfo, CaptureTimecodeSourceType,
    TimecodeMetadataSampleBuffer,
};

use crate::callback::{ArcContext, SerializedCallback};
use crate::error::report_callback_error;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{optional_json_cstring, parse_json_and_free};
use crate::output::{AVCaptureOutputDataDroppedReason, CaptureOutputRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `AVCaptureVideoDataOutput`.
pub struct VideoOutputSettings {
    /// The pixel format reported by `AVCaptureVideoDataOutput`.
    pub pixel_format: u32,
    /// The width component.
    pub width: Option<i32>,
    /// The height component.
    pub height: Option<i32>,
}

impl VideoOutputSettings {
    #[must_use]
    /// Creates a new `AVCaptureVideoDataOutput` wrapper.
    pub const fn new(pixel_format: u32) -> Self {
        Self {
            pixel_format,
            width: None,
            height: None,
        }
    }

    #[must_use]
    /// Returns video settings configured for BGRA pixels.
    pub const fn bgra() -> Self {
        Self::new(u32::from_be_bytes(*b"BGRA"))
    }

    #[must_use]
    /// Returns a copy with explicit dimensions set.
    pub const fn with_dimensions(mut self, width: i32, height: i32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureVideoDataOutput` state.
pub struct VideoDataOutputInfo {
    /// The connection count reported by `AVCaptureVideoDataOutput`.
    pub connection_count: usize,
    /// The always discards late video frames reported by `AVCaptureVideoDataOutput`.
    pub always_discards_late_video_frames: bool,
    #[serde(
        rename = "availableVideoCVPixelFormatTypes",
        alias = "availableVideoCvPixelFormatTypes"
    )]
    /// The available video cv pixel format types reported by `AVCaptureVideoDataOutput`.
    pub available_video_cv_pixel_format_types: Vec<u32>,
    /// The callback installed reported by `AVCaptureVideoDataOutput`.
    pub callback_installed: bool,
    /// The video settings reported by `AVCaptureVideoDataOutput`.
    pub video_settings: Option<VideoOutputSettings>,
    /// The dropped sample count reported by `AVCaptureVideoDataOutput`.
    pub dropped_sample_count: usize,
    /// The last dropped sample reason reported by `AVCaptureVideoDataOutput`.
    pub last_dropped_sample_reason: Option<AVCaptureOutputDataDroppedReason>,
}

#[derive(Debug, Clone)]
/// Event delivered by `AVCaptureVideoDataOutputSampleBufferDelegate`.
pub enum VideoDataOutputEvent {
    /// A captured video sample and its image buffer, when present.
    Sample {
        /// The retained sample buffer.
        sample_buffer: CMSampleBuffer,
        /// The retained image buffer associated with the sample.
        pixel_buffer: Option<CVPixelBuffer>,
    },
    /// A frame dropped by the native capture pipeline.
    Dropped {
        /// The retained dropped-frame sample buffer.
        sample_buffer: CMSampleBuffer,
        /// The native drop reason, when one was supplied.
        reason: Option<AVCaptureOutputDataDroppedReason>,
        /// Total dropped frames observed by this output.
        total: u64,
    },
}

type VideoSampleCallbackState = SerializedCallback<(CMSampleBuffer, Option<CVPixelBuffer>)>;
type VideoEventCallbackState = SerializedCallback<VideoDataOutputEvent>;

/// Safe wrapper around `AVCaptureVideoDataOutput`.
#[derive(Debug)]
/// Wraps `AVCaptureVideoDataOutput`.
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

impl crate::output::sealed::Sealed for VideoDataOutput {}

impl VideoDataOutput {
    /// Creates a new `AVCaptureVideoDataOutput` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::video_data_output::av_capture_video_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureVideoDataOutput` state.
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

    /// Returns the connection count reported by `AVCaptureVideoDataOutput`.
    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    /// Returns the available video cv pixel format types reported by `AVCaptureVideoDataOutput`.
    pub fn available_video_cv_pixel_format_types(&self) -> Result<Vec<u32>, AVCaptureError> {
        Ok(self.info()?.available_video_cv_pixel_format_types)
    }

    /// Corresponds to `AVCaptureVideoDataOutput.video_settings`.
    pub fn video_settings(&self) -> Result<Option<VideoOutputSettings>, AVCaptureError> {
        Ok(self.info()?.video_settings)
    }

    /// Corresponds to `AVCaptureVideoDataOutput.always_discards_late_video_frames`.
    pub fn always_discards_late_video_frames(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.always_discards_late_video_frames)
    }

    /// Corresponds to `AVCaptureVideoDataOutput.callback_installed`.
    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    /// Returns the dropped sample count reported by `AVCaptureVideoDataOutput`.
    pub fn dropped_sample_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.dropped_sample_count)
    }

    /// Corresponds to `AVCaptureVideoDataOutput.last_dropped_sample_reason`.
    pub fn last_dropped_sample_reason(
        &self,
    ) -> Result<Option<AVCaptureOutputDataDroppedReason>, AVCaptureError> {
        Ok(self.info()?.last_dropped_sample_reason)
    }

    /// Sets the video settings on `AVCaptureVideoDataOutput`.
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

    /// Sets the always discards late video frames on `AVCaptureVideoDataOutput`.
    pub fn set_always_discards_late_video_frames(&self, enabled: bool) {
        unsafe {
            ffi::video_data_output::av_capture_video_output_set_always_discards_late_video_frames(
                self.ptr, enabled,
            );
        }
    }

    /// Sets the sample-buffer handler on `AVCaptureVideoDataOutput`.
    pub fn set_sample_buffer_handler<F>(
        &self,
        queue_label: Option<&str>,
        mut callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(CMSampleBuffer, Option<CVPixelBuffer>) + Send + 'static,
    {
        let queue_label = queue_label.unwrap_or("avcapture-video-output");
        let queue_label = CString::new(queue_label).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
        })?;
        let state = ArcContext::new(VideoSampleCallbackState::new(
            move |(sample_buffer, pixel_buffer)| callback(sample_buffer, pixel_buffer),
        ));
        let userdata = state.as_ptr();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_data_output::av_capture_video_output_set_sample_buffer_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(video_sample_trampoline),
                userdata,
                Some(video_sample_callback_retain),
                Some(video_sample_callback_release),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets a handler that receives both captured samples and native dropped-frame events.
    pub fn set_sample_buffer_event_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(VideoDataOutputEvent) + Send + 'static,
    {
        let queue_label = queue_label.unwrap_or("avcapture-video-output");
        let queue_label = CString::new(queue_label).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
        })?;
        let state = ArcContext::new(VideoEventCallbackState::new(callback));
        let userdata = state.as_ptr();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_data_output::av_capture_video_output_set_sample_buffer_event_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(video_event_trampoline),
                userdata,
                Some(video_event_callback_retain),
                Some(video_event_callback_release),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Clears the sample buffer handler on `AVCaptureVideoDataOutput`.
    pub fn clear_sample_buffer_handler(&self) {
        unsafe {
            ffi::video_data_output::av_capture_video_output_clear_sample_buffer_callback(self.ptr);
        }
    }

    /// Clears the sample and dropped-frame event handler on `AVCaptureVideoDataOutput`.
    pub fn clear_sample_buffer_event_handler(&self) {
        unsafe {
            ffi::video_data_output::av_capture_video_output_clear_sample_buffer_event_callback(
                self.ptr,
            );
        }
    }
}

#[allow(unused_unsafe)]
unsafe extern "C" fn video_sample_trampoline(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
) {
    let sample_buffer = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let pixel_buffer = unsafe { CVPixelBuffer::from_raw(pixel_buffer) };
    let Some(state) = ArcContext::<VideoSampleCallbackState>::get(userdata) else {
        drop(sample_buffer);
        drop(pixel_buffer);
        return;
    };
    let Some(sample_buffer) = sample_buffer else {
        return;
    };
    state.dispatch("video_sample_trampoline", (sample_buffer, pixel_buffer));
}

#[allow(unused_unsafe)]
unsafe extern "C" fn video_event_trampoline(
    userdata: *mut c_void,
    kind: i32,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
    dropped_reason: *mut c_char,
    dropped_total: u64,
) {
    let sample_buffer = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let pixel_buffer = unsafe { CVPixelBuffer::from_raw(pixel_buffer) };
    let reason = if dropped_reason.is_null() {
        None
    } else {
        let reason = CStr::from_ptr(dropped_reason)
            .to_string_lossy()
            .into_owned();
        ffi::core::avc_string_free(dropped_reason);
        Some(AVCaptureOutputDataDroppedReason::from_raw(reason))
    };
    let Some(state) = ArcContext::<VideoEventCallbackState>::get(userdata) else {
        drop(sample_buffer);
        drop(pixel_buffer);
        return;
    };
    let event = match (kind, sample_buffer) {
        (0, Some(sample_buffer)) => VideoDataOutputEvent::Sample {
            sample_buffer,
            pixel_buffer,
        },
        (1, Some(sample_buffer)) => VideoDataOutputEvent::Dropped {
            sample_buffer,
            reason,
            total: dropped_total,
        },
        (unknown, sample_buffer) => {
            drop(sample_buffer);
            drop(pixel_buffer);
            report_callback_error(
                "video_event_trampoline",
                AVCaptureError::BridgeProtocol(format!(
                    "unknown video sample event kind {unknown}"
                )),
            );
            return;
        }
    };
    state.dispatch("video_event_trampoline", event);
}

unsafe extern "C" fn video_sample_callback_retain(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    ArcContext::<VideoSampleCallbackState>::retain(userdata);
}

unsafe extern "C" fn video_sample_callback_release(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    ArcContext::<VideoSampleCallbackState>::release(userdata);
}

unsafe extern "C" fn video_event_callback_retain(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    ArcContext::<VideoEventCallbackState>::retain(userdata);
}

unsafe extern "C" fn video_event_callback_release(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    ArcContext::<VideoEventCallbackState>::release(userdata);
}

#[cfg(test)]
mod tests {
    use super::{AVCaptureOutputDataDroppedReason, VideoDataOutput};
    use crate::ffi;
    use std::ffi::CString;

    #[test]
    fn synthetic_native_drops_increment_count_with_optional_reason() {
        let output = VideoDataOutput::new().expect("video output should be constructible");
        let first = unsafe {
            ffi::video_data_output::av_capture_video_output_record_drop_for_testing(
                output.ptr,
                core::ptr::null(),
            )
        };
        assert_eq!(first, 1);

        let reason = CString::new("lateData").expect("drop reason CString");
        let second = unsafe {
            ffi::video_data_output::av_capture_video_output_record_drop_for_testing(
                output.ptr,
                reason.as_ptr(),
            )
        };
        assert_eq!(second, 2);

        let info = output.info().expect("video output info should decode");
        assert_eq!(info.dropped_sample_count, 2);
        assert_eq!(
            info.last_dropped_sample_reason,
            Some(AVCaptureOutputDataDroppedReason::LateData)
        );
    }
}
