use core::ffi::{c_char, c_void};

extern "C" {
    pub fn av_camera_calibration_data_release(camera_calibration_data: *mut c_void);
    pub fn av_camera_calibration_data_info_json(
        camera_calibration_data: *mut c_void,
        out_error_message: *mut *mut c_char,
    ) -> *mut c_char;
}
