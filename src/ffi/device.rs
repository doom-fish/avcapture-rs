use core::ffi::{c_char, c_void};

use apple_cf::cm::CMTime;

extern "C" {
    pub fn av_capture_authorization_status(
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_devices_json(
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_default_device(
        media_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_default_device_for_type(
        device_type: *const c_char,
        media_type: *const c_char,
        position: i32,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_with_unique_id(
        unique_id: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_release(device: *mut c_void);
    pub fn av_capture_device_info_json(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_details_json(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_supports_session_preset(
        device: *mut c_void,
        preset: *const c_char,
    ) -> bool;
    pub fn av_capture_device_formats_count(device: *mut c_void) -> usize;
    pub fn av_capture_device_format_at_index(
        device: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_active_format(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_active_video_min_frame_duration(device: *mut c_void) -> CMTime;
    pub fn av_capture_device_active_video_max_frame_duration(device: *mut c_void) -> CMTime;
    pub fn av_capture_device_is_exposure_mode_supported(device: *mut c_void, mode: i32) -> bool;
    pub fn av_capture_device_is_focus_mode_supported(device: *mut c_void, mode: i32) -> bool;
    pub fn av_capture_device_is_white_balance_mode_supported(
        device: *mut c_void,
        mode: i32,
    ) -> bool;
    pub fn av_capture_device_lock_for_configuration(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_unlock_for_configuration(device: *mut c_void);
    pub fn av_capture_device_set_active_format(
        device: *mut c_void,
        format: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_active_video_min_frame_duration(
        device: *mut c_void,
        duration: CMTime,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_active_video_max_frame_duration(
        device: *mut c_void,
        duration: CMTime,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_exposure_mode(
        device: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_focus_mode(
        device: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_white_balance_mode(
        device: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_torch_mode(
        device: *mut c_void,
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_torch_level(
        device: *mut c_void,
        level: f32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_active_color_space(
        device: *mut c_void,
        color_space: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_input_sources_count(device: *mut c_void) -> usize;
    pub fn av_capture_device_input_source_at_index(
        device: *mut c_void,
        index: usize,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_active_input_source(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_set_active_input_source(
        device: *mut c_void,
        input_source: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_input_source_release(input_source: *mut c_void);
    pub fn av_capture_device_input_source_info_json(
        input_source: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_set_transport_controls_playback_mode(
        device: *mut c_void,
        mode: i32,
        speed: f32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_primary_constituent_device_switching_behavior(
        device: *mut c_void,
        behavior: i32,
        conditions: u64,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_center_stage_control_mode() -> i32;
    pub fn av_capture_device_center_stage_enabled() -> i32;
    pub fn av_capture_device_set_center_stage_control_mode(
        mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_center_stage_enabled(
        enabled: bool,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_preferred_microphone_mode() -> i32;
    pub fn av_capture_device_active_microphone_mode() -> i32;
    pub fn av_capture_device_show_system_user_interface(
        system_user_interface: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_reaction_effects_enabled() -> i32;
    pub fn av_capture_device_reaction_effect_gestures_enabled() -> i32;
    pub fn av_capture_device_perform_reaction_effect(
        device: *mut c_void,
        reaction_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_reaction_system_image_name_for_type(
        reaction_type: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_rotation_coordinator_create(
        device: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_device_rotation_coordinator_release(coordinator: *mut c_void);
    pub fn av_capture_device_rotation_coordinator_info_json(
        coordinator: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_device_set_cinematic_video_tracking_focus_at_point(
        device: *mut c_void,
        x: f64,
        y: f64,
        focus_mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_cinematic_video_fixed_focus_at_point(
        device: *mut c_void,
        x: f64,
        y: f64,
        focus_mode: i32,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_set_camera_lens_smudge_detection(
        device: *mut c_void,
        enabled: bool,
        has_interval: bool,
        interval: CMTime,
        out_error_message: *mut *mut c_char,
    ) -> i32;
    pub fn av_capture_device_max_available_torch_level() -> f32;
}
