#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::device::CaptureFlashMode;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{parse_json_and_free, VideoDimensions};
use crate::output::CaptureOutputRef;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoOutputInfo {
    pub connection_count: usize,
    pub available_photo_codec_types: Vec<String>,
    pub available_photo_file_types: Vec<String>,
    pub available_photo_pixel_format_types: Vec<u32>,
    pub available_raw_photo_pixel_format_types: Vec<u32>,
    pub supported_flash_modes: Vec<CaptureFlashMode>,
    pub max_photo_dimensions: Option<VideoDimensions>,
    pub capture_readiness: Option<i32>,
    pub high_resolution_capture_enabled: bool,
    pub responsive_capture_enabled: Option<bool>,
    pub callback_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoCaptureResult {
    pub unique_id: i64,
    pub error: Option<String>,
}

struct PhotoCaptureCallbackState {
    callback: Box<dyn FnMut(PhotoCaptureResult) + Send + 'static>,
}

/// Safe wrapper around `AVCapturePhotoOutput`.
pub struct PhotoOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PhotoOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::photo_output::av_capture_photo_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureOutputRef for PhotoOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl PhotoOutput {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::photo_output::av_capture_photo_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<PhotoOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::photo_output::av_capture_photo_output_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn available_photo_codec_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_photo_codec_types)
    }

    pub fn available_photo_file_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_photo_file_types)
    }

    pub fn available_photo_pixel_format_types(&self) -> Result<Vec<u32>, AVCaptureError> {
        Ok(self.info()?.available_photo_pixel_format_types)
    }

    pub fn available_raw_photo_pixel_format_types(&self) -> Result<Vec<u32>, AVCaptureError> {
        Ok(self.info()?.available_raw_photo_pixel_format_types)
    }

    pub fn supported_flash_modes(&self) -> Result<Vec<CaptureFlashMode>, AVCaptureError> {
        Ok(self.info()?.supported_flash_modes)
    }

    pub fn max_photo_dimensions(&self) -> Result<Option<VideoDimensions>, AVCaptureError> {
        Ok(self.info()?.max_photo_dimensions)
    }

    pub fn capture_readiness(&self) -> Result<Option<i32>, AVCaptureError> {
        Ok(self.info()?.capture_readiness)
    }

    pub fn high_resolution_capture_enabled(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.high_resolution_capture_enabled)
    }

    pub fn responsive_capture_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.responsive_capture_enabled)
    }

    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    pub fn set_high_resolution_capture_enabled(&self, enabled: bool) {
        unsafe {
            ffi::photo_output::av_capture_photo_output_set_high_resolution_capture_enabled(
                self.ptr, enabled,
            );
        }
    }

    pub fn set_responsive_capture_enabled(&self, enabled: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_set_responsive_capture_enabled(
                self.ptr,
                enabled,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn capture_photo<F>(&self, callback: F) -> Result<(), AVCaptureError>
    where
        F: FnMut(PhotoCaptureResult) + Send + 'static,
    {
        let state = Box::new(PhotoCaptureCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_capture_photo(
                self.ptr,
                Some(photo_capture_trampoline),
                userdata,
                Some(photo_capture_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { photo_capture_callback_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

unsafe extern "C" fn photo_capture_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<PhotoCaptureCallbackState>().as_mut() else {
        return;
    };
    let Ok(result) = parse_json_and_free::<PhotoCaptureResult>(payload) else {
        return;
    };
    (state.callback)(result);
}

unsafe extern "C" fn photo_capture_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(userdata.cast::<PhotoCaptureCallbackState>()));
}
