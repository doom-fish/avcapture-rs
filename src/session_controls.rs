use core::ffi::{c_char, c_void};
use core::ops::Deref;
use core::ptr;
use std::ffi::CString;

use serde::{Deserialize, Serialize};

use crate::device::CaptureDevice;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cstring, json_cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureControl` state.
pub struct CaptureControlInfo {
    /// The callback kind reported by the underlying API.
    pub kind: String,
    /// The enabled reported by `AVCaptureControl`.
    pub enabled: bool,
    /// The localized title reported by `AVCaptureControl`.
    pub localized_title: Option<String>,
    /// The symbol name reported by `AVCaptureControl`.
    pub symbol_name: Option<String>,
    /// The accessibility identifier reported by `AVCaptureControl`.
    pub accessibility_identifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureSlider` state.
pub struct CaptureSliderInfo {
    #[serde(flatten)]
    /// The shared `AVCaptureControl` state.
    pub control: CaptureControlInfo,
    /// The value reported by `AVCaptureSlider`.
    pub value: f32,
    /// The min value reported by `AVCaptureSlider`.
    pub min_value: Option<f32>,
    /// The max value reported by `AVCaptureSlider`.
    pub max_value: Option<f32>,
    /// The step reported by `AVCaptureSlider`.
    pub step: Option<f32>,
    /// The values reported by `AVCaptureSlider`.
    pub values: Vec<f32>,
    /// The prominent values reported by `AVCaptureSlider`.
    pub prominent_values: Vec<f32>,
    /// The localized value format reported by `AVCaptureSlider`.
    pub localized_value_format: Option<String>,
    /// The has action handler reported by `AVCaptureSlider`.
    pub has_action_handler: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureIndexPicker` state.
pub struct CaptureIndexPickerInfo {
    #[serde(flatten)]
    /// The shared `AVCaptureControl` state.
    pub control: CaptureControlInfo,
    /// The selected index reported by `AVCaptureIndexPicker`.
    pub selected_index: usize,
    /// The number of indexes reported by `AVCaptureIndexPicker`.
    pub number_of_indexes: usize,
    /// The localized index titles reported by `AVCaptureIndexPicker`.
    pub localized_index_titles: Vec<String>,
    /// The has action handler reported by `AVCaptureIndexPicker`.
    pub has_action_handler: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event payload derived from `AVCaptureSession` callbacks.
pub struct CaptureSessionControlsEvent {
    /// The callback kind reported by the underlying API.
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event payload derived from `AVCaptureSession` callbacks.
pub struct CaptureSessionDeferredStartEvent {
    /// The callback kind reported by the underlying API.
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SliderActionPayload {
    value: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndexPickerActionPayload {
    selected_index: usize,
}

struct SliderCallbackState {
    callback: Box<dyn FnMut(f32) + Send + 'static>,
}

struct IndexPickerCallbackState {
    callback: Box<dyn FnMut(usize) + Send + 'static>,
}

struct ControlsDelegateCallbackState {
    callback: Box<dyn FnMut(CaptureSessionControlsEvent) + Send + 'static>,
}

struct DeferredStartDelegateCallbackState {
    callback: Box<dyn FnMut(CaptureSessionDeferredStartEvent) + Send + 'static>,
}

#[derive(Debug)]
/// Wraps `AVCaptureControl`.
pub struct CaptureControl {
    pub(crate) ptr: *mut c_void,
}

impl Drop for CaptureControl {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::session::av_capture_control_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureControl {
    pub(crate) const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    pub(crate) const fn control_ptr(&self) -> *mut c_void {
        self.ptr
    }

    /// Returns a snapshot of `AVCaptureControl` state.
    pub fn info(&self) -> Result<CaptureControlInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::session::av_capture_control_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(error_from_status(
                ffi::status::SESSION_ERROR,
                err,
                "failed to read capture control info",
            ));
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureControl.kind`.
    pub fn kind(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.kind)
    }

    /// Returns whether `AVCaptureControl` is enabled.
    pub fn is_enabled(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.enabled)
    }

    /// Sets the enabled on `AVCaptureControl`.
    pub fn set_enabled(&self, enabled: bool) {
        unsafe { ffi::session::av_capture_control_set_enabled(self.ptr, enabled) };
    }

    /// Corresponds to `AVCaptureControl.localized_title`.
    pub fn localized_title(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.localized_title)
    }

    /// Corresponds to `AVCaptureControl.symbol_name`.
    pub fn symbol_name(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.symbol_name)
    }

    /// Corresponds to `AVCaptureControl.accessibility_identifier`.
    pub fn accessibility_identifier(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.accessibility_identifier)
    }

    /// Returns whether `AVCaptureControl` is index picker.
    pub fn is_index_picker(&self) -> Result<bool, AVCaptureError> {
        Ok(self.kind()? == "indexPicker")
    }

    /// Returns whether `AVCaptureControl` is slider.
    pub fn is_slider(&self) -> Result<bool, AVCaptureError> {
        Ok(self.kind()? == "slider")
    }

    /// Returns whether `AVCaptureControl` is system exposure bias slider.
    pub fn is_system_exposure_bias_slider(&self) -> Result<bool, AVCaptureError> {
        Ok(self.kind()? == "systemExposureBiasSlider")
    }

    /// Returns whether `AVCaptureControl` is system zoom slider.
    pub fn is_system_zoom_slider(&self) -> Result<bool, AVCaptureError> {
        Ok(self.kind()? == "systemZoomSlider")
    }
}

#[derive(Debug)]
/// Wraps `AVCaptureIndexPicker`.
pub struct CaptureIndexPicker {
    control: CaptureControl,
}

impl Deref for CaptureIndexPicker {
    type Target = CaptureControl;

    fn deref(&self) -> &Self::Target {
        &self.control
    }
}

impl CaptureIndexPicker {
    pub(crate) fn new(
        localized_title: &str,
        symbol_name: &str,
        number_of_indexes: usize,
    ) -> Result<Self, AVCaptureError> {
        if number_of_indexes == 0 {
            return Err(AVCaptureError::InvalidArgument(
                "index picker number_of_indexes must be greater than 0".to_owned(),
            ));
        }
        let localized_title = cstring(localized_title, "index picker localized title")?;
        let symbol_name = cstring(symbol_name, "index picker symbol name")?;
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_index_picker_create(
                localized_title.as_ptr(),
                symbol_name.as_ptr(),
                number_of_indexes,
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture index picker",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    pub(crate) fn new_with_titles(
        localized_title: &str,
        symbol_name: &str,
        localized_index_titles: &[&str],
    ) -> Result<Self, AVCaptureError> {
        if localized_index_titles.is_empty() {
            return Err(AVCaptureError::InvalidArgument(
                "index picker localized_index_titles must not be empty".to_owned(),
            ));
        }
        let localized_title = cstring(localized_title, "index picker localized title")?;
        let symbol_name = cstring(symbol_name, "index picker symbol name")?;
        let titles = localized_index_titles
            .iter()
            .map(|title| (*title).to_owned())
            .collect::<Vec<_>>();
        let titles_json = json_cstring(&titles, "index picker localized index titles")?;
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_index_picker_create_with_titles_json(
                localized_title.as_ptr(),
                symbol_name.as_ptr(),
                titles_json.as_ptr(),
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture index picker",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    /// Returns a snapshot of `AVCaptureIndexPicker` state.
    pub fn info(&self) -> Result<CaptureIndexPickerInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::session::av_capture_index_picker_info_json(self.control.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(error_from_status(
                ffi::status::SESSION_ERROR,
                err,
                "failed to read capture index picker info",
            ));
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureIndexPicker.selected_index`.
    pub fn selected_index(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.selected_index)
    }

    /// Corresponds to `AVCaptureIndexPicker.number_of_indexes`.
    pub fn number_of_indexes(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.number_of_indexes)
    }

    /// Corresponds to `AVCaptureIndexPicker.localized_index_titles`.
    pub fn localized_index_titles(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.localized_index_titles)
    }

    /// Returns whether `AVCaptureIndexPicker` has action handler.
    pub fn has_action_handler(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.has_action_handler)
    }

    /// Sets the selected index on `AVCaptureIndexPicker`.
    pub fn set_selected_index(&self, selected_index: usize) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::session::av_capture_index_picker_set_selected_index(
                self.control.ptr,
                selected_index,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(error_from_status(
                status,
                err,
                "failed to set capture index picker selected index",
            ));
        }
        Ok(())
    }

    /// Sets the action handler on `AVCaptureIndexPicker`.
    pub fn set_action_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(usize) + Send + 'static,
    {
        let queue_label = queue_label_cstring(
            queue_label,
            "avcapture-index-picker-action",
            "index picker action queue label",
        )?;
        let state = Box::new(IndexPickerCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::session::av_capture_index_picker_set_action_callback(
                self.control.ptr,
                queue_label.as_ptr(),
                Some(index_picker_action_trampoline),
                userdata,
                Some(index_picker_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { index_picker_callback_drop(userdata) };
            return Err(error_from_status(
                status,
                err,
                "failed to install capture index picker action handler",
            ));
        }
        Ok(())
    }

    /// Clears the action handler on `AVCaptureIndexPicker`.
    pub fn clear_action_handler(&self) {
        unsafe { ffi::session::av_capture_index_picker_clear_action_callback(self.control.ptr) };
    }
}

#[derive(Debug)]
/// Wraps `AVCaptureSlider`.
pub struct CaptureSlider {
    control: CaptureControl,
}

impl Deref for CaptureSlider {
    type Target = CaptureControl;

    fn deref(&self) -> &Self::Target {
        &self.control
    }
}

impl CaptureSlider {
    pub(crate) fn new(
        localized_title: &str,
        symbol_name: &str,
        min_value: f32,
        max_value: f32,
    ) -> Result<Self, AVCaptureError> {
        validate_slider_bounds(min_value, max_value)?;
        let localized_title = cstring(localized_title, "slider localized title")?;
        let symbol_name = cstring(symbol_name, "slider symbol name")?;
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_slider_create(
                localized_title.as_ptr(),
                symbol_name.as_ptr(),
                min_value,
                max_value,
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    pub(crate) fn new_with_step(
        localized_title: &str,
        symbol_name: &str,
        min_value: f32,
        max_value: f32,
        step: f32,
    ) -> Result<Self, AVCaptureError> {
        validate_slider_bounds(min_value, max_value)?;
        if !step.is_finite() || step <= 0.0 {
            return Err(AVCaptureError::InvalidArgument(
                "slider step must be a finite value greater than 0".to_owned(),
            ));
        }
        let localized_title = cstring(localized_title, "slider localized title")?;
        let symbol_name = cstring(symbol_name, "slider symbol name")?;
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_slider_create_with_step(
                localized_title.as_ptr(),
                symbol_name.as_ptr(),
                min_value,
                max_value,
                step,
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    pub(crate) fn new_with_values(
        localized_title: &str,
        symbol_name: &str,
        values: &[f32],
    ) -> Result<Self, AVCaptureError> {
        if values.is_empty() {
            return Err(AVCaptureError::InvalidArgument(
                "slider values must not be empty".to_owned(),
            ));
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(AVCaptureError::InvalidArgument(
                "slider values must all be finite".to_owned(),
            ));
        }
        let localized_title = cstring(localized_title, "slider localized title")?;
        let symbol_name = cstring(symbol_name, "slider symbol name")?;
        let values_json = json_cstring(&values, "slider values")?;
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_slider_create_with_values_json(
                localized_title.as_ptr(),
                symbol_name.as_ptr(),
                values_json.as_ptr(),
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    /// Returns a snapshot of `AVCaptureSlider` state.
    pub fn info(&self) -> Result<CaptureSliderInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::session::av_capture_slider_info_json(self.control.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(error_from_status(
                ffi::status::SESSION_ERROR,
                err,
                "failed to read capture slider info",
            ));
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureSlider.value`.
    pub fn value(&self) -> Result<f32, AVCaptureError> {
        Ok(self.info()?.value)
    }

    /// Corresponds to `AVCaptureSlider.min_value`.
    pub fn min_value(&self) -> Result<Option<f32>, AVCaptureError> {
        Ok(self.info()?.min_value)
    }

    /// Corresponds to `AVCaptureSlider.max_value`.
    pub fn max_value(&self) -> Result<Option<f32>, AVCaptureError> {
        Ok(self.info()?.max_value)
    }

    /// Corresponds to `AVCaptureSlider.values`.
    pub fn values(&self) -> Result<Vec<f32>, AVCaptureError> {
        Ok(self.info()?.values)
    }

    /// Corresponds to `AVCaptureSlider.localized_value_format`.
    pub fn localized_value_format(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.localized_value_format)
    }

    /// Corresponds to `AVCaptureSlider.prominent_values`.
    pub fn prominent_values(&self) -> Result<Vec<f32>, AVCaptureError> {
        Ok(self.info()?.prominent_values)
    }

    /// Returns whether `AVCaptureSlider` has action handler.
    pub fn has_action_handler(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.has_action_handler)
    }

    /// Sets the value on `AVCaptureSlider`.
    pub fn set_value(&self, value: f32) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::session::av_capture_slider_set_value(self.control.ptr, value, &mut err) };
        if status != ffi::status::OK {
            return Err(error_from_status(
                status,
                err,
                "failed to set capture slider value",
            ));
        }
        Ok(())
    }

    /// Sets the action handler on `AVCaptureSlider`.
    pub fn set_action_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let queue_label = queue_label_cstring(
            queue_label,
            "avcapture-slider-action",
            "slider action queue label",
        )?;
        let state = Box::new(SliderCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::session::av_capture_slider_set_action_callback(
                self.control.ptr,
                queue_label.as_ptr(),
                Some(slider_action_trampoline),
                userdata,
                Some(slider_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { slider_callback_drop(userdata) };
            return Err(error_from_status(
                status,
                err,
                "failed to install capture slider action handler",
            ));
        }
        Ok(())
    }

    /// Clears the action handler on `AVCaptureSlider`.
    pub fn clear_action_handler(&self) {
        unsafe { ffi::session::av_capture_slider_clear_action_callback(self.control.ptr) };
    }
}

#[derive(Debug)]
/// Wraps `AVCaptureSystemExposureBiasSlider`.
pub struct CaptureSystemExposureBiasSlider {
    control: CaptureControl,
}

impl Deref for CaptureSystemExposureBiasSlider {
    type Target = CaptureControl;

    fn deref(&self) -> &Self::Target {
        &self.control
    }
}

impl CaptureSystemExposureBiasSlider {
    pub(crate) fn new(device: &CaptureDevice) -> Result<Self, AVCaptureError> {
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_system_exposure_bias_slider_create(
                device.ptr,
                None,
                ptr::null_mut(),
                None,
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture system exposure bias slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    pub(crate) fn new_with_handler<F>(
        device: &CaptureDevice,
        callback: F,
    ) -> Result<Self, AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let state = Box::new(SliderCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_system_exposure_bias_slider_create(
                device.ptr,
                Some(slider_action_trampoline),
                userdata,
                Some(slider_callback_drop),
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            unsafe { slider_callback_drop(userdata) };
            return Err(error_from_status(
                status,
                err,
                "failed to create capture system exposure bias slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }
}

#[derive(Debug)]
/// Wraps `AVCaptureSystemZoomSlider`.
pub struct CaptureSystemZoomSlider {
    control: CaptureControl,
}

impl Deref for CaptureSystemZoomSlider {
    type Target = CaptureControl;

    fn deref(&self) -> &Self::Target {
        &self.control
    }
}

impl CaptureSystemZoomSlider {
    pub(crate) fn new(device: &CaptureDevice) -> Result<Self, AVCaptureError> {
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_system_zoom_slider_create(
                device.ptr,
                None,
                ptr::null_mut(),
                None,
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(error_from_status(
                status,
                err,
                "failed to create capture system zoom slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }

    pub(crate) fn new_with_handler<F>(
        device: &CaptureDevice,
        callback: F,
    ) -> Result<Self, AVCaptureError>
    where
        F: FnMut(f32) + Send + 'static,
    {
        let state = Box::new(SliderCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut status = ffi::status::SESSION_ERROR;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_system_zoom_slider_create(
                device.ptr,
                Some(slider_action_trampoline),
                userdata,
                Some(slider_callback_drop),
                &mut status,
                &mut err,
            )
        };
        if ptr.is_null() {
            unsafe { slider_callback_drop(userdata) };
            return Err(error_from_status(
                status,
                err,
                "failed to create capture system zoom slider",
            ));
        }
        Ok(Self {
            control: CaptureControl::from_raw(ptr),
        })
    }
}

pub(super) fn session_controls_count(session_ptr: *mut c_void) -> usize {
    unsafe { ffi::session::av_capture_session_controls_count(session_ptr) }
}

pub(super) fn session_controls(
    session_ptr: *mut c_void,
) -> Result<Vec<CaptureControl>, AVCaptureError> {
    let count = session_controls_count(session_ptr);
    let mut controls = Vec::with_capacity(count);
    for index in 0..count {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::session::av_capture_session_control_at_index(session_ptr, index, &mut err)
        };
        if ptr.is_null() {
            return Err(error_from_status(
                ffi::status::SESSION_ERROR,
                err,
                "failed to read capture session control",
            ));
        }
        controls.push(CaptureControl::from_raw(ptr));
    }
    Ok(controls)
}

pub(super) fn session_can_add_control(session_ptr: *mut c_void, control: &CaptureControl) -> bool {
    unsafe { ffi::session::av_capture_session_can_add_control(session_ptr, control.control_ptr()) }
}

pub(super) fn session_add_control(
    session_ptr: *mut c_void,
    control: &CaptureControl,
) -> Result<(), AVCaptureError> {
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::session::av_capture_session_add_control(session_ptr, control.control_ptr(), &mut err)
    };
    if status != ffi::status::OK {
        return Err(error_from_status(
            status,
            err,
            "failed to add capture control to session",
        ));
    }
    Ok(())
}

pub(super) fn session_remove_control(session_ptr: *mut c_void, control: &CaptureControl) {
    unsafe { ffi::session::av_capture_session_remove_control(session_ptr, control.control_ptr()) };
}

pub(super) fn install_controls_delegate_handler<F>(
    session_ptr: *mut c_void,
    queue_label: Option<&str>,
    callback: F,
) -> Result<(), AVCaptureError>
where
    F: FnMut(CaptureSessionControlsEvent) + Send + 'static,
{
    let queue_label = queue_label_cstring(
        queue_label,
        "avcapture-session-controls",
        "session controls delegate queue label",
    )?;
    let state = Box::new(ControlsDelegateCallbackState {
        callback: Box::new(callback),
    });
    let userdata = Box::into_raw(state).cast::<c_void>();
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::session::av_capture_session_set_controls_delegate_callback(
            session_ptr,
            queue_label.as_ptr(),
            Some(session_controls_delegate_trampoline),
            userdata,
            Some(session_controls_delegate_callback_drop),
            &mut err,
        )
    };
    if status != ffi::status::OK {
        unsafe { session_controls_delegate_callback_drop(userdata) };
        return Err(error_from_status(
            status,
            err,
            "failed to install session controls delegate handler",
        ));
    }
    Ok(())
}

pub(super) fn clear_controls_delegate_handler(session_ptr: *mut c_void) {
    unsafe { ffi::session::av_capture_session_clear_controls_delegate_callback(session_ptr) };
}

pub(super) fn install_deferred_start_delegate_handler<F>(
    session_ptr: *mut c_void,
    queue_label: Option<&str>,
    callback: F,
) -> Result<(), AVCaptureError>
where
    F: FnMut(CaptureSessionDeferredStartEvent) + Send + 'static,
{
    let queue_label = queue_label_cstring(
        queue_label,
        "avcapture-session-deferred-start",
        "session deferred start delegate queue label",
    )?;
    let state = Box::new(DeferredStartDelegateCallbackState {
        callback: Box::new(callback),
    });
    let userdata = Box::into_raw(state).cast::<c_void>();
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        ffi::session::av_capture_session_set_deferred_start_delegate_callback(
            session_ptr,
            queue_label.as_ptr(),
            Some(session_deferred_start_delegate_trampoline),
            userdata,
            Some(session_deferred_start_delegate_callback_drop),
            &mut err,
        )
    };
    if status != ffi::status::OK {
        unsafe { session_deferred_start_delegate_callback_drop(userdata) };
        return Err(error_from_status(
            status,
            err,
            "failed to install session deferred start delegate handler",
        ));
    }
    Ok(())
}

pub(super) fn clear_deferred_start_delegate_handler(session_ptr: *mut c_void) {
    unsafe { ffi::session::av_capture_session_clear_deferred_start_delegate_callback(session_ptr) };
}

unsafe extern "C" fn slider_action_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<SliderCallbackState>().as_mut() else {
        return;
    };
    let Ok(payload) = parse_json_and_free::<SliderActionPayload>(payload) else {
        return;
    };
    (state.callback)(payload.value);
}

unsafe extern "C" fn slider_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(userdata.cast::<SliderCallbackState>()));
}

unsafe extern "C" fn index_picker_action_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<IndexPickerCallbackState>().as_mut() else {
        return;
    };
    let Ok(payload) = parse_json_and_free::<IndexPickerActionPayload>(payload) else {
        return;
    };
    (state.callback)(payload.selected_index);
}

unsafe extern "C" fn index_picker_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(userdata.cast::<IndexPickerCallbackState>()));
}

unsafe extern "C" fn session_controls_delegate_trampoline(
    userdata: *mut c_void,
    payload: *mut c_char,
) {
    let Some(state) = userdata.cast::<ControlsDelegateCallbackState>().as_mut() else {
        return;
    };
    let Ok(event) = parse_json_and_free::<CaptureSessionControlsEvent>(payload) else {
        return;
    };
    (state.callback)(event);
}

unsafe extern "C" fn session_controls_delegate_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(
        userdata.cast::<ControlsDelegateCallbackState>(),
    ));
}

unsafe extern "C" fn session_deferred_start_delegate_trampoline(
    userdata: *mut c_void,
    payload: *mut c_char,
) {
    let Some(state) = userdata
        .cast::<DeferredStartDelegateCallbackState>()
        .as_mut()
    else {
        return;
    };
    let Ok(event) = parse_json_and_free::<CaptureSessionDeferredStartEvent>(payload) else {
        return;
    };
    (state.callback)(event);
}

unsafe extern "C" fn session_deferred_start_delegate_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(
        userdata.cast::<DeferredStartDelegateCallbackState>(),
    ));
}

fn validate_slider_bounds(min_value: f32, max_value: f32) -> Result<(), AVCaptureError> {
    if !min_value.is_finite() || !max_value.is_finite() {
        return Err(AVCaptureError::InvalidArgument(
            "slider min_value and max_value must be finite".to_owned(),
        ));
    }
    if min_value >= max_value {
        return Err(AVCaptureError::InvalidArgument(
            "slider min_value must be less than max_value".to_owned(),
        ));
    }
    Ok(())
}

fn queue_label_cstring(
    queue_label: Option<&str>,
    default_label: &str,
    what: &str,
) -> Result<CString, AVCaptureError> {
    cstring(queue_label.unwrap_or(default_label), what)
}

fn error_from_status(status: i32, err: *mut c_char, fallback: &str) -> AVCaptureError {
    if err.is_null() {
        match status {
            ffi::status::INVALID_ARGUMENT => AVCaptureError::InvalidArgument(fallback.to_owned()),
            ffi::status::DEVICE_ERROR => AVCaptureError::DeviceError(fallback.to_owned()),
            ffi::status::INPUT_ERROR => AVCaptureError::InputError(fallback.to_owned()),
            ffi::status::SESSION_ERROR => AVCaptureError::SessionError(fallback.to_owned()),
            ffi::status::OUTPUT_ERROR => AVCaptureError::OutputError(fallback.to_owned()),
            ffi::status::CALLBACK_ERROR => AVCaptureError::CallbackError(fallback.to_owned()),
            ffi::status::OPERATION_FAILED => AVCaptureError::OperationFailed(fallback.to_owned()),
            _ => AVCaptureError::OperationFailed(format!("{fallback} (status {status})")),
        }
    } else {
        unsafe { from_swift(status, err) }
    }
}
