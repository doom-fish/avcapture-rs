#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
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

struct VideoCallbackState {
    callback: Box<dyn FnMut(CMSampleBuffer, Option<CVPixelBuffer>) + Send + 'static>,
    ref_count: AtomicUsize,
}

impl VideoCallbackState {
    /// Increment the reference count.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, live `VideoCallbackState`.
    unsafe fn retain(ptr: *mut Self) {
        unsafe { &*ptr }.ref_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement the reference count, freeing the state if it reaches zero.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a valid, live `VideoCallbackState`. After this call,
    /// `ptr` must not be used if the state was freed.
    unsafe fn release(ptr: *mut Self) {
        if ptr.is_null() {
            return;
        }
        let prev = unsafe { &*ptr }.ref_count.fetch_sub(1, Ordering::Release);
        if prev == 1 {
            // Acquire fence pairs with the Release stores of every other thread
            // that previously held a reference, so the freeing thread observes
            // all their writes. This is the canonical Arc-style refcount drop.
            core::sync::atomic::fence(Ordering::Acquire);
            drop(unsafe { Box::from_raw(ptr) });
        }
    }
}

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
            ref_count: AtomicUsize::new(1),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_data_output::av_capture_video_output_set_sample_buffer_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(video_sample_trampoline),
                userdata,
                Some(video_callback_retain),
                Some(video_callback_release),
                &mut err,
            )
        };
        // On success the Swift callback box took a +1 via `video_callback_retain`;
        // drop our creation reference so the state is owned solely by Swift and is
        // freed only once the box's `deinit` runs (after any in-flight callback).
        // On error Swift never retained, so this releases the final reference.
        unsafe { video_callback_release(userdata) };
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
}

unsafe extern "C" fn video_sample_trampoline(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
) {
    // SAFETY: `userdata` is the `Box<VideoCallbackState>` cast to `*mut c_void`
    // in `set_sample_buffer_handler`. It is non-null and properly aligned for
    // the entire lifetime of this callback registration.
    let Some(state) = userdata.cast::<VideoCallbackState>().as_mut() else {
        return;
    };
    // SAFETY: `sample_buffer` is a `CMSampleBufferRef` at +1 retain passed from
    // the Swift bridge via `Unmanaged.passRetained(...).toOpaque()`.
    let Some(sample_buffer) = CMSampleBuffer::from_raw(sample_buffer) else {
        return;
    };
    let pixel_buffer = CVPixelBuffer::from_raw(pixel_buffer);
    // User closures can panic; catch them here so the panic doesn't unwind
    // across the `extern "C"` boundary (which is UB).
    doom_fish_utils::panic_safe::catch_user_panic("video_sample_trampoline", || {
        (state.callback)(sample_buffer, pixel_buffer);
    });
}

unsafe extern "C" fn video_callback_retain(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: `userdata` is the `Box<VideoCallbackState>` cast to `*mut c_void`
    // in `set_sample_buffer_handler`, kept alive by the Swift callback box.
    unsafe { VideoCallbackState::retain(userdata.cast::<VideoCallbackState>()) };
}

unsafe extern "C" fn video_callback_release(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    // SAFETY: `userdata` was created by `Box::into_raw(Box::new(VideoCallbackState { .. }))`
    // in `set_sample_buffer_handler`. `release` frees the box once the last
    // reference (Rust creation ref + Swift callback box) is dropped.
    unsafe { VideoCallbackState::release(userdata.cast::<VideoCallbackState>()) };
}
