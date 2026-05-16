#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use apple_cf::cm::CMTime;
use serde::{Deserialize, Serialize};

use crate::device::CaptureFlashMode;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, parse_json_and_free};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
pub enum PhotoQualityPrioritization {
    Speed,
    Balanced,
    Quality,
    Unknown(i32),
}

impl PhotoQualityPrioritization {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Speed,
            2 => Self::Balanced,
            3 => Self::Quality,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Speed => 1,
            Self::Balanced => 2,
            Self::Quality => 3,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for PhotoQualityPrioritization {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<PhotoQualityPrioritization> for i32 {
    fn from(value: PhotoQualityPrioritization) -> Self {
        value.as_raw()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoSettingsInfo {
    pub unique_id: i64,
    pub processed_file_type: Option<String>,
    pub flash_mode: Option<CaptureFlashMode>,
    pub photo_quality_prioritization: Option<PhotoQualityPrioritization>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoInfo {
    pub unique_id: i64,
    #[serde(with = "cm_time_serde")]
    pub timestamp: CMTime,
    pub photo_count: usize,
    pub pixel_buffer_available: bool,
    pub constant_color_confidence_map_available: Option<bool>,
    pub constant_color_center_weighted_mean_confidence_level: Option<f32>,
    pub constant_color_fallback_photo: Option<bool>,
}

/// Safe wrapper around `AVCapturePhotoSettings`.
pub struct PhotoSettings {
    pub(crate) ptr: *mut c_void,
}

impl Drop for PhotoSettings {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::photo::av_capture_photo_settings_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl PhotoSettings {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::photo::av_capture_photo_settings_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn copy_with_unique_id(&self) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::photo::av_capture_photo_settings_copy_with_unique_id(self.ptr, &mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<PhotoSettingsInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::photo::av_capture_photo_settings_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn unique_id(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    pub fn processed_file_type(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.processed_file_type)
    }

    pub fn flash_mode(&self) -> Result<Option<CaptureFlashMode>, AVCaptureError> {
        Ok(self.info()?.flash_mode)
    }

    pub fn photo_quality_prioritization(
        &self,
    ) -> Result<Option<PhotoQualityPrioritization>, AVCaptureError> {
        Ok(self.info()?.photo_quality_prioritization)
    }

    pub fn set_flash_mode(&self, mode: CaptureFlashMode) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo::av_capture_photo_settings_set_flash_mode(self.ptr, mode.as_raw(), &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_photo_quality_prioritization(
        &self,
        prioritization: PhotoQualityPrioritization,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo::av_capture_photo_settings_set_photo_quality_prioritization(
                self.ptr,
                prioritization.as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

/// Safe wrapper around `AVCapturePhoto`.
#[derive(Debug)]
pub struct Photo {
    pub(crate) ptr: *mut c_void,
}

impl Drop for Photo {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::photo::av_capture_photo_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl Photo {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub fn info(&self) -> Result<PhotoInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::photo::av_capture_photo_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn unique_id(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    pub fn timestamp(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.timestamp)
    }

    pub fn photo_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.photo_count)
    }

    pub fn pixel_buffer_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.pixel_buffer_available)
    }
}
