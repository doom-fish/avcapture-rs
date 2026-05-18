#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use serde::Deserialize;

use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{json_cstring, parse_json_and_free, CaptureRect};
use crate::output::CaptureOutputRef;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureMetadataOutput` state.
pub struct MetadataOutputInfo {
    /// The connection count reported by `AVCaptureMetadataOutput`.
    pub connection_count: usize,
    /// The metadata object types reported by `AVCaptureMetadataOutput`.
    pub metadata_object_types: Vec<String>,
    /// The available metadata object types reported by `AVCaptureMetadataOutput`.
    pub available_metadata_object_types: Vec<String>,
    /// The rect of interest reported by `AVCaptureMetadataOutput`.
    pub rect_of_interest: CaptureRect,
    /// The callback installed reported by `AVCaptureMetadataOutput`.
    pub callback_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `AVMetadataObject`.
pub struct MetadataObject {
    /// The object type reported by `AVMetadataObject`.
    pub object_type: String,
    /// The string value reported by `AVMetadataObject`.
    pub string_value: Option<String>,
    /// The bounds reported by `AVMetadataObject`.
    pub bounds: CaptureRect,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event payload produced by `AVCaptureMetadataOutputObjectsDelegate` callbacks.
pub struct MetadataObjectsEvent {
    /// The metadata objects delivered by the callback.
    pub objects: Vec<MetadataObject>,
}

struct MetadataCallbackState {
    callback: Box<dyn FnMut(MetadataObjectsEvent) + Send + 'static>,
}

/// Safe wrapper around `AVCaptureMetadataOutput`.
#[derive(Debug)]
/// Wraps `AVCaptureMetadataOutput`.
pub struct MetadataOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for MetadataOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::metadata_output::av_capture_metadata_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureOutputRef for MetadataOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl MetadataOutput {
    /// Creates a new `AVCaptureMetadataOutput` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::metadata_output::av_capture_metadata_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureMetadataOutput` state.
    pub fn info(&self) -> Result<MetadataOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::metadata_output::av_capture_metadata_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the connection count reported by `AVCaptureMetadataOutput`.
    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    /// Corresponds to `AVCaptureMetadataOutput.metadata_object_types`.
    pub fn metadata_object_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.metadata_object_types)
    }

    /// Returns the available metadata object types reported by `AVCaptureMetadataOutput`.
    pub fn available_metadata_object_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_metadata_object_types)
    }

    /// Corresponds to `AVCaptureMetadataOutput.rect_of_interest`.
    pub fn rect_of_interest(&self) -> Result<CaptureRect, AVCaptureError> {
        Ok(self.info()?.rect_of_interest)
    }

    /// Corresponds to `AVCaptureMetadataOutput.callback_installed`.
    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    /// Sets the metadata object types on `AVCaptureMetadataOutput`.
    pub fn set_metadata_object_types<I, S>(&self, types: I) -> Result<(), AVCaptureError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let values = types
            .into_iter()
            .map(|value| value.as_ref().to_owned())
            .collect::<Vec<_>>();
        let json = json_cstring(&values, "metadata object types")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::metadata_output::av_capture_metadata_output_set_metadata_object_types_json(
                self.ptr,
                json.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the rect of interest on `AVCaptureMetadataOutput`.
    pub fn set_rect_of_interest(&self, rect: &CaptureRect) -> Result<(), AVCaptureError> {
        let json = json_cstring(rect, "metadata output rect of interest")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::metadata_output::av_capture_metadata_output_set_rect_of_interest_json(
                self.ptr,
                json.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the metadata objects handler on `AVCaptureMetadataOutput`.
    pub fn set_metadata_objects_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(MetadataObjectsEvent) + Send + 'static,
    {
        let queue_label = queue_label.unwrap_or("avcapture-metadata-output");
        let queue_label = CString::new(queue_label).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
        })?;
        let state = Box::new(MetadataCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::metadata_output::av_capture_metadata_output_set_metadata_objects_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(metadata_callback_trampoline),
                userdata,
                Some(metadata_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { metadata_callback_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Clears the metadata objects handler on `AVCaptureMetadataOutput`.
    pub fn clear_metadata_objects_handler(&self) {
        unsafe {
            ffi::metadata_output::av_capture_metadata_output_clear_metadata_objects_callback(
                self.ptr,
            );
        }
    }
}

unsafe extern "C" fn metadata_callback_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<MetadataCallbackState>().as_mut() else {
        return;
    };
    let Ok(event) = parse_json_and_free::<MetadataObjectsEvent>(payload) else {
        return;
    };
    (state.callback)(event);
}

unsafe extern "C" fn metadata_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(userdata.cast::<MetadataCallbackState>()));
}
