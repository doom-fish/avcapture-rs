//! Errors produced by the `AVCapture` bridge.

use core::fmt;

use crate::ffi;

/// Top-level error type returned by fallible APIs in this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// `AVCapture` values.
pub enum AVCaptureError {
    /// Invalid caller input (UTF-8 / NUL / unsupported configuration).
    InvalidArgument(String),
    /// Capture-device discovery or lookup failed.
    DeviceError(String),
    /// Device-input creation failed.
    InputError(String),
    /// Session creation or configuration failed.
    SessionError(String),
    /// Output creation or configuration failed.
    OutputError(String),
    /// Callback installation failed.
    CallbackError(String),
    /// A generic operation on an existing object failed.
    OperationFailed(String),
}

impl fmt::Display for AVCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::DeviceError(message) => write!(f, "capture device error: {message}"),
            Self::InputError(message) => write!(f, "capture input error: {message}"),
            Self::SessionError(message) => write!(f, "capture session error: {message}"),
            Self::OutputError(message) => write!(f, "capture output error: {message}"),
            Self::CallbackError(message) => write!(f, "capture callback error: {message}"),
            Self::OperationFailed(message) => write!(f, "operation failed: {message}"),
        }
    }
}

impl std::error::Error for AVCaptureError {}

/// Corresponds to `AVCapture.from_swift`.
///
/// # Safety
///
/// The caller must ensure the raw inputs satisfy the bridge invariants expected by the underlying API.
pub unsafe fn from_swift(status: i32, error_str: *mut core::ffi::c_char) -> AVCaptureError {
    let message = if error_str.is_null() {
        String::new()
    } else {
        let s = core::ffi::CStr::from_ptr(error_str)
            .to_string_lossy()
            .into_owned();
        ffi::core::avc_string_free(error_str);
        s
    };

    match status {
        ffi::status::INVALID_ARGUMENT => AVCaptureError::InvalidArgument(message),
        ffi::status::DEVICE_ERROR => AVCaptureError::DeviceError(message),
        ffi::status::INPUT_ERROR => AVCaptureError::InputError(message),
        ffi::status::SESSION_ERROR => AVCaptureError::SessionError(message),
        ffi::status::OUTPUT_ERROR => AVCaptureError::OutputError(message),
        ffi::status::CALLBACK_ERROR => AVCaptureError::CallbackError(message),
        ffi::status::OPERATION_FAILED => AVCaptureError::OperationFailed(message),
        _ => AVCaptureError::OperationFailed(format!("unknown status {status}: {message}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{from_swift, AVCaptureError};
    use crate::ffi;

    #[test]
    fn display_formats_each_error_variant_with_expected_prefix() {
        assert_eq!(
            AVCaptureError::InvalidArgument("bad value".to_owned()).to_string(),
            "invalid argument: bad value"
        );
        assert_eq!(
            AVCaptureError::DeviceError("offline".to_owned()).to_string(),
            "capture device error: offline"
        );
        assert_eq!(
            AVCaptureError::InputError("missing input".to_owned()).to_string(),
            "capture input error: missing input"
        );
        assert_eq!(
            AVCaptureError::SessionError("stopped".to_owned()).to_string(),
            "capture session error: stopped"
        );
        assert_eq!(
            AVCaptureError::OutputError("write failed".to_owned()).to_string(),
            "capture output error: write failed"
        );
        assert_eq!(
            AVCaptureError::CallbackError("delegate failed".to_owned()).to_string(),
            "capture callback error: delegate failed"
        );
        assert_eq!(
            AVCaptureError::OperationFailed("bridge failed".to_owned()).to_string(),
            "operation failed: bridge failed"
        );
    }

    #[test]
    fn from_swift_maps_known_status_codes() {
        let cases = [
            (
                ffi::status::INVALID_ARGUMENT,
                AVCaptureError::InvalidArgument(String::new()),
            ),
            (
                ffi::status::DEVICE_ERROR,
                AVCaptureError::DeviceError(String::new()),
            ),
            (
                ffi::status::INPUT_ERROR,
                AVCaptureError::InputError(String::new()),
            ),
            (
                ffi::status::SESSION_ERROR,
                AVCaptureError::SessionError(String::new()),
            ),
            (
                ffi::status::OUTPUT_ERROR,
                AVCaptureError::OutputError(String::new()),
            ),
            (
                ffi::status::CALLBACK_ERROR,
                AVCaptureError::CallbackError(String::new()),
            ),
            (
                ffi::status::OPERATION_FAILED,
                AVCaptureError::OperationFailed(String::new()),
            ),
        ];

        for (status, expected) in cases {
            let error = unsafe { from_swift(status, core::ptr::null_mut()) };
            assert_eq!(error, expected);
        }
    }

    #[test]
    fn from_swift_maps_unknown_status_codes_to_operation_failed() {
        let error = unsafe { from_swift(-42, core::ptr::null_mut()) };

        assert_eq!(
            error,
            AVCaptureError::OperationFailed("unknown status -42: ".to_owned())
        );
    }
}
