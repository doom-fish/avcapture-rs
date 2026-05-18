#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::{Deserialize, Serialize};

use super::VideoPreviewLayer;
use crate::device::CaptureDevice;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{json_cstring, parse_json_and_free, CaptureRect, VideoDimensions};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeskViewApplicationInfo {
    pub runtime_supported: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeskViewApplicationLaunchConfigurationInfo {
    pub main_window_frame: CaptureRect,
    #[serde(alias = "requiresSetUpModeCompletion")]
    pub requires_setup_mode_completion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDisplaySupportInfo {
    pub should_match_frame_rate_supported: bool,
    pub bypass_color_space_conversion_supported: bool,
    pub preferred_resolution_supported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDisplayConfigurationInfo {
    pub should_match_frame_rate: bool,
    pub bypass_color_space_conversion: bool,
    pub preferred_resolution: VideoDimensions,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDisplayConfiguratorInfo {
    pub device_available: bool,
    pub preview_layer_available: bool,
    pub active: bool,
    pub active_external_display_frame_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeskViewApplicationCompletionPayload {
    error: Option<String>,
}

struct DeskViewCompletionState {
    callback: Box<dyn FnMut(Result<(), AVCaptureError>) + Send + 'static>,
}

#[derive(Debug)]
pub struct DeskViewApplication {
    ptr: *mut c_void,
}

impl Drop for DeskViewApplication {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::desk_view_application::av_capture_desk_view_application_release(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl DeskViewApplication {
    fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_create(&mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<DeskViewApplicationInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_info_json(
                self.ptr, &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn present(&self) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_present(
                self.ptr,
                None,
                ptr::null_mut(),
                None,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn present_with_launch_configuration(
        &self,
        launch_configuration: &DeskViewApplicationLaunchConfiguration,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_present_with_launch_configuration(
                self.ptr,
                launch_configuration.ptr,
                None,
                ptr::null_mut(),
                None,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn present_with_completion_handler<F>(&self, callback: F) -> Result<(), AVCaptureError>
    where
        F: FnMut(Result<(), AVCaptureError>) + Send + 'static,
    {
        let state = Box::new(DeskViewCompletionState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_present(
                self.ptr,
                Some(desk_view_completion_trampoline),
                userdata,
                Some(desk_view_completion_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { desk_view_completion_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn present_with_launch_configuration_and_completion_handler<F>(
        &self,
        launch_configuration: &DeskViewApplicationLaunchConfiguration,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(Result<(), AVCaptureError>) + Send + 'static,
    {
        let state = Box::new(DeskViewCompletionState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_present_with_launch_configuration(
                self.ptr,
                launch_configuration.ptr,
                Some(desk_view_completion_trampoline),
                userdata,
                Some(desk_view_completion_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { desk_view_completion_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct DeskViewApplicationLaunchConfiguration {
    ptr: *mut c_void,
}

impl Drop for DeskViewApplicationLaunchConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::desk_view_application::av_capture_desk_view_application_launch_configuration_release(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl DeskViewApplicationLaunchConfiguration {
    fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_launch_configuration_create(
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<DeskViewApplicationLaunchConfigurationInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_launch_configuration_info_json(
                self.ptr,
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn main_window_frame(&self) -> Result<CaptureRect, AVCaptureError> {
        Ok(self.info()?.main_window_frame)
    }

    pub fn requires_setup_mode_completion(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.requires_setup_mode_completion)
    }

    pub fn set_main_window_frame(&self, frame: &CaptureRect) -> Result<(), AVCaptureError> {
        let frame = json_cstring(frame, "desk view main window frame")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_launch_configuration_set_main_window_frame_json(
                self.ptr,
                frame.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    pub fn set_requires_setup_mode_completion(&self, required: bool) {
        unsafe {
            ffi::desk_view_application::av_capture_desk_view_application_launch_configuration_set_requires_setup_mode_completion(
                self.ptr,
                required,
            );
        }
    }
}

#[derive(Debug)]
pub struct ExternalDisplayConfiguration {
    ptr: *mut c_void,
}

impl Drop for ExternalDisplayConfiguration {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::external_display::av_capture_external_display_configuration_release(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl ExternalDisplayConfiguration {
    fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::external_display::av_capture_external_display_configuration_create(&mut err)
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<ExternalDisplayConfigurationInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::external_display::av_capture_external_display_configuration_info_json(
                self.ptr, &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn should_match_frame_rate(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.should_match_frame_rate)
    }

    pub fn bypass_color_space_conversion(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.bypass_color_space_conversion)
    }

    pub fn preferred_resolution(&self) -> Result<VideoDimensions, AVCaptureError> {
        Ok(self.info()?.preferred_resolution)
    }

    pub fn set_should_match_frame_rate(&self, enabled: bool) {
        unsafe {
            ffi::external_display::av_capture_external_display_configuration_set_should_match_frame_rate(
                self.ptr,
                enabled,
            );
        }
    }

    pub fn set_bypass_color_space_conversion(&self, enabled: bool) {
        unsafe {
            ffi::external_display::av_capture_external_display_configuration_set_bypass_color_space_conversion(
                self.ptr,
                enabled,
            );
        }
    }

    pub fn set_preferred_resolution(
        &self,
        preferred_resolution: &VideoDimensions,
    ) -> Result<(), AVCaptureError> {
        let preferred_resolution = json_cstring(
            preferred_resolution,
            "external display preferred resolution",
        )?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::external_display::av_capture_external_display_configuration_set_preferred_resolution_json(
                self.ptr,
                preferred_resolution.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ExternalDisplayConfigurator {
    ptr: *mut c_void,
}

impl Drop for ExternalDisplayConfigurator {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::external_display::av_capture_external_display_configurator_release(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl ExternalDisplayConfigurator {
    fn new(
        device: &CaptureDevice,
        preview_layer: &VideoPreviewLayer,
        configuration: &ExternalDisplayConfiguration,
    ) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::external_display::av_capture_external_display_configurator_create(
                device.ptr,
                preview_layer.ptr,
                configuration.ptr,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    pub fn info(&self) -> Result<ExternalDisplayConfiguratorInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::external_display::av_capture_external_display_configurator_info_json(
                self.ptr, &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn is_active(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.active)
    }

    pub fn active_external_display_frame_rate(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.active_external_display_frame_rate)
    }

    pub fn device_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.device_available)
    }

    pub fn preview_layer_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.preview_layer_available)
    }

    pub fn stop(&self) {
        unsafe { ffi::external_display::av_capture_external_display_configurator_stop(self.ptr) };
    }
}

impl VideoPreviewLayer {
    pub fn desk_view_application() -> Result<DeskViewApplication, AVCaptureError> {
        DeskViewApplication::new()
    }

    pub fn desk_view_application_launch_configuration(
    ) -> Result<DeskViewApplicationLaunchConfiguration, AVCaptureError> {
        DeskViewApplicationLaunchConfiguration::new()
    }

    pub fn external_display_support_info() -> Result<ExternalDisplaySupportInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::external_display::av_capture_external_display_support_info_json(&mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn external_display_configuration() -> Result<ExternalDisplayConfiguration, AVCaptureError>
    {
        ExternalDisplayConfiguration::new()
    }

    pub fn external_display_configurator(
        &self,
        device: &CaptureDevice,
        configuration: &ExternalDisplayConfiguration,
    ) -> Result<ExternalDisplayConfigurator, AVCaptureError> {
        ExternalDisplayConfigurator::new(device, self, configuration)
    }
}

unsafe extern "C" fn desk_view_completion_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<DeskViewCompletionState>().as_mut() else {
        return;
    };
    let result = match parse_json_and_free::<DeskViewApplicationCompletionPayload>(payload) {
        Ok(payload) => payload.error.map_or_else(
            || Ok(()),
            |message| Err(AVCaptureError::OperationFailed(message)),
        ),
        Err(err) => Err(err),
    };
    (state.callback)(result);
}

unsafe extern "C" fn desk_view_completion_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(userdata.cast::<DeskViewCompletionState>()));
}
