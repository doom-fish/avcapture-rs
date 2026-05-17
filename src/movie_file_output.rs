#![allow(clippy::missing_errors_doc, clippy::must_use_candidate, dead_code)]

use core::ffi::{c_char, c_void};
use core::ptr;
use std::path::Path;

use apple_cf::cm::{CMSampleBuffer, CMTime};
use serde::Deserialize;

use crate::audio_data_output::AudioOutputSettings;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, cstring, optional_json_cstring, parse_json_and_free};
use crate::output::CaptureOutputRef;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieFileOutputInfo {
    pub connection_count: usize,
    pub is_recording: bool,
    pub is_recording_paused: bool,
    pub output_file_url: Option<String>,
    #[serde(with = "cm_time_serde")]
    pub recorded_duration: CMTime,
    pub recorded_file_size: i64,
    #[serde(with = "cm_time_serde")]
    pub max_recorded_duration: CMTime,
    pub max_recorded_file_size: i64,
    pub min_free_disk_space_limit: i64,
    #[serde(with = "cm_time_serde")]
    pub movie_fragment_interval: CMTime,
    pub metadata_count: usize,
    pub spatial_video_capture_enabled: Option<bool>,
    pub callback_installed: bool,
    pub sample_buffer_boundary_callback_installed: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFileOutputInfo {
    pub connection_count: usize,
    pub is_recording: bool,
    pub is_recording_paused: bool,
    pub output_file_url: Option<String>,
    #[serde(with = "cm_time_serde")]
    pub recorded_duration: CMTime,
    pub recorded_file_size: i64,
    #[serde(with = "cm_time_serde")]
    pub max_recorded_duration: CMTime,
    pub max_recorded_file_size: i64,
    pub min_free_disk_space_limit: i64,
    pub metadata_count: usize,
    pub available_output_file_types: Vec<String>,
    pub audio_settings: Option<AudioOutputSettings>,
    pub callback_installed: bool,
    pub sample_buffer_boundary_callback_installed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MovieRecordingEventKind {
    Started,
    Paused,
    Resumed,
    WillFinish,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioFileRecordingEventKind {
    Started,
    Paused,
    Resumed,
    WillFinish,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieRecordingEvent {
    pub kind: MovieRecordingEventKind,
    pub file_url: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioFileRecordingEvent {
    pub kind: AudioFileRecordingEventKind,
    pub file_url: String,
    pub error: Option<String>,
}

struct MovieRecordingCallbackState {
    callback: Box<dyn FnMut(MovieRecordingEvent) + Send + 'static>,
}

struct AudioFileRecordingCallbackState {
    callback: Box<dyn FnMut(AudioFileRecordingEvent) + Send + 'static>,
}

struct FileOutputSampleBufferCallbackState {
    callback: Box<dyn FnMut(CMSampleBuffer) + Send + 'static>,
}

/// Safe wrapper around `AVCaptureMovieFileOutput`.
pub struct MovieFileOutput {
    pub(crate) ptr: *mut c_void,
}

/// Safe wrapper around `AVCaptureAudioFileOutput`.
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

impl MovieFileOutput {
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::movie_file_output::av_capture_movie_file_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

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

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn is_recording(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording)
    }

    pub fn is_recording_paused(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording_paused)
    }

    pub fn output_file_url(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.output_file_url)
    }

    pub fn recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.recorded_duration)
    }

    pub fn recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.recorded_file_size)
    }

    pub fn max_recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.max_recorded_duration)
    }

    pub fn max_recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.max_recorded_file_size)
    }

    pub fn min_free_disk_space_limit(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.min_free_disk_space_limit)
    }

    pub fn movie_fragment_interval(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.movie_fragment_interval)
    }

    pub fn metadata_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.metadata_count)
    }

    pub fn spatial_video_capture_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.spatial_video_capture_enabled)
    }

    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    pub fn sample_buffer_boundary_callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.sample_buffer_boundary_callback_installed)
    }

    pub fn start_recording<P: AsRef<Path>>(&self, output_path: P) -> Result<(), AVCaptureError> {
        let output_path = output_path_cstring(output_path, "movie output path")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
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

    pub fn start_recording_with_handler<P, F>(
        &self,
        output_path: P,
        callback: F,
    ) -> Result<(), AVCaptureError>
    where
        P: AsRef<Path>,
        F: FnMut(MovieRecordingEvent) + Send + 'static,
    {
        let output_path = output_path_cstring(output_path, "movie output path")?;
        let state = Box::new(MovieRecordingCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                Some(movie_recording_trampoline),
                userdata,
                Some(movie_recording_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { movie_recording_callback_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

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

    pub fn clear_sample_buffer_boundary_handler(&self) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_clear_sample_buffer_boundary_callback(
                self.ptr,
            );
        }
    }

    pub fn stop_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_movie_file_output_stop_recording(self.ptr) };
    }

    pub fn pause_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_movie_file_output_pause_recording(self.ptr) };
    }

    pub fn resume_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_movie_file_output_resume_recording(self.ptr) };
    }

    pub fn set_max_recorded_duration(&self, duration: CMTime) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_max_recorded_duration(
                self.ptr, duration,
            );
        }
    }

    pub fn set_max_recorded_file_size(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_max_recorded_file_size(
                self.ptr, bytes,
            );
        }
    }

    pub fn set_min_free_disk_space_limit(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_min_free_disk_space_limit(
                self.ptr, bytes,
            );
        }
    }

    pub fn set_movie_fragment_interval(&self, interval: CMTime) {
        unsafe {
            ffi::movie_file_output::av_capture_movie_file_output_set_movie_fragment_interval(
                self.ptr, interval,
            );
        }
    }

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
    pub fn new() -> Result<Self, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::movie_file_output::av_capture_audio_file_output_create(&mut err) };
        if ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self { ptr })
    }

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

    pub fn connection_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.connection_count)
    }

    pub fn is_recording(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording)
    }

    pub fn is_recording_paused(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.is_recording_paused)
    }

    pub fn output_file_url(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.info()?.output_file_url)
    }

    pub fn recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.recorded_duration)
    }

    pub fn recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.recorded_file_size)
    }

    pub fn max_recorded_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.max_recorded_duration)
    }

    pub fn max_recorded_file_size(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.max_recorded_file_size)
    }

    pub fn min_free_disk_space_limit(&self) -> Result<i64, AVCaptureError> {
        Ok(self.info()?.min_free_disk_space_limit)
    }

    pub fn metadata_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.metadata_count)
    }

    pub fn available_output_file_types(&self) -> Result<Vec<String>, AVCaptureError> {
        Ok(self.info()?.available_output_file_types)
    }

    pub fn audio_settings(&self) -> Result<Option<AudioOutputSettings>, AVCaptureError> {
        Ok(self.info()?.audio_settings)
    }

    pub fn callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.callback_installed)
    }

    pub fn sample_buffer_boundary_callback_installed(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.sample_buffer_boundary_callback_installed)
    }

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

    pub fn start_recording<P: AsRef<Path>>(
        &self,
        output_path: P,
        output_file_type: &str,
    ) -> Result<(), AVCaptureError> {
        let output_path = output_path_cstring(output_path, "audio file output path")?;
        let output_file_type = cstring(output_file_type, "audio file output type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                output_file_type.as_ptr(),
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
        let output_path = output_path_cstring(output_path, "audio file output path")?;
        let output_file_type = cstring(output_file_type, "audio file output type")?;
        let state = Box::new(AudioFileRecordingCallbackState {
            callback: Box::new(callback),
        });
        let userdata = Box::into_raw(state).cast::<c_void>();
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_start_recording(
                self.ptr,
                output_path.as_ptr(),
                output_file_type.as_ptr(),
                Some(audio_file_recording_trampoline),
                userdata,
                Some(audio_file_recording_callback_drop),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            unsafe { audio_file_recording_callback_drop(userdata) };
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

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

    pub fn clear_sample_buffer_boundary_handler(&self) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_clear_sample_buffer_boundary_callback(
                self.ptr,
            );
        }
    }

    pub fn stop_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_stop_recording(self.ptr) };
    }

    pub fn pause_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_pause_recording(self.ptr) };
    }

    pub fn resume_recording(&self) {
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_resume_recording(self.ptr) };
    }

    pub fn set_max_recorded_duration(&self, duration: CMTime) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_set_max_recorded_duration(
                self.ptr, duration,
            );
        }
    }

    pub fn set_max_recorded_file_size(&self, bytes: i64) {
        unsafe {
            ffi::movie_file_output::av_capture_audio_file_output_set_max_recorded_file_size(
                self.ptr, bytes,
            );
        }
    }

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
    let state = Box::new(FileOutputSampleBufferCallbackState {
        callback: Box::new(callback),
    });
    let userdata = Box::into_raw(state).cast::<c_void>();
    let mut err: *mut c_char = ptr::null_mut();
    let status = unsafe {
        register_callback(
            ptr,
            Some(file_output_sample_buffer_trampoline),
            userdata,
            Some(file_output_sample_buffer_callback_drop),
            &mut err,
        )
    };
    if status != ffi::status::OK {
        unsafe { file_output_sample_buffer_callback_drop(userdata) };
        return Err(unsafe { from_swift(status, err) });
    }
    Ok(())
}

fn output_path_cstring<P: AsRef<Path>>(
    output_path: P,
    what: &str,
) -> Result<std::ffi::CString, AVCaptureError> {
    let output_path = output_path.as_ref().to_string_lossy().into_owned();
    cstring(&output_path, what)
}

unsafe extern "C" fn file_output_sample_buffer_trampoline(
    userdata: *mut c_void,
    sample_buffer: *mut c_void,
) {
    let Some(state) = userdata
        .cast::<FileOutputSampleBufferCallbackState>()
        .as_mut()
    else {
        return;
    };
    let Some(sample_buffer) = CMSampleBuffer::from_raw(sample_buffer) else {
        return;
    };
    (state.callback)(sample_buffer);
}

unsafe extern "C" fn file_output_sample_buffer_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(
        userdata.cast::<FileOutputSampleBufferCallbackState>(),
    ));
}

unsafe extern "C" fn movie_recording_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<MovieRecordingCallbackState>().as_mut() else {
        return;
    };
    let Ok(event) = parse_json_and_free::<MovieRecordingEvent>(payload) else {
        return;
    };
    (state.callback)(event);
}

unsafe extern "C" fn movie_recording_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(
        userdata.cast::<MovieRecordingCallbackState>(),
    ));
}

unsafe extern "C" fn audio_file_recording_trampoline(userdata: *mut c_void, payload: *mut c_char) {
    let Some(state) = userdata.cast::<AudioFileRecordingCallbackState>().as_mut() else {
        return;
    };
    let Ok(event) = parse_json_and_free::<AudioFileRecordingEvent>(payload) else {
        return;
    };
    (state.callback)(event);
}

unsafe extern "C" fn audio_file_recording_callback_drop(userdata: *mut c_void) {
    if userdata.is_null() {
        return;
    }
    drop(Box::from_raw(
        userdata.cast::<AudioFileRecordingCallbackState>(),
    ));
}
