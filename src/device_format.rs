#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::{Deserialize, Serialize};

use apple_cf::cm::CMTime;

use crate::device::MediaType;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, parse_json_and_free, VideoDimensions};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `CMFormatDescription` state.
pub struct FormatDescriptionInfo {
    /// The media type reported by `CMFormatDescription`.
    pub media_type: String,
    /// The media type code reported by `CMFormatDescription`.
    pub media_type_code: u32,
    /// The media subtype reported by `CMFormatDescription`.
    pub media_subtype: String,
    /// The media subtype code reported by `CMFormatDescription`.
    pub media_subtype_code: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `AVFrameRateRange`.
pub struct FrameRateRange {
    /// The min frame rate reported by `AVFrameRateRange`.
    pub min_frame_rate: f64,
    /// The max frame rate reported by `AVFrameRateRange`.
    pub max_frame_rate: f64,
    #[serde(with = "cm_time_serde")]
    /// The min frame duration reported by `AVFrameRateRange`.
    pub min_frame_duration: CMTime,
    #[serde(with = "cm_time_serde")]
    /// The max frame duration reported by `AVFrameRateRange`.
    pub max_frame_duration: CMTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureDeviceFormat` state.
pub struct CaptureDeviceFormatInfo {
    /// The media type reported by `AVCaptureDeviceFormat`.
    pub media_type: MediaType,
    /// The format description reported by `AVCaptureDeviceFormat`.
    pub format_description: FormatDescriptionInfo,
    /// The video supported frame rate ranges reported by `AVCaptureDeviceFormat`.
    pub video_supported_frame_rate_ranges: Vec<FrameRateRange>,
    /// The high resolution still image dimensions reported by `AVCaptureDeviceFormat`.
    pub high_resolution_still_image_dimensions: Option<VideoDimensions>,
    /// The supported max photo dimensions reported by `AVCaptureDeviceFormat`.
    pub supported_max_photo_dimensions: Vec<VideoDimensions>,
}

/// Safe wrapper around `AVCaptureDeviceFormat`.
#[derive(Debug)]
/// Wraps `AVCaptureDeviceFormat`.
pub struct CaptureDeviceFormat {
    pub(crate) ptr: *mut c_void,
}

impl CaptureDeviceFormat {
    /// Wraps an existing `AVCaptureDeviceFormat` pointer.
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureDeviceFormat` state.
    pub fn info(&self) -> Result<CaptureDeviceFormatInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::device_format::av_capture_device_format_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureDeviceFormat.media_type`.
    pub fn media_type(&self) -> Result<MediaType, AVCaptureError> {
        Ok(self.info()?.media_type)
    }

    /// Corresponds to `AVCaptureDeviceFormat.format_description`.
    pub fn format_description(&self) -> Result<FormatDescriptionInfo, AVCaptureError> {
        Ok(self.info()?.format_description)
    }

    /// Corresponds to `AVCaptureDeviceFormat.video_supported_frame_rate_ranges`.
    pub fn video_supported_frame_rate_ranges(&self) -> Result<Vec<FrameRateRange>, AVCaptureError> {
        Ok(self.info()?.video_supported_frame_rate_ranges)
    }

    /// Corresponds to `AVCaptureDeviceFormat.high_resolution_still_image_dimensions`.
    pub fn high_resolution_still_image_dimensions(
        &self,
    ) -> Result<Option<VideoDimensions>, AVCaptureError> {
        Ok(self.info()?.high_resolution_still_image_dimensions)
    }

    /// Corresponds to `AVCaptureDeviceFormat.supported_max_photo_dimensions`.
    pub fn supported_max_photo_dimensions(&self) -> Result<Vec<VideoDimensions>, AVCaptureError> {
        Ok(self.info()?.supported_max_photo_dimensions)
    }
}

impl Drop for CaptureDeviceFormat {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device_format::av_capture_device_format_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}
