#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::{Deserialize, Serialize};

use apple_cf::cm::CMTime;

use crate::device::MediaType;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, parse_json_and_free};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureConnectionInfo {
    pub input_port_count: usize,
    pub media_types: Vec<MediaType>,
    pub enabled: bool,
    pub active: bool,
    pub supports_video_mirroring: bool,
    pub video_mirrored: bool,
    pub automatically_adjusts_video_mirroring: bool,
    pub video_rotation_angle: Option<f64>,
    pub supports_video_min_frame_duration: bool,
    #[serde(with = "cm_time_serde")]
    pub video_min_frame_duration: CMTime,
    pub supports_video_max_frame_duration: bool,
    #[serde(with = "cm_time_serde")]
    pub video_max_frame_duration: CMTime,
}

/// Safe wrapper around `AVCaptureConnection`.
pub struct CaptureConnection {
    ptr: *mut c_void,
}

impl CaptureConnection {
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn info(&self) -> Result<CaptureConnectionInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::connection::av_capture_connection_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn media_types(&self) -> Result<Vec<MediaType>, AVCaptureError> {
        Ok(self.info()?.media_types)
    }

    pub fn input_port_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.input_port_count)
    }

    pub fn is_enabled(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.enabled)
    }

    pub fn is_active(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.active)
    }

    pub fn supports_video_mirroring(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_video_mirroring)
    }

    pub fn is_video_mirrored(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.video_mirrored)
    }

    pub fn automatically_adjusts_video_mirroring(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.automatically_adjusts_video_mirroring)
    }

    pub fn video_rotation_angle(&self) -> Result<Option<f64>, AVCaptureError> {
        Ok(self.info()?.video_rotation_angle)
    }

    pub fn supports_video_min_frame_duration(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_video_min_frame_duration)
    }

    pub fn video_min_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.video_min_frame_duration)
    }

    pub fn supports_video_max_frame_duration(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_video_max_frame_duration)
    }

    pub fn video_max_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.video_max_frame_duration)
    }

    pub fn set_enabled(&self, enabled: bool) {
        unsafe { ffi::connection::av_capture_connection_set_enabled(self.ptr, enabled) };
    }

    pub fn set_automatically_adjusts_video_mirroring(&self, enabled: bool) {
        unsafe {
            ffi::connection::av_capture_connection_set_automatically_adjusts_video_mirroring(
                self.ptr, enabled,
            );
        }
    }

    pub fn set_video_mirrored(&self, mirrored: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::connection::av_capture_connection_set_video_mirrored(self.ptr, mirrored, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_video_rotation_angle(&self, angle: f64) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::connection::av_capture_connection_set_video_rotation_angle(
                self.ptr, angle, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl Drop for CaptureConnection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::connection::av_capture_connection_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}
