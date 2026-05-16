#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::Deserialize;

use crate::device::CaptureDevice;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::parse_json_and_free;
use crate::input::CaptureInputRef;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInputInfo {
    pub device_unique_id: String,
    pub device_localized_name: String,
    pub ports_count: usize,
}

/// Safe wrapper around `AVCaptureDeviceInput`.
pub struct DeviceInput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for DeviceInput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device_input::av_capture_device_input_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl DeviceInput {
    pub fn new(device: &CaptureDevice) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::device_input::av_capture_device_input_create(device.ptr, &mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<DeviceInputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::device_input::av_capture_device_input_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::INPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn device_unique_id(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.device_unique_id)
    }

    pub fn device_localized_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.device_localized_name)
    }

    pub fn ports_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.ports_count)
    }
}

impl CaptureInputRef for DeviceInput {
    fn input_ptr(&self) -> *mut c_void {
        self.ptr
    }
}
