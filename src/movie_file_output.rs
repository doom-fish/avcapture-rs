#![allow(clippy::missing_errors_doc, clippy::must_use_candidate, dead_code)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use apple_cf::cm::{CMSampleBuffer, CMTime};
use serde::Deserialize;

use crate::audio_data_output::AudioOutputSettings;
use crate::callback::{ArcContext, SerializedCallback};
use crate::error::{from_swift, report_callback_error, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, cstring, optional_json_cstring, parse_json_and_free};
use crate::output::CaptureOutputRef;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureMovieFileOutput` state.
pub struct MovieFileOutputInfo {
    /// The connection count reported by `AVCaptureMovieFileOutput`.
    pub connection_count: usize,
    /// The is recording reported by `AVCaptureMovieFileOutput`.
    pub is_recording: bool,
    /// The is recording paused reported by `AVCaptureMovieFileOutput`.
    pub is_recording_paused: bool,
    #[serde(rename = "outputFileURL", alias = "outputFileUrl")]
    /// The output file url reported by `AVCaptureMovieFileOutput`.
    pub output_file_url: Option<String>,
    #[serde(with = "cm_time_serde")]
    /// The recorded duration reported by `AVCaptureMovieFileOutput`.
    pub recorded_duration: CMTime,
    /// The recorded file size reported by `AVCaptureMovieFileOutput`.
    pub recorded_file_size: i64,
    #[serde(with = "cm_time_serde")]
    /// The max recorded duration reported by `AVCaptureMovieFileOutput`.
    pub max_recorded_duration: CMTime,
    /// The max recorded file size reported by `AVCaptureMovieFileOutput`.
    pub max_recorded_file_size: i64,
    /// The min free disk space limit reported by `AVCaptureMovieFileOutput`.
    pub min_free_disk_space_limit: i64,
    #[serde(with = "cm_time_serde")]
    /// The movie fragment interval reported by `AVCaptureMovieFileOutput`.
    pub movie_fragment_interval: CMTime,
    /// The metadata count reported by `AVCaptureMovieFileOutput`.
    pub metadata_count: usize,
    /// The spatial video capture enabled reported by `AVCaptureMovieFileOutput`.
    pub spatial_video_capture_enabled: Option<bool>,
    /// The callback installed reported by `AVCaptureMovieFileOutput`.
    pub callback_installed: bool,
    /// The sample buffer boundary callback installed reported by `AVCaptureMovieFileOutput`.
    pub sample_buffer_boundary_callback_installed: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureAudioFileOutput` state.
pub struct AudioFileOutputInfo {
    /// The connection count reported by `AVCaptureAudioFileOutput`.
    pub connection_count: usize,
    /// The is recording reported by `AVCaptureAudioFileOutput`.
    pub is_recording: bool,
    /// The is recording paused reported by `AVCaptureAudioFileOutput`.
    pub is_recording_paused: bool,
    #[serde(rename = "outputFileURL", alias = "outputFileUrl")]
    /// The output file url reported by `AVCaptureAudioFileOutput`.
    pub output_file_url: Option<String>,
    #[serde(with = "cm_time_serde")]
    /// The recorded duration reported by `AVCaptureAudioFileOutput`.
    pub recorded_duration: CMTime,
    /// The recorded file size reported by `AVCaptureAudioFileOutput`.
    pub recorded_file_size: i64,
    #[serde(with = "cm_time_serde")]
    /// The max recorded duration reported by `AVCaptureAudioFileOutput`.
    pub max_recorded_duration: CMTime,
    /// The max recorded file size reported by `AVCaptureAudioFileOutput`.
    pub max_recorded_file_size: i64,
    /// The min free disk space limit reported by `AVCaptureAudioFileOutput`.
    pub min_free_disk_space_limit: i64,
    /// The metadata count reported by `AVCaptureAudioFileOutput`.
    pub metadata_count: usize,
    /// The available output file types reported by `AVCaptureAudioFileOutput`.
    pub available_output_file_types: Vec<String>,
    /// The audio settings reported by `AVCaptureAudioFileOutput`.
    pub audio_settings: Option<AudioOutputSettings>,
    /// The callback installed reported by `AVCaptureAudioFileOutput`.
    pub callback_installed: bool,
    /// The sample buffer boundary callback installed reported by `AVCaptureAudioFileOutput`.
    pub sample_buffer_boundary_callback_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event-kind values produced by `AVCaptureFileOutputRecordingDelegate` callbacks.
pub enum MovieRecordingEventKind {
    /// Corresponds to the `Started` case.
    Started,
    /// Corresponds to the `Paused` case.
    Paused,
    /// Corresponds to the `Resumed` case.
    Resumed,
    /// Corresponds to the `WillFinish` case.
    WillFinish,
    /// Corresponds to the `Finished` case.
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Event-kind values produced by `AVCaptureFileOutputRecordingDelegate` callbacks.
pub enum AudioFileRecordingEventKind {
    /// Corresponds to the `Started` case.
    Started,
    /// Corresponds to the `Paused` case.
    Paused,
    /// Corresponds to the `Resumed` case.
    Resumed,
    /// Corresponds to the `WillFinish` case.
    WillFinish,
    /// Corresponds to the `Finished` case.
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Recording event payload derived from `AVCaptureFileOutputRecordingDelegate` callbacks.
pub struct MovieRecordingEvent {
    /// The callback kind reported by the underlying API.
    pub kind: MovieRecordingEventKind,
    #[serde(rename = "fileURL", alias = "fileUrl")]
    /// The file url reported by `AVCaptureFileOutputRecordingDelegate`.
    pub file_url: String,
    /// The error message, if any.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Recording event payload derived from `AVCaptureFileOutputRecordingDelegate` callbacks.
pub struct AudioFileRecordingEvent {
    /// The callback kind reported by the underlying API.
    pub kind: AudioFileRecordingEventKind,
    #[serde(rename = "fileURL", alias = "fileUrl")]
    /// The file url reported by `AVCaptureFileOutputRecordingDelegate`.
    pub file_url: String,
    /// The error message, if any.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
/// Policy used when a recording destination already exists.
pub enum RecordingOverwritePolicy {
    /// Fail without modifying the existing path.
    #[default]
    FailIfExists = 0,
    /// Atomically replace an existing regular file after recording completes.
    OverwriteRegularFile = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Options controlling recording destination handling.
pub struct RecordingOptions {
    /// The explicit overwrite policy.
    pub overwrite_policy: RecordingOverwritePolicy,
}

impl RecordingOptions {
    #[must_use]
    /// Creates options that may replace an existing regular file.
    pub const fn overwrite_regular_file() -> Self {
        Self {
            overwrite_policy: RecordingOverwritePolicy::OverwriteRegularFile,
        }
    }
}

type MovieRecordingCallbackState = SerializedCallback<MovieRecordingEvent>;
type AudioFileRecordingCallbackState = SerializedCallback<AudioFileRecordingEvent>;
type FileOutputSampleBufferCallbackState = SerializedCallback<CMSampleBuffer>;

/// Safe wrapper around `AVCaptureMovieFileOutput`.
#[derive(Debug)]
/// Wraps `AVCaptureMovieFileOutput`.
pub struct MovieFileOutput {
    pub(crate) ptr: *mut c_void,
}

/// Safe wrapper around `AVCaptureAudioFileOutput`.
#[derive(Debug)]
/// Wraps `AVCaptureAudioFileOutput`.
pub struct AudioFileOutput {
    pub(crate) ptr: *mut c_void,
}

impl Drop for MovieFileOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::movie_file_output::av_capture_movie_file_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl Drop for AudioFileOutput {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::movie_file_output::av_capture_audio_file_output_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureOutputRef for MovieFileOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl CaptureOutputRef for AudioFileOutput {
    fn output_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl crate::output::sealed::Sealed for MovieFileOutput {}
impl crate::output::sealed::Sealed for AudioFileOutput {}

impl MovieFileOutput {
    /// Creates a new `AVCaptureMovieFileOutput` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::movie_file_output::av_capture_movie_file_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureMovieFileOutput` state.
    pub fn info(&self) -> Result<MovieFileOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the connection count reported by `AVCaptureMovieFileOutput`.
    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    /// Returns whether `AVCaptureMovieFileOutput` is recording.
    pub fn is_recording(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording)
    }

    /// Returns whether `AVCaptureMovieFileOutput` is recording paused.
    pub fn is_recording_paused(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording_paused)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.output_file_url`.
    pub fn output_file_url(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.output_file_url)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.recorded_duration`.
    pub fn recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.recorded_duration)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.recorded_file_size`.
    pub fn recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.recorded_file_size)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.max_recorded_duration`.
    pub fn max_recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.max_recorded_duration)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.max_recorded_file_size`.
    pub fn max_recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.max_recorded_file_size)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.min_free_disk_space_limit`.
    pub fn min_free_disk_space_limit(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.min_free_disk_space_limit)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.movie_fragment_interval`.
    pub fn movie_fragment_interval(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.movie_fragment_interval)
    }

    /// Returns the metadata count reported by `AVCaptureMovieFileOutput`.
    pub fn metadata_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.metadata_count)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.spatial_video_capture_enabled`.
    pub fn spatial_video_capture_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.spatial_video_capture_enabled)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.callback_installed`.
    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    /// Corresponds to `AVCaptureMovieFileOutput.sample_buffer_boundary_callback_installed`.
    pub fn sample_buffer_boundary_callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.sample_buffer_boundary_callback_installed)
    }

    /// Starts recording with `AVCaptureMovieFileOutput`.
    pub fn start_recording<P: AsRef<Path>>(&self, output_path: P) -> Result<(), AVCaptureError> {
        self.start_recording_with_options(output_path, RecordingOptions::default())
    }

    /// Starts recording with explicit destination handling options.
    pub fn start_recording_with_options<P: AsRef<Path>>(
        &self,
        output_path: P,
        options: RecordingOptions,
    ) -> Result<(), AVCaptureError> {
        let output_path = output_path_bytes(output_path, "movie output path")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                output_path.len(),
                options.overwrite_policy as i32,
                None,
                ptr::null_mut(),
                None,
                None,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Starts recording with `AVCaptureMovieFileOutput` and installs a callback.
    pub fn start_recording_with_handler<P, F>(
        &self,
        output_path: P,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        P: AsRef<Path>,
        F: FnMut(MovieRecordingEvent) + Send + 'static,
    {
        self.start_recording_with_handler_and_options(
            output_path,
            RecordingOptions::default(),
            callback,
        )
    }

    /// Starts recording with a handler and explicit destination handling options.
    pub fn start_recording_with_handler_and_options<P, F>(
        &self,
        output_path: P,
        options: RecordingOptions,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        P: AsRef<Path>,
        F: FnMut(MovieRecordingEvent) + Send + 'static,
    {
        let output_path = output_path_bytes(output_path, "movie output path")?;
        let state = ArcContext::new(MovieRecordingCallbackState::new(callback));
        let userdata = state.as_ptr();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                output_path.len(),
                options.overwrite_policy as i32,
                Some(movie_recording_trampoline),
                userdata,
                Some(movie_recording_callback_retain),
                Some(movie_recording_callback_release),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the sample-buffer boundary handler on `AVCaptureMovieFileOutput`.
    pub fn set_sample_buffer_boundary_handler<F>(&self, callback: F) -> Result<(), AVCaptureError>
    where
        F: FnMut(CMSampleBuffer) + Send + 'static,
    {
        set_file_output_sample_buffer_boundary_handler(
            self.ptr,
            ffi::movie_file_output::av_capture_movie_file_output_set_sample_buffer_boundary_callback,
            callback,
        )
    }

    /// Clears the sample buffer boundary handler on `AVCaptureMovieFileOutput`.
    pub fn clear_sample_buffer_boundary_handler(&self) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_clear_sample_buffer_boundary_callback(
                self.ptr,
            );
        }
    }

    /// Corresponds to `AVCaptureMovieFileOutput.stop_recording`.
    pub fn stop_recording(&self) -> bool {
        unsafe { ffi::movie_file_output::av_capture_movie_file_output_stop_recording(self.ptr) }
    }

    /// Corresponds to `AVCaptureMovieFileOutput.pause_recording`.
    pub fn pause_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_movie_file_output_pause_recording(self.ptr) };
    }

    /// Corresponds to `AVCaptureMovieFileOutput.resume_recording`.
    pub fn resume_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_movie_file_output_resume_recording(self.ptr) };
    }

    /// Sets the max recorded duration on `AVCaptureMovieFileOutput`.
    pub fn set_max_recorded_duration(&self, duration: CMTime) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_max_recorded_duration(
                self.ptr, duration,
            );
        }
    }

    /// Sets the max recorded file size on `AVCaptureMovieFileOutput`.
    pub fn set_max_recorded_file_size(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_max_recorded_file_size(
                self.ptr, bytes,
            );
        }
    }

    /// Sets the min free disk space limit on `AVCaptureMovieFileOutput`.
    pub fn set_min_free_disk_space_limit(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_min_free_disk_space_limit(
                self.ptr, bytes,
            );
        }
    }

    /// Sets the movie fragment interval on `AVCaptureMovieFileOutput`.
    pub fn set_movie_fragment_interval(&self, interval: CMTime) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_movie_fragment_interval(
                self.ptr, interval,
            );
        }
    }

    /// Sets the spatial video capture enabled on `AVCaptureMovieFileOutput`.
    pub fn set_spatial_video_capture_enabled(&self, enabled: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_spatial_video_capture_enabled(
                self.ptr, enabled, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl AudioFileOutput {
    /// Creates a new `AVCaptureAudioFileOutput` wrapper.
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::movie_file_output::av_capture_audio_file_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

    /// Returns a snapshot of `AVCaptureAudioFileOutput` state.
    pub fn info(&self) -> Result<AudioFileOutputInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the connection count reported by `AVCaptureAudioFileOutput`.
    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    /// Returns whether `AVCaptureAudioFileOutput` is recording.
    pub fn is_recording(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording)
    }

    /// Returns whether `AVCaptureAudioFileOutput` is recording paused.
    pub fn is_recording_paused(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording_paused)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.output_file_url`.
    pub fn output_file_url(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.output_file_url)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.recorded_duration`.
    pub fn recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.recorded_duration)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.recorded_file_size`.
    pub fn recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.recorded_file_size)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.max_recorded_duration`.
    pub fn max_recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.max_recorded_duration)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.max_recorded_file_size`.
    pub fn max_recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.max_recorded_file_size)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.min_free_disk_space_limit`.
    pub fn min_free_disk_space_limit(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.min_free_disk_space_limit)
    }

    /// Returns the metadata count reported by `AVCaptureAudioFileOutput`.
    pub fn metadata_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.metadata_count)
    }

    /// Returns the available output file types reported by `AVCaptureAudioFileOutput`.
    pub fn available_output_file_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_output_file_types)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.audio_settings`.
    pub fn audio_settings(&self) -> Result<Option<AudioOutputSettings>, AVCaptureError> {
        Ok(self.info()?.audio_settings)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.callback_installed`.
    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    /// Corresponds to `AVCaptureAudioFileOutput.sample_buffer_boundary_callback_installed`.
    pub fn sample_buffer_boundary_callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.sample_buffer_boundary_callback_installed)
    }

    /// Sets the audio settings on `AVCaptureAudioFileOutput`.
    pub fn set_audio_settings(
        &self,
        settings: Option<&AudioOutputSettings>,
    ) -> Result<(), AVCaptureError> {
        let settings = optional_json_cstring(settings, "audio file output settings")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_set_audio_settings_json(
                self.ptr,
                settings.as_ref().map_or(ptr::null(), |json| json.as_ptr()),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Starts recording with `AVCaptureAudioFileOutput`.
    pub fn start_recording<P: AsRef<Path>>(
        &self,
        output_path: P,
        output_file_type: &str,
    ) -> Result<(), AVCaptureError> {
        self.start_recording_with_options(
            output_path,
            output_file_type,
            RecordingOptions::default(),
        )
    }

    /// Starts recording with explicit destination handling options.
    pub fn start_recording_with_options<P: AsRef<Path>>(
        &self,
        output_path: P,
        output_file_type: &str,
        options: RecordingOptions,
    ) -> Result<(), AVCaptureError> {
        let output_path = output_path_bytes(output_path, "audio file output path")?;
        let output_file_type = cstring(output_file_type, "audio file output type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                output_path.len(),
                output_file_type.as_ptr(),
                options.overwrite_policy as i32,
                None,
                ptr::null_mut(),
                None,
                None,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Starts recording with `AVCaptureAudioFileOutput` and installs a callback.
    pub fn start_recording_with_handler<P, F>(
        &self,
        output_path: P,
        output_file_type: &str,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        P: AsRef<Path>,
        F: FnMut(AudioFileRecordingEvent) + Send + 'static,
    {
        self.start_recording_with_handler_and_options(
            output_path,
            output_file_type,
            RecordingOptions::default(),
            callback,
        )
    }

    /// Starts recording with a handler and explicit destination handling options.
    pub fn start_recording_with_handler_and_options<P, F>(
        &self,
        output_path: P,
        output_file_type: &str,
        options: RecordingOptions,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        P: AsRef<Path>,
        F: FnMut(AudioFileRecordingEvent) + Send + 'static,
    {
        let output_path = output_path_bytes(output_path, "audio file output path")?;
        let output_file_type = cstring(output_file_type, "audio file output type")?;
        let state = ArcContext::new(AudioFileRecordingCallbackState::new(callback));
        let userdata = state.as_ptr();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                output_path.len(),
                output_file_type.as_ptr(),
                options.overwrite_policy as i32,
                Some(audio_file_recording_trampoline),
                userdata,
                Some(audio_file_recording_callback_retain),
                Some(audio_file_recording_callback_release),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the sample-buffer boundary handler on `AVCaptureAudioFileOutput`.
    pub fn set_sample_buffer_boundary_handler<F>(&self, callback: F) -> Result<(), AVCaptureError>
    where
        F: FnMut(CMSampleBuffer) + Send + 'static,
    {
        set_file_output_sample_buffer_boundary_handler(
            self.ptr,
            ffi::movie_file_output::av_capture_audio_file_output_set_sample_buffer_boundary_callback,
            callback,
        )
    }

    /// Clears the sample buffer boundary handler on `AVCaptureAudioFileOutput`.
    pub fn clear_sample_buffer_boundary_handler(&self) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_clear_sample_buffer_boundary_callback(
                self.ptr,
            );
        }
    }

    /// Corresponds to `AVCaptureAudioFileOutput.stop_recording`.
    pub fn stop_recording(&self) -> bool {
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_stop_recording(self.ptr) }
    }

    /// Corresponds to `AVCaptureAudioFileOutput.pause_recording`.
    pub fn pause_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_pause_recording(self.ptr) };
    }

    /// Corresponds to `AVCaptureAudioFileOutput.resume_recording`.
    pub fn resume_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_resume_recording(self.ptr) };
    }

    /// Sets the max recorded duration on `AVCaptureAudioFileOutput`.
    pub fn set_max_recorded_duration(&self, duration: CMTime) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_set_max_recorded_duration(
                self.ptr, duration,
            );
        }
    }

    /// Sets the max recorded file size on `AVCaptureAudioFileOutput`.
    pub fn set_max_recorded_file_size(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_set_max_recorded_file_size(
                self.ptr, bytes,
            );
        }
    }

    /// Sets the min free disk space limit on `AVCaptureAudioFileOutput`.
    pub fn set_min_free_disk_space_limit(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_set_min_free_disk_space_limit(
                self.ptr, bytes,
            );
        }
    }
}

type FileOutputSampleBufferBoundaryCallbackRegistrar = unsafe extern "C" fn(
    output: *mut c_void,
    callback: Option<ffi::AudioSampleCallback>,
    userdata: *mut c_void,
    retain_userdata: Option<ffi::RetainCallback>,
    drop_userdata: Option<ffi::DropCallback>,
    out_error_message: *mut *mut c_char,
) -> i32;

fn set_file_output_sample_buffer_boundary_handler<F>(
    ptr: *mut c_void,
    register_callback: FileOutputSampleBufferBoundaryCallbackRegistrar,
    callback: F,
) -> Result<(), AVCaptureError>
where
    F: FnMut(CMSampleBuffer) + Send + 'static,
{
    let state = ArcContext::new(FileOutputSampleBufferCallbackState::new(callback));
    let userdata = state.as_ptr();
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        register_callback(
            ptr,
            Some(file_output_sample_buffer_trampoline),
            userdata,
            Some(file_output_sample_buffer_callback_retain),
            Some(file_output_sample_buffer_callback_release),
            &mut err,
        )
    };
    if status != ffi::status::OK {
        return Err(unsafe { from_swift(status, err) });
    }
    Ok(())
}

pub(crate) fn output_path_bytes<P: AsRef<Path>>(
    output_path: P,
    what: &str,
) -> Result<Vec<u8>, AVCaptureError> {
    let output_path = output_path.as_ref();
    let output_path = if output_path.is_absolute() {
        output_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| {
                AVCaptureError::InvalidArgument(format!(
                    "failed to resolve relative {what}: {error}"
                ))
            })?
            .join(output_path)
    };
    let bytes = output_path.as_os_str().as_bytes().to_vec();
    if bytes.is_empty() {
        return Err(AVCaptureError::InvalidArgument(format!(
            "{what} must not be empty"
        )));
    }
    if bytes.contains(&0) {
        return Err(AVCaptureError::InvalidArgument(format!(
            "{what} contains a NUL byte"
        )));
    }
    Ok(bytes)
}

#[allow(unused_unsafe)]
unsafe extern "C" fn file_output_sample_buffer_trampoline(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
) {
    let Some(state) = ArcContext::<FileOutputSampleBufferCallbackState>::get(userdata) else {
        return;
    };
    let Some(sample_buffer) = (unsafe { CMSampleBuffer::from_raw(sample_buffer) }) else {
        return;
    };
    // User closures can panic; catch them here so the panic doesn't unwind
    // across the `extern "C"` boundary (which is UB).
    state.dispatch("file_output_sample_buffer_trampoline", sample_buffer);
}

unsafe extern "C" fn file_output_sample_buffer_callback_retain(userdata: *mut c_void) {
    ArcContext::<FileOutputSampleBufferCallbackState>::retain(userdata);
}

unsafe extern "C" fn file_output_sample_buffer_callback_release(userdata: *mut c_void) {
    ArcContext::<FileOutputSampleBufferCallbackState>::release(userdata);
}

unsafe extern "C" fn movie_recording_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = ArcContext::<MovieRecordingCallbackState>::get(userdata) else {
        return;
    };
    let event = match parse_json_and_free::<MovieRecordingEvent>(payload) {
        Ok(event) => event,
        Err(error) => {
            report_callback_error("movie_recording_trampoline", error);
            return;
        }
    };
    state.dispatch("movie_recording_trampoline", event);
}

unsafe extern "C" fn movie_recording_callback_retain(userdata: *mut c_void) {
    ArcContext::<MovieRecordingCallbackState>::retain(userdata);
}

unsafe extern "C" fn movie_recording_callback_release(userdata: *mut c_void) {
    ArcContext::<MovieRecordingCallbackState>::release(userdata);
}

unsafe extern "C" fn audio_file_recording_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = ArcContext::<AudioFileRecordingCallbackState>::get(userdata) else {
        return;
    };
    let event = match parse_json_and_free::<AudioFileRecordingEvent>(payload) {
        Ok(event) => event,
        Err(error) => {
            report_callback_error("audio_file_recording_trampoline", error);
            return;
        }
    };
    state.dispatch("audio_file_recording_trampoline", event);
}

unsafe extern "C" fn audio_file_recording_callback_retain(userdata: *mut c_void) {
    ArcContext::<AudioFileRecordingCallbackState>::retain(userdata);
}

unsafe extern "C" fn audio_file_recording_callback_release(userdata: *mut c_void) {
    ArcContext::<AudioFileRecordingCallbackState>::release(userdata);
}

#[cfg(test)]
mod tests {
    use super::{output_path_bytes, RecordingOverwritePolicy};
    use crate::ffi;
    use std::ffi::CStr;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};

    fn artifact_path(name: &str) -> PathBuf {
        std::env::current_dir()
            .expect("current directory")
            .join("target")
            .join("test-artifacts")
            .join(format!("{name}-{}", std::process::id()))
    }

    fn remove_file_if_present(path: &Path) {
        if fs::symlink_metadata(path).is_ok() {
            fs::remove_file(path).expect("remove test file or symlink");
        }
    }

    fn finalize_for_test(
        path: &Path,
        policy: RecordingOverwritePolicy,
        native_error_mode: i32,
    ) -> (i32, bool, String) {
        let path = output_path_bytes(path, "synthetic recording path").expect("path should encode");
        let mut had_error = false;
        let mut error = core::ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_recording_destination_finalize_for_testing(
                path.as_ptr(),
                path.len(),
                policy as i32,
                native_error_mode,
                &mut had_error,
                &mut error,
            )
        };
        let error = if error.is_null() {
            String::new()
        } else {
            let message = unsafe { CStr::from_ptr(error) }
                .to_string_lossy()
                .into_owned();
            unsafe { ffi::core::avc_string_free(error) };
            message
        };
        (status, had_error, error)
    }

    #[test]
    fn native_error_requires_explicit_success_before_staging_is_moved() {
        let directory = artifact_path("recording-native-error");
        fs::create_dir_all(&directory).expect("create test directory");
        let destination = directory.join("capture.mov");
        fs::write(&destination, b"original").expect("write destination sentinel");

        let (status, had_error, error) = finalize_for_test(
            &destination,
            RecordingOverwritePolicy::OverwriteRegularFile,
            1,
        );
        assert_eq!(status, ffi::status::OK, "{error}");
        assert!(had_error);
        assert_eq!(
            fs::read(&destination).expect("read preserved destination"),
            b"original"
        );
        assert!(!fs::read_dir(&directory)
            .expect("read test directory")
            .any(|entry| entry
                .expect("directory entry")
                .file_name()
                .to_string_lossy()
                .contains(".capture.mov.avcapture-")));

        let (status, had_error, error) = finalize_for_test(
            &destination,
            RecordingOverwritePolicy::OverwriteRegularFile,
            3,
        );
        assert_eq!(status, ffi::status::OK, "{error}");
        assert!(had_error);
        assert_eq!(
            fs::read(&destination).expect("read destination after unsuccessful error"),
            b"original"
        );

        let (status, had_error, error) = finalize_for_test(
            &destination,
            RecordingOverwritePolicy::OverwriteRegularFile,
            2,
        );
        assert_eq!(status, ffi::status::OK, "{error}");
        assert!(had_error);
        assert_eq!(
            fs::read(&destination).expect("read finalized destination"),
            b"staged"
        );

        fs::remove_file(&destination).expect("remove finalized destination");
        fs::remove_dir(&directory).expect("remove test directory");
    }

    #[test]
    fn symlinked_parent_is_allowed_but_destination_symlink_is_rejected() {
        let real_parent = artifact_path("recording-real-parent");
        let linked_parent = artifact_path("recording-linked-parent");
        remove_file_if_present(&linked_parent);
        fs::create_dir_all(&real_parent).expect("create real parent");
        symlink(&real_parent, &linked_parent).expect("create parent symlink");

        let linked_destination = linked_parent.join("capture.mov");
        let (status, had_error, error) = finalize_for_test(
            &linked_destination,
            RecordingOverwritePolicy::FailIfExists,
            0,
        );
        assert_eq!(status, ffi::status::OK, "{error}");
        assert!(!had_error);
        assert_eq!(
            fs::read(real_parent.join("capture.mov")).expect("read staged result"),
            b"staged"
        );

        fs::remove_file(real_parent.join("capture.mov")).expect("remove staged result");
        fs::remove_file(&linked_parent).expect("remove parent symlink");
        fs::remove_dir(&real_parent).expect("remove real parent");

        let destination_parent = artifact_path("recording-destination-symlink");
        fs::create_dir_all(&destination_parent).expect("create destination parent");
        let symlink_target = destination_parent.join("target.mov");
        let destination_symlink = destination_parent.join("capture.mov");
        fs::write(&symlink_target, b"target").expect("write symlink target");
        symlink(&symlink_target, &destination_symlink).expect("create destination symlink");

        let (status, _had_error, _error) = finalize_for_test(
            &destination_symlink,
            RecordingOverwritePolicy::OverwriteRegularFile,
            0,
        );
        assert_eq!(status, ffi::status::INVALID_ARGUMENT);
        assert!(fs::symlink_metadata(&destination_symlink)
            .expect("destination symlink metadata")
            .file_type()
            .is_symlink());
        assert_eq!(
            fs::read(&symlink_target).expect("read untouched symlink target"),
            b"target"
        );

        fs::remove_file(&destination_symlink).expect("remove destination symlink");
        fs::remove_file(&symlink_target).expect("remove symlink target");
        fs::remove_dir(&destination_parent).expect("remove destination parent");
    }
}
