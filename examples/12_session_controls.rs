mod support;

use avcapture::prelude::*;

fn main() -> support::ExampleResult {
    let session = CaptureSession::new()?;
    println!(
        "runtime error notification: {}",
        CaptureSession::RUNTIME_ERROR_NOTIFICATION
    );
    println!("session info: {:?}", session.info()?);

    let info = session.info()?;
    if info.supports_controls.is_none() {
        println!("capture controls require macOS 15.0 or newer");
        return Ok(());
    }

    let slider = CaptureSession::slider("Exposure", "sun.max", -1.0, 1.0)?;
    slider.set_action_handler(None, |value| {
        println!("slider action: {value}");
    })?;
    slider.set_value(0.5)?;
    println!("custom slider: {:?}", slider.info()?);

    let picker = CaptureSession::index_picker_with_titles(
        "Capture mode",
        "camera.filters",
        &["Auto", "Manual"],
    )?;
    picker.set_action_handler(None, |selected_index| {
        println!("picker action: {selected_index}");
    })?;
    picker.set_selected_index(1)?;
    println!("custom index picker: {:?}", picker.info()?);

    if session.supports_controls()? {
        session.set_controls_delegate_handler(None, |event| {
            println!("session controls event: {:?}", event);
        })?;

        if session.can_add_control(&slider)? {
            session.add_control(&slider)?;
        }
        if session.can_add_control(&picker)? {
            session.add_control(&picker)?;
        }

        println!("session controls count: {}", session.controls_count()?);
        for control in session.controls()? {
            println!("session control: {:?}", control.info()?);
        }

        session.remove_control(&picker);
        session.remove_control(&slider);
        session.clear_controls_delegate_handler();
    } else {
        println!("session controls are not supported on this system");
    }

    if session.deferred_start_supported()? {
        session.set_deferred_start_delegate_handler(None, |event| {
            println!("deferred start event: {:?}", event);
        })?;
        println!(
            "automatically runs deferred start: {:?}",
            session.automatically_runs_deferred_start()?
        );
        session.clear_deferred_start_delegate_handler();
    }

    if let Some(device) = CaptureDevice::default(&MediaType::Video)? {
        let zoom_slider = CaptureSession::system_zoom_slider(&device)?;
        println!("system zoom slider: {:?}", zoom_slider.info()?);

        match CaptureSession::system_exposure_bias_slider(&device) {
            Ok(exposure_slider) => {
                println!("system exposure bias slider: {:?}", exposure_slider.info()?);
            }
            Err(err) => support::print_skip("system exposure bias slider", err),
        }
    } else {
        support::print_no_device("system sliders");
    }

    Ok(())
}
