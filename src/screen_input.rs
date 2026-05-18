#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::Deserialize;

use apple_cf::cm::CMTime;

use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, parse_json_and_free, CaptureRect};
use crate::input::CaptureInputRef;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureScreenInput` state.
pub struct ScreenInputInfo {
    /// The display id reported by `AVCaptureScreenInput`.
    pub display_id: u32,
    #[serde(with = "cm_time_serde")]
    /// The min frame duration reported by `AVCaptureScreenInput`.
    pub min_frame_duration: CMTime,
    /// The crop rect reported by `AVCaptureScreenInput`.
    pub crop_rect: CaptureRect,
    /// The scale factor reported by `AVCaptureScreenInput`.
    pub scale_factor: f64,
    /// The captures mouse clicks reported by `AVCaptureScreenInput`.
    pub captures_mouse_clicks: bool,
    /// The captures cursor reported by `AVCaptureScreenInput`.
    pub captures_cursor: bool,
    /// The removes duplicate frames reported by `AVCaptureScreenInput`.
    pub removes_duplicate_frames: bool,
}

/// Safe wrapper around `AVCaptureScreenInput`.
#[derive(Debug)]
/// Wraps `AVCaptureScreenInput`.
pub struct ScreenInput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for ScreenInput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::screen_input::av_capture_screen_input_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl ScreenInput {
    /// Creates a new `AVCaptureScreenInput` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::screen_input::av_capture_screen_input_create_main_display(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Corresponds to `AVCaptureScreenInput.with_display_id`.
    pub fn with_display_id(display_id: u32) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::screen_input::av_capture_screen_input_create_with_display_id(display_id, &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureScreenInput` state.
    pub fn info(&self) -> Result<ScreenInputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::screen_input::av_capture_screen_input_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureScreenInput.display_id`.
    pub fn display_id(&self) -> Result<u32, AVCaptureError> {
        Ok(self.info()?.display_id)
    }

    /// Corresponds to `AVCaptureScreenInput.min_frame_duration`.
    pub fn min_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.min_frame_duration)
    }

    /// Corresponds to `AVCaptureScreenInput.crop_rect`.
    pub fn crop_rect(&self) -> Result<CaptureRect, AVCaptureError> {
        Ok(self.info()?.crop_rect)
    }

    /// Corresponds to `AVCaptureScreenInput.scale_factor`.
    pub fn scale_factor(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.scale_factor)
    }

    /// Corresponds to `AVCaptureScreenInput.captures_mouse_clicks`.
    pub fn captures_mouse_clicks(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.captures_mouse_clicks)
    }

    /// Corresponds to `AVCaptureScreenInput.captures_cursor`.
    pub fn captures_cursor(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.captures_cursor)
    }

    /// Corresponds to `AVCaptureScreenInput.removes_duplicate_frames`.
    pub fn removes_duplicate_frames(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.removes_duplicate_frames)
    }

    /// Sets the min frame duration on `AVCaptureScreenInput`.
    pub fn set_min_frame_duration(&self, duration: CMTime) {
        unsafe {
            ffi::screen_input::av_capture_screen_input_set_min_frame_duration(self.ptr, duration);
        }
    }

    /// Sets the crop rect on `AVCaptureScreenInput`.
    pub fn set_crop_rect(&self, rect: CaptureRect) {
        unsafe {
            ffi::screen_input::av_capture_screen_input_set_crop_rect(
                self.ptr,
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            );
        }
    }

    /// Sets the scale factor on `AVCaptureScreenInput`.
    pub fn set_scale_factor(&self, scale_factor: f64) {
        unsafe {
            ffi::screen_input::av_capture_screen_input_set_scale_factor(self.ptr, scale_factor);
        }
    }

    /// Sets the captures mouse clicks on `AVCaptureScreenInput`.
    pub fn set_captures_mouse_clicks(&self, enabled: bool) {
        unsafe {
            ffi::screen_input::av_capture_screen_input_set_captures_mouse_clicks(self.ptr, enabled);
        }
    }

    /// Sets the captures cursor on `AVCaptureScreenInput`.
    pub fn set_captures_cursor(&self, enabled: bool) {
        unsafe {
            ffi::screen_input::av_capture_screen_input_set_captures_cursor(self.ptr, enabled);
        }
    }

    /// Sets the removes duplicate frames on `AVCaptureScreenInput`.
    pub fn set_removes_duplicate_frames(&self, enabled: bool) {
        unsafe {
            ffi::screen_input::av_capture_screen_input_set_removes_duplicate_frames(
                self.ptr, enabled,
            );
        }
    }
}

impl CaptureInputRef for ScreenInput {
    fn input_ptr(&self) -> *mut c_void {
        self.ptr
    }
}
