#![cfg(feature = "async")]
#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use apple_cf::cm::CMSampleBuffer;
use apple_cf::cv::CVPixelBuffer;
use doom_fish_utils::completion::{AsyncCompletion, AsyncCompletionFuture};
use doom_fish_utils::stream::{AsyncStreamSender, BoundedAsyncStream};
use serde::Deserialize;
use std::ffi::{c_char, c_void, CStr, CString};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::callback::ArcContext;
use crate::error::{from_swift, report_callback_error, AVCaptureError};
use crate::helpers::{cstring, parse_bridge_json};
use crate::{ffi, CaptureRect, MetadataObject};

const AVC_STREAM_BRIDGE_ERROR_KIND: i32 = -1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Event payload derived from `AVCaptureSession` callbacks.
pub enum SessionRunningEvent {
    /// Corresponds to the `Started` case.
    Started,
    /// Corresponds to the `Stopped` case.
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Event payload derived from `AVCaptureSession` runtime-error notifications.
pub struct SessionErrorEvent {
    /// The textual description reported by the underlying API.
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Event-kind values produced by `AVCaptureSession` callbacks.
pub enum InterruptionKind {
    /// Corresponds to the `Interrupted` case.
    Interrupted,
    /// Corresponds to the `InterruptionEnded` case.
    InterruptionEnded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Event payload derived from `AVCaptureSession` interruption notifications.
pub struct InterruptionEvent {
    /// The callback kind reported by the underlying API.
    pub kind: InterruptionKind,
}

/// A video sample-buffer event delivered at ~60 Hz from the capture pipeline.
///
/// `Clone` is a **reference-count increment** (`CFRetain`) on the underlying
/// `CMSampleBufferRef` — it does **not** copy frame pixel data. That said,
/// cloning a live sample buffer extends its lifetime, which delays reuse of
/// the backing pixel memory. Prefer moving or consuming the event rather than
/// cloning it in the ~60 Hz hot path.
#[derive(Debug, Clone)]
/// Event payload derived from `AVCaptureVideoDataOutputSampleBufferDelegate` callbacks.
pub struct VideoSampleBufferEvent {
    /// The retained `CMSampleBuffer` delivered by the callback.
    pub sample_buffer: CMSampleBuffer,
    /// The associated `CVPixelBuffer`, if one was delivered.
    pub pixel_buffer: Option<CVPixelBuffer>,
}

/// An audio sample-buffer event delivered from the capture pipeline.
///
/// `Clone` is a **reference-count increment** (`CFRetain`) on the underlying
/// `CMSampleBufferRef` — it does **not** copy audio PCM data. Prefer moving
/// the event rather than cloning it.
#[derive(Debug, Clone)]
/// Event payload derived from `AVCaptureAudioDataOutputSampleBufferDelegate` callbacks.
pub struct AudioSampleBufferEvent {
    /// The retained `CMSampleBuffer` delivered by the callback.
    pub sample_buffer: CMSampleBuffer,
}

/// A file-output sample-buffer boundary event delivered from `AVCaptureFileOutputDelegate`.
///
/// `Clone` is a **reference-count increment** (`CFRetain`) on the underlying
/// `CMSampleBufferRef` — it does **not** copy sample data.
#[derive(Debug, Clone)]
pub struct FileOutputSampleBufferEvent {
    /// The retained `CMSampleBuffer` delivered by the callback.
    pub sample_buffer: CMSampleBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Event-kind values produced by `AVCaptureFileOutputRecordingDelegate` callbacks.
pub enum FileRecordingKind {
    /// Corresponds to the `Started` case.
    Started,
    /// Corresponds to the `Paused` case.
    Paused,
    /// Corresponds to the `Resumed` case.
    Resumed,
    /// Corresponds to the `WillFinish` case.
    WillFinish,
    /// Corresponds to the `Finished` case.
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Async recording event payload derived from `AVCaptureFileOutputRecordingDelegate` callbacks.
pub struct FileRecordingStreamEvent {
    /// The callback kind reported by the underlying API.
    pub kind: FileRecordingKind,
    /// The file url reported by `AVCaptureFileOutputRecordingDelegate`.
    pub file_url: String,
    /// The error message, if any.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
/// Async event payload produced by `AVCaptureMetadataOutputObjectsDelegate` callbacks.
pub struct MetadataObjectsStreamEvent {
    /// The metadata objects delivered by the callback.
    pub objects: Vec<MetadataObject>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionErrorPayload {
    error_description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileRecordingPayload {
    #[serde(rename = "fileURL", alias = "fileUrl")]
    file_url: String,
    error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataObjectPayload {
    object_type: String,
    string_value: Option<String>,
    bounds: CaptureRect,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataObjectsPayload {
    objects: Vec<MetadataObjectPayload>,
}

impl From<MetadataObjectPayload> for MetadataObject {
    fn from(value: MetadataObjectPayload) -> Self {
        Self {
            object_type: value.object_type,
            string_value: value.string_value,
            bounds: value.bounds,
        }
    }
}

fn capture_bridge_err(msg: String) -> AVCaptureError {
    if let Some(message) = msg.strip_prefix("capture operation cancelled: ") {
        return AVCaptureError::Cancelled(message.to_owned());
    }
    AVCaptureError::OperationFailed(msg)
}

struct PhotoFutureSignal<T> {
    context: usize,
    completed: AtomicBool,
    _marker: std::marker::PhantomData<fn(T)>,
}

impl<T> PhotoFutureSignal<T> {
    fn new(context: *mut c_void) -> Self {
        Self {
            context: context as usize,
            completed: AtomicBool::new(false),
            _marker: std::marker::PhantomData,
        }
    }

    fn complete(&self, result: Result<T, AVCaptureError>) {
        if self.completed.swap(true, Ordering::AcqRel) {
            return;
        }
        unsafe {
            match result {
                Ok(value) => AsyncCompletion::complete_ok(self.context as *mut c_void, value),
                Err(error) => {
                    AsyncCompletion::<T>::complete_err(
                        self.context as *mut c_void,
                        error.to_string(),
                    );
                }
            }
        }
    }

    fn cancel(&self) {
        self.complete(Err(AVCaptureError::Cancelled(
            "photo capture future was cancelled; the native capture continues to final cleanup"
                .to_owned(),
        )));
    }
}

impl<T> Drop for PhotoFutureSignal<T> {
    fn drop(&mut self) {
        if !self.completed.swap(true, Ordering::AcqRel) {
            unsafe {
                AsyncCompletion::<T>::complete_err(
                    self.context as *mut c_void,
                    "photo capture completion owner was dropped".to_owned(),
                );
            }
        }
    }
}

/// Future returned by [`PhotoCaptureEventFuture::start`] and
/// [`PhotoCaptureEventFuture::start_with_settings`].
pub struct PhotoCaptureEventFuture {
    inner: AsyncCompletionFuture<crate::PhotoCaptureEvent>,
    signal: Arc<PhotoFutureSignal<crate::PhotoCaptureEvent>>,
    output: Option<crate::PhotoOutput>,
}

impl std::fmt::Debug for PhotoCaptureEventFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhotoCaptureEventFuture")
            .finish_non_exhaustive()
    }
}

impl PhotoCaptureEventFuture {
    /// Starts a default-settings `AVCapturePhotoOutput` capture and resolves to
    /// the detailed final capture event.
    pub fn start(output: &crate::PhotoOutput) -> Result<Self, AVCaptureError> {
        let settings = crate::PhotoSettings::new()?;
        Self::start_with_settings(output, &settings)
    }

    /// Starts a capture with caller-provided settings and resolves to the
    /// detailed final capture event.
    pub fn start_with_settings(
        output: &crate::PhotoOutput,
        settings: &crate::PhotoSettings,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = AsyncCompletion::create();
        let signal = Arc::new(PhotoFutureSignal::new(ctx));
        let callback_signal = Arc::clone(&signal);
        if let Err(error) = output.capture_photo_with_settings_result(settings, move |result| {
            callback_signal.complete(result);
        }) {
            signal.complete(Err(error.clone()));
            return Err(error);
        }
        Ok(Self {
            inner,
            signal,
            output: Some(output.clone()),
        })
    }

    /// Stops waiting for the result. The native capture itself continues until
    /// its delegate receives the final completion callback.
    pub fn cancel(&mut self) {
        self.signal.cancel();
        self.output = None;
    }
}

impl Drop for PhotoCaptureEventFuture {
    fn drop(&mut self) {
        self.signal.cancel();
        self.output = None;
    }
}

impl Future for PhotoCaptureEventFuture {
    type Output = Result<crate::PhotoCaptureEvent, AVCaptureError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let result = Pin::new(&mut self.inner).poll(cx);
        if result.is_ready() {
            self.output = None;
        }
        result.map(|result| result.map_err(capture_bridge_err))
    }
}

/// Future returned by [`PhotoCaptureResultFuture::start`] and
/// [`PhotoCaptureResultFuture::start_with_settings`].
pub struct PhotoCaptureResultFuture {
    inner: AsyncCompletionFuture<crate::PhotoCaptureResult>,
    signal: Arc<PhotoFutureSignal<crate::PhotoCaptureResult>>,
    output: Option<crate::PhotoOutput>,
}

impl std::fmt::Debug for PhotoCaptureResultFuture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhotoCaptureResultFuture")
            .finish_non_exhaustive()
    }
}

impl PhotoCaptureResultFuture {
    /// Starts a default-settings `AVCapturePhotoOutput` capture and resolves to
    /// the final success result.
    pub fn start(output: &crate::PhotoOutput) -> Result<Self, AVCaptureError> {
        let settings = crate::PhotoSettings::new()?;
        Self::start_with_settings(output, &settings)
    }

    /// Starts a capture with caller-provided settings and resolves to the final
    /// success result.
    pub fn start_with_settings(
        output: &crate::PhotoOutput,
        settings: &crate::PhotoSettings,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = AsyncCompletion::create();
        let signal = Arc::new(PhotoFutureSignal::new(ctx));
        let callback_signal = Arc::clone(&signal);
        if let Err(error) = output.capture_photo_with_settings_result(settings, move |result| {
            callback_signal.complete(result.map(|event| crate::PhotoCaptureResult {
                unique_id: event.unique_id,
                error: event.error,
            }));
        }) {
            signal.complete(Err(error.clone()));
            return Err(error);
        }
        Ok(Self {
            inner,
            signal,
            output: Some(output.clone()),
        })
    }

    /// Stops waiting for the result. The native capture itself continues until
    /// its delegate receives the final completion callback.
    pub fn cancel(&mut self) {
        self.signal.cancel();
        self.output = None;
    }
}

impl Drop for PhotoCaptureResultFuture {
    fn drop(&mut self) {
        self.signal.cancel();
        self.output = None;
    }
}

impl Future for PhotoCaptureResultFuture {
    type Output = Result<crate::PhotoCaptureResult, AVCaptureError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let result = Pin::new(&mut self.inner).poll(cx);
        if result.is_ready() {
            self.output = None;
        }
        result.map(|result| {
            let result = result.map_err(capture_bridge_err)?;
            if let Some(error) = result.error.clone() {
                return Err(AVCaptureError::OperationFailed(error));
            }
            Ok(result)
        })
    }
}

#[derive(Debug)]
struct StreamHandle {
    ptr: *mut c_void,
    drop_fn: unsafe fn(*mut c_void),
}

impl StreamHandle {
    const fn new(ptr: *mut c_void, drop_fn: unsafe fn(*mut c_void)) -> Self {
        Self { ptr, drop_fn }
    }
}

impl Drop for StreamHandle {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        // SAFETY: `self.ptr` is a valid Swift bridge handle created by the
        // corresponding `avcapture_*_subscribe` / `avcapture_*_start` function
        // and owned exclusively by this `StreamHandle`. It is non-null (checked
        // above) and has not been freed yet (this is the first and only drop).
        unsafe { (self.drop_fn)(self.ptr) };
        self.ptr = std::ptr::null_mut();
    }
}

// SAFETY: `StreamHandle` holds an opaque Swift ARC-managed pointer and a C
// function pointer. The Swift objects are documented as safe to send across
// threads. The only mutation of `self.ptr` happens inside `drop(&mut self)`,
// which requires exclusive access, so `Sync` (shared-reference access) is
// sound: no two threads can reach the mutating path at the same time.
unsafe impl Send for StreamHandle {}
unsafe impl Sync for StreamHandle {}

macro_rules! impl_stream_common {
    ($ty:ident, $event:ty) => {
        impl $ty {
            /// Returns the next buffered event.
            pub fn next(&self) -> doom_fish_utils::stream::NextItem<'_, $event> {
                self.inner.next()
            }

            /// Returns the next buffered event if one is available.
            pub fn try_next(&self) -> Option<$event> {
                self.inner.try_next()
            }

            /// Returns the number of currently buffered events.
            pub fn buffered_count(&self) -> usize {
                self.inner.buffered_count()
            }

            /// Returns whether the stream has been closed.
            pub fn is_closed(&self) -> bool {
                self.inner.is_closed()
            }
        }
    };
}

fn stream_parts<T>(capacity: usize) -> (BoundedAsyncStream<T>, *mut c_void) {
    let (inner, sender) = BoundedAsyncStream::new(capacity);
    let ctx = ArcContext::new(sender).into_raw();
    (inner, ctx)
}

unsafe fn sender_from_ctx<T>(ctx: *mut c_void) -> Option<&'static AsyncStreamSender<T>> {
    ArcContext::<AsyncStreamSender<T>>::get(ctx)
}

unsafe fn take_json_str(payload: *mut c_char) -> String {
    if payload.is_null() {
        return String::new();
    }
    // SAFETY: `payload` is a nul-terminated C string allocated by Swift's
    // `ffiString` helper and must be freed with `avc_string_free`. The pointer
    // is non-null (checked above) and valid for reads up to and including the
    // nul terminator. We copy the bytes into an owned `String` before freeing.
    let s = CStr::from_ptr(payload).to_string_lossy().into_owned();
    ffi::core::avc_string_free(payload);
    s
}

unsafe fn parse_json_payload<T: serde::de::DeserializeOwned>(
    payload: *mut c_char,
) -> Result<T, AVCaptureError> {
    // SAFETY: delegates to `take_json_str` which upholds all pointer invariants.
    let json = take_json_str(payload);
    parse_bridge_json(&json)
}

macro_rules! stream_context_release {
    ($name:ident, $event:ty) => {
        unsafe extern "C" fn $name(ctx: *mut c_void) {
            ArcContext::<AsyncStreamSender<$event>>::release(ctx);
        }
    };
}

stream_context_release!(release_session_running_ctx, SessionRunningEvent);
stream_context_release!(release_session_error_ctx, SessionErrorEvent);
stream_context_release!(release_session_interruption_ctx, InterruptionEvent);
stream_context_release!(release_video_sample_ctx, VideoSampleBufferEvent);
stream_context_release!(
    release_video_data_output_event_ctx,
    crate::VideoDataOutputEvent
);
stream_context_release!(release_audio_sample_ctx, AudioSampleBufferEvent);
stream_context_release!(release_file_recording_ctx, FileRecordingStreamEvent);
stream_context_release!(
    release_file_output_sample_buffer_ctx,
    FileOutputSampleBufferEvent
);
stream_context_release!(release_metadata_objects_ctx, MetadataObjectsStreamEvent);

unsafe fn unsubscribe_session_running(handle: *mut c_void) {
    // SAFETY: `handle` is the non-null pointer returned by
    // `avcapture_session_running_subscribe` and has not been freed yet.
    ffi::async_stream::avcapture_session_running_unsubscribe(handle);
}

unsafe fn unsubscribe_session_error(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_session_error_unsubscribe(handle);
}

unsafe fn unsubscribe_session_interruption(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_session_interruption_unsubscribe(handle);
}

unsafe fn unsubscribe_video_sample(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_video_sample_unsubscribe(handle);
}

unsafe fn unsubscribe_video_data_output_event(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_video_data_output_event_unsubscribe(handle);
}

unsafe fn unsubscribe_audio_sample(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_audio_sample_unsubscribe(handle);
}

unsafe fn stop_file_recording(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_file_recording_stream_stop(handle);
}

unsafe fn stop_audio_file_recording(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_audio_file_recording_stream_stop(handle);
}

unsafe fn unsubscribe_movie_file_boundary(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_movie_file_boundary_unsubscribe(handle);
}

unsafe fn unsubscribe_audio_file_boundary(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_audio_file_boundary_unsubscribe(handle);
}

unsafe fn unsubscribe_metadata_objects(handle: *mut c_void) {
    // SAFETY: same contract as `unsubscribe_session_running`.
    ffi::async_stream::avcapture_metadata_objects_unsubscribe(handle);
}

const fn file_recording_kind(kind: i32) -> Option<FileRecordingKind> {
    match kind {
        0 => Some(FileRecordingKind::Started),
        1 => Some(FileRecordingKind::Paused),
        2 => Some(FileRecordingKind::Resumed),
        3 => Some(FileRecordingKind::WillFinish),
        4 => Some(FileRecordingKind::Finished),
        _ => None,
    }
}

/// # Safety
/// Called by the Swift bridge from any thread. `ctx` is an independently
/// retained `Arc<AsyncStreamSender<_>>` owned by the native callback owner.
unsafe extern "C" fn session_running_cb(kind: i32, payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<SessionRunningEvent>(ctx) else {
        let _ = take_json_str(payload);
        return;
    };
    let event = match kind {
        0 => SessionRunningEvent::Started,
        1 => SessionRunningEvent::Stopped,
        unknown => {
            let detail = take_json_str(payload);
            report_callback_error(
                "session_running_cb",
                AVCaptureError::BridgeProtocol(format!(
                    "unknown session running event kind {unknown}: {detail}"
                )),
            );
            return;
        }
    };
    let _ = take_json_str(payload);
    sender.push(event);
}

/// # Safety
/// Same contract as `session_running_cb`. `payload` is an owned C string on
/// `kind == 0`; for any other `kind` it is null or forwarded to `take_json_str`
/// for cleanup.
unsafe extern "C" fn session_error_cb(kind: i32, payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<SessionErrorEvent>(ctx) else {
        let _ = take_json_str(payload);
        return;
    };
    if kind != 0 {
        if kind == -1 {
            report_callback_error(
                "session_error_cb",
                AVCaptureError::BridgeProtocol(take_json_str(payload)),
            );
            return;
        }
        let _ = take_json_str(payload);
        report_callback_error(
            "session_error_cb",
            AVCaptureError::BridgeProtocol(format!("unknown session error event kind {kind}")),
        );
        return;
    }
    let payload = match parse_json_payload::<SessionErrorPayload>(payload) {
        Ok(payload) => payload,
        Err(error) => {
            report_callback_error("session_error_cb", error);
            return;
        }
    };
    sender.push(SessionErrorEvent {
        description: payload.error_description,
    });
}

/// # Safety
/// Same contract as `session_running_cb`.
unsafe extern "C" fn session_interruption_cb(kind: i32, payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<InterruptionEvent>(ctx) else {
        let _ = take_json_str(payload);
        return;
    };
    let kind = match kind {
        0 => InterruptionKind::Interrupted,
        1 => InterruptionKind::InterruptionEnded,
        unknown => {
            let detail = take_json_str(payload);
            report_callback_error(
                "session_interruption_cb",
                AVCaptureError::BridgeProtocol(format!(
                    "unknown session interruption event kind {unknown}: {detail}"
                )),
            );
            return;
        }
    };
    let _ = take_json_str(payload);
    sender.push(InterruptionEvent { kind });
}

/// # Safety
/// Called from the capture dispatch queue (see `VideoSampleStreamBridge`).
/// `sample_buffer` is a `CMSampleBufferRef` at +1 retain (`passRetained`);
/// `pixel_buffer` is a `CVPixelBufferRef` at +1 retain, or null.
/// Both are consumed (released) by the `CMSampleBuffer`/`CVPixelBuffer` drop
/// impls, either immediately (early returns) or when the event is eventually
/// popped or displaced from the `BoundedAsyncStream` ring buffer.
#[allow(unused_unsafe)]
unsafe extern "C" fn video_sample_cb(
    ctx: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
) {
    let sample = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let pixel = unsafe { CVPixelBuffer::from_raw(pixel_buffer) };
    let Some(sender) = sender_from_ctx::<VideoSampleBufferEvent>(ctx) else {
        drop(sample);
        drop(pixel);
        return;
    };
    let Some(sample_buffer) = sample else {
        return;
    };
    sender.push(VideoSampleBufferEvent {
        sample_buffer,
        pixel_buffer: pixel,
    });
}

#[allow(unused_unsafe)]
unsafe extern "C" fn video_data_output_event_cb(
    ctx: *mut c_void,
    kind: i32,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
    dropped_reason: *mut c_char,
    dropped_total: u64,
) {
    let sample = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let pixel = unsafe { CVPixelBuffer::from_raw(pixel_buffer) };
    let reason = if dropped_reason.is_null() {
        None
    } else {
        Some(crate::AVCaptureOutputDataDroppedReason::from_raw(
            take_json_str(dropped_reason),
        ))
    };
    let Some(sender) = sender_from_ctx::<crate::VideoDataOutputEvent>(ctx) else {
        drop(sample);
        drop(pixel);
        return;
    };
    let event = match (kind, sample) {
        (0, Some(sample_buffer)) => crate::VideoDataOutputEvent::Sample {
            sample_buffer,
            pixel_buffer: pixel,
        },
        (1, Some(sample_buffer)) => crate::VideoDataOutputEvent::Dropped {
            sample_buffer,
            reason,
            total: dropped_total,
        },
        (unknown, sample_buffer) => {
            drop(sample_buffer);
            drop(pixel);
            report_callback_error(
                "video_data_output_event_cb",
                AVCaptureError::BridgeProtocol(format!(
                    "unknown video data output stream event kind {unknown}"
                )),
            );
            return;
        }
    };
    sender.push(event);
}

/// # Safety
/// Same as `video_sample_cb` but audio-only. `sample_buffer` is a
/// `CMSampleBufferRef` at +1 retain.
#[allow(unused_unsafe)]
unsafe extern "C" fn audio_sample_cb(ctx: *mut c_void, sample_buffer: *mut c_void) {
    let sample = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let Some(sender) = sender_from_ctx::<AudioSampleBufferEvent>(ctx) else {
        drop(sample);
        return;
    };
    let Some(sample_buffer) = sample else {
        return;
    };
    sender.push(AudioSampleBufferEvent { sample_buffer });
}

/// # Safety
/// Same as `audio_sample_cb`, but used for file-output sample-buffer boundary
/// delivery.
#[allow(unused_unsafe)]
unsafe extern "C" fn file_output_sample_buffer_cb(ctx: *mut c_void, sample_buffer: *mut c_void) {
    let sample = unsafe { CMSampleBuffer::from_raw(sample_buffer) };
    let Some(sender) = sender_from_ctx::<FileOutputSampleBufferEvent>(ctx) else {
        drop(sample);
        return;
    };
    let Some(sample_buffer) = sample else {
        return;
    };
    sender.push(FileOutputSampleBufferEvent { sample_buffer });
}

/// # Safety
/// Same contract as `session_error_cb`.
unsafe extern "C" fn file_recording_cb(kind: i32, payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<FileRecordingStreamEvent>(ctx) else {
        let _ = take_json_str(payload);
        return;
    };
    if kind == AVC_STREAM_BRIDGE_ERROR_KIND {
        report_callback_error(
            "file_recording_cb",
            AVCaptureError::BridgeProtocol(take_json_str(payload)),
        );
        return;
    }
    let Some(kind) = file_recording_kind(kind) else {
        let _ = take_json_str(payload);
        report_callback_error(
            "file_recording_cb",
            AVCaptureError::BridgeProtocol(format!("unknown file recording event kind {kind}")),
        );
        return;
    };
    let payload = match parse_json_payload::<FileRecordingPayload>(payload) {
        Ok(payload) => payload,
        Err(error) => {
            report_callback_error("file_recording_cb", error);
            return;
        }
    };
    sender.push(FileRecordingStreamEvent {
        kind,
        file_url: payload.file_url,
        error: payload.error,
    });
}

/// # Safety
/// Same contract as `session_error_cb`.
unsafe extern "C" fn metadata_objects_cb(kind: i32, payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<MetadataObjectsStreamEvent>(ctx) else {
        let _ = take_json_str(payload);
        return;
    };
    if kind != 0 {
        if kind == AVC_STREAM_BRIDGE_ERROR_KIND {
            report_callback_error(
                "metadata_objects_cb",
                AVCaptureError::BridgeProtocol(take_json_str(payload)),
            );
            return;
        }
        let _ = take_json_str(payload);
        report_callback_error(
            "metadata_objects_cb",
            AVCaptureError::BridgeProtocol(format!("unknown metadata event kind {kind}")),
        );
        return;
    }
    let payload = match parse_json_payload::<MetadataObjectsPayload>(payload) {
        Ok(payload) => payload,
        Err(error) => {
            report_callback_error("metadata_objects_cb", error);
            return;
        }
    };
    sender.push(MetadataObjectsStreamEvent {
        objects: payload.objects.into_iter().map(Into::into).collect(),
    });
}

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureSession`.
pub struct SessionRunningStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<SessionRunningEvent>,
}

impl SessionRunningStream {
    /// Subscribes to `AVCaptureSession` updates with the given buffer capacity.
    pub fn subscribe(session: &crate::CaptureSession, capacity: usize) -> Self {
        let (inner, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_session_running_subscribe_owned(
                session.ptr,
                Some(session_running_cb),
                ctx,
                Some(release_session_running_ctx),
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "session running stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_session_running),
            inner,
        }
    }
}

impl_stream_common!(SessionRunningStream, SessionRunningEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureSession`.
pub struct SessionErrorStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<SessionErrorEvent>,
}

impl SessionErrorStream {
    /// Subscribes to `AVCaptureSession` updates with the given buffer capacity.
    pub fn subscribe(session: &crate::CaptureSession, capacity: usize) -> Self {
        let (inner, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_session_error_subscribe_owned(
                session.ptr,
                Some(session_error_cb),
                ctx,
                Some(release_session_error_ctx),
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "session error stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_session_error),
            inner,
        }
    }
}

impl_stream_common!(SessionErrorStream, SessionErrorEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureSession`.
pub struct SessionInterruptionStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<InterruptionEvent>,
}

impl SessionInterruptionStream {
    /// Subscribes to `AVCaptureSession` updates with the given buffer capacity.
    pub fn subscribe(session: &crate::CaptureSession, capacity: usize) -> Self {
        let (inner, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_session_interruption_subscribe_owned(
                session.ptr,
                Some(session_interruption_cb),
                ctx,
                Some(release_session_interruption_ctx),
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "session interruption stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_session_interruption),
            inner,
        }
    }
}

impl_stream_common!(SessionInterruptionStream, InterruptionEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureVideoDataOutput`.
pub struct VideoSampleBufferStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<VideoSampleBufferEvent>,
}

impl VideoSampleBufferStream {
    /// Subscribes to `AVCaptureVideoDataOutput` updates with the given buffer capacity.
    ///
    /// # Panics
    ///
    /// Panics if another handler or stream owns the native delegate slot. Use
    /// [`Self::try_subscribe`] to receive a typed error instead.
    pub fn subscribe(output: &crate::VideoDataOutput, capacity: usize) -> Self {
        Self::try_subscribe(output, capacity)
            .expect("video sample-buffer delegate slot is already occupied")
    }

    /// Tries to subscribe without replacing an existing native delegate owner.
    pub fn try_subscribe(
        output: &crate::VideoDataOutput,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-video-stream").expect("queue label is valid");
        let mut status = ffi::status::OK;
        let mut err = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_video_sample_subscribe_owned(
                output.ptr,
                queue_label.as_ptr(),
                Some(video_sample_cb),
                ctx,
                Some(release_video_sample_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_video_sample),
            inner,
        })
    }
}

impl_stream_common!(VideoSampleBufferStream, VideoSampleBufferEvent);

#[derive(Debug)]
/// Opt-in async stream of video samples and native dropped-frame events.
pub struct VideoDataOutputEventStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<crate::VideoDataOutputEvent>,
}

impl VideoDataOutputEventStream {
    /// Subscribes to sample and dropped-frame events without replacing another delegate owner.
    pub fn subscribe(
        output: &crate::VideoDataOutput,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-video-event-stream").expect("queue label is valid");
        let mut status = ffi::status::OK;
        let mut err = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_video_data_output_event_subscribe(
                output.ptr,
                queue_label.as_ptr(),
                Some(video_data_output_event_cb),
                ctx,
                Some(release_video_data_output_event_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_video_data_output_event),
            inner,
        })
    }
}

impl_stream_common!(VideoDataOutputEventStream, crate::VideoDataOutputEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureAudioDataOutput`.
pub struct AudioSampleBufferStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<AudioSampleBufferEvent>,
}

impl AudioSampleBufferStream {
    /// Subscribes to `AVCaptureAudioDataOutput` updates with the given buffer capacity.
    pub fn subscribe(
        output: &crate::AudioDataOutput,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-audio-stream").expect("queue label is valid");
        let mut status = ffi::status::OK;
        let mut err = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_audio_sample_subscribe_owned(
                output.ptr,
                queue_label.as_ptr(),
                Some(audio_sample_cb),
                ctx,
                Some(release_audio_sample_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_audio_sample),
            inner,
        })
    }
}

impl_stream_common!(AudioSampleBufferStream, AudioSampleBufferEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureMovieFileOutput`.
pub struct FileRecordingStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<FileRecordingStreamEvent>,
}

impl FileRecordingStream {
    /// Starts `AVCaptureMovieFileOutput` event delivery and returns an async stream.
    pub fn start(
        output: &crate::MovieFileOutput,
        path: &Path,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        Self::start_with_options(output, path, crate::RecordingOptions::default(), capacity)
    }

    /// Starts recording with explicit destination handling options.
    pub fn start_with_options(
        output: &crate::MovieFileOutput,
        path: &Path,
        options: crate::RecordingOptions,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let path = crate::movie_file_output::output_path_bytes(path, "movie file output path")?;
        let (inner, ctx) = stream_parts(capacity);
        let mut err: *mut c_char = std::ptr::null_mut();
        let mut status = ffi::status::OK;
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_file_recording_stream_start_with_options_owned(
                output.ptr,
                path.as_ptr(),
                path.len(),
                options.overwrite_policy as i32,
                Some(file_recording_cb),
                ctx,
                Some(release_file_recording_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, stop_file_recording),
            inner,
        })
    }

    /// Requests stop and waits for the native finalization callback.
    pub async fn stop_and_finalize(self) -> Result<FileRecordingStreamEvent, AVCaptureError> {
        unsafe {
            ffi::async_stream::avcapture_file_recording_stream_request_stop(self._handle.ptr);
        }
        while let Some(event) = self.inner.next().await {
            if event.kind == FileRecordingKind::Finished {
                if let Some(error) = event.error.clone() {
                    return Err(AVCaptureError::OutputError(error));
                }
                return Ok(event);
            }
        }
        Err(AVCaptureError::OperationFailed(
            "recording stream closed before native finalization".to_owned(),
        ))
    }
}

impl_stream_common!(FileRecordingStream, FileRecordingStreamEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureAudioFileOutput`.
pub struct AudioFileRecordingStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<FileRecordingStreamEvent>,
}

impl AudioFileRecordingStream {
    /// Starts `AVCaptureAudioFileOutput` event delivery and returns an async stream.
    pub fn start(
        output: &crate::AudioFileOutput,
        path: &Path,
        output_file_type: &str,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        Self::start_with_options(
            output,
            path,
            output_file_type,
            crate::RecordingOptions::default(),
            capacity,
        )
    }

    /// Starts recording with explicit destination handling options.
    pub fn start_with_options(
        output: &crate::AudioFileOutput,
        path: &Path,
        output_file_type: &str,
        options: crate::RecordingOptions,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let path = crate::movie_file_output::output_path_bytes(path, "audio file output path")?;
        let output_file_type = cstring(output_file_type, "audio file output type")?;
        let (inner, ctx) = stream_parts(capacity);
        let mut err: *mut c_char = std::ptr::null_mut();
        let mut status = ffi::status::OK;
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_audio_file_recording_stream_start_with_options_owned(
                output.ptr,
                path.as_ptr(),
                path.len(),
                output_file_type.as_ptr(),
                options.overwrite_policy as i32,
                Some(file_recording_cb),
                ctx,
                Some(release_file_recording_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, stop_audio_file_recording),
            inner,
        })
    }

    /// Requests stop and waits for the native finalization callback.
    pub async fn stop_and_finalize(self) -> Result<FileRecordingStreamEvent, AVCaptureError> {
        unsafe {
            ffi::async_stream::avcapture_audio_file_recording_stream_request_stop(self._handle.ptr);
        }
        while let Some(event) = self.inner.next().await {
            if event.kind == FileRecordingKind::Finished {
                if let Some(error) = event.error.clone() {
                    return Err(AVCaptureError::OutputError(error));
                }
                return Ok(event);
            }
        }
        Err(AVCaptureError::OperationFailed(
            "audio recording stream closed before native finalization".to_owned(),
        ))
    }
}

impl_stream_common!(AudioFileRecordingStream, FileRecordingStreamEvent);

#[derive(Debug)]
/// Async stream of file-output sample-buffer boundary events sourced from `AVCaptureMovieFileOutput`.
pub struct MovieFileSampleBufferBoundaryStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<FileOutputSampleBufferEvent>,
}

impl MovieFileSampleBufferBoundaryStream {
    /// Subscribes to `AVCaptureMovieFileOutput` sample-buffer boundary callbacks.
    pub fn subscribe(
        output: &crate::MovieFileOutput,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = stream_parts(capacity);
        let mut status = ffi::status::OK;
        let mut err = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_movie_file_boundary_subscribe_owned(
                output.ptr,
                Some(file_output_sample_buffer_cb),
                ctx,
                Some(release_file_output_sample_buffer_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_movie_file_boundary),
            inner,
        })
    }
}

impl_stream_common!(
    MovieFileSampleBufferBoundaryStream,
    FileOutputSampleBufferEvent
);

#[derive(Debug)]
/// Async stream of file-output sample-buffer boundary events sourced from `AVCaptureAudioFileOutput`.
pub struct AudioFileSampleBufferBoundaryStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<FileOutputSampleBufferEvent>,
}

impl AudioFileSampleBufferBoundaryStream {
    /// Subscribes to `AVCaptureAudioFileOutput` sample-buffer boundary callbacks.
    pub fn subscribe(
        output: &crate::AudioFileOutput,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = stream_parts(capacity);
        let mut status = ffi::status::OK;
        let mut err = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_audio_file_boundary_subscribe_owned(
                output.ptr,
                Some(file_output_sample_buffer_cb),
                ctx,
                Some(release_file_output_sample_buffer_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_audio_file_boundary),
            inner,
        })
    }
}

impl_stream_common!(
    AudioFileSampleBufferBoundaryStream,
    FileOutputSampleBufferEvent
);

#[derive(Debug)]
/// Async stream of readiness changes sourced from `AVCapturePhotoOutputReadinessCoordinator`.
pub struct PhotoCaptureReadinessStream {
    coordinator: crate::PhotoOutputReadinessCoordinator,
    inner: BoundedAsyncStream<crate::PhotoOutputCaptureReadiness>,
}

impl PhotoCaptureReadinessStream {
    /// Starts readiness observation from an owned readiness coordinator.
    pub fn from_coordinator(
        coordinator: crate::PhotoOutputReadinessCoordinator,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, sender) = BoundedAsyncStream::new(capacity);
        coordinator.set_capture_readiness_handler(move |readiness| {
            sender.push(readiness);
        })?;
        Ok(Self { coordinator, inner })
    }

    /// Creates a readiness coordinator from the output and starts async observation.
    pub fn subscribe(output: &crate::PhotoOutput, capacity: usize) -> Result<Self, AVCaptureError> {
        Self::from_coordinator(output.readiness_coordinator()?, capacity)
    }

    /// Returns the owned readiness coordinator backing this stream.
    pub const fn coordinator(&self) -> &crate::PhotoOutputReadinessCoordinator {
        &self.coordinator
    }
}

impl_stream_common!(
    PhotoCaptureReadinessStream,
    crate::PhotoOutputCaptureReadiness
);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureMetadataOutput`.
pub struct MetadataObjectsStream {
    _handle: StreamHandle,
    inner: BoundedAsyncStream<MetadataObjectsStreamEvent>,
}

impl MetadataObjectsStream {
    /// Subscribes to `AVCaptureMetadataOutput` updates with the given buffer capacity.
    pub fn subscribe(
        output: &crate::MetadataOutput,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-metadata-stream").expect("queue label is valid");
        let mut status = ffi::status::OK;
        let mut err = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_metadata_objects_subscribe_owned(
                output.ptr,
                queue_label.as_ptr(),
                Some(metadata_objects_cb),
                ctx,
                Some(release_metadata_objects_ctx),
                &mut status,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_metadata_objects),
            inner,
        })
    }
}

impl_stream_common!(MetadataObjectsStream, MetadataObjectsStreamEvent);

#[cfg(test)]
mod tests {
    use super::{capture_bridge_err, PhotoFutureSignal};
    use doom_fish_utils::completion::AsyncCompletion;
    use std::sync::Arc;

    #[test]
    fn photo_future_cancellation_completes_context_once() {
        let (future, context) = AsyncCompletion::<u32>::create();
        let signal = Arc::new(PhotoFutureSignal::<u32>::new(context));
        signal.cancel();
        signal.cancel();

        let error = pollster::block_on(future).expect_err("cancelled future should fail");
        assert!(error.contains("cancelled"));
    }

    #[test]
    fn photo_future_cancellation_maps_to_typed_error() {
        assert!(matches!(
            capture_bridge_err(
                "capture operation cancelled: native capture continues".to_owned()
            ),
            crate::AVCaptureError::Cancelled(message)
                if message == "native capture continues"
        ));
    }
}
