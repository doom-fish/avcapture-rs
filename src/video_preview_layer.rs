#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::Deserialize;

use crate::connection::CaptureConnection;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, parse_json_and_free};
use crate::session::CaptureSession;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPreviewLayerInfo {
    pub session_attached: bool,
    pub connection_present: bool,
    pub video_gravity: String,
}

/// Safe wrapper around `AVCaptureVideoPreviewLayer`.
pub struct VideoPreviewLayer {
    ptr: *mut c_void,
}

impl Drop for VideoPreviewLayer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::video_preview_layer::av_capture_video_preview_layer_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl VideoPreviewLayer {
    pub fn new(session: &CaptureSession) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_create(session.ptr, &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<VideoPreviewLayerInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn session_attached(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.session_attached)
    }

    pub fn connection_present(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.connection_present)
    }

    pub fn video_gravity(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.video_gravity)
    }

    pub fn connection(&self) -> Result<Option<CaptureConnection>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_connection(self.ptr, &mut err)
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(CaptureConnection::from_raw(ptr)))
    }

    pub fn set_video_gravity(&self, video_gravity: impl AsRef<str>) -> Result<(), AVCaptureError> {
        let video_gravity = cstring(video_gravity.as_ref(), "video gravity")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_video_gravity(
                self.ptr,
                video_gravity.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}
