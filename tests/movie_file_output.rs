mod common;

use core::ffi::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::fs;
use std::ptr;

use avcapture::{ffi, prelude::*};
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireEnvelope<T> {
    schema_version: u64,
    payload: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCaptureOutputInfo {
    connection_count: usize,
    deferred_start_supported: Option<bool>,
    deferred_start_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAudioSettings {
    sample_rate: Option<f64>,
    channel_count: Option<u32>,
    bits_per_channel: u32,
    is_float: bool,
    is_non_interleaved: bool,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAudioFileOutputInfo {
    connection_count: usize,
    is_recording: bool,
    is_recording_paused: bool,
    #[serde(rename = "outputFileURL")]
    output_file_url: Option<String>,
    available_output_file_types: Vec<String>,
    audio_settings: Option<RawAudioSettings>,
    callback_installed: bool,
    sample_buffer_boundary_callback_installed: bool,
}

unsafe fn decode_json<T: DeserializeOwned>(json_ptr: *mut c_char) -> T {
    let json = CStr::from_ptr(json_ptr).to_string_lossy().into_owned();
    ffi::core::avc_string_free(json_ptr);
    let envelope: WireEnvelope<T> =
        serde_json::from_str(&json).expect("bridge returned invalid JSON");
    assert_eq!(envelope.schema_version, 1);
    envelope.payload
}

unsafe fn take_error(err_ptr: *mut c_char) -> String {
    if err_ptr.is_null() {
        return String::new();
    }
    let error = CStr::from_ptr(err_ptr).to_string_lossy().into_owned();
    ffi::core::avc_string_free(err_ptr);
    error
}

#[allow(clippy::missing_const_for_fn)]
unsafe extern "C" fn sample_buffer_boundary_callback(
    _userdata: *mut c_void,
    _sample_buffer: *mut c_void,
) {
}

#[test]
fn movie_file_output_smoke() -> common::TestResult {
    let output = MovieFileOutput::new()?;
    output.set_max_recorded_file_size(8 * 1024 * 1024);
    output.set_min_free_disk_space_limit(0);
    output.set_max_recorded_duration(output.max_recorded_duration()?);
    output.set_movie_fragment_interval(output.movie_fragment_interval()?);
    let info = output.info()?;
    assert!(!info.is_recording);
    let output_info = output.output_info()?;
    assert_eq!(output_info.connection_count, info.connection_count);
    assert_eq!(
        output.deferred_start_supported()?,
        output_info.deferred_start_supported
    );
    assert_eq!(
        output.deferred_start_enabled()?,
        output_info.deferred_start_enabled
    );
    if output_info.deferred_start_enabled == Some(true) {
        assert_eq!(output_info.deferred_start_supported, Some(true));
    }
    assert!(!output.callback_installed()?);
    assert!(!output.sample_buffer_boundary_callback_installed()?);
    output.set_sample_buffer_boundary_handler(|_sample| {})?;
    assert!(output.sample_buffer_boundary_callback_installed()?);
    output.clear_sample_buffer_boundary_handler();
    assert!(!output.sample_buffer_boundary_callback_installed()?);

    let artifact_dir = std::env::current_dir()?
        .join("target")
        .join("test-artifacts");
    fs::create_dir_all(&artifact_dir)?;
    let artifact_path = artifact_dir.join("movie-file-output-smoke.mov");
    fs::write(&artifact_path, b"sentinel")?;
    let err = output
        .start_recording_with_handler(&artifact_path, |event| {
            eprintln!("unexpected movie recording callback: {event:?}");
        })
        .expect_err("disconnected movie output should refuse recording requests");
    assert_eq!(
        err,
        AVCaptureError::OutputFileExists(artifact_path.display().to_string())
    );
    assert_eq!(fs::read(&artifact_path)?, b"sentinel");
    fs::remove_file(&artifact_path)?;

    let directory_path = artifact_dir.join("movie-output-directory");
    fs::create_dir_all(&directory_path)?;
    let err = output
        .start_recording_with_options(&directory_path, RecordingOptions::overwrite_regular_file())
        .expect_err("explicit overwrite must reject directories");
    assert!(matches!(err, AVCaptureError::InvalidArgument(_)));
    assert!(directory_path.is_dir());
    fs::remove_dir(&directory_path)?;
    Ok(())
}

#[test]
#[allow(clippy::too_many_lines)]
fn audio_file_output_ffi_smoke() -> common::TestResult {
    let mut err: *mut c_char = ptr::null_mut();
    let output_ptr =
        unsafe { ffi::movie_file_output::av_capture_audio_file_output_create(&mut err) };
    assert!(err.is_null(), "unexpected create error: {err:?}");
    assert!(!output_ptr.is_null());

    let settings_json = serde_json::to_string(&serde_json::json!({
        "schemaVersion": 1,
        "payload": AudioOutputSettings::pcm_i16(48_000.0, 2),
    }))?;
    let settings_json = CString::new(settings_json)?;
    let status = unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_set_audio_settings_json(
            output_ptr,
            settings_json.as_ptr(),
            &mut err,
        )
    };
    assert_eq!(
        status,
        ffi::status::OK,
        "unexpected settings error: {}",
        unsafe { take_error(err) }
    );

    let info_json = unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_info_json(output_ptr, &mut err)
    };
    assert!(err.is_null(), "unexpected info error: {err:?}");
    let info: RawAudioFileOutputInfo = unsafe { decode_json(info_json) };
    assert_eq!(info.connection_count, 0);
    assert!(!info.is_recording);
    assert!(!info.is_recording_paused);
    assert!(info.output_file_url.is_none());
    assert!(!info.callback_installed);
    assert!(!info.sample_buffer_boundary_callback_installed);
    assert!(!info.available_output_file_types.is_empty());
    if let Some(settings) = info.audio_settings {
        assert_eq!(settings.sample_rate, Some(48_000.0));
        assert_eq!(settings.channel_count, Some(2));
        assert_eq!(settings.bits_per_channel, 16);
        assert!(!settings.is_float);
        assert!(!settings.is_non_interleaved);
    }

    let output_info_json =
        unsafe { ffi::output::av_capture_output_info_json(output_ptr, &mut err) };
    assert!(err.is_null(), "unexpected generic info error: {err:?}");
    let output_info: RawCaptureOutputInfo = unsafe { decode_json(output_info_json) };
    assert_eq!(output_info.connection_count, info.connection_count);
    if output_info.deferred_start_enabled == Some(true) {
        assert_eq!(output_info.deferred_start_supported, Some(true));
    }

    let status = unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_set_sample_buffer_boundary_callback(
            output_ptr,
            Some(sample_buffer_boundary_callback),
            ptr::null_mut(),
            None,
            None,
            &mut err,
        )
    };
    assert_eq!(
        status,
        ffi::status::OK,
        "unexpected boundary callback error: {}",
        unsafe { take_error(err) }
    );
    let boundary_info_json = unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_info_json(output_ptr, &mut err)
    };
    assert!(err.is_null(), "unexpected boundary info error: {err:?}");
    let boundary_info: RawAudioFileOutputInfo = unsafe { decode_json(boundary_info_json) };
    assert!(boundary_info.sample_buffer_boundary_callback_installed);
    unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_clear_sample_buffer_boundary_callback(
            output_ptr,
        );
    }
    let cleared_boundary_info_json = unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_info_json(output_ptr, &mut err)
    };
    assert!(
        err.is_null(),
        "unexpected cleared boundary info error: {err:?}"
    );
    let cleared_boundary_info: RawAudioFileOutputInfo =
        unsafe { decode_json(cleared_boundary_info_json) };
    assert!(!cleared_boundary_info.sample_buffer_boundary_callback_installed);

    let artifact_dir = std::env::current_dir()?
        .join("target")
        .join("test-artifacts");
    fs::create_dir_all(&artifact_dir)?;
    let artifact_path = artifact_dir.join("audio-file-output-smoke.caf");
    let artifact_path = artifact_path.as_os_str().as_encoded_bytes();
    let output_file_type = CString::new(info.available_output_file_types[0].clone())?;
    let status = unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_start_recording(
            output_ptr,
            artifact_path.as_ptr(),
            artifact_path.len(),
            output_file_type.as_ptr(),
            RecordingOverwritePolicy::FailIfExists as i32,
            None,
            ptr::null_mut(),
            None,
            None,
            &mut err,
        )
    };
    assert_eq!(status, ffi::status::OUTPUT_ERROR);
    let error = unsafe { take_error(err) };
    assert!(
        error.contains("not attached to a session"),
        "unexpected start error: {error}"
    );

    unsafe {
        ffi::movie_file_output::av_capture_audio_file_output_release(output_ptr.cast::<c_void>());
    }
    Ok(())
}

#[test]
fn recording_payload_fixtures_use_url_acronyms() {
    let movie: MovieRecordingEvent =
        serde_json::from_str(r#"{"kind":"started","fileURL":"/tmp/movie.mov","error":null}"#)
            .expect("Swift movie recording payload should decode");
    assert_eq!(movie.file_url, "/tmp/movie.mov");

    let audio: AudioFileRecordingEvent =
        serde_json::from_str(r#"{"kind":"finished","fileURL":"/tmp/audio.caf","error":null}"#)
            .expect("Swift audio recording payload should decode");
    assert_eq!(audio.file_url, "/tmp/audio.caf");
}
