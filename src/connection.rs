#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;
use serde::{Deserialize, Serialize};

use apple_cf::cm::CMTime;

use crate::device::MediaType;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, parse_json_and_free};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureAudioChannel` state.
pub struct CaptureAudioChannelInfo {
    /// The average power level reported by `AVCaptureAudioChannel`.
    pub average_power_level: f32,
    /// The peak hold level reported by `AVCaptureAudioChannel`.
    pub peak_hold_level: f32,
    /// The volume reported by `AVCaptureAudioChannel`.
    pub volume: Option<f32>,
    /// The enabled reported by `AVCaptureAudioChannel`.
    pub enabled: Option<bool>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureConnection` state.
pub struct CaptureConnectionInfo {
    /// The input port count reported by `AVCaptureConnection`.
    pub input_port_count: usize,
    /// The media types reported by `AVCaptureConnection`.
    pub media_types: Vec<MediaType>,
    /// The enabled reported by `AVCaptureConnection`.
    pub enabled: bool,
    /// The active reported by `AVCaptureConnection`.
    pub active: bool,
    /// The supports video mirroring reported by `AVCaptureConnection`.
    pub supports_video_mirroring: bool,
    /// The video mirrored reported by `AVCaptureConnection`.
    pub video_mirrored: bool,
    /// The automatically adjusts video mirroring reported by `AVCaptureConnection`.
    pub automatically_adjusts_video_mirroring: bool,
    /// The video rotation angle reported by `AVCaptureConnection`.
    pub video_rotation_angle: Option<f64>,
    /// The supports video min frame duration reported by `AVCaptureConnection`.
    pub supports_video_min_frame_duration: bool,
    #[serde(with = "cm_time_serde")]
    /// The video min frame duration reported by `AVCaptureConnection`.
    pub video_min_frame_duration: CMTime,
    /// The supports video max frame duration reported by `AVCaptureConnection`.
    pub supports_video_max_frame_duration: bool,
    #[serde(with = "cm_time_serde")]
    /// The video max frame duration reported by `AVCaptureConnection`.
    pub video_max_frame_duration: CMTime,
    #[serde(default)]
    /// The audio channels reported by `AVCaptureConnection`.
    pub audio_channels: Vec<CaptureAudioChannelInfo>,
}

/// Safe wrapper around `AVCaptureAudioChannel`.
#[derive(Debug)]
/// Wraps `AVCaptureAudioChannel`.
pub struct CaptureAudioChannel {
    ptr: *mut c_void,
}

impl CaptureAudioChannel {
    /// Wraps an existing `AVCaptureAudioChannel` pointer.
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureAudioChannel` state.
    pub fn info(&self) -> Result<CaptureAudioChannelInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::connection::av_capture_audio_channel_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureAudioChannel.average_power_level`.
    pub fn average_power_level(&self) -> Result<f32, AVCaptureError> {
        Ok(self.info()?.average_power_level)
    }

    /// Corresponds to `AVCaptureAudioChannel.peak_hold_level`.
    pub fn peak_hold_level(&self) -> Result<f32, AVCaptureError> {
        Ok(self.info()?.peak_hold_level)
    }

    /// Corresponds to `AVCaptureAudioChannel.volume`.
    pub fn volume(&self) -> Result<Option<f32>, AVCaptureError> {
        Ok(self.info()?.volume)
    }

    /// Returns whether `AVCaptureAudioChannel` is enabled.
    pub fn is_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.info()?.enabled)
    }

    /// Sets the volume on `AVCaptureAudioChannel`.
    pub fn set_volume(&self, volume: f32) -> Result<(), AVCaptureError> {
        if !volume.is_finite() || volume < 0.0 {
            return Err(AVCaptureError::InvalidArgument(
                "audio channel volume must be finite and non-negative".to_owned(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::connection::av_capture_audio_channel_set_volume(self.ptr, volume, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the enabled on `AVCaptureAudioChannel`.
    pub fn set_enabled(&self, enabled: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::connection::av_capture_audio_channel_set_enabled(self.ptr, enabled, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl Drop for CaptureAudioChannel {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::connection::av_capture_audio_channel_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

/// Safe wrapper around `AVCaptureConnection`.
#[derive(Debug)]
/// Wraps `AVCaptureConnection`.
pub struct CaptureConnection {
    ptr: *mut c_void,
}

impl CaptureConnection {
    /// Wraps an existing `AVCaptureConnection` pointer.
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureConnection` state.
    pub fn info(&self) -> Result<CaptureConnectionInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::connection::av_capture_connection_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureConnection.media_types`.
    pub fn media_types(&self) -> Result<Vec<MediaType>, AVCaptureError> {
        Ok(self.info()?.media_types)
    }

    /// Returns the input port count reported by `AVCaptureConnection`.
    pub fn input_port_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.input_port_count)
    }

    /// Returns whether `AVCaptureConnection` is enabled.
    pub fn is_enabled(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.enabled)
    }

    /// Returns whether `AVCaptureConnection` is active.
    pub fn is_active(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.active)
    }

    /// Returns whether `AVCaptureConnection` supports video mirroring.
    pub fn supports_video_mirroring(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_video_mirroring)
    }

    /// Returns whether `AVCaptureConnection` is video mirrored.
    pub fn is_video_mirrored(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.video_mirrored)
    }

    /// Corresponds to `AVCaptureConnection.automatically_adjusts_video_mirroring`.
    pub fn automatically_adjusts_video_mirroring(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.automatically_adjusts_video_mirroring)
    }

    /// Corresponds to `AVCaptureConnection.video_rotation_angle`.
    pub fn video_rotation_angle(&self) -> Result<Option<f64>, AVCaptureError> {
        Ok(self.info()?.video_rotation_angle)
    }

    /// Returns whether `AVCaptureConnection` supports video min frame duration.
    pub fn supports_video_min_frame_duration(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_video_min_frame_duration)
    }

    /// Corresponds to `AVCaptureConnection.video_min_frame_duration`.
    pub fn video_min_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.video_min_frame_duration)
    }

    /// Returns whether `AVCaptureConnection` supports video max frame duration.
    pub fn supports_video_max_frame_duration(&self) -> Result<bool, AVCaptureError> {
        Ok(self.info()?.supports_video_max_frame_duration)
    }

    /// Corresponds to `AVCaptureConnection.video_max_frame_duration`.
    pub fn video_max_frame_duration(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.info()?.video_max_frame_duration)
    }

    /// Corresponds to `AVCaptureConnection.audio_channels_info`.
    pub fn audio_channels_info(&self) -> Result<Vec<CaptureAudioChannelInfo>, AVCaptureError> {
        Ok(self.info()?.audio_channels)
    }

    /// Returns the audio channel count reported by `AVCaptureConnection`.
    pub fn audio_channel_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.info()?.audio_channels.len())
    }

    /// Corresponds to `AVCaptureConnection.audio_channels`.
    pub fn audio_channels(&self) -> Result<Vec<CaptureAudioChannel>, AVCaptureError> {
        let count =
            unsafe { ffi::connection::av_capture_connection_audio_channels_count(self.ptr) };
        let mut channels = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::connection::av_capture_connection_audio_channel_at_index(
                    self.ptr, index, &mut err,
                )
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
            }
            channels.push(CaptureAudioChannel::from_raw(ptr));
        }
        Ok(channels)
    }

    /// Sets the enabled on `AVCaptureConnection`.
    pub fn set_enabled(&self, enabled: bool) {
        unsafe { ffi::connection::av_capture_connection_set_enabled(self.ptr, enabled) };
    }

    /// Sets the automatically adjusts video mirroring on `AVCaptureConnection`.
    pub fn set_automatically_adjusts_video_mirroring(&self, enabled: bool) {
        unsafe {
            ffi::connection::av_capture_connection_set_automatically_adjusts_video_mirroring(
                self.ptr, enabled,
            );
        }
    }

    /// Sets the video mirrored on `AVCaptureConnection`.
    pub fn set_video_mirrored(&self, mirrored: bool) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::connection::av_capture_connection_set_video_mirrored(self.ptr, mirrored, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the video rotation angle on `AVCaptureConnection`.
    pub fn set_video_rotation_angle(&self, angle: f64) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::connection::av_capture_connection_set_video_rotation_angle(
                self.ptr, angle, &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl Drop for CaptureConnection {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::connection::av_capture_connection_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}
