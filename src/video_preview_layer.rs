#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::{Deserialize, Serialize};

#[path = "video_preview_layer_display.rs"]
mod display_support;

pub use self::display_support::{
    DeskViewApplication, DeskViewApplicationInfo, DeskViewApplicationLaunchConfiguration,
    DeskViewApplicationLaunchConfigurationInfo, ExternalDisplayConfiguration,
    ExternalDisplayConfigurationInfo, ExternalDisplayConfigurator, ExternalDisplayConfiguratorInfo,
    ExternalDisplaySupportInfo,
};

use crate::connection::CaptureConnection;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, json_cstring, parse_json_and_free, CaptureRect};
use crate::session::CaptureSession;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPreviewLayerInfo {
    pub session_attached: bool,
    pub connection_present: bool,
    pub video_gravity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapturePointPayload {
    x: f64,
    y: f64,
}

impl CapturePointPayload {
    const fn from_tuple((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }

    const fn into_tuple(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

/// Safe wrapper around `AVCaptureVideoPreviewLayer`.
#[derive(Debug)]
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

    pub fn set_session(&self, session: &CaptureSession) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_session(
                self.ptr,
                session.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn clear_session(&self) {
        unsafe { ffi::video_preview_layer::av_capture_video_preview_layer_clear_session(self.ptr) };
    }

    pub fn set_session_with_no_connection(
        &self,
        session: &CaptureSession,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_session_with_no_connection(
                self.ptr,
                session.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn capture_device_point_of_interest_for_point(
        &self,
        point: (f64, f64),
    ) -> Result<(f64, f64), AVCaptureError> {
        let point = json_cstring(
            &CapturePointPayload::from_tuple(point),
            "preview layer point",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json(
                self.ptr,
                point.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<CapturePointPayload>(json_ptr)?.into_tuple())
    }

    pub fn point_for_capture_device_point_of_interest(
        &self,
        point: (f64, f64),
    ) -> Result<(f64, f64), AVCaptureError> {
        let point = json_cstring(
            &CapturePointPayload::from_tuple(point),
            "preview layer capture device point",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_point_for_capture_device_point_of_interest_json(
                self.ptr,
                point.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<CapturePointPayload>(json_ptr)?.into_tuple())
    }

    pub fn metadata_output_rect_of_interest_for_rect(
        &self,
        rect: &CaptureRect,
    ) -> Result<CaptureRect, AVCaptureError> {
        let rect = json_cstring(rect, "preview layer metadata output rect")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_metadata_output_rect_of_interest_for_rect_json(
                self.ptr,
                rect.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn rect_for_metadata_output_rect_of_interest(
        &self,
        rect: &CaptureRect,
    ) -> Result<CaptureRect, AVCaptureError> {
        let rect = json_cstring(rect, "preview layer metadata rect of interest")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_rect_for_metadata_output_rect_of_interest_json(
                self.ptr,
                rect.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }
}
