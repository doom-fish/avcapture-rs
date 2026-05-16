#![allow(dead_code)]

use std::fmt::Display;

use avcapture::prelude::*;

pub type ExampleResult = Result<(), Box<dyn std::error::Error>>;

pub fn print_skip(context: &str, err: impl Display) {
    println!("skipping {context}: {err}");
}

pub fn print_no_device(context: &str) {
    println!("skipping {context}: no capture device available");
}

pub fn default_video_or_audio_device() -> Result<Option<CaptureDevice>, AVCaptureError> {
    if let Some(device) = CaptureDevice::default(&MediaType::Video)? {
        return Ok(Some(device));
    }
    CaptureDevice::default(&MediaType::Audio)
}
