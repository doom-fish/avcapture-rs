#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::{Deserialize, Serialize};

use crate::connection::CaptureConnection;
use crate::device::MediaType;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOutputInfo {
    pub connection_count: usize,
}

pub trait CaptureOutputRef {
    fn output_ptr(&self) -> *mut c_void;

    fn output_info(&self) -> Result<CaptureOutputInfo, AVCaptureError> {
        output_info_from_ptr(self.output_ptr())
    }

    fn connections(&self) -> Result<Vec<CaptureConnection>, AVCaptureError> {
        connections_from_output_ptr(self.output_ptr())
    }

    fn connection(
        &self,
        media_type: &MediaType,
    ) -> Result<Option<CaptureConnection>, AVCaptureError> {
        connection_from_output_ptr(self.output_ptr(), media_type)
    }
}

pub fn output_info_from_ptr(ptr_value: *mut c_void) -> Result<CaptureOutputInfo, AVCaptureError> {
    let mut err: *mut c_char = ptr::null_mut();
    let json_ptr = unsafe { ffi::output::av_capture_output_info_json(ptr_value, &mut err) };
    if json_ptr.is_null() {
        return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
    }
    parse_json_and_free(json_ptr)
}

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
