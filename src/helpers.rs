#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};

use apple_cf::cm::CMTime;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::AVCaptureError;
use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
/// Wraps `CGPoint` values used by `AVCapture*` APIs.
pub struct CapturePoint {
    /// The horizontal coordinate.
    pub x: f64,
    /// The vertical coordinate.
    pub y: f64,
}

impl CapturePoint {
    #[must_use]
    /// Creates a new `CGPoint` wrapper.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
/// Wraps `CGSize` values used by `AVCapture*` APIs.
pub struct CaptureSize {
    /// The width component.
    pub width: f64,
    /// The height component.
    pub height: f64,
}

impl CaptureSize {
    #[must_use]
    /// Creates a new `CGSize` wrapper.
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
/// Wraps `CGRect` values used by `AVCapture*` APIs.
pub struct CaptureRect {
    #[serde(flatten)]
    /// The origin point.
    pub origin: CapturePoint,
    #[serde(flatten)]
    /// The size value.
    pub size: CaptureSize,
}

impl CaptureRect {
    #[must_use]
    /// Creates a new `CGRect` wrapper.
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: CapturePoint::new(x, y),
            size: CaptureSize::new(width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `CMVideoDimensions` values used by `AVCapture*` APIs.
pub struct VideoDimensions {
    /// The width component.
    pub width: i32,
    /// The height component.
    pub height: i32,
}

impl VideoDimensions {
    #[must_use]
    /// Creates a new `CMVideoDimensions` wrapper.
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

/// Corresponds to `AVCapture.cstring`.
pub fn cstring(value: &str, what: &str) -> Result<CString, AVCaptureError> {
    CString::new(value).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("{what} contains NUL byte: {error}"))
    })
}

/// Corresponds to `AVCapture.json_cstring`.
pub fn json_cstring<T: Serialize>(value: &T, what: &str) -> Result<CString, AVCaptureError> {
    let json = serde_json::to_string(value).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("failed to encode {what}: {error}"))
    })?;
    cstring(&json, what)
}

/// Corresponds to `AVCapture.optional_json_cstring`.
pub fn optional_json_cstring<T: Serialize>(
    value: Option<&T>,
    what: &str,
) -> Result<Option<CString>, AVCaptureError> {
    value.map(|value| json_cstring(value, what)).transpose()
}

/// Corresponds to `AVCapture.parse_json_and_free`.
pub fn parse_json_and_free<T: DeserializeOwned>(
    json_ptr: *mut c_char,
) -> Result<T, AVCaptureError> {
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::core::avc_string_free(json_ptr) };
    serde_json::from_str::<T>(&json).map_err(|error| {
        AVCaptureError::OperationFailed(format!("failed to decode bridge JSON: {error}"))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct CMTimePayload {
    value: i64,
    timescale: i32,
    flags: u32,
    epoch: i64,
}

impl From<CMTime> for CMTimePayload {
    fn from(value: CMTime) -> Self {
        Self {
            value: value.value,
            timescale: value.timescale,
            flags: value.flags,
            epoch: value.epoch,
        }
    }
}

impl From<CMTimePayload> for CMTime {
    fn from(value: CMTimePayload) -> Self {
        Self {
            value: value.value,
            timescale: value.timescale,
            flags: value.flags,
            epoch: value.epoch,
        }
    }
}

/// Module covering `CMTime` support.
pub mod cm_time_serde {
    use super::{CMTime, CMTimePayload, Deserialize, Deserializer, Serialize, Serializer};

    /// Corresponds to `AVCapture.serialize`.
    pub fn serialize<S>(value: &CMTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CMTimePayload::from(*value).serialize(serializer)
    }

    /// Corresponds to `AVCapture.deserialize`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<CMTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let payload = CMTimePayload::deserialize(deserializer)?;
        Ok(payload.into())
    }
}
