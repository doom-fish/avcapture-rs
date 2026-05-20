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
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::ffi::{c_char, c_void, CStr, CString};
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::error::{from_swift, AVCaptureError};
use crate::helpers::cstring;
use crate::{ffi, CaptureRect, MetadataObject};

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

const fn capture_bridge_err(msg: String) -> AVCaptureError {
    AVCaptureError::OperationFailed(msg)
}

/// Future returned by [`PhotoCaptureEventFuture::start`] and
/// [`PhotoCaptureEventFuture::start_with_settings`].
pub struct PhotoCaptureEventFuture {
    inner: AsyncCompletionFuture<crate::PhotoCaptureEvent>,
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
        let ctx = ctx as usize;
        output.capture_photo_with_settings(settings, move |event| unsafe {
            AsyncCompletion::complete_ok(ctx as *mut c_void, event);
        })?;
        Ok(Self { inner })
    }
}

impl Future for PhotoCaptureEventFuture {
    type Output = Result<crate::PhotoCaptureEvent, AVCaptureError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner)
            .poll(cx)
            .map(|result| result.map_err(capture_bridge_err))
    }
}

/// Future returned by [`PhotoCaptureResultFuture::start`] and
/// [`PhotoCaptureResultFuture::start_with_settings`].
pub struct PhotoCaptureResultFuture {
    inner: AsyncCompletionFuture<crate::PhotoCaptureResult>,
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
        let (inner, ctx) = AsyncCompletion::create();
        let ctx = ctx as usize;
        output.capture_photo(move |result| unsafe {
            AsyncCompletion::complete_ok(ctx as *mut c_void, result);
        })?;
        Ok(Self { inner })
    }

    /// Starts a capture with caller-provided settings and resolves to the final
    /// success result.
    pub fn start_with_settings(
        output: &crate::PhotoOutput,
        settings: &crate::PhotoSettings,
    ) -> Result<Self, AVCaptureError> {
        let (inner, ctx) = AsyncCompletion::create();
        let ctx = ctx as usize;
        output.capture_photo_with_settings(settings, move |event| unsafe {
            AsyncCompletion::complete_ok(
                ctx as *mut c_void,
                crate::PhotoCaptureResult {
                    unique_id: event.unique_id,
                    error: event.error,
                },
            );
        })?;
        Ok(Self { inner })
    }
}

impl Future for PhotoCaptureResultFuture {
    type Output = Result<crate::PhotoCaptureResult, AVCaptureError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.inner).poll(cx).map(|result| {
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

#[derive(Debug)]
struct SenderBox<T>(*mut AsyncStreamSender<T>);

impl<T> SenderBox<T> {
    fn new(sender: AsyncStreamSender<T>) -> Self {
        Self(Box::into_raw(Box::new(sender)))
    }

    const fn as_ptr(&self) -> *mut AsyncStreamSender<T> {
        self.0
    }
}

impl<T> Drop for SenderBox<T> {
    fn drop(&mut self) {
        if self.0.is_null() {
            return;
        }
        // SAFETY: `self.0` was allocated by `Box::into_raw(Box::new(sender))`
        // in `SenderBox::new` and is owned exclusively by this wrapper. It is
        // non-null (checked above) and has not been freed before (this is the
        // only drop site). After reconstituting the `Box` it is immediately
        // dropped, so there is no double-free.
        unsafe { drop(Box::from_raw(self.0)) };
        self.0 = std::ptr::null_mut();
    }
}

// SAFETY: `SenderBox<T>` owns a heap-allocated `AsyncStreamSender<T>` behind a
// raw pointer. `AsyncStreamSender<T>` is `Send` when `T: Send`, so transferring
// a `SenderBox<T>` to another thread is safe under the same condition.
// `Sync` is also sound: no two threads can observe the interior pointer
// simultaneously because the only mutable use is in `drop(&mut self)`.
unsafe impl<T: Send> Send for SenderBox<T> {}
unsafe impl<T: Send> Sync for SenderBox<T> {}

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

fn stream_parts<T>(capacity: usize) -> (BoundedAsyncStream<T>, SenderBox<T>, *mut c_void) {
    let (inner, sender) = BoundedAsyncStream::new(capacity);
    let sender_box = SenderBox::new(sender);
    let ctx = sender_box.as_ptr().cast::<c_void>();
    (inner, sender_box, ctx)
}

unsafe fn sender_from_ctx<T>(ctx: *mut c_void) -> Option<&'static AsyncStreamSender<T>> {
    // SAFETY: `ctx` is the `SenderBox::as_ptr()` cast to `*mut c_void` stored
    // when the stream was subscribed. The `SenderBox` is kept alive for the
    // entire lifetime of the subscription (it lives inside the stream struct
    // alongside the `StreamHandle`, and the handle is dropped before the box).
    // The `'static` lifetime is safe because we only ever access this reference
    // while the SenderBox is alive — any call that reaches here happens within
    // an active Swift delegate callback, and the Swift bridge drains in-flight
    // callbacks before releasing the bridge object (see AsyncStream.swift deinit),
    // which in turn means the SenderBox is still live.
    ctx.cast::<AsyncStreamSender<T>>().as_ref()
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

unsafe fn parse_json_payload<T: DeserializeOwned>(payload: *mut c_char) -> Option<T> {
    // SAFETY: delegates to `take_json_str` which upholds all pointer invariants.
    let json = take_json_str(payload);
    serde_json::from_str(&json).ok()
}

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
/// Called by the Swift bridge from any thread. `ctx` is the `SenderBox` raw
/// pointer held alive for the duration of the subscription. `payload` is either
/// null or an owned C string allocated by Swift.
unsafe extern "C" fn session_running_cb(kind: i32, _payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<SessionRunningEvent>(ctx) else {
        return;
    };
    let event = match kind {
        0 => SessionRunningEvent::Started,
        1 => SessionRunningEvent::Stopped,
        _ => return,
    };
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
        let _ = take_json_str(payload);
        return;
    }
    let Some(payload) = parse_json_payload::<SessionErrorPayload>(payload) else {
        return;
    };
    sender.push(SessionErrorEvent {
        description: payload.error_description,
    });
}

/// # Safety
/// Same contract as `session_running_cb`.
unsafe extern "C" fn session_interruption_cb(kind: i32, _payload: *mut c_char, ctx: *mut c_void) {
    let Some(sender) = sender_from_ctx::<InterruptionEvent>(ctx) else {
        return;
    };
    let kind = match kind {
        0 => InterruptionKind::Interrupted,
        1 => InterruptionKind::InterruptionEnded,
        _ => return,
    };
    sender.push(InterruptionEvent { kind });
}

/// # Safety
/// Called from the capture dispatch queue (see `VideoSampleStreamBridge`).
/// `sample_buffer` is a `CMSampleBufferRef` at +1 retain (`passRetained`);
/// `pixel_buffer` is a `CVPixelBufferRef` at +1 retain, or null.
/// Both are consumed (released) by the `CMSampleBuffer`/`CVPixelBuffer` drop
/// impls, either immediately (early returns) or when the event is eventually
/// popped or displaced from the `BoundedAsyncStream` ring buffer.
unsafe extern "C" fn video_sample_cb(
    ctx: *mut c_void,
    sample_buffer: *mut c_void,
    pixel_buffer: *mut c_void,
) {
    let sample = CMSampleBuffer::from_raw(sample_buffer);
    let pixel = CVPixelBuffer::from_raw(pixel_buffer);
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

/// # Safety
/// Same as `video_sample_cb` but audio-only. `sample_buffer` is a
/// `CMSampleBufferRef` at +1 retain.
unsafe extern "C" fn audio_sample_cb(ctx: *mut c_void, sample_buffer: *mut c_void) {
    let sample = CMSampleBuffer::from_raw(sample_buffer);
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
unsafe extern "C" fn file_output_sample_buffer_cb(ctx: *mut c_void, sample_buffer: *mut c_void) {
    let sample = CMSampleBuffer::from_raw(sample_buffer);
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
    let Some(kind) = file_recording_kind(kind) else {
        let _ = take_json_str(payload);
        return;
    };
    let Some(payload) = parse_json_payload::<FileRecordingPayload>(payload) else {
        return;
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
        let _ = take_json_str(payload);
        return;
    }
    let Some(payload) = parse_json_payload::<MetadataObjectsPayload>(payload) else {
        return;
    };
    sender.push(MetadataObjectsStreamEvent {
        objects: payload.objects.into_iter().map(Into::into).collect(),
    });
}

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureSession`.
pub struct SessionRunningStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<SessionRunningEvent>,
    inner: BoundedAsyncStream<SessionRunningEvent>,
}

impl SessionRunningStream {
    /// Subscribes to `AVCaptureSession` updates with the given buffer capacity.
    pub fn subscribe(session: &crate::CaptureSession, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_session_running_subscribe(
                session.ptr,
                Some(session_running_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "session running stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_session_running),
            _sender_box: sender_box,
            inner,
        }
    }
}

impl_stream_common!(SessionRunningStream, SessionRunningEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureSession`.
pub struct SessionErrorStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<SessionErrorEvent>,
    inner: BoundedAsyncStream<SessionErrorEvent>,
}

impl SessionErrorStream {
    /// Subscribes to `AVCaptureSession` updates with the given buffer capacity.
    pub fn subscribe(session: &crate::CaptureSession, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_session_error_subscribe(
                session.ptr,
                Some(session_error_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "session error stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_session_error),
            _sender_box: sender_box,
            inner,
        }
    }
}

impl_stream_common!(SessionErrorStream, SessionErrorEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureSession`.
pub struct SessionInterruptionStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<InterruptionEvent>,
    inner: BoundedAsyncStream<InterruptionEvent>,
}

impl SessionInterruptionStream {
    /// Subscribes to `AVCaptureSession` updates with the given buffer capacity.
    pub fn subscribe(session: &crate::CaptureSession, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_session_interruption_subscribe(
                session.ptr,
                Some(session_interruption_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "session interruption stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_session_interruption),
            _sender_box: sender_box,
            inner,
        }
    }
}

impl_stream_common!(SessionInterruptionStream, InterruptionEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureVideoDataOutput`.
pub struct VideoSampleBufferStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<VideoSampleBufferEvent>,
    inner: BoundedAsyncStream<VideoSampleBufferEvent>,
}

impl VideoSampleBufferStream {
    /// Subscribes to `AVCaptureVideoDataOutput` updates with the given buffer capacity.
    pub fn subscribe(output: &crate::VideoDataOutput, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-video-stream").expect("queue label is valid");
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_video_sample_subscribe(
                output.ptr,
                queue_label.as_ptr(),
                Some(video_sample_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "video sample stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_video_sample),
            _sender_box: sender_box,
            inner,
        }
    }
}

impl_stream_common!(VideoSampleBufferStream, VideoSampleBufferEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureAudioDataOutput`.
pub struct AudioSampleBufferStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<AudioSampleBufferEvent>,
    inner: BoundedAsyncStream<AudioSampleBufferEvent>,
}

impl AudioSampleBufferStream {
    /// Subscribes to `AVCaptureAudioDataOutput` updates with the given buffer capacity.
    pub fn subscribe(output: &crate::AudioDataOutput, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-audio-stream").expect("queue label is valid");
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_audio_sample_subscribe(
                output.ptr,
                queue_label.as_ptr(),
                Some(audio_sample_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "audio sample stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_audio_sample),
            _sender_box: sender_box,
            inner,
        }
    }
}

impl_stream_common!(AudioSampleBufferStream, AudioSampleBufferEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureMovieFileOutput`.
pub struct FileRecordingStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<FileRecordingStreamEvent>,
    inner: BoundedAsyncStream<FileRecordingStreamEvent>,
}

impl FileRecordingStream {
    /// Starts `AVCaptureMovieFileOutput` event delivery and returns an async stream.
    pub fn start(
        output: &crate::MovieFileOutput,
        path: &Path,
        capacity: usize,
    ) -> Result<Self, AVCaptureError> {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let path = cstring(&path.to_string_lossy(), "movie file output path")?;
        let mut err: *mut c_char = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_file_recording_stream_start(
                output.ptr,
                path.as_ptr(),
                Some(file_recording_cb),
                ctx,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, stop_file_recording),
            _sender_box: sender_box,
            inner,
        })
    }
}

impl_stream_common!(FileRecordingStream, FileRecordingStreamEvent);

#[derive(Debug)]
/// Async stream of events sourced from `AVCaptureAudioFileOutput`.
pub struct AudioFileRecordingStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<FileRecordingStreamEvent>,
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
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let path = cstring(&path.to_string_lossy(), "audio file output path")?;
        let output_file_type = cstring(output_file_type, "audio file output type")?;
        let mut err: *mut c_char = std::ptr::null_mut();
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_audio_file_recording_stream_start(
                output.ptr,
                path.as_ptr(),
                output_file_type.as_ptr(),
                Some(file_recording_cb),
                ctx,
                &mut err,
            )
        };
        if handle_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OUTPUT_ERROR, err) });
        }
        Ok(Self {
            _handle: StreamHandle::new(handle_ptr, stop_audio_file_recording),
            _sender_box: sender_box,
            inner,
        })
    }
}

impl_stream_common!(AudioFileRecordingStream, FileRecordingStreamEvent);

#[derive(Debug)]
/// Async stream of file-output sample-buffer boundary events sourced from `AVCaptureMovieFileOutput`.
pub struct MovieFileSampleBufferBoundaryStream {
    _handle: StreamHandle,
    _sender_box: SenderBox<FileOutputSampleBufferEvent>,
    inner: BoundedAsyncStream<FileOutputSampleBufferEvent>,
}

impl MovieFileSampleBufferBoundaryStream {
    /// Subscribes to `AVCaptureMovieFileOutput` sample-buffer boundary callbacks.
    pub fn subscribe(output: &crate::MovieFileOutput, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_movie_file_boundary_subscribe(
                output.ptr,
                Some(file_output_sample_buffer_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "movie file boundary stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_movie_file_boundary),
            _sender_box: sender_box,
            inner,
        }
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
    _sender_box: SenderBox<FileOutputSampleBufferEvent>,
    inner: BoundedAsyncStream<FileOutputSampleBufferEvent>,
}

impl AudioFileSampleBufferBoundaryStream {
    /// Subscribes to `AVCaptureAudioFileOutput` sample-buffer boundary callbacks.
    pub fn subscribe(output: &crate::AudioFileOutput, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_audio_file_boundary_subscribe(
                output.ptr,
                Some(file_output_sample_buffer_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "audio file boundary stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_audio_file_boundary),
            _sender_box: sender_box,
            inner,
        }
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
    _sender_box: SenderBox<MetadataObjectsStreamEvent>,
    inner: BoundedAsyncStream<MetadataObjectsStreamEvent>,
}

impl MetadataObjectsStream {
    /// Subscribes to `AVCaptureMetadataOutput` updates with the given buffer capacity.
    pub fn subscribe(output: &crate::MetadataOutput, capacity: usize) -> Self {
        let (inner, sender_box, ctx) = stream_parts(capacity);
        let queue_label =
            CString::new("avcapture-async-metadata-stream").expect("queue label is valid");
        let handle_ptr = unsafe {
            ffi::async_stream::avcapture_metadata_objects_subscribe(
                output.ptr,
                queue_label.as_ptr(),
                Some(metadata_objects_cb),
                ctx,
            )
        };
        assert!(
            !handle_ptr.is_null(),
            "metadata objects stream subscribe failed"
        );
        Self {
            _handle: StreamHandle::new(handle_ptr, unsubscribe_metadata_objects),
            _sender_box: sender_box,
            inner,
        }
    }
}

impl_stream_common!(MetadataObjectsStream, MetadataObjectsStreamEvent);
