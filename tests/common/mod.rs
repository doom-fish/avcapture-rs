#![allow(dead_code)]

use std::fmt::Display;

use avcapture::prelude::*;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub fn skip(context: &str, err: impl Display) {
    eprintln!("skipping {context}: {err}");
}

pub fn skip_no_device(context: &str) {
    eprintln!("skipping {context}: no capture device available");
}

pub fn default_video_or_audio_device() -> Result<Option<CaptureDevice>, AVCaptureError> {
    if let Some(device) = CaptureDevice::default(&MediaType::Video)? {
        return Ok(Some(device));
    }
    CaptureDevice::default(&MediaType::Audio)
}
