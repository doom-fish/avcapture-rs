use core::ffi::c_char;

extern "C" {
    pub fn avc_string_free(s: *mut c_char);
}
