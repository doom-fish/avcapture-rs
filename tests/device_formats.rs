mod common;

use avcapture::prelude::*;

#[test]
fn device_formats_smoke() -> common::TestResult {
    let Some(device) = CaptureDevice::default(&MediaType::Video)? else {
        common::skip_no_device("device formats");
        return Ok(());
    };

    let formats = device.formats()?;
    assert_eq!(device.formats_count()?, formats.len());
    let details = device.details()?;
    assert_eq!(device.exposure_mode()?, details.exposure_mode);
    if let Some(mode) = details.exposure_mode {
        assert!(device.is_exposure_mode_supported(mode));
    }
    if let Some(active_format) = device.active_format()? {
        let _ = active_format.info()?;
    }
    if let Some(first_format) = formats.first() {
        let _ = first_format.info()?;
    }
    Ok(())
}

#[test]
fn device_format_ranges_smoke() -> common::TestResult {
    let Some(device) = CaptureDevice::default(&MediaType::Video)? else {
        common::skip_no_device("device format ranges");
        return Ok(());
    };

    let Some(format) = device.active_format()? else {
        return Ok(());
    };

    let info = format.info()?;
    for range in &info.video_supported_frame_rate_ranges {
        assert!(range.min_frame_rate <= range.max_frame_rate);
    }
    if let Some(range) = info.system_recommended_exposure_bias_range.as_ref() {
        assert!(range.min_exposure_bias <= range.max_exposure_bias);
    }
    if let Some(range) = info.system_recommended_video_zoom_range.as_ref() {
        assert!(range.min_zoom_factor <= range.max_zoom_factor);
    }
    for range in &info.supported_video_zoom_ranges_for_depth_data_delivery {
        assert!(range.min_zoom_factor <= range.max_zoom_factor);
    }

    let _ = format.video_supported_frame_rate_ranges()?;
    let _ = format.video_frame_rate_range_for_center_stage()?;
    let _ = format.video_frame_rate_range_for_portrait_effect()?;
    let _ = format.video_frame_rate_range_for_studio_light()?;
    let _ = format.video_frame_rate_range_for_reaction_effects_in_progress()?;
    let _ = format.video_frame_rate_range_for_background_replacement()?;
    let _ = format.video_frame_rate_range_for_cinematic_video()?;
    let _ = format.system_recommended_video_zoom_range()?;
    let _ = format.system_recommended_exposure_bias_range()?;
    let _ = format.supported_video_zoom_ranges_for_depth_data_delivery()?;
    Ok(())
}
