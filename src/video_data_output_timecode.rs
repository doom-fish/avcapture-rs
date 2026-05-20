#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::ffi::CString;

use apple_cf::cm::CMTime;
use serde::{Deserialize, Serialize};

use super::VideoDataOutput;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, json_cstring, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
/// `AVCaptureTimecodeSource` values.
pub enum CaptureTimecodeSourceType {
    /// Corresponds to the `FrameCount` case.
    FrameCount,
    /// Corresponds to the `RealTimeClock` case.
    RealTimeClock,
    /// Corresponds to the `External` case.
    External,
    /// A value not recognized by this crate.
    Unknown(String),
}

impl CaptureTimecodeSourceType {
    #[must_use]
    /// Returns the raw SDK value for `AVCaptureTimecodeSource`.
    pub fn as_raw(&self) -> &str {
        match self {
            Self::FrameCount => "frameCount",
            Self::RealTimeClock => "realTimeClock",
            Self::External => "external",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
    /// Wraps an existing `AVCaptureTimecodeSource` pointer.
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "frameCount" => Self::FrameCount,
            "realTimeClock" => Self::RealTimeClock,
            "external" => Self::External,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl From<String> for CaptureTimecodeSourceType {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<CaptureTimecodeSourceType> for String {
    fn from(value: CaptureTimecodeSourceType) -> Self {
        value.as_raw().to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
/// `AVCaptureTimecodeGenerator` values.
pub enum CaptureTimecodeGeneratorSynchronizationStatus {
    /// A value not recognized by this crate.
    Unknown,
    /// Corresponds to the `SourceSelected` case.
    SourceSelected,
    /// Corresponds to the `Synchronizing` case.
    Synchronizing,
    /// Corresponds to the `Synchronized` case.
    Synchronized,
    /// Corresponds to the `TimedOut` case.
    TimedOut,
    /// Corresponds to the `SourceUnavailable` case.
    SourceUnavailable,
    /// Corresponds to the `SourceUnsupported` case.
    SourceUnsupported,
    /// Corresponds to the `NotRequired` case.
    NotRequired,
    /// A value not recognized by this crate.
    UnknownValue(String),
}

impl CaptureTimecodeGeneratorSynchronizationStatus {
    #[must_use]
    /// Returns the raw SDK value for `AVCaptureTimecodeGenerator`.
    pub fn as_raw(&self) -> &str {
        match self {
            Self::Unknown => "unknown",
            Self::SourceSelected => "sourceSelected",
            Self::Synchronizing => "synchronizing",
            Self::Synchronized => "synchronized",
            Self::TimedOut => "timedOut",
            Self::SourceUnavailable => "sourceUnavailable",
            Self::SourceUnsupported => "sourceUnsupported",
            Self::NotRequired => "notRequired",
            Self::UnknownValue(raw) => raw.as_str(),
        }
    }

    #[must_use]
    /// Wraps an existing `AVCaptureTimecodeGenerator` pointer.
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "unknown" => Self::Unknown,
            "sourceSelected" => Self::SourceSelected,
            "synchronizing" => Self::Synchronizing,
            "synchronized" => Self::Synchronized,
            "timedOut" => Self::TimedOut,
            "sourceUnavailable" => Self::SourceUnavailable,
            "sourceUnsupported" => Self::SourceUnsupported,
            "notRequired" => Self::NotRequired,
            other => Self::UnknownValue(other.to_owned()),
        }
    }
}

impl From<String> for CaptureTimecodeGeneratorSynchronizationStatus {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<CaptureTimecodeGeneratorSynchronizationStatus> for String {
    fn from(value: CaptureTimecodeGeneratorSynchronizationStatus) -> Self {
        value.as_raw().to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureTimecodeSource` state.
pub struct CaptureTimecodeSourceInfo {
    /// The display name reported by `AVCaptureTimecodeSource`.
    pub display_name: String,
    /// The source type reported by `AVCaptureTimecodeSource`.
    pub source_type: CaptureTimecodeSourceType,
    /// The uuid reported by `AVCaptureTimecodeSource`.
    pub uuid: String,
}

impl CaptureTimecodeSourceInfo {
    #[must_use]
    /// Returns the raw source-type string.
    pub fn source_type_raw(&self) -> &str {
        self.source_type.as_raw()
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `AVCaptureTimecode`.
pub struct CaptureTimecode {
    /// The hours reported by `AVCaptureTimecode`.
    pub hours: u8,
    /// The minutes reported by `AVCaptureTimecode`.
    pub minutes: u8,
    /// The seconds reported by `AVCaptureTimecode`.
    pub seconds: u8,
    /// The frames reported by `AVCaptureTimecode`.
    pub frames: u8,
    /// The user bits reported by `AVCaptureTimecode`.
    pub user_bits: u32,
    #[serde(with = "cm_time_serde")]
    /// The frame duration reported by `AVCaptureTimecode`.
    pub frame_duration: CMTime,
    /// The source type reported by `AVCaptureTimecode`.
    pub source_type: CaptureTimecodeSourceType,
}

impl CaptureTimecode {
    fn new(
        hours: u8,
        minutes: u8,
        seconds: u8,
        frames: u8,
        user_bits: u32,
        frame_duration: CMTime,
        source_type: impl AsRef<str>,
    ) -> Result<Self, AVCaptureError> {
        if hours >= 24 {
            return Err(AVCaptureError::InvalidArgument(
                "timecode hours must be less than 24".to_owned(),
            ));
        }
        if minutes >= 60 {
            return Err(AVCaptureError::InvalidArgument(
                "timecode minutes must be less than 60".to_owned(),
            ));
        }
        if seconds >= 60 {
            return Err(AVCaptureError::InvalidArgument(
                "timecode seconds must be less than 60".to_owned(),
            ));
        }
        Ok(Self {
            hours,
            minutes,
            seconds,
            frames,
            user_bits,
            frame_duration,
            source_type: CaptureTimecodeSourceType::from_raw(source_type.as_ref()),
        })
    }

    #[must_use]
    /// Returns the raw source-type string.
    pub fn source_type_raw(&self) -> &str {
        self.source_type.as_raw()
    }

    /// Corresponds to `AVCaptureTimecode.advanced_by_frames`.
    pub fn advanced_by_frames(&self, frames_to_add: i64) -> Result<Self, AVCaptureError> {
        let timecode = json_cstring(self, "capture timecode")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::timecode::av_capture_timecode_advanced_by_frames_json(
                timecode.as_ptr(),
                frames_to_add,
                &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureTimecode.create_metadata_sample_buffer_associated_with_presentation_time_stamp`.
    pub fn create_metadata_sample_buffer_associated_with_presentation_time_stamp(
        &self,
        presentation_time_stamp: CMTime,
    ) -> Result<TimecodeMetadataSampleBuffer, AVCaptureError> {
        let timecode = json_cstring(self, "capture timecode")?;
        let presentation_time_stamp =
            cm_time_json(presentation_time_stamp, "presentation time stamp")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::timecode::av_capture_timecode_create_metadata_sample_buffer_associated_with_presentation_time_stamp(
                timecode.as_ptr(),
                presentation_time_stamp.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(TimecodeMetadataSampleBuffer { ptr })
    }

    /// Corresponds to `AVCaptureTimecode.create_metadata_sample_buffer_for_duration`.
    pub fn create_metadata_sample_buffer_for_duration(
        &self,
        duration: CMTime,
    ) -> Result<TimecodeMetadataSampleBuffer, AVCaptureError> {
        let timecode = json_cstring(self, "capture timecode")?;
        let duration = cm_time_json(duration, "timecode duration")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::timecode::av_capture_timecode_create_metadata_sample_buffer_for_duration(
                timecode.as_ptr(),
                duration.as_ptr(),
                &mut err,
            )
        };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(TimecodeMetadataSampleBuffer { ptr })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureTimecodeGenerator` state.
pub struct CaptureTimecodeGeneratorInfo {
    /// The available source count reported by `AVCaptureTimecodeGenerator`.
    pub available_source_count: usize,
    /// The current source reported by `AVCaptureTimecodeGenerator`.
    pub current_source: Option<CaptureTimecodeSourceInfo>,
    /// The synchronization timeout reported by `AVCaptureTimecodeGenerator`.
    pub synchronization_timeout: f64,
    /// The timecode alignment offset reported by `AVCaptureTimecodeGenerator`.
    pub timecode_alignment_offset: f64,
    #[serde(with = "cm_time_serde")]
    /// The timecode frame duration reported by `AVCaptureTimecodeGenerator`.
    pub timecode_frame_duration: CMTime,
    /// The delegate installed reported by `AVCaptureTimecodeGenerator`.
    pub delegate_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event payload derived from `AVCaptureTimecodeGeneratorDelegate` callbacks.
pub struct CaptureTimecodeGeneratorEvent {
    /// The callback kind reported by the underlying API.
    pub kind: String,
    /// The timecode reported by `AVCaptureTimecodeGeneratorDelegate`.
    pub timecode: Option<CaptureTimecode>,
    /// The source reported by `AVCaptureTimecodeGeneratorDelegate`.
    pub source: Option<CaptureTimecodeSourceInfo>,
    /// The synchronization status reported by `AVCaptureTimecodeGeneratorDelegate`.
    pub synchronization_status: Option<CaptureTimecodeGeneratorSynchronizationStatus>,
    #[serde(default)]
    /// The available sources reported by `AVCaptureTimecodeGeneratorDelegate`.
    pub available_sources: Vec<CaptureTimecodeSourceInfo>,
}

struct TimecodeDelegateCallbackState {
    callback: Box<dyn FnMut(CaptureTimecodeGeneratorEvent) + Send + 'static>,
}

#[derive(Debug)]
/// Wraps `AVCaptureTimecode`.
pub struct TimecodeMetadataSampleBuffer {
    ptr: *mut c_void,
}

impl Drop for TimecodeMetadataSampleBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::timecode::av_capture_timecode_metadata_sample_buffer_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl TimecodeMetadataSampleBuffer {
    /// Returns the raw pointer backing `AVCaptureTimecode`.
    pub const fn raw_ptr(&self) -> *mut c_void {
        self.ptr
    }

    #[must_use]
    /// Returns whether `AVCaptureTimecode` is available.
    pub fn is_available(&self) -> bool {
        !self.ptr.is_null()
    }
}

#[derive(Debug)]
/// Wraps `AVCaptureTimecodeSource`.
pub struct CaptureTimecodeSource {
    ptr: *mut c_void,
}

impl Drop for CaptureTimecodeSource {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::timecode::av_capture_timecode_source_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureTimecodeSource {
    const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureTimecodeSource` state.
    pub fn info(&self) -> Result<CaptureTimecodeSourceInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::timecode::av_capture_timecode_source_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureTimecodeSource.display_name`.
    pub fn display_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.display_name)
    }

    /// Returns the source type reported by the underlying API.
    pub fn source_type(&self) -> Result<CaptureTimecodeSourceType, AVCaptureError> {
        Ok(self.info()?.source_type)
    }

    /// Returns the raw source-type string.
    pub fn source_type_raw(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.source_type_raw().to_owned())
    }

    /// Corresponds to `AVCaptureTimecodeSource.uuid`.
    pub fn uuid(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.uuid)
    }
}

#[derive(Debug)]
/// Wraps `AVCaptureTimecodeGenerator`.
pub struct CaptureTimecodeGenerator {
    ptr: *mut c_void,
}

impl Drop for CaptureTimecodeGenerator {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::timecode::av_capture_timecode_generator_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureTimecodeGenerator {
    fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::timecode::av_capture_timecode_generator_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureTimecodeGenerator` state.
    pub fn info(&self) -> Result<CaptureTimecodeGeneratorInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::timecode::av_capture_timecode_generator_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the available sources reported by `AVCaptureTimecodeGenerator`.
    pub fn available_sources(&self) -> Result<Vec<CaptureTimecodeSource>, AVCaptureError> {
        let count = unsafe {
            ffi::timecode::av_capture_timecode_generator_available_sources_count(self.ptr)
        };
        let mut sources = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::timecode::av_capture_timecode_generator_available_source_at_index(
                    self.ptr, index, &mut err,
                )
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
            }
            sources.push(CaptureTimecodeSource::from_raw(ptr));
        }
        Ok(sources)
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.current_source`.
    pub fn current_source(&self) -> Result<Option<CaptureTimecodeSource>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::timecode::av_capture_timecode_generator_current_source(self.ptr, &mut err)
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(Some(CaptureTimecodeSource::from_raw(ptr)))
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.synchronization_timeout`.
    pub fn synchronization_timeout(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.synchronization_timeout)
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.timecode_alignment_offset`.
    pub fn timecode_alignment_offset(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.timecode_alignment_offset)
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.timecode_frame_duration`.
    pub fn timecode_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.timecode_frame_duration)
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.delegate_installed`.
    pub fn delegate_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.delegate_installed)
    }

    /// Sets the synchronization timeout on `AVCaptureTimecodeGenerator`.
    pub fn set_synchronization_timeout(
        &self,
        synchronization_timeout: f64,
    ) -> Result<(), AVCaptureError> {
        if !synchronization_timeout.is_finite() || synchronization_timeout < 0.0 {
            return Err(AVCaptureError::InvalidArgument(
                "timecode synchronization timeout must be finite and non-negative".to_owned(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::timecode::av_capture_timecode_generator_set_synchronization_timeout(
                self.ptr,
                synchronization_timeout,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the timecode alignment offset on `AVCaptureTimecodeGenerator`.
    pub fn set_timecode_alignment_offset(
        &self,
        timecode_alignment_offset: f64,
    ) -> Result<(), AVCaptureError> {
        if !timecode_alignment_offset.is_finite() {
            return Err(AVCaptureError::InvalidArgument(
                "timecode alignment offset must be finite".to_owned(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::timecode::av_capture_timecode_generator_set_timecode_alignment_offset(
                self.ptr,
                timecode_alignment_offset,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the timecode frame duration on `AVCaptureTimecodeGenerator`.
    pub fn set_timecode_frame_duration(
        &self,
        frame_duration: CMTime,
    ) -> Result<(), AVCaptureError> {
        let frame_duration = cm_time_json(frame_duration, "timecode frame duration")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::timecode::av_capture_timecode_generator_set_timecode_frame_duration_json(
                self.ptr,
                frame_duration.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.start_synchronization`.
    pub fn start_synchronization(
        &self,
        source: &CaptureTimecodeSource,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::timecode::av_capture_timecode_generator_start_synchronization(
                self.ptr, source.ptr, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCaptureTimecodeGenerator.generate_initial_timecode`.
    pub fn generate_initial_timecode(&self) -> Result<CaptureTimecode, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::timecode::av_capture_timecode_generator_generate_initial_timecode_json(
                self.ptr, &mut err,
            )
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Sets the delegate handler on `AVCaptureTimecodeGenerator`.
    pub fn set_delegate_handler<F>(
        &self,
        queue_label: Option<&str>,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        F: FnMut(CaptureTimecodeGeneratorEvent) + Send + 'static,
    {
        let queue_label = queue_label.unwrap_or("avcapture-timecode-generator");
        let queue_label = CString::new(queue_label).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("queue label contains NUL byte: {error}"))
        })?;
        let state = Box::new(TimecodeDelegateCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::timecode::av_capture_timecode_generator_set_delegate_callback(
                self.ptr,
                queue_label.as_ptr(),
                Some(timecode_delegate_trampoline),
                userdata,
                Some(timecode_delegate_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { timecode_delegate_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Clears the delegate handler on `AVCaptureTimecodeGenerator`.
    pub fn clear_delegate_handler(&self) {
        unsafe { ffi::timecode::av_capture_timecode_generator_clear_delegate_callback(self.ptr) };
    }
}

impl VideoDataOutput {
    /// Creates a new `AVCaptureTimecodeGenerator` wrapper.
    pub fn timecode_generator() -> Result<CaptureTimecodeGenerator, AVCaptureError> {
        CaptureTimecodeGenerator::new()
    }

    /// Returns the frame-count `AVCaptureTimecodeSource`.
    pub fn frame_count_timecode_source() -> Result<CaptureTimecodeSource, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::timecode::av_capture_timecode_source_frame_count(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(CaptureTimecodeSource::from_raw(ptr))
    }

    /// Returns the real-time-clock `AVCaptureTimecodeSource`.
    pub fn real_time_clock_timecode_source() -> Result<CaptureTimecodeSource, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::timecode::av_capture_timecode_source_real_time_clock(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(CaptureTimecodeSource::from_raw(ptr))
    }

    /// Creates an `AVCaptureTimecode` value.
    pub fn timecode(
        hours: u8,
        minutes: u8,
        seconds: u8,
        frames: u8,
        user_bits: u32,
        frame_duration: CMTime,
        source_type: impl AsRef<str>,
    ) -> Result<CaptureTimecode, AVCaptureError> {
        CaptureTimecode::new(
            hours,
            minutes,
            seconds,
            frames,
            user_bits,
            frame_duration,
            source_type,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct CMTimePayload {
    value: i64,
    timescale: i32,
    flags: u32,
    epoch: i64,
}

impl From<CMTime> for CMTimePayload {
    fn from(value: CMTime) -> Self {
        Self {
            value: value.value,
            timescale: value.timescale,
            flags: value.flags,
            epoch: value.epoch,
        }
    }
}

fn cm_time_json(time: CMTime, what: &str) -> Result<CString, AVCaptureError> {
    json_cstring(&CMTimePayload::from(time), what)
}

unsafe extern "C" fn timecode_delegate_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<TimecodeDelegateCallbackState>().as_mut() else {
        return;
    };
    let Ok(event) = parse_json_and_free::<CaptureTimecodeGeneratorEvent>(payload) else {
        return;
    };
    (state.callback)(event);
}

unsafe extern "C" fn timecode_delegate_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(
        userdata.cast::<TimecodeDelegateCallbackState>(),
    ));
}

#[cfg(test)]
mod tests {
    use core::ptr;

    use apple_cf::cm::CMTime;

    use super::{
        CaptureTimecode, CaptureTimecodeGenerator,
        CaptureTimecodeGeneratorSynchronizationStatus, CaptureTimecodeSourceInfo,
        CaptureTimecodeSourceType,
    };
    use crate::error::AVCaptureError;

    fn sample_frame_duration() -> CMTime {
        CMTime {
            value: 1001,
            timescale: 30_000,
            flags: 1,
            epoch: 0,
        }
    }

    #[test]
    fn source_type_round_trips_known_and_unknown_values() {
        assert_eq!(
            CaptureTimecodeSourceType::from_raw("frameCount"),
            CaptureTimecodeSourceType::FrameCount
        );
        assert_eq!(CaptureTimecodeSourceType::External.as_raw(), "external");

        let custom = CaptureTimecodeSourceType::from_raw("customSource");
        assert_eq!(custom, CaptureTimecodeSourceType::Unknown("customSource".to_owned()));
        assert_eq!(custom.as_raw(), "customSource");
    }

    #[test]
    fn synchronization_status_round_trips_known_and_unknown_values() {
        assert_eq!(
            CaptureTimecodeGeneratorSynchronizationStatus::from_raw("synchronized"),
            CaptureTimecodeGeneratorSynchronizationStatus::Synchronized
        );
        assert_eq!(
            CaptureTimecodeGeneratorSynchronizationStatus::TimedOut.as_raw(),
            "timedOut"
        );

        let custom = CaptureTimecodeGeneratorSynchronizationStatus::from_raw("delayed");
        assert_eq!(
            custom,
            CaptureTimecodeGeneratorSynchronizationStatus::UnknownValue("delayed".to_owned())
        );
        assert_eq!(custom.as_raw(), "delayed");
    }

    #[test]
    fn timecode_new_preserves_frame_duration_and_source_type() {
        let frame_duration = sample_frame_duration();
        let timecode =
            CaptureTimecode::new(1, 2, 3, 4, 0xAABB_CCDD, frame_duration, "realTimeClock")
                .expect("valid timecode should build");

        assert_eq!(timecode.hours, 1);
        assert_eq!(timecode.minutes, 2);
        assert_eq!(timecode.seconds, 3);
        assert_eq!(timecode.frames, 4);
        assert_eq!(timecode.user_bits, 0xAABB_CCDD);
        assert_eq!(timecode.frame_duration, frame_duration);
        assert_eq!(timecode.source_type, CaptureTimecodeSourceType::RealTimeClock);
        assert_eq!(timecode.source_type_raw(), "realTimeClock");
    }

    #[test]
    fn timecode_new_rejects_invalid_clock_fields() {
        assert!(matches!(
            CaptureTimecode::new(24, 0, 0, 0, 0, sample_frame_duration(), "frameCount"),
            Err(AVCaptureError::InvalidArgument(message))
                if message == "timecode hours must be less than 24"
        ));
        assert!(matches!(
            CaptureTimecode::new(0, 60, 0, 0, 0, sample_frame_duration(), "frameCount"),
            Err(AVCaptureError::InvalidArgument(message))
                if message == "timecode minutes must be less than 60"
        ));
        assert!(matches!(
            CaptureTimecode::new(0, 0, 60, 0, 0, sample_frame_duration(), "frameCount"),
            Err(AVCaptureError::InvalidArgument(message))
                if message == "timecode seconds must be less than 60"
        ));
    }

    #[test]
    fn timecode_deserialization_preserves_frame_duration() {
        let timecode: CaptureTimecode = serde_json::from_str(
            r#"{
                "hours": 5,
                "minutes": 6,
                "seconds": 7,
                "frames": 8,
                "userBits": 9,
                "frameDuration": {
                    "value": 1001,
                    "timescale": 30000,
                    "flags": 1,
                    "epoch": 0
                },
                "sourceType": "external"
            }"#,
        )
        .expect("timecode JSON should decode");

        assert_eq!(timecode.frame_duration, sample_frame_duration());
        assert_eq!(timecode.source_type, CaptureTimecodeSourceType::External);
    }

    #[test]
    fn source_info_exposes_raw_source_type() {
        let info = CaptureTimecodeSourceInfo {
            display_name: "Frame Count".to_owned(),
            source_type: CaptureTimecodeSourceType::FrameCount,
            uuid: "test-uuid".to_owned(),
        };

        assert_eq!(info.source_type_raw(), "frameCount");
    }

    #[test]
    fn generator_validation_rejects_invalid_numeric_inputs() {
        let generator = CaptureTimecodeGenerator { ptr: ptr::null_mut() };

        assert!(matches!(
            generator.set_synchronization_timeout(-0.1),
            Err(AVCaptureError::InvalidArgument(message))
                if message == "timecode synchronization timeout must be finite and non-negative"
        ));
        assert!(matches!(
            generator.set_synchronization_timeout(f64::INFINITY),
            Err(AVCaptureError::InvalidArgument(message))
                if message == "timecode synchronization timeout must be finite and non-negative"
        ));
        assert!(matches!(
            generator.set_timecode_alignment_offset(f64::NAN),
            Err(AVCaptureError::InvalidArgument(message))
                if message == "timecode alignment offset must be finite"
        ));
    }
}
