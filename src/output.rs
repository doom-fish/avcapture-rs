#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::connection::CaptureConnection;
use crate::device::MediaType;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureOutput` state.
pub struct CaptureOutputInfo {
    /// The connection count reported by `AVCaptureOutput`.
    pub connection_count: usize,
    /// The deferred start supported reported by `AVCaptureOutput`.
    pub deferred_start_supported: Option<bool>,
    /// The deferred start enabled reported by `AVCaptureOutput`.
    pub deferred_start_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// `AVCaptureOutputDataDroppedReason` values.
pub enum AVCaptureOutputDataDroppedReason {
    /// Corresponds to the `None` case.
    None,
    /// Corresponds to the `LateData` case.
    LateData,
    /// Corresponds to the `OutOfBuffers` case.
    OutOfBuffers,
    /// Corresponds to the `Discontinuity` case.
    Discontinuity,
    /// A value not recognized by this crate.
    Unknown(String),
}

impl AVCaptureOutputDataDroppedReason {
    #[must_use]
    /// Wraps an existing `AVCaptureOutputDataDroppedReason` pointer.
    pub fn from_raw(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        match raw.as_str() {
            "none" => Self::None,
            "lateData" | "frameWasLate" => Self::LateData,
            "outOfBuffers" => Self::OutOfBuffers,
            "discontinuity" => Self::Discontinuity,
            _ => Self::Unknown(raw),
        }
    }

    #[must_use]
    /// Returns the raw SDK value for `AVCaptureOutputDataDroppedReason`.
    pub fn as_raw(&self) -> &str {
        match self {
            Self::None => "none",
            Self::LateData => "lateData",
            Self::OutOfBuffers => "outOfBuffers",
            Self::Discontinuity => "discontinuity",
            Self::Unknown(raw) => raw,
        }
    }
}

impl Serialize for AVCaptureOutputDataDroppedReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_raw())
    }
}

impl<'de> Deserialize<'de> for AVCaptureOutputDataDroppedReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from_raw(raw))
    }
}

impl fmt::Display for AVCaptureOutputDataDroppedReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_raw())
    }
}

/// Alias for `AVCaptureOutputDataDroppedReason`.
pub type CaptureOutputDataDroppedReason = AVCaptureOutputDataDroppedReason;

/// Shared helper methods for wrappers backed by `AVCaptureOutput`.
pub trait CaptureOutputRef {
    /// Returns the raw `AVCaptureOutput` pointer.
    fn output_ptr(&self) -> *mut c_void;

    /// Returns a snapshot of `AVCaptureOutput` state.
    fn output_info(&self) -> Result<CaptureOutputInfo, AVCaptureError> {
        output_info_from_ptr(self.output_ptr())
    }

    /// Returns the connections reported by the underlying API.
    fn connections(&self) -> Result<Vec<CaptureConnection>, AVCaptureError> {
        connections_from_output_ptr(self.output_ptr())
    }

    /// Returns the connection matching the requested media type, if available.
    fn connection(
        &self,
        media_type: &MediaType,
    ) -> Result<Option<CaptureConnection>, AVCaptureError> {
        connection_from_output_ptr(self.output_ptr(), media_type)
    }

    /// Corresponds to `AVCaptureOutput.deferred_start_supported`.
    fn deferred_start_supported(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.output_info()?.deferred_start_supported)
    }

    /// Corresponds to `AVCaptureOutput.deferred_start_enabled`.
    fn deferred_start_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.output_info()?.deferred_start_enabled)
    }
}

/// Corresponds to `AVCapture.output_info_from_ptr`.
pub fn output_info_from_ptr(ptr_value: *mut c_void) -> Result<CaptureOutputInfo, AVCaptureError> {
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe { ffi::output::av_capture_output_info_json(ptr_value, &mut err) };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
    }
    parse_json_and_free(json_ptr)
}

/// Corresponds to `AVCapture.connections_from_output_ptr`.
pub fn connections_from_output_ptr(
    ptr_value: *mut c_void,
) -> Result<Vec<CaptureConnection>, AVCaptureError> {
    let count = unsafe { ffi::output::av_capture_output_connections_count(ptr_value) };
    let mut connections = Vec::with_capacity(count);
    for index in 0..count {
        let mut err: *mut c_char = ptr::null_mut();
        let connection_ptr = unsafe {
            ffi::output::av_capture_output_connection_at_index(ptr_value, index, &mut err)
        };
        if connection_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        connections.push(CaptureConnection::from_raw(connection_ptr));
    }
    Ok(connections)
}

/// Corresponds to `AVCapture.connection_from_output_ptr`.
pub fn connection_from_output_ptr(
    ptr_value: *mut c_void,
    media_type: &MediaType,
) -> Result<Option<CaptureConnection>, AVCaptureError> {
    let media_type = cstring(media_type.as_raw(), "media type")?;
    let mut err: *mut c_char = ptr::null_mut();
    let connection_ptr = unsafe {
        ffi::output::av_capture_output_connection_for_media_type(
            ptr_value,
            media_type.as_ptr(),
            &mut err,
        )
    };
    if connection_ptr.is_null() {
        if err.is_null() {
            return Ok(None);
        }
        return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
    }
    Ok(Some(CaptureConnection::from_raw(connection_ptr)))
}
