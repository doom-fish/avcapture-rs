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
pub enum CaptureTimecodeSourceType {
    FrameCount,
    RealTimeClock,
    External,
    Unknown(String),
}

impl CaptureTimecodeSourceType {
    #[must_use]
    pub fn as_raw(&self) -> &str {
        match self {
            Self::FrameCount => "frameCount",
            Self::RealTimeClock => "realTimeClock",
            Self::External => "external",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
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
pub enum CaptureTimecodeGeneratorSynchronizationStatus {
    Unknown,
    SourceSelected,
    Synchronizing,
    Synchronized,
    TimedOut,
    SourceUnavailable,
    SourceUnsupported,
    NotRequired,
    UnknownValue(String),
}

impl CaptureTimecodeGeneratorSynchronizationStatus {
    #[must_use]
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
pub struct CaptureTimecodeSourceInfo {
    pub display_name: String,
    pub source_type: CaptureTimecodeSourceType,
    pub uuid: String,
}

impl CaptureTimecodeSourceInfo {
    #[must_use]
    pub fn source_type_raw(&self) -> &str {
        self.source_type.as_raw()
    }
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub user_bits: u32,
    #[serde(with = "cm_time_serde")]
    pub frame_duration: CMTime,
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
    pub fn source_type_raw(&self) -> &str {
        self.source_type.as_raw()
    }

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
pub struct CaptureTimecodeGeneratorInfo {
    pub available_source_count: usize,
    pub current_source: Option<CaptureTimecodeSourceInfo>,
    pub synchronization_timeout: f64,
    pub timecode_alignment_offset: f64,
    #[serde(with = "cm_time_serde")]
    pub timecode_frame_duration: CMTime,
    pub delegate_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureTimecodeGeneratorEvent {
    pub kind: String,
    pub timecode: Option<CaptureTimecode>,
    pub source: Option<CaptureTimecodeSourceInfo>,
    pub synchronization_status: Option<CaptureTimecodeGeneratorSynchronizationStatus>,
    #[serde(default)]
    pub available_sources: Vec<CaptureTimecodeSourceInfo>,
}

struct TimecodeDelegateCallbackState {
    callback: Box<dyn FnMut(CaptureTimecodeGeneratorEvent) + Send + 'static>,
}

#[derive(Debug)]
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
    pub const fn raw_ptr(&self) -> *mut c_void {
        self.ptr
    }

    #[must_use]
    pub fn is_available(&self) -> bool {
        !self.ptr.is_null()
    }
}

#[derive(Debug)]
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

    pub fn info(&self) -> Result<CaptureTimecodeSourceInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::timecode::av_capture_timecode_source_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    pub fn display_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.display_name)
    }

    pub fn source_type(&self) -> Result<CaptureTimecodeSourceType, AVCaptureError> {
        Ok(self.info()?.source_type)
    }

    pub fn source_type_raw(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.source_type_raw().to_owned())
    }

    pub fn uuid(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.uuid)
    }
}

#[derive(Debug)]
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

    pub fn info(&self) -> Result<CaptureTimecodeGeneratorInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::timecode::av_capture_timecode_generator_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

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

    pub fn synchronization_timeout(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.synchronization_timeout)
    }

    pub fn timecode_alignment_offset(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.timecode_alignment_offset)
    }

    pub fn timecode_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.timecode_frame_duration)
    }

    pub fn delegate_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.delegate_installed)
    }

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

    pub fn clear_delegate_handler(&self) {
        unsafe { ffi::timecode::av_capture_timecode_generator_clear_delegate_callback(self.ptr) };
    }
}

impl VideoDataOutput {
    pub fn timecode_generator() -> Result<CaptureTimecodeGenerator, AVCaptureError> {
        CaptureTimecodeGenerator::new()
    }

    pub fn frame_count_timecode_source() -> Result<CaptureTimecodeSource, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::timecode::av_capture_timecode_source_frame_count(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(CaptureTimecodeSource::from_raw(ptr))
    }

    pub fn real_time_clock_timecode_source() -> Result<CaptureTimecodeSource, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::timecode::av_capture_timecode_source_real_time_clock(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        Ok(CaptureTimecodeSource::from_raw(ptr))
    }

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
