//! Errors produced by the `AVCapture` bridge.

use core::fmt;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

use crate::ffi;

const MAX_CALLBACK_DIAGNOSTICS: usize = 64;

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
    /// A native delegate slot is already owned by another registration.
    DelegateSlotOccupied(String),
    /// The requested API is unavailable on the current platform or SDK.
    UnsupportedPlatform(String),
    /// The operation must be performed on the main thread.
    MainThreadRequired(String),
    /// A recording destination already exists.
    OutputFileExists(String),
    /// A callback or future was cancelled before native completion.
    Cancelled(String),
    /// The Rust/Swift bridge payload violated the versioned wire contract.
    BridgeProtocol(String),
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
            Self::DelegateSlotOccupied(message) => {
                write!(f, "capture delegate slot occupied: {message}")
            }
            Self::UnsupportedPlatform(message) => write!(f, "unsupported platform: {message}"),
            Self::MainThreadRequired(message) => write!(f, "main thread required: {message}"),
            Self::OutputFileExists(message) => write!(f, "output file exists: {message}"),
            Self::Cancelled(message) => write!(f, "capture operation cancelled: {message}"),
            Self::BridgeProtocol(message) => write!(f, "capture bridge protocol error: {message}"),
            Self::OperationFailed(message) => write!(f, "operation failed: {message}"),
        }
    }
}

impl std::error::Error for AVCaptureError {}

/// An error observed while decoding or dispatching an asynchronous native callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackDiagnostic {
    /// The callback trampoline or stream that observed the error.
    pub source: String,
    /// The bridge error that prevented normal event delivery.
    pub error: AVCaptureError,
}

static CALLBACK_DIAGNOSTICS: OnceLock<Mutex<VecDeque<CallbackDiagnostic>>> = OnceLock::new();

pub(crate) fn report_callback_error(source: &'static str, error: AVCaptureError) {
    eprintln!("avcapture callback error in {source}: {error}");
    let diagnostics = CALLBACK_DIAGNOSTICS.get_or_init(|| Mutex::new(VecDeque::new()));
    let mut diagnostics = diagnostics
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if diagnostics.len() == MAX_CALLBACK_DIAGNOSTICS {
        diagnostics.pop_front();
    }
    diagnostics.push_back(CallbackDiagnostic {
        source: source.to_owned(),
        error,
    });
}

/// Removes and returns callback diagnostics accumulated since the previous call.
pub fn take_callback_diagnostics() -> Vec<CallbackDiagnostic> {
    let Some(diagnostics) = CALLBACK_DIAGNOSTICS.get() else {
        return Vec::new();
    };
    diagnostics
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .drain(..)
        .collect()
}

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
        ffi::status::DELEGATE_SLOT_OCCUPIED => AVCaptureError::DelegateSlotOccupied(message),
        ffi::status::UNSUPPORTED_PLATFORM => AVCaptureError::UnsupportedPlatform(message),
        ffi::status::MAIN_THREAD_REQUIRED => AVCaptureError::MainThreadRequired(message),
        ffi::status::OUTPUT_FILE_EXISTS => AVCaptureError::OutputFileExists(message),
        ffi::status::CANCELLED => AVCaptureError::Cancelled(message),
        ffi::status::BRIDGE_PROTOCOL => AVCaptureError::BridgeProtocol(message),
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
            AVCaptureError::DelegateSlotOccupied("video output".to_owned()).to_string(),
            "capture delegate slot occupied: video output"
        );
        assert_eq!(
            AVCaptureError::UnsupportedPlatform("pixel formats".to_owned()).to_string(),
            "unsupported platform: pixel formats"
        );
        assert_eq!(
            AVCaptureError::MainThreadRequired("preview layer".to_owned()).to_string(),
            "main thread required: preview layer"
        );
        assert_eq!(
            AVCaptureError::OutputFileExists("/tmp/capture.mov".to_owned()).to_string(),
            "output file exists: /tmp/capture.mov"
        );
        assert_eq!(
            AVCaptureError::Cancelled("photo future dropped".to_owned()).to_string(),
            "capture operation cancelled: photo future dropped"
        );
        assert_eq!(
            AVCaptureError::BridgeProtocol("schema mismatch".to_owned()).to_string(),
            "capture bridge protocol error: schema mismatch"
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
                ffi::status::DELEGATE_SLOT_OCCUPIED,
                AVCaptureError::DelegateSlotOccupied(String::new()),
            ),
            (
                ffi::status::UNSUPPORTED_PLATFORM,
                AVCaptureError::UnsupportedPlatform(String::new()),
            ),
            (
                ffi::status::MAIN_THREAD_REQUIRED,
                AVCaptureError::MainThreadRequired(String::new()),
            ),
            (
                ffi::status::OUTPUT_FILE_EXISTS,
                AVCaptureError::OutputFileExists(String::new()),
            ),
            (
                ffi::status::CANCELLED,
                AVCaptureError::Cancelled(String::new()),
            ),
            (
                ffi::status::BRIDGE_PROTOCOL,
                AVCaptureError::BridgeProtocol(String::new()),
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
