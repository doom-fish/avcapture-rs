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
pub struct CaptureRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl CaptureRect {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDimensions {
    pub width: i32,
    pub height: i32,
}

impl VideoDimensions {
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

pub fn cstring(value: &str, what: &str) -> Result<CString, AVCaptureError> {
    CString::new(value).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("{what} contains NUL byte: {error}"))
    })
}

pub fn json_cstring<T: Serialize>(value: &T, what: &str) -> Result<CString, AVCaptureError> {
    let json = serde_json::to_string(value).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("failed to encode {what}: {error}"))
    })?;
    cstring(&json, what)
}

pub fn optional_json_cstring<T: Serialize>(
    value: Option<&T>,
    what: &str,
) -> Result<Option<CString>, AVCaptureError> {
    value.map(|value| json_cstring(value, what)).transpose()
}

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

pub mod cm_time_serde {
    use super::{CMTime, CMTimePayload, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &CMTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        CMTimePayload::from(*value).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CMTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let payload = CMTimePayload::deserialize(deserializer)?;
        Ok(payload.into())
    }
}
