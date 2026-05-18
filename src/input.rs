#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::{Deserialize, Serialize};

use crate::device::MediaType;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::parse_json_and_free;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureInputPort` state.
pub struct CaptureInputPortInfo {
    /// The media type reported by `AVCaptureInputPort`.
    pub media_type: MediaType,
    /// The enabled reported by `AVCaptureInputPort`.
    pub enabled: bool,
    /// The has format description reported by `AVCaptureInputPort`.
    pub has_format_description: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureInput` state.
pub struct CaptureInputInfo {
    /// The ports reported by `AVCaptureInput`.
    pub ports: Vec<CaptureInputPortInfo>,
}

impl CaptureInputInfo {
    #[must_use]
    /// Returns the number of input ports reported by `AVCaptureInput`.
    pub fn ports_count(&self) -> usize {
        self.ports.len()
    }
}

/// Shared helper methods for wrappers backed by `AVCaptureInput`.
pub trait CaptureInputRef {
    /// Returns the raw `AVCaptureInput` pointer.
    fn input_ptr(&self) -> *mut c_void;

    /// Returns a snapshot of `AVCaptureInput` state.
    fn input_info(&self) -> Result<CaptureInputInfo, AVCaptureError> {
        input_info_from_ptr(self.input_ptr())
    }

    /// Returns the input ports reported by `AVCaptureInput`.
    fn ports(&self) -> Result<Vec<CaptureInputPortInfo>, AVCaptureError> {
        Ok(self.input_info()?.ports)
    }

    /// Returns the number of input ports reported by `AVCaptureInput`.
    fn ports_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.input_info()?.ports_count())
    }
}

/// Corresponds to `AVCapture.input_info_from_ptr`.
pub fn input_info_from_ptr(ptr_value: *mut c_void) -> Result<CaptureInputInfo, AVCaptureError> {
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe { ffi::input::av_capture_input_info_json(ptr_value, &mut err) };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
    }
    parse_json_and_free(json_ptr)
}
