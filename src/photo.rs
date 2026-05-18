#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use apple_cf::cm::CMTime;
use serde::{Deserialize, Serialize};

use crate::device::CaptureFlashMode;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, parse_json_and_free, VideoDimensions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
/// `AVCapturePhotoOutputQualityPrioritization` values.
pub enum PhotoQualityPrioritization {
    /// Corresponds to the `Speed` case.
    Speed,
    /// Corresponds to the `Balanced` case.
    Balanced,
    /// Corresponds to the `Quality` case.
    Quality,
    /// A value not recognized by this crate.
    Unknown(i32),
}

impl PhotoQualityPrioritization {
    #[must_use]
    /// Wraps an existing `AVCapturePhotoOutputQualityPrioritization` pointer.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Speed,
            2 => Self::Balanced,
            3 => Self::Quality,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    /// Returns the raw SDK value for `AVCapturePhotoOutputQualityPrioritization`.
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
/// Snapshot of `AVCapturePhotoSettings` state.
pub struct PhotoSettingsInfo {
    /// The unique id reported by `AVCapturePhotoSettings`.
    pub unique_id: i64,
    /// The processed file type reported by `AVCapturePhotoSettings`.
    pub processed_file_type: Option<String>,
    /// The flash mode reported by `AVCapturePhotoSettings`.
    pub flash_mode: Option<CaptureFlashMode>,
    /// The photo quality prioritization reported by `AVCapturePhotoSettings`.
    pub photo_quality_prioritization: Option<PhotoQualityPrioritization>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureResolvedPhotoSettings` state.
pub struct ResolvedPhotoSettingsInfo {
    /// The unique id reported by `AVCaptureResolvedPhotoSettings`.
    pub unique_id: i64,
    /// The photo dimensions reported by `AVCaptureResolvedPhotoSettings`.
    pub photo_dimensions: VideoDimensions,
    /// The expected photo count reported by `AVCaptureResolvedPhotoSettings`.
    pub expected_photo_count: usize,
    /// The fast capture prioritization enabled reported by `AVCaptureResolvedPhotoSettings`.
    pub fast_capture_prioritization_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCapturePhoto` state.
pub struct PhotoInfo {
    /// The unique id reported by `AVCapturePhoto`.
    pub unique_id: i64,
    #[serde(with = "cm_time_serde")]
    /// The timestamp reported by `AVCapturePhoto`.
    pub timestamp: CMTime,
    /// The photo count reported by `AVCapturePhoto`.
    pub photo_count: usize,
    /// The pixel buffer available reported by `AVCapturePhoto`.
    pub pixel_buffer_available: bool,
    /// The constant color confidence map available reported by `AVCapturePhoto`.
    pub constant_color_confidence_map_available: Option<bool>,
    /// The constant color center weighted mean confidence level reported by `AVCapturePhoto`.
    pub constant_color_center_weighted_mean_confidence_level: Option<f32>,
    /// The constant color fallback photo reported by `AVCapturePhoto`.
    pub constant_color_fallback_photo: Option<bool>,
    /// The resolved settings reported by `AVCapturePhoto`.
    pub resolved_settings: ResolvedPhotoSettingsInfo,
}

/// Safe wrapper around `AVCapturePhotoSettings`.
#[derive(Debug)]
/// Wraps `AVCapturePhotoSettings`.
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
    /// Creates a new `AVCapturePhotoSettings` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::photo::av_capture_photo_settings_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Corresponds to `AVCapturePhotoSettings.copy_with_unique_id`.
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

    /// Returns a snapshot of `AVCapturePhotoSettings` state.
    pub fn info(&self) -> Result<PhotoSettingsInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::photo::av_capture_photo_settings_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCapturePhotoSettings.unique_id`.
    pub fn unique_id(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    /// Corresponds to `AVCapturePhotoSettings.processed_file_type`.
    pub fn processed_file_type(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.processed_file_type)
    }

    /// Corresponds to `AVCapturePhotoSettings.flash_mode`.
    pub fn flash_mode(&self) -> Result<Option<CaptureFlashMode>, AVCaptureError> {
        Ok(self.info()?.flash_mode)
    }

    /// Corresponds to `AVCapturePhotoSettings.photo_quality_prioritization`.
    pub fn photo_quality_prioritization(
        &self,
    ) -> Result<Option<PhotoQualityPrioritization>, AVCaptureError> {
        Ok(self.info()?.photo_quality_prioritization)
    }

    /// Sets the flash mode on `AVCapturePhotoSettings`.
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

    /// Sets the photo quality prioritization on `AVCapturePhotoSettings`.
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

/// Safe wrapper around `AVCaptureResolvedPhotoSettings`.
#[derive(Debug)]
/// Wraps `AVCaptureResolvedPhotoSettings`.
pub struct ResolvedPhotoSettings {
    pub(crate) ptr: *mut c_void,
}

impl Drop for ResolvedPhotoSettings {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::photo::av_capture_resolved_photo_settings_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl ResolvedPhotoSettings {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureResolvedPhotoSettings` state.
    pub fn info(&self) -> Result<ResolvedPhotoSettingsInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::photo::av_capture_resolved_photo_settings_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureResolvedPhotoSettings.unique_id`.
    pub fn unique_id(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    /// Corresponds to `AVCaptureResolvedPhotoSettings.photo_dimensions`.
    pub fn photo_dimensions(&self) -> Result<VideoDimensions, AVCaptureError> {
        Ok(self.info()?.photo_dimensions)
    }

    /// Returns the expected photo count reported by `AVCaptureResolvedPhotoSettings`.
    pub fn expected_photo_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.expected_photo_count)
    }

    /// Corresponds to `AVCaptureResolvedPhotoSettings.fast_capture_prioritization_enabled`.
    pub fn fast_capture_prioritization_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.fast_capture_prioritization_enabled)
    }
}

/// Safe wrapper around `AVCapturePhoto`.
#[derive(Debug)]
/// Wraps `AVCapturePhoto`.
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

    /// Returns a snapshot of `AVCapturePhoto` state.
    pub fn info(&self) -> Result<PhotoInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::photo::av_capture_photo_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCapturePhoto.unique_id`.
    pub fn unique_id(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    /// Corresponds to `AVCapturePhoto.timestamp`.
    pub fn timestamp(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.timestamp)
    }

    /// Returns the photo count reported by `AVCapturePhoto`.
    pub fn photo_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.photo_count)
    }

    /// Corresponds to `AVCapturePhoto.pixel_buffer_available`.
    pub fn pixel_buffer_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.pixel_buffer_available)
    }

    /// Corresponds to `AVCapturePhoto.resolved_settings_info`.
    pub fn resolved_settings_info(&self) -> Result<ResolvedPhotoSettingsInfo, AVCaptureError> {
        Ok(self.info()?.resolved_settings)
    }

    /// Corresponds to `AVCapturePhoto.resolved_settings`.
    pub fn resolved_settings(&self) -> Result<ResolvedPhotoSettings, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::photo::av_capture_photo_resolved_settings(self.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(ResolvedPhotoSettings::from_raw(ptr))
    }
}
