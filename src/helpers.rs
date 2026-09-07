#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::c_char;
use std::ffi::{CStr, CString};

use apple_cf::cm::CMTime;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::AVCaptureError;
use crate::ffi;

pub(crate) const BRIDGE_SCHEMA_VERSION: u64 = 1;

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
    let payload = serde_json::to_value(value).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("failed to encode {what}: {error}"))
    })?;
    let value = serde_json::json!({
        "schemaVersion": BRIDGE_SCHEMA_VERSION,
        "payload": payload,
    });
    let json = serde_json::to_string(&value).map_err(|error| {
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
    if json_ptr.is_null() {
        return Err(AVCaptureError::BridgeProtocol(
            "bridge returned a null JSON payload".to_owned(),
        ));
    }
    let json = unsafe { CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::core::avc_string_free(json_ptr) };
    parse_bridge_json(&json)
}

pub(crate) fn parse_bridge_json<T: DeserializeOwned>(json: &str) -> Result<T, AVCaptureError> {
    let value = serde_json::from_str::<serde_json::Value>(json).map_err(|error| {
        AVCaptureError::BridgeProtocol(format!("failed to decode bridge JSON: {error}"))
    })?;
    let object = value.as_object().ok_or_else(|| {
        AVCaptureError::BridgeProtocol("bridge JSON payload is not an envelope object".to_owned())
    })?;
    let version = object
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            AVCaptureError::BridgeProtocol(
                "bridge JSON payload is missing schemaVersion".to_owned(),
            )
        })?;
    if version != BRIDGE_SCHEMA_VERSION {
        return Err(AVCaptureError::BridgeProtocol(format!(
            "bridge JSON schema version {version} is unsupported; expected {BRIDGE_SCHEMA_VERSION}"
        )));
    }
    let payload = object.get("payload").cloned().ok_or_else(|| {
        AVCaptureError::BridgeProtocol("bridge JSON envelope is missing payload".to_owned())
    })?;
    serde_json::from_value(payload).map_err(|error| {
        AVCaptureError::BridgeProtocol(format!("failed to decode bridge JSON: {error}"))
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

#[cfg(test)]
mod tests {
    use super::{json_cstring, parse_bridge_json};
    use crate::AVCaptureError;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Payload {
        value: u32,
    }

    #[test]
    fn bridge_json_uses_a_versioned_envelope_for_objects_and_arrays() {
        let object =
            json_cstring(&Payload { value: 7 }, "payload").expect("object payload should encode");
        let object = object.to_str().expect("JSON should be UTF-8");
        assert_eq!(
            parse_bridge_json::<Payload>(object).expect("object payload should decode"),
            Payload { value: 7 }
        );

        let array = json_cstring(&vec![1_u32, 2, 3], "array").expect("array payload should encode");
        let array = array.to_str().expect("JSON should be UTF-8");
        assert_eq!(
            parse_bridge_json::<Vec<u32>>(array).expect("array payload should decode"),
            vec![1, 2, 3]
        );

        let scalar = json_cstring(&"figure.wave", "scalar").expect("scalar should encode");
        assert_eq!(
            parse_bridge_json::<String>(scalar.to_str().expect("JSON should be UTF-8"))
                .expect("scalar payload should decode"),
            "figure.wave"
        );
    }

    #[test]
    fn bridge_json_rejects_missing_or_unknown_schema_versions() {
        assert!(matches!(
            parse_bridge_json::<Payload>(r#"{"payload":{"value":7}}"#),
            Err(AVCaptureError::BridgeProtocol(_))
        ));
        assert!(matches!(
            parse_bridge_json::<Payload>(r#"{"schemaVersion":2,"payload":{"value":7}}"#),
            Err(AVCaptureError::BridgeProtocol(_))
        ));
    }
}
