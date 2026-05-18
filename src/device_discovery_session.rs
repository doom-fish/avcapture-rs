#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::Serialize;

use crate::device::{CaptureDevice, CaptureDeviceType, MediaType};
use crate::device_position::CaptureDevicePosition;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::cstring;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiscoveryCriteria {
    device_types: Vec<CaptureDeviceType>,
    media_type: Option<MediaType>,
    position: CaptureDevicePosition,
}

/// Safe wrapper around `AVCaptureDeviceDiscoverySession`.
#[derive(Debug)]
pub struct CaptureDeviceDiscoverySession {
    ptr: *mut c_void,
}

impl Drop for CaptureDeviceDiscoverySession {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::device_discovery_session::av_capture_device_discovery_session_release(
                    self.ptr,
                );
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureDeviceDiscoverySession {
    pub fn new(
        device_types: &[CaptureDeviceType],
        media_type: Option<&MediaType>,
        position: CaptureDevicePosition,
    ) -> Result<Self, AVCaptureError> {
        let criteria = DiscoveryCriteria {
            device_types: device_types.to_vec(),
            media_type: media_type.cloned(),
            position,
        };
        let json = serde_json::to_string(&criteria).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("failed to encode discovery criteria: {error}"))
        })?;
        let json = cstring(&json, "discovery criteria")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::device_discovery_session::av_capture_device_discovery_session_create(
                json.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    pub fn devices(&self) -> Result<Vec<CaptureDevice>, AVCaptureError> {
        let count = unsafe {
            ffi::device_discovery_session::av_capture_device_discovery_session_devices_count(
                self.ptr,
            )
        };
        let mut devices = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::device_discovery_session::av_capture_device_discovery_session_device_at_index(
                    self.ptr, index, &mut err,
                )
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
            }
            devices.push(CaptureDevice { ptr });
        }
        Ok(devices)
    }
}
