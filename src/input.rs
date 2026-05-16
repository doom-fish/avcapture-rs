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
pub struct CaptureInputPortInfo {
    pub media_type: MediaType,
    pub enabled: bool,
    pub has_format_description: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInputInfo {
    pub ports: Vec<CaptureInputPortInfo>,
}

impl CaptureInputInfo {
    #[must_use]
    pub fn ports_count(&self) -> usize {
        self.ports.len()
    }
}

pub trait CaptureInputRef {
    fn input_ptr(&self) -> *mut c_void;

    fn input_info(&self) -> Result<CaptureInputInfo, AVCaptureError> {
        input_info_from_ptr(self.input_ptr())
    }

    fn ports(&self) -> Result<Vec<CaptureInputPortInfo>, AVCaptureError> {
        Ok(self.input_info()?.ports)
    }

    fn ports_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.input_info()?.ports_count())
    }
}

pub fn input_info_from_ptr(ptr_value: *mut c_void) -> Result<CaptureInputInfo, AVCaptureError> {
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe { ffi::input::av_capture_input_info_json(ptr_value, &mut err) };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
    }
    parse_json_and_free(json_ptr)
}
