use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_capture_external_display_support_info_json(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;

    pub fn av_capture_external_display_configuration_create(
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_external_display_configuration_release(configuration: *mut c_void);
    pub fn av_capture_external_display_configuration_info_json(
        configuration: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_external_display_configuration_set_should_match_frame_rate(
        configuration: *mut c_void,
        should_match_frame_rate: bool,
    );
    pub fn av_capture_external_display_configuration_set_bypass_color_space_conversion(
        configuration: *mut c_void,
        bypass_color_space_conversion: bool,
    );
    pub fn av_capture_external_display_configuration_set_preferred_resolution_json(
        configuration: *mut c_void,
        preferred_resolution_json: *const c_char,
        out_error_message: *mut *mut c_char,
    ) -> i32;

    pub fn av_capture_external_display_configurator_create(
        device: *mut c_void,
        preview_layer: *mut c_void,
        configuration: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_void;
    pub fn av_capture_external_display_configurator_release(configurator: *mut c_void);
    pub fn av_capture_external_display_configurator_info_json(
        configurator: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
    pub fn av_capture_external_display_configurator_stop(configurator: *mut c_void);
}
