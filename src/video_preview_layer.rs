#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::marker::PhantomData;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[path = "video_preview_layer_display.rs"]
mod display_support;

pub use self::display_support::{
    DeskViewApplication, DeskViewApplicationInfo, DeskViewApplicationLaunchConfiguration,
    DeskViewApplicationLaunchConfigurationInfo, ExternalDisplayConfiguration,
    ExternalDisplayConfigurationInfo, ExternalDisplayConfigurator, ExternalDisplayConfiguratorInfo,
    ExternalDisplaySupportInfo,
};

use crate::connection::CaptureConnection;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, json_cstring, parse_json_and_free, CaptureRect};
use crate::session::CaptureSession;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureVideoPreviewLayer` state.
pub struct VideoPreviewLayerInfo {
    /// The session attached reported by `AVCaptureVideoPreviewLayer`.
    pub session_attached: bool,
    /// The connection present reported by `AVCaptureVideoPreviewLayer`.
    pub connection_present: bool,
    /// The video gravity reported by `AVCaptureVideoPreviewLayer`.
    pub video_gravity: String,
    /// The frame in the caller-owned host layer's coordinate space.
    pub frame: CaptureRect,
    /// The layer bounds used by capture geometry conversions.
    pub bounds: CaptureRect,
    /// The layer contents scale.
    pub contents_scale: f64,
    /// Whether the layer currently has a caller-owned superlayer.
    pub has_superlayer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapturePointPayload {
    x: f64,
    y: f64,
}

impl CapturePointPayload {
    const fn from_tuple((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }

    const fn into_tuple(self) -> (f64, f64) {
        (self.x, self.y)
    }
}

/// Safe wrapper around `AVCaptureVideoPreviewLayer`.
#[derive(Debug)]
/// Wraps `AVCaptureVideoPreviewLayer`.
pub struct VideoPreviewLayer {
    ptr: *mut c_void,
    _main_thread_only: PhantomData<Rc<()>>,
}

/// A +1 retained opaque `CALayer` handle.
#[derive(Debug)]
pub struct RetainedNativeLayer {
    ptr: *mut c_void,
    _main_thread_only: PhantomData<Rc<()>>,
}

impl RetainedNativeLayer {
    /// Returns the retained native `CALayer` pointer without transferring ownership.
    #[must_use]
    pub const fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl Drop for RetainedNativeLayer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::video_preview_layer::av_capture_video_preview_layer_native_layer_release(
                    self.ptr,
                );
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl Drop for VideoPreviewLayer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::video_preview_layer::av_capture_video_preview_layer_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl VideoPreviewLayer {
    /// Creates a new `AVCaptureVideoPreviewLayer` wrapper.
    pub fn new(session: &CaptureSession) -> Result<Self, AVCaptureError> {
        let mut status = ffi::status::OK;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_create(
                session.ptr,
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            ptr,
            _main_thread_only: PhantomData,
        })
    }

    /// Returns a snapshot of `AVCaptureVideoPreviewLayer` state.
    pub fn info(&self) -> Result<VideoPreviewLayerInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.session_attached`.
    pub fn session_attached(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.session_attached)
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.connection_present`.
    pub fn connection_present(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.connection_present)
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.video_gravity`.
    pub fn video_gravity(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.video_gravity)
    }

    /// Returns the preview layer frame.
    pub fn frame(&self) -> Result<CaptureRect, AVCaptureError> {
        Ok(self.info()?.frame)
    }

    /// Returns the preview layer bounds.
    pub fn bounds(&self) -> Result<CaptureRect, AVCaptureError> {
        Ok(self.info()?.bounds)
    }

    /// Returns the preview layer contents scale.
    pub fn contents_scale(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.contents_scale)
    }

    /// Returns whether the preview layer is attached to a caller-owned host layer.
    pub fn has_superlayer(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.has_superlayer)
    }

    /// Returns a borrowed +0 `CALayer` pointer valid while `self` remains alive.
    #[must_use]
    pub fn as_native_layer_ptr(&self) -> *mut c_void {
        unsafe { ffi::video_preview_layer::av_capture_video_preview_layer_native_layer(self.ptr) }
    }

    /// Returns a +1 retained opaque `CALayer` handle.
    pub fn retained_native_layer(&self) -> Result<RetainedNativeLayer, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_native_layer_retained(
                self.ptr, &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(RetainedNativeLayer {
            ptr,
            _main_thread_only: PhantomData,
        })
    }

    /// Returns the connection matching the requested media type, if available.
    pub fn connection(&self) -> Result<Option<CaptureConnection>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_connection(self.ptr, &mut err)
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(unsafe {
            CaptureConnection::from_retained_bridge_box(ptr)
        }))
    }

    /// Sets the video gravity on `AVCaptureVideoPreviewLayer`.
    pub fn set_video_gravity(&self, video_gravity: impl AsRef<str>) -> Result<(), AVCaptureError> {
        let video_gravity = cstring(video_gravity.as_ref(), "video gravity")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_video_gravity(
                self.ptr,
                video_gravity.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the session on `AVCaptureVideoPreviewLayer`.
    pub fn set_session(&self, session: &CaptureSession) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_session(
                self.ptr,
                session.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Clears the session on `AVCaptureVideoPreviewLayer`.
    pub fn clear_session(&self) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_clear_session(
                self.ptr, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the session with no connection on `AVCaptureVideoPreviewLayer`.
    pub fn set_session_with_no_connection(
        &self,
        session: &CaptureSession,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_session_with_no_connection(
                self.ptr,
                session.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.capture_device_point_of_interest_for_point`.
    pub fn capture_device_point_of_interest_for_point(
        &self,
        point: (f64, f64),
    ) -> Result<(f64, f64), AVCaptureError> {
        let point = json_cstring(
            &CapturePointPayload::from_tuple(point),
            "preview layer point",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_capture_device_point_of_interest_for_point_json(
                self.ptr,
                point.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<CapturePointPayload>(json_ptr)?.into_tuple())
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.point_for_capture_device_point_of_interest`.
    pub fn point_for_capture_device_point_of_interest(
        &self,
        point: (f64, f64),
    ) -> Result<(f64, f64), AVCaptureError> {
        let point = json_cstring(
            &CapturePointPayload::from_tuple(point),
            "preview layer capture device point",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_point_for_capture_device_point_of_interest_json(
                self.ptr,
                point.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(parse_json_and_free::<CapturePointPayload>(json_ptr)?.into_tuple())
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.metadata_output_rect_of_interest_for_rect`.
    pub fn metadata_output_rect_of_interest_for_rect(
        &self,
        rect: &CaptureRect,
    ) -> Result<CaptureRect, AVCaptureError> {
        let rect = json_cstring(rect, "preview layer metadata output rect")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_metadata_output_rect_of_interest_for_rect_json(
                self.ptr,
                rect.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureVideoPreviewLayer.rect_for_metadata_output_rect_of_interest`.
    pub fn rect_for_metadata_output_rect_of_interest(
        &self,
        rect: &CaptureRect,
    ) -> Result<CaptureRect, AVCaptureError> {
        let rect = json_cstring(rect, "preview layer metadata rect of interest")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_rect_for_metadata_output_rect_of_interest_json(
                self.ptr,
                rect.as_ptr(),
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Sets the preview layer frame without implicit Core Animation actions.
    pub fn set_frame(&self, frame: &CaptureRect) -> Result<(), AVCaptureError> {
        self.set_rect_property(
            frame,
            "preview layer frame",
            ffi::video_preview_layer::av_capture_video_preview_layer_set_frame_json,
        )
    }

    /// Sets the preview layer bounds without implicit Core Animation actions.
    pub fn set_bounds(&self, bounds: &CaptureRect) -> Result<(), AVCaptureError> {
        self.set_rect_property(
            bounds,
            "preview layer bounds",
            ffi::video_preview_layer::av_capture_video_preview_layer_set_bounds_json,
        )
    }

    /// Sets the preview layer contents scale.
    pub fn set_contents_scale(&self, contents_scale: f64) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_set_contents_scale(
                self.ptr,
                contents_scale,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Marks the preview layer as needing layout.
    pub fn set_needs_layout(&self) -> Result<(), AVCaptureError> {
        self.call_layout(ffi::video_preview_layer::av_capture_video_preview_layer_set_needs_layout)
    }

    /// Performs pending preview layer layout immediately.
    pub fn layout_if_needed(&self) -> Result<(), AVCaptureError> {
        self.call_layout(ffi::video_preview_layer::av_capture_video_preview_layer_layout_if_needed)
    }

    /// Adds this preview layer to a caller-owned `CALayer`.
    ///
    /// # Safety
    ///
    /// `host_layer` must be a live borrowed `CALayer` pointer for the duration of this call.
    pub unsafe fn attach_to_host_layer(
        &self,
        host_layer: *mut c_void,
    ) -> Result<(), AVCaptureError> {
        if host_layer.is_null() {
            return Err(AVCaptureError::InvalidArgument(
                "host layer pointer must not be null".to_owned(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_attach_to_host_layer(
                self.ptr, host_layer, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Removes this preview layer from its caller-owned superlayer.
    pub fn detach_from_host_layer(&self) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::video_preview_layer::av_capture_video_preview_layer_detach_from_host_layer(
                self.ptr, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    fn set_rect_property(
        &self,
        rect: &CaptureRect,
        what: &str,
        setter: unsafe extern "C" fn(*mut c_void, *const c_char, *mut *mut c_char) -> i32,
    ) -> Result<(), AVCaptureError> {
        let rect = json_cstring(rect, what)?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { setter(self.ptr, rect.as_ptr(), &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    fn call_layout(
        &self,
        operation: unsafe extern "C" fn(*mut c_void, *mut *mut c_char) -> i32,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe { operation(self.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::VideoPreviewLayerInfo;
    use crate::CaptureRect;

    #[test]
    fn preview_layer_payload_includes_hosting_geometry() {
        let info: VideoPreviewLayerInfo = serde_json::from_str(
            r#"{
                "sessionAttached": true,
                "connectionPresent": false,
                "videoGravity": "resizeAspect",
                "frame": {"x": 10.0, "y": 20.0, "width": 640.0, "height": 360.0},
                "bounds": {"x": 0.0, "y": 0.0, "width": 640.0, "height": 360.0},
                "contentsScale": 2.0,
                "hasSuperlayer": true
            }"#,
        )
        .expect("preview layer fixture should decode");

        assert_eq!(info.frame, CaptureRect::new(10.0, 20.0, 640.0, 360.0));
        assert_eq!(info.bounds, CaptureRect::new(0.0, 0.0, 640.0, 360.0));
        assert_eq!(info.contents_scale, 2.0);
        assert!(info.has_superlayer);
    }
}
