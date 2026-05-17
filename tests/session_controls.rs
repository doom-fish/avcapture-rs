mod common;

use avcapture::prelude::*;

#[test]
fn session_controls_surface_smoke() -> common::TestResult {
    let session = CaptureSession::new()?;

    assert_eq!(
        CaptureSession::RUNTIME_ERROR_NOTIFICATION,
        "AVCaptureSessionRuntimeErrorNotification"
    );
    assert_eq!(CaptureSession::ERROR_KEY, "AVCaptureSessionErrorKey");
    assert_eq!(
        CaptureSession::DID_START_RUNNING_NOTIFICATION,
        "AVCaptureSessionDidStartRunningNotification"
    );
    assert_eq!(
        CaptureSession::DID_STOP_RUNNING_NOTIFICATION,
        "AVCaptureSessionDidStopRunningNotification"
    );
    assert_eq!(
        CaptureSession::WAS_INTERRUPTED_NOTIFICATION,
        "AVCaptureSessionWasInterruptedNotification"
    );
    assert_eq!(
        CaptureSession::INTERRUPTION_ENDED_NOTIFICATION,
        "AVCaptureSessionInterruptionEndedNotification"
    );

    let info = session.info()?;
    assert_eq!(info.controls_count.unwrap_or(0), session.controls_count()?);
    assert_eq!(session.controls()?.len(), session.controls_count()?);
    assert_eq!(
        info.manual_deferred_start_supported,
        session.manual_deferred_start_supported()?
    );
    assert_eq!(
        info.automatically_runs_deferred_start,
        session.automatically_runs_deferred_start()?
    );

    if info.supports_controls.is_none() {
        common::skip(
            "session controls",
            "capture controls require macOS 15.0 or newer",
        );
        return Ok(());
    }

    let slider = CaptureSession::slider("Exposure", "sun.max", -1.0, 1.0)?;
    assert_eq!(slider.localized_title()?.as_deref(), Some("Exposure"));
    assert_eq!(slider.symbol_name()?.as_deref(), Some("sun.max"));
    assert_eq!(slider.value()?, -1.0);
    slider.set_action_handler(None, |_| {})?;
    assert!(slider.has_action_handler()?);
    slider.set_value(0.5)?;
    let slider_info = slider.info()?;
    assert_eq!(slider_info.value, 0.5);
    assert_eq!(slider_info.min_value, Some(-1.0));
    assert_eq!(slider_info.max_value, Some(1.0));
    assert!(slider_info.values.is_empty());

    let picker = CaptureSession::index_picker_with_titles(
        "Capture mode",
        "camera.filters",
        &["Auto", "Manual"],
    )?;
    assert_eq!(picker.localized_title()?.as_deref(), Some("Capture mode"));
    assert_eq!(picker.symbol_name()?.as_deref(), Some("camera.filters"));
    picker.set_action_handler(None, |_| {})?;
    assert!(picker.has_action_handler()?);
    picker.set_selected_index(1)?;
    let picker_info = picker.info()?;
    assert_eq!(picker_info.selected_index, 1);
    assert_eq!(picker_info.number_of_indexes, 2);
    assert_eq!(picker_info.localized_index_titles, vec!["Auto", "Manual"]);

    if session.supports_controls()? {
        session.set_controls_delegate_handler(None, |_| {})?;
        assert!(session.controls_delegate_installed()?);
        assert!(session.can_add_control(&slider)?);
        assert!(session.can_add_control(&picker)?);

        session.add_control(&slider)?;
        session.add_control(&picker)?;

        let controls = session.controls()?;
        assert_eq!(controls.len(), session.controls_count()?);
        assert!(controls
            .iter()
            .any(|control| control.kind().ok().as_deref() == Some("slider")));
        assert!(controls
            .iter()
            .any(|control| control.kind().ok().as_deref() == Some("indexPicker")));

        session.remove_control(&picker);
        session.remove_control(&slider);
        session.clear_controls_delegate_handler();
        assert!(!session.controls_delegate_installed()?);
    } else {
        assert!(!session.can_add_control(&slider)?);
        assert!(session.set_controls_delegate_handler(None, |_| {}).is_err());
    }

    slider.clear_action_handler();
    picker.clear_action_handler();
    assert!(!slider.has_action_handler()?);
    assert!(!picker.has_action_handler()?);

    if session.deferred_start_supported()? {
        session.set_deferred_start_delegate_handler(None, |_| {})?;
        assert!(session.deferred_start_delegate_installed()?);
        session.clear_deferred_start_delegate_handler();
        assert!(!session.deferred_start_delegate_installed()?);
    } else {
        assert!(session
            .set_deferred_start_delegate_handler(None, |_| {})
            .is_err());
    }

    if let Some(device) = CaptureDevice::default(&MediaType::Video)? {
        let zoom_slider = CaptureSession::system_zoom_slider(&device)?;
        assert!(zoom_slider.is_system_zoom_slider()?);
        let zoom_slider_with_handler =
            CaptureSession::system_zoom_slider_with_handler(&device, |_| {})?;
        assert!(zoom_slider_with_handler.is_system_zoom_slider()?);

        match CaptureSession::system_exposure_bias_slider(&device) {
            Ok(exposure_slider) => {
                assert!(exposure_slider.is_system_exposure_bias_slider()?);
                let exposure_slider_with_handler =
                    CaptureSession::system_exposure_bias_slider_with_handler(&device, |_| {})?;
                assert!(exposure_slider_with_handler.is_system_exposure_bias_slider()?);
            }
            Err(err) => common::skip("system exposure bias slider", err),
        }
    } else {
        common::skip_no_device("system sliders");
    }

    Ok(())
}
