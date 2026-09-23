#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::{Deserialize, Serialize};

use crate::callback::{ArcContext, SerializedCallback};
use crate::device::CaptureFlashMode;
use crate::error::{from_swift, report_callback_error, AVCaptureError};
use crate::ffi;
use crate::helpers::{parse_json_and_free, VideoDimensions};
use crate::output::CaptureOutputRef;
use crate::photo::{
    Photo, PhotoQualityPrioritization, PhotoSettings, PhotoSettingsInfo, ResolvedPhotoSettingsInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
/// `AVCapturePhotoOutput` values.
pub enum PhotoOutputCaptureReadiness {
    /// Corresponds to the `SessionNotRunning` case.
    SessionNotRunning,
    /// Corresponds to the `Ready` case.
    Ready,
    /// Corresponds to the `NotReadyMomentarily` case.
    NotReadyMomentarily,
    /// Corresponds to the `NotReadyWaitingForCapture` case.
    NotReadyWaitingForCapture,
    /// Corresponds to the `NotReadyWaitingForProcessing` case.
    NotReadyWaitingForProcessing,
    /// A value not recognized by this crate.
    Unknown(i32),
}

impl PhotoOutputCaptureReadiness {
    #[must_use]
    /// Decodes an `AVCapturePhotoOutput.CaptureReadiness` raw value.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::SessionNotRunning,
            1 => Self::Ready,
            2 => Self::NotReadyMomentarily,
            3 => Self::NotReadyWaitingForCapture,
            4 => Self::NotReadyWaitingForProcessing,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    /// Returns the raw SDK value for `AVCapturePhotoOutput`.
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::SessionNotRunning => 0,
            Self::Ready => 1,
            Self::NotReadyMomentarily => 2,
            Self::NotReadyWaitingForCapture => 3,
            Self::NotReadyWaitingForProcessing => 4,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for PhotoOutputCaptureReadiness {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<PhotoOutputCaptureReadiness> for i32 {
    fn from(value: PhotoOutputCaptureReadiness) -> Self {
        value.as_raw()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCapturePhotoOutput` state.
pub struct PhotoOutputInfo {
    /// The connection count reported by `AVCapturePhotoOutput`.
    pub connection_count: usize,
    /// The available photo codec types reported by `AVCapturePhotoOutput`.
    pub available_photo_codec_types: Vec<String>,
    /// The available photo file types reported by `AVCapturePhotoOutput`.
    pub available_photo_file_types: Vec<String>,
    /// The available photo pixel format types reported by `AVCapturePhotoOutput`.
    pub available_photo_pixel_format_types: Vec<u32>,
    /// The available raw photo pixel format types reported by `AVCapturePhotoOutput`.
    pub available_raw_photo_pixel_format_types: Option<Vec<u32>>,
    /// The supported flash modes reported by `AVCapturePhotoOutput`.
    pub supported_flash_modes: Vec<CaptureFlashMode>,
    /// The max photo dimensions reported by `AVCapturePhotoOutput`.
    pub max_photo_dimensions: Option<VideoDimensions>,
    /// The capture readiness reported by `AVCapturePhotoOutput`.
    pub capture_readiness: Option<PhotoOutputCaptureReadiness>,
    /// The max photo quality prioritization reported by `AVCapturePhotoOutput`.
    pub max_photo_quality_prioritization: Option<PhotoQualityPrioritization>,
    /// The high resolution capture enabled reported by `AVCapturePhotoOutput`.
    pub high_resolution_capture_enabled: bool,
    /// The responsive capture enabled reported by `AVCapturePhotoOutput`.
    pub responsive_capture_enabled: Option<bool>,
    /// The callback installed reported by `AVCapturePhotoOutput`.
    pub callback_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Result payload produced by `AVCapturePhotoOutput` capture callbacks.
pub struct PhotoCaptureResult {
    #[serde(rename = "uniqueID", alias = "uniqueId")]
    /// The unique id reported by `AVCapturePhotoOutput`.
    pub unique_id: i64,
    /// The error message, if any.
    pub error: Option<String>,
}

#[derive(Debug)]
/// Detailed event payload produced by `AVCapturePhotoCaptureDelegate` callbacks.
pub struct PhotoCaptureEvent {
    /// The unique id reported by `AVCapturePhotoCaptureDelegate`.
    pub unique_id: i64,
    /// The error message, if any.
    pub error: Option<String>,
    /// The resolved settings reported by `AVCapturePhotoCaptureDelegate`.
    pub resolved_settings: ResolvedPhotoSettingsInfo,
    /// The captured `Photo`, if one was produced.
    pub photo: Option<Photo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PhotoCaptureEventPayload {
    #[serde(rename = "uniqueID", alias = "uniqueId")]
    unique_id: i64,
    error: Option<String>,
    resolved_settings: ResolvedPhotoSettingsInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PhotoOutputReadinessPayload {
    capture_readiness: PhotoOutputCaptureReadiness,
}

type PhotoCaptureEventCallbackState = SerializedCallback<Result<PhotoCaptureEvent, AVCaptureError>>;
type PhotoOutputReadinessCallbackState = SerializedCallback<PhotoOutputCaptureReadiness>;

/// Safe wrapper around `AVCapturePhotoOutput`.
#[derive(Debug)]
/// Wraps `AVCapturePhotoOutput`.
pub struct PhotoOutput {
    pub(crate) ptr: *mut c_void,
}

impl Clone for PhotoOutput {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { ffi::photo_output::av_capture_photo_output_retain(self.ptr) },
        }
    }
}

impl Drop for PhotoOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::photo_output::av_capture_photo_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureOutputRef for PhotoOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl crate::output::sealed::Sealed for PhotoOutput {}

impl PhotoOutput {
    /// Creates a new `AVCapturePhotoOutput` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::photo_output::av_capture_photo_output_create(&raw mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCapturePhotoOutput` state.
    pub fn info(&self) -> Result<PhotoOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::photo_output::av_capture_photo_output_info_json(self.ptr, &raw mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the connection count reported by `AVCapturePhotoOutput`.
    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    /// Returns the available photo codec types reported by `AVCapturePhotoOutput`.
    pub fn available_photo_codec_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_photo_codec_types)
    }

    /// Returns the available photo file types reported by `AVCapturePhotoOutput`.
    pub fn available_photo_file_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_photo_file_types)
    }

    /// Returns the available photo pixel format types reported by `AVCapturePhotoOutput`.
    pub fn available_photo_pixel_format_types(&self) -> Result<Vec<u32>, AVCaptureError> {
        Ok(self.info()?.available_photo_pixel_format_types)
    }

    /// Returns the available raw photo pixel format types reported by `AVCapturePhotoOutput`.
    pub fn available_raw_photo_pixel_format_types(
        &self,
    ) -> Result<Option<Vec<u32>>, AVCaptureError> {
        Ok(self.info()?.available_raw_photo_pixel_format_types)
    }

    /// Corresponds to `AVCapturePhotoOutput.supported_flash_modes`.
    pub fn supported_flash_modes(&self) -> Result<Vec<CaptureFlashMode>, AVCaptureError> {
        Ok(self.info()?.supported_flash_modes)
    }

    /// Corresponds to `AVCapturePhotoOutput.max_photo_dimensions`.
    pub fn max_photo_dimensions(&self) -> Result<Option<VideoDimensions>, AVCaptureError> {
        Ok(self.info()?.max_photo_dimensions)
    }

    /// Corresponds to `AVCapturePhotoOutput.capture_readiness`.
    pub fn capture_readiness(&self) -> Result<Option<PhotoOutputCaptureReadiness>, AVCaptureError> {
        Ok(self.info()?.capture_readiness)
    }

    /// Corresponds to `AVCapturePhotoOutput.max_photo_quality_prioritization`.
    pub fn max_photo_quality_prioritization(
        &self,
    ) -> Result<Option<PhotoQualityPrioritization>, AVCaptureError> {
        Ok(self.info()?.max_photo_quality_prioritization)
    }

    /// Corresponds to `AVCapturePhotoOutput.high_resolution_capture_enabled`.
    pub fn high_resolution_capture_enabled(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.high_resolution_capture_enabled)
    }

    /// Corresponds to `AVCapturePhotoOutput.responsive_capture_enabled`.
    pub fn responsive_capture_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.responsive_capture_enabled)
    }

    /// Corresponds to `AVCapturePhotoOutput.callback_installed`.
    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    /// Corresponds to `AVCapturePhotoOutput.readiness_coordinator`.
    pub fn readiness_coordinator(&self) -> Result<PhotoOutputReadinessCoordinator, AVCaptureError> {
        PhotoOutputReadinessCoordinator::new(self)
    }

    /// Sets the high resolution capture enabled on `AVCapturePhotoOutput`.
    pub fn set_high_resolution_capture_enabled(&self, enabled: bool) {
        unsafe {
            ffi::photo_output::av_capture_photo_output_set_high_resolution_capture_enabled(
                self.ptr, enabled,
            );
        }
    }

    /// Sets the max photo quality prioritization on `AVCapturePhotoOutput`.
    pub fn set_max_photo_quality_prioritization(
        &self,
        prioritization: PhotoQualityPrioritization,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_set_max_photo_quality_prioritization(
                self.ptr,
                prioritization.as_raw(),
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the responsive capture enabled on `AVCapturePhotoOutput`.
    pub fn set_responsive_capture_enabled(&self, enabled: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_set_responsive_capture_enabled(
                self.ptr,
                enabled,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Captures a photo with `AVCapturePhotoOutput` using default settings.
    pub fn capture_photo<F>(&self, mut callback: F) -> Result<(), AVCaptureError>
    where
        F: FnMut(PhotoCaptureResult) + Send + 'static,
    {
        let settings = PhotoSettings::new()?;
        self.capture_photo_with_settings(&settings, move |event| {
            callback(PhotoCaptureResult {
                unique_id: event.unique_id,
                error: event.error,
            });
        })
    }

    /// Captures a photo with `AVCapturePhotoOutput` using the provided settings.
    pub fn capture_photo_with_settings<F>(
        &self,
        settings: &PhotoSettings,
        mut callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(PhotoCaptureEvent) + Send + 'static,
    {
        self.capture_photo_with_settings_result(settings, move |result| match result {
            Ok(event) => callback(event),
            Err(error) => report_callback_error("photo capture callback", error),
        })
    }

    pub(crate) fn capture_photo_with_settings_result<F>(
        &self,
        settings: &PhotoSettings,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(Result<PhotoCaptureEvent, AVCaptureError>) + Send + 'static,
    {
        validate_photo_settings(&settings.info()?, &self.info()?)?;
        let state = ArcContext::new(PhotoCaptureEventCallbackState::new(callback));
        let userdata = state.as_ptr();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_capture_photo(
                self.ptr,
                settings.ptr,
                Some(photo_capture_event_trampoline),
                userdata,
                Some(photo_capture_event_callback_retain),
                Some(photo_capture_event_callback_release),
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

fn validate_photo_settings(
    settings: &PhotoSettingsInfo,
    output: &PhotoOutputInfo,
) -> Result<(), AVCaptureError> {
    if settings.used_for_capture {
        return Err(AVCaptureError::InvalidArgument(
            "photo settings have already been used for capture; create a copy with a new unique ID"
                .to_owned(),
        ));
    }
    if let Some(mode) = settings.flash_mode {
        if mode != CaptureFlashMode::Off && !output.supported_flash_modes.contains(&mode) {
            return Err(AVCaptureError::InvalidArgument(format!(
                "flash mode {mode:?} is not supported by the photo output (supported: {:?})",
                output.supported_flash_modes
            )));
        }
    }
    if let (Some(requested), Some(maximum)) = (
        settings.photo_quality_prioritization,
        output.max_photo_quality_prioritization,
    ) {
        if requested.as_raw() > maximum.as_raw() {
            return Err(AVCaptureError::InvalidArgument(format!(
                "photo quality prioritization {requested:?} exceeds the photo output's maximum {maximum:?}; raise it with set_max_photo_quality_prioritization first"
            )));
        }
    }
    Ok(())
}

/// Safe wrapper around `AVCapturePhotoOutputReadinessCoordinator`.
#[derive(Debug)]
/// Wraps `AVCapturePhotoOutputReadinessCoordinator`.
pub struct PhotoOutputReadinessCoordinator {
    ptr: *mut c_void,
}

impl Drop for PhotoOutputReadinessCoordinator {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::photo_output::av_capture_photo_output_readiness_coordinator_release(self.ptr);
            }
            self.ptr = ptr::null_mut();
        }
    }
}

impl PhotoOutputReadinessCoordinator {
    /// Creates a new `AVCapturePhotoOutputReadinessCoordinator` wrapper.
    pub fn new(output: &PhotoOutput) -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::photo_output::av_capture_photo_output_readiness_coordinator_create(
                output.ptr,
                &raw mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Corresponds to `AVCapturePhotoOutputReadinessCoordinator.capture_readiness`.
    pub fn capture_readiness(&self) -> Result<PhotoOutputCaptureReadiness, AVCaptureError> {
        let mut raw = 0;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_readiness_coordinator_capture_readiness(
                self.ptr,
                &raw mut raw,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(PhotoOutputCaptureReadiness::from_raw(raw))
    }

    /// Sets the capture-readiness handler on `AVCapturePhotoOutputReadinessCoordinator`.
    pub fn set_capture_readiness_handler<F>(&self, callback: F) -> Result<(), AVCaptureError>
    where
        F: FnMut(PhotoOutputCaptureReadiness) + Send + 'static,
    {
        let state = ArcContext::new(PhotoOutputReadinessCallbackState::new(callback));
        let userdata = state.as_ptr();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_readiness_coordinator_set_callback(
                self.ptr,
                Some(photo_output_readiness_trampoline),
                userdata,
                Some(photo_output_readiness_callback_retain),
                Some(photo_output_readiness_callback_release),
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Clears the capture readiness handler on `AVCapturePhotoOutputReadinessCoordinator`.
    pub fn clear_capture_readiness_handler(&self) {
        unsafe {
            ffi::photo_output::av_capture_photo_output_readiness_coordinator_clear_callback(
                self.ptr,
            );
        }
    }

    /// Corresponds to `AVCapturePhotoOutputReadinessCoordinator.start_tracking_capture_request`.
    pub fn start_tracking_capture_request(
        &self,
        settings: &PhotoSettings,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_readiness_coordinator_start_tracking_capture_request(
                self.ptr,
                settings.ptr,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCapturePhotoOutputReadinessCoordinator.stop_tracking_capture_request`.
    pub fn stop_tracking_capture_request(
        &self,
        settings_unique_id: i64,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::photo_output::av_capture_photo_output_readiness_coordinator_stop_tracking_capture_request(
                self.ptr,
                settings_unique_id,
                &raw mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCapturePhotoOutputReadinessCoordinator.stop_tracking_capture_request_for_settings`.
    pub fn stop_tracking_capture_request_for_settings(
        &self,
        settings: &PhotoSettings,
    ) -> Result<(), AVCaptureError> {
        self.stop_tracking_capture_request(settings.unique_id()?)
    }
}

// SAFETY: The readiness coordinator is an opaque ARC-managed wrapper that may
// be sent across thread boundaries for async readiness observation.
unsafe impl Send for PhotoOutputReadinessCoordinator {}

unsafe extern "C" fn photo_capture_event_trampoline(
    userdata: *mut c_void,
    photo_ptr: *mut c_void,
    payload: *mut c_char,
) {
    let result = match parse_json_and_free::<PhotoCaptureEventPayload>(payload) {
        Ok(result) => result,
        Err(error) => {
            if !photo_ptr.is_null() {
                ffi::photo::av_capture_photo_release(photo_ptr);
            }
            if let Some(state) = ArcContext::<PhotoCaptureEventCallbackState>::get(userdata) {
                state.dispatch("photo_capture_event_trampoline", Err(error));
            } else {
                report_callback_error(
                    "photo_capture_event_trampoline",
                    AVCaptureError::BridgeProtocol(
                        "photo callback context was null during decode failure".to_owned(),
                    ),
                );
            }
            return;
        }
    };
    let Some(state) = ArcContext::<PhotoCaptureEventCallbackState>::get(userdata) else {
        if !photo_ptr.is_null() {
            ffi::photo::av_capture_photo_release(photo_ptr);
        }
        return;
    };
    let photo = if photo_ptr.is_null() {
        None
    } else {
        Some(unsafe { Photo::from_retained_bridge_box(photo_ptr) })
    };
    // User closures can panic; catch them here so the panic doesn't unwind
    // across the `extern "C"` boundary (which is UB).
    state.dispatch(
        "photo_capture_event_trampoline",
        Ok(PhotoCaptureEvent {
            unique_id: result.unique_id,
            error: result.error,
            resolved_settings: result.resolved_settings,
            photo,
        }),
    );
}

unsafe extern "C" fn photo_capture_event_callback_retain(userdata: *mut c_void) {
    ArcContext::<PhotoCaptureEventCallbackState>::retain(userdata);
}

unsafe extern "C" fn photo_capture_event_callback_release(userdata: *mut c_void) {
    ArcContext::<PhotoCaptureEventCallbackState>::release(userdata);
}

unsafe extern "C" fn photo_output_readiness_callback_retain(userdata: *mut c_void) {
    ArcContext::<PhotoOutputReadinessCallbackState>::retain(userdata);
}

unsafe extern "C" fn photo_output_readiness_callback_release(userdata: *mut c_void) {
    ArcContext::<PhotoOutputReadinessCallbackState>::release(userdata);
}

unsafe extern "C" fn photo_output_readiness_trampoline(
    userdata: *mut c_void,
    payload: *mut c_char,
) {
    let payload = match parse_json_and_free::<PhotoOutputReadinessPayload>(payload) {
        Ok(payload) => payload,
        Err(error) => {
            report_callback_error("photo_output_readiness_trampoline", error);
            return;
        }
    };
    let Some(state) = ArcContext::<PhotoOutputReadinessCallbackState>::get(userdata) else {
        return;
    };
    state.dispatch(
        "photo_output_readiness_trampoline",
        payload.capture_readiness,
    );
}

#[cfg(test)]
mod tests {
    use super::{validate_photo_settings, PhotoOutputInfo};
    use crate::device::CaptureFlashMode;
    use crate::error::AVCaptureError;
    use crate::photo::{PhotoQualityPrioritization, PhotoSettingsInfo};

    fn settings(
        flash_mode: Option<CaptureFlashMode>,
        photo_quality_prioritization: Option<PhotoQualityPrioritization>,
    ) -> PhotoSettingsInfo {
        PhotoSettingsInfo {
            unique_id: 1,
            processed_file_type: None,
            flash_mode,
            photo_quality_prioritization,
            used_for_capture: false,
        }
    }

    fn output(
        supported_flash_modes: Vec<CaptureFlashMode>,
        max_photo_quality_prioritization: Option<PhotoQualityPrioritization>,
    ) -> PhotoOutputInfo {
        PhotoOutputInfo {
            connection_count: 1,
            available_photo_codec_types: Vec::new(),
            available_photo_file_types: Vec::new(),
            available_photo_pixel_format_types: Vec::new(),
            available_raw_photo_pixel_format_types: None,
            supported_flash_modes,
            max_photo_dimensions: None,
            capture_readiness: None,
            max_photo_quality_prioritization,
            high_resolution_capture_enabled: false,
            responsive_capture_enabled: None,
            callback_installed: false,
        }
    }

    const fn is_invalid_argument(result: &Result<(), AVCaptureError>) -> bool {
        matches!(result, Err(AVCaptureError::InvalidArgument(_)))
    }

    #[test]
    fn flash_mode_must_be_supported_by_the_output() {
        let flashless = output(
            vec![CaptureFlashMode::Off],
            Some(PhotoQualityPrioritization::Balanced),
        );

        assert!(is_invalid_argument(&validate_photo_settings(
            &settings(Some(CaptureFlashMode::Auto), None),
            &flashless
        )));
        assert!(is_invalid_argument(&validate_photo_settings(
            &settings(Some(CaptureFlashMode::On), None),
            &flashless
        )));
        assert!(
            validate_photo_settings(&settings(Some(CaptureFlashMode::Off), None), &flashless)
                .is_ok()
        );

        let with_flash = output(
            vec![
                CaptureFlashMode::Off,
                CaptureFlashMode::On,
                CaptureFlashMode::Auto,
            ],
            None,
        );
        assert!(validate_photo_settings(
            &settings(Some(CaptureFlashMode::Auto), None),
            &with_flash
        )
        .is_ok());
    }

    #[test]
    fn flash_off_is_accepted_even_without_reported_modes() {
        assert!(validate_photo_settings(
            &settings(Some(CaptureFlashMode::Off), None),
            &output(Vec::new(), None)
        )
        .is_ok());
    }

    #[test]
    fn quality_prioritization_may_not_exceed_the_output_maximum() {
        let balanced = output(
            vec![CaptureFlashMode::Off],
            Some(PhotoQualityPrioritization::Balanced),
        );

        assert!(is_invalid_argument(&validate_photo_settings(
            &settings(None, Some(PhotoQualityPrioritization::Quality)),
            &balanced
        )));
        assert!(validate_photo_settings(
            &settings(None, Some(PhotoQualityPrioritization::Balanced)),
            &balanced
        )
        .is_ok());
        assert!(validate_photo_settings(
            &settings(None, Some(PhotoQualityPrioritization::Speed)),
            &balanced
        )
        .is_ok());
        assert!(validate_photo_settings(
            &settings(None, Some(PhotoQualityPrioritization::Quality)),
            &output(Vec::new(), Some(PhotoQualityPrioritization::Quality))
        )
        .is_ok());
    }

    #[test]
    fn settings_without_reported_values_are_not_rejected() {
        assert!(validate_photo_settings(&settings(None, None), &output(Vec::new(), None)).is_ok());
    }

    #[test]
    fn used_settings_are_rejected() {
        let mut used = settings(None, None);
        used.used_for_capture = true;

        assert!(is_invalid_argument(&validate_photo_settings(
            &used,
            &output(vec![CaptureFlashMode::Off], None)
        )));
    }
}
