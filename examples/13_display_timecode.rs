mod support;

use apple_cf::cm::CMTime;
use avcapture::prelude::*;

fn frame_duration_30fps() -> CMTime {
    CMTime {
        value: 1,
        timescale: 30,
        flags: 1,
        epoch: 0,
    }
}

fn main() -> support::ExampleResult {
    match VideoPreviewLayer::desk_view_application() {
        Ok(application) => {
            println!("desk view application info: {:?}", application.info()?);
            let launch_configuration =
                VideoPreviewLayer::desk_view_application_launch_configuration()?;
            launch_configuration
                .set_main_window_frame(&CaptureRect::new(32.0, 48.0, 640.0, 360.0))?;
            launch_configuration.set_requires_setup_mode_completion(true);
            println!(
                "desk view launch configuration: {:?}",
                launch_configuration.info()?
            );
        }
        Err(err) => {
            support::print_skip("desk view application", err);
            return Ok(());
        }
    }

    match VideoPreviewLayer::external_display_support_info() {
        Ok(support_info) => {
            println!("external display support: {:?}", support_info);
            let configuration = VideoPreviewLayer::external_display_configuration()?;
            configuration
                .set_should_match_frame_rate(support_info.should_match_frame_rate_supported);
            configuration.set_bypass_color_space_conversion(
                support_info.bypass_color_space_conversion_supported,
            );
            configuration.set_preferred_resolution(&VideoDimensions::new(1920, 1080))?;
            println!(
                "external display configuration: {:?}",
                configuration.info()?
            );

            let session = CaptureSession::new()?;
            let preview_layer = VideoPreviewLayer::new(&session)?;
            if let Some(device) = CaptureDevice::default(&MediaType::Video)? {
                match preview_layer.external_display_configurator(&device, &configuration) {
                    Ok(configurator) => {
                        println!("external display configurator: {:?}", configurator.info()?);
                        configurator.stop();
                    }
                    Err(err) => support::print_skip("external display configurator", err),
                }
            } else {
                support::print_no_device("external display configurator");
            }
        }
        Err(err) => support::print_skip("external display support", err),
    }

    match VideoDataOutput::timecode_generator() {
        Ok(generator) => {
            let frame_duration = frame_duration_30fps();
            generator.set_synchronization_timeout(2.0)?;
            generator.set_timecode_alignment_offset(0.25)?;
            generator.set_timecode_frame_duration(frame_duration)?;
            generator.set_delegate_handler(Some("avcapture-example-timecode"), |event| {
                println!("timecode generator event: {event:?}");
            })?;

            let frame_count_source = VideoDataOutput::frame_count_timecode_source()?;
            let real_time_clock_source = VideoDataOutput::real_time_clock_timecode_source()?;
            println!("frame-count source: {:?}", frame_count_source.info()?);
            println!(
                "real-time clock source: {:?}",
                real_time_clock_source.info()?
            );
            println!(
                "available timecode source count: {}",
                generator.available_sources()?.len()
            );

            generator.start_synchronization(&frame_count_source)?;
            println!(
                "current source: {:?}",
                generator.current_source()?.map(|source| source.info())
            );
            let initial_timecode = generator.generate_initial_timecode()?;
            let advanced_timecode = initial_timecode.advanced_by_frames(90)?;
            println!("initial timecode: {:?}", initial_timecode);
            println!("advanced timecode: {:?}", advanced_timecode);

            let stamped_buffer = advanced_timecode
                .create_metadata_sample_buffer_associated_with_presentation_time_stamp(CMTime {
                    value: 10,
                    timescale: 30,
                    flags: 1,
                    epoch: 0,
                })?;
            let duration_buffer =
                advanced_timecode.create_metadata_sample_buffer_for_duration(frame_duration)?;
            println!(
                "stamped metadata sample buffer available: {}",
                stamped_buffer.is_available()
            );
            println!(
                "duration metadata sample buffer available: {}",
                duration_buffer.is_available()
            );
            generator.clear_delegate_handler();
        }
        Err(err) => support::print_skip("timecode generator", err),
    }

    Ok(())
}
