mod common;

use core::ffi::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::ptr;

use avcapture::{ffi, prelude::*};
use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCaptureOutputInfo {
    connection_count: usize,
    deferred_start_supported: Option<bool>,
    deferred_start_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAudioPreviewOutputInfo {
    connection_count: usize,
    output_device_unique_id: Option<String>,
    volume: f32,
}

unsafe fn decode_json<T: DeserializeOwned>(json_ptr: *mut c_char) -> T {
    let json = CStr::from_ptr(json_ptr).to_string_lossy().into_owned();
    ffi::core::avc_string_free(json_ptr);
    serde_json::from_str(&json).expect("bridge returned invalid JSON")
}

#[test]
fn data_outputs_smoke() -> common::TestResult {
    let video_output = VideoDataOutput::new()?;
    video_output
        .set_video_settings(Some(&VideoOutputSettings::bgra().with_dimensions(640, 480)))?;
    video_output
        .set_sample_buffer_handler(Some("avcapture-test-video"), |_sample, _pixel_buffer| {})?;
    assert!(video_output.callback_installed()?);
    let video_info = video_output.info()?;
    assert_eq!(
        video_output.dropped_sample_count()?,
        video_info.dropped_sample_count
    );
    assert_eq!(
        video_output
            .last_dropped_sample_reason()?
            .as_ref()
            .map(AVCaptureOutputDataDroppedReason::as_raw),
        video_info
            .last_dropped_sample_reason
            .as_ref()
            .map(AVCaptureOutputDataDroppedReason::as_raw)
    );
    assert_eq!(video_info.dropped_sample_count, 0);
    assert!(video_info.last_dropped_sample_reason.is_none());
    let video_output_info = video_output.output_info()?;
    assert_eq!(
        video_output.deferred_start_supported()?,
        video_output_info.deferred_start_supported
    );
    assert_eq!(
        video_output.deferred_start_enabled()?,
        video_output_info.deferred_start_enabled
    );
    if video_output_info.deferred_start_enabled == Some(true) {
        assert_eq!(video_output_info.deferred_start_supported, Some(true));
    }
    video_output.clear_sample_buffer_handler();
    assert!(!video_output.callback_installed()?);

    let audio_output = AudioDataOutput::new()?;
    audio_output.set_audio_settings(Some(&AudioOutputSettings::pcm_i16(48_000.0, 2)))?;
    audio_output.set_sample_buffer_handler(Some("avcapture-test-audio"), |_sample| {})?;
    assert!(audio_output.callback_installed()?);
    let audio_info = audio_output.info()?;
    assert_eq!(
        audio_output.dropped_sample_count()?,
        audio_info.dropped_sample_count
    );
    assert_eq!(
        audio_output.last_dropped_sample_reason()?,
        audio_info.last_dropped_sample_reason
    );
    assert_eq!(audio_info.dropped_sample_count, 0);
    assert!(audio_info.last_dropped_sample_reason.is_none());
    let audio_output_info = audio_output.output_info()?;
    assert_eq!(
        audio_output.deferred_start_supported()?,
        audio_output_info.deferred_start_supported
    );
    assert_eq!(
        audio_output.deferred_start_enabled()?,
        audio_output_info.deferred_start_enabled
    );
    if audio_output_info.deferred_start_enabled == Some(true) {
        assert_eq!(audio_output_info.deferred_start_supported, Some(true));
    }
    audio_output.clear_sample_buffer_handler();
    assert!(!audio_output.callback_installed()?);
    Ok(())
}

#[test]
fn dropped_reason_wrapper_deserializes_known_and_unknown_values() {
    let audio_info: AudioDataOutputInfo = serde_json::from_str(
        r#"{
            "connectionCount": 0,
            "callbackInstalled": false,
            "audioSettings": null,
            "droppedSampleCount": 1,
            "lastDroppedSampleReason": "lateData"
        }"#,
    )
    .expect("audio dropped reason JSON should deserialize");
    assert_eq!(
        audio_info
            .last_dropped_sample_reason
            .as_ref()
            .map(AVCaptureOutputDataDroppedReason::as_raw),
        Some("lateData")
    );

    let video_info: VideoDataOutputInfo = serde_json::from_str(
        r#"{
            "connectionCount": 0,
            "alwaysDiscardsLateVideoFrames": true,
            "availableVideoCvPixelFormatTypes": [],
            "callbackInstalled": false,
            "videoSettings": null,
            "droppedSampleCount": 2,
            "lastDroppedSampleReason": "vendorSpecificReason"
        }"#,
    )
    .expect("video dropped reason JSON should deserialize");
    let reason = video_info
        .last_dropped_sample_reason
        .as_ref()
        .expect("video dropped reason should be present");
    assert_eq!(reason.as_raw(), "vendorSpecificReason");
    assert_eq!(
        serde_json::to_string(reason).expect("wrapped dropped reason should serialize"),
        "\"vendorSpecificReason\""
    );
}

#[test]
fn audio_preview_output_ffi_smoke() {
    let mut err: *mut c_char = ptr::null_mut();
    let output_ptr =
        unsafe { ffi::audio_data_output::av_capture_audio_preview_output_create(&mut err) };
    assert!(err.is_null(), "unexpected create error: {err:?}");
    assert!(!output_ptr.is_null());

    let info_json = unsafe {
        ffi::audio_data_output::av_capture_audio_preview_output_info_json(output_ptr, &mut err)
    };
    assert!(err.is_null(), "unexpected info error: {err:?}");
    let info: RawAudioPreviewOutputInfo = unsafe { decode_json(info_json) };
    assert_eq!(info.connection_count, 0);
    assert!(info.output_device_unique_id.is_none());
    assert!(info.volume.is_finite());

    unsafe { ffi::audio_data_output::av_capture_audio_preview_output_set_volume(output_ptr, 0.25) };
    let unique_id = CString::new("avcapture-test-device").expect("CString conversion failed");
    unsafe {
        ffi::audio_data_output::av_capture_audio_preview_output_set_output_device_unique_id(
            output_ptr,
            unique_id.as_ptr(),
        );
    }

    let updated_json = unsafe {
        ffi::audio_data_output::av_capture_audio_preview_output_info_json(output_ptr, &mut err)
    };
    assert!(err.is_null(), "unexpected updated info error: {err:?}");
    let updated: RawAudioPreviewOutputInfo = unsafe { decode_json(updated_json) };
    assert!(
        updated.output_device_unique_id.is_none()
            || updated.output_device_unique_id.as_deref() == Some("avcapture-test-device")
    );
    assert!((updated.volume - 0.25).abs() < f32::EPSILON * 8.0);

    let output_info_json =
        unsafe { ffi::output::av_capture_output_info_json(output_ptr, &mut err) };
    assert!(err.is_null(), "unexpected generic info error: {err:?}");
    let output_info: RawCaptureOutputInfo = unsafe { decode_json(output_info_json) };
    assert_eq!(output_info.connection_count, updated.connection_count);
    if output_info.deferred_start_enabled == Some(true) {
        assert_eq!(output_info.deferred_start_supported, Some(true));
    }

    unsafe {
        ffi::audio_data_output::av_capture_audio_preview_output_set_output_device_unique_id(
            output_ptr,
            ptr::null(),
        );
    }
    let cleared_json = unsafe {
        ffi::audio_data_output::av_capture_audio_preview_output_info_json(output_ptr, &mut err)
    };
    assert!(err.is_null(), "unexpected cleared info error: {err:?}");
    let cleared: RawAudioPreviewOutputInfo = unsafe { decode_json(cleared_json) };
    assert!(cleared.output_device_unique_id.is_none());

    unsafe {
        ffi::audio_data_output::av_capture_audio_preview_output_release(
            output_ptr.cast::<c_void>(),
        );
    }
}
