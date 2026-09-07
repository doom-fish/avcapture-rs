//! Basic compilation + type tests for async stream surfaces.
//! These tests verify subscribe → stream-is-open → drop-handle semantics
//! without requiring real hardware.

#[cfg(feature = "async")]
mod async_stream {
    use core::ffi::{c_char, c_void};
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use avcapture::async_api::*;

    unsafe extern "C" fn legacy_video_callback(
        _userdata: *mut c_void,
        _sample_buffer: *mut c_void,
        _pixel_buffer: *mut c_void,
    ) {
    }

    unsafe extern "C" fn video_event_callback(
        _userdata: *mut c_void,
        _kind: i32,
        _sample_buffer: *mut c_void,
        _pixel_buffer: *mut c_void,
        _dropped_reason: *mut c_char,
        _dropped_total: u64,
    ) {
    }

    unsafe extern "C" fn audio_sample_callback(
        _userdata: *mut c_void,
        _sample_buffer: *mut c_void,
    ) {
    }

    unsafe extern "C" fn stream_event_callback(
        _kind: i32,
        payload: *mut c_char,
        _ctx: *mut c_void,
    ) {
        if !payload.is_null() {
            unsafe { avcapture::ffi::core::avc_string_free(payload) };
        }
    }

    unsafe fn take_error(error: *mut c_char) -> String {
        if error.is_null() {
            return String::new();
        }
        let message = unsafe { std::ffi::CStr::from_ptr(error) }
            .to_string_lossy()
            .into_owned();
        unsafe { avcapture::ffi::core::avc_string_free(error) };
        message
    }

    unsafe extern "C" fn count_context_drop(context: *mut c_void) {
        let counter = unsafe { &*context.cast::<AtomicUsize>() };
        counter.fetch_add(1, Ordering::SeqCst);
    }

    const fn assert_next_item<T>(_: doom_fish_utils::stream::NextItem<'_, T>) {}

    fn assert_future_result<T, U>(_: T)
    where
        T: std::future::Future<Output = Result<U, avcapture::AVCaptureError>>,
    {
    }

    fn normalize_type(value: &str) -> String {
        value
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect()
    }

    fn source_signature(
        source: &str,
        marker: &str,
        symbol: &str,
        body_delimiter: char,
    ) -> (String, String, usize) {
        let marker = format!("{marker}{symbol}(");
        let start = source
            .find(&marker)
            .unwrap_or_else(|| panic!("missing signature for {symbol}"));
        let open = start + marker.len() - 1;
        let mut depth = 0_usize;
        let mut close = None;
        for (offset, character) in source[open..].char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(open + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let close = close.unwrap_or_else(|| panic!("unterminated signature for {symbol}"));
        let parameters = source[open + 1..close]
            .split(',')
            .filter_map(|parameter| {
                parameter
                    .split_once(':')
                    .map(|(_, value)| normalize_type(value))
            })
            .collect::<Vec<_>>()
            .join(",");
        let suffix = source[close + 1..]
            .split(body_delimiter)
            .next()
            .expect("signature suffix");
        let return_type = suffix
            .split_once("->")
            .map_or_else(|| "()".to_owned(), |(_, value)| normalize_type(value));
        (parameters, return_type, start)
    }

    fn check_session_running_stream_api(session: &avcapture::CaptureSession) {
        let s = SessionRunningStream::subscribe(session, 8);
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_session_error_stream_api(session: &avcapture::CaptureSession) {
        let s = SessionErrorStream::subscribe(session, 8);
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_session_interruption_stream_api(session: &avcapture::CaptureSession) {
        let s = SessionInterruptionStream::subscribe(session, 8);
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_video_sample_stream_api(output: &avcapture::VideoDataOutput) {
        let s: VideoSampleBufferStream = VideoSampleBufferStream::subscribe(output, 8);
        let _ = s.buffered_count();
        if let Some(event) = s.try_next() {
            let _sample = event.sample_buffer;
            let _pixel = event.pixel_buffer;
        }
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn assert_legacy_video_event_fields(event: VideoSampleBufferEvent) {
        let _sample = event.sample_buffer;
        let _pixel = event.pixel_buffer;
    }

    fn check_audio_sample_stream_api(output: &avcapture::AudioDataOutput) {
        let Ok(s) = AudioSampleBufferStream::subscribe(output, 8) else {
            return;
        };
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_metadata_objects_stream_api(output: &avcapture::MetadataOutput) {
        let Ok(s) = MetadataObjectsStream::subscribe(output, 8) else {
            return;
        };
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_photo_capture_future_api(
        output: &avcapture::PhotoOutput,
    ) -> Result<(), avcapture::AVCaptureError> {
        let settings = avcapture::PhotoSettings::new()?;
        if let Ok(future) = PhotoCaptureResultFuture::start(output) {
            assert_future_result(future);
        }
        if let Ok(future) = PhotoCaptureResultFuture::start_with_settings(output, &settings) {
            assert_future_result(future);
        }
        if let Ok(future) = PhotoCaptureEventFuture::start(output) {
            assert_future_result(future);
        }
        if let Ok(future) = PhotoCaptureEventFuture::start_with_settings(output, &settings) {
            assert_future_result(future);
        }
        Ok(())
    }

    fn check_photo_readiness_stream_api(stream: &PhotoCaptureReadinessStream) {
        let _ = stream.coordinator();
        let _ = stream.buffered_count();
        let _ = stream.try_next();
        assert_next_item(stream.next());
        let _ = stream.is_closed();
    }

    #[test]
    fn session_running_stream_closes_on_drop() {
        let Ok(session) = avcapture::CaptureSession::new() else {
            println!("skip: no session available");
            return;
        };
        check_session_running_stream_api(&session);
        let stream = SessionRunningStream::subscribe(&session, 8);
        assert!(!stream.is_closed(), "stream should be open after subscribe");
        drop(stream);
    }

    #[test]
    fn session_error_stream_closes_on_drop() {
        let Ok(session) = avcapture::CaptureSession::new() else {
            println!("skip: no session available");
            return;
        };
        check_session_error_stream_api(&session);
        let stream = SessionErrorStream::subscribe(&session, 8);
        assert!(!stream.is_closed(), "stream should be open after subscribe");
        drop(stream);
    }

    #[test]
    fn session_interruption_stream_closes_on_drop() {
        let Ok(session) = avcapture::CaptureSession::new() else {
            println!("skip: no session available");
            return;
        };
        check_session_interruption_stream_api(&session);
        let stream = SessionInterruptionStream::subscribe(&session, 8);
        assert!(!stream.is_closed(), "stream should be open after subscribe");
        drop(stream);
    }

    #[test]
    fn sample_streams_compile_and_drop() -> Result<(), Box<dyn std::error::Error>> {
        let video = avcapture::VideoDataOutput::new()?;
        let audio = avcapture::AudioDataOutput::new()?;
        check_video_sample_stream_api(&video);
        check_audio_sample_stream_api(&audio);

        if let Ok(metadata) = avcapture::MetadataOutput::new() {
            check_metadata_objects_stream_api(&metadata);
        }

        Ok(())
    }

    #[test]
    fn file_recording_stream_requires_attached_output() -> Result<(), Box<dyn std::error::Error>> {
        let output = avcapture::MovieFileOutput::new()?;
        let artifact_dir = std::env::current_dir()?
            .join("target")
            .join("test-artifacts");
        fs::create_dir_all(&artifact_dir)?;
        let artifact_path = artifact_dir.join("async-file-recording-stream.mov");
        let Err(err) = FileRecordingStream::start(&output, &artifact_path, 8) else {
            panic!("disconnected movie output should refuse recording requests");
        };
        assert!(matches!(
            err,
            avcapture::AVCaptureError::OutputError(_)
                | avcapture::AVCaptureError::OperationFailed(_)
        ));
        Ok(())
    }

    #[test]
    fn audio_file_recording_stream_requires_attached_output(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = avcapture::AudioFileOutput::new()?;
        let artifact_dir = std::env::current_dir()?
            .join("target")
            .join("test-artifacts");
        fs::create_dir_all(&artifact_dir)?;
        let artifact_path = artifact_dir.join("async-audio-file-recording-stream.caf");
        let output_type = output
            .available_output_file_types()?
            .into_iter()
            .next()
            .ok_or("expected an available audio output type")?;
        let Err(err) = AudioFileRecordingStream::start(&output, &artifact_path, &output_type, 8)
        else {
            panic!("disconnected audio output should refuse recording requests");
        };
        assert!(matches!(
            err,
            avcapture::AVCaptureError::OutputError(_)
                | avcapture::AVCaptureError::OperationFailed(_)
        ));
        Ok(())
    }

    #[test]
    fn file_output_boundary_streams_compile_and_drop() -> Result<(), Box<dyn std::error::Error>> {
        let movie = avcapture::MovieFileOutput::new()?;
        let audio = avcapture::AudioFileOutput::new()?;

        let movie_stream = MovieFileSampleBufferBoundaryStream::subscribe(&movie, 8)?;
        let _ = movie_stream.buffered_count();
        let _ = movie_stream.try_next();
        assert_next_item(movie_stream.next());
        assert!(!movie_stream.is_closed());
        drop(movie_stream);

        let audio_stream = AudioFileSampleBufferBoundaryStream::subscribe(&audio, 8)?;
        let _ = audio_stream.buffered_count();
        let _ = audio_stream.try_next();
        assert_next_item(audio_stream.next());
        assert!(!audio_stream.is_closed());
        drop(audio_stream);

        Ok(())
    }

    #[test]
    fn delegate_slots_reject_competing_sync_and_async_registrations(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let video = avcapture::VideoDataOutput::new()?;
        video.set_sample_buffer_handler(None, |_sample, _pixel| {})?;
        let video_error = VideoSampleBufferStream::try_subscribe(&video, 2)
            .expect_err("video stream must not replace a sync delegate");
        assert!(matches!(
            video_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        let event_error = video
            .set_sample_buffer_event_handler(None, |_event| {})
            .expect_err("event handler must not replace a legacy sync delegate");
        assert!(matches!(
            event_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        video.clear_sample_buffer_handler();

        video.set_sample_buffer_event_handler(None, |_event| {})?;
        let legacy_error = video
            .set_sample_buffer_handler(None, |_sample, _pixel| {})
            .expect_err("legacy handler must not replace an event handler");
        assert!(matches!(
            legacy_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        video.clear_sample_buffer_event_handler();

        let video_stream = VideoSampleBufferStream::try_subscribe(&video, 2)?;
        let event_stream_error = VideoDataOutputEventStream::subscribe(&video, 2)
            .expect_err("event stream must not replace a legacy stream");
        assert!(matches!(
            event_stream_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        let video_error = video
            .set_sample_buffer_handler(None, |_sample, _pixel| {})
            .expect_err("sync video handler must not replace a stream delegate");
        assert!(matches!(
            video_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        drop(video_stream);
        video.set_sample_buffer_handler(None, |_sample, _pixel| {})?;
        video.clear_sample_buffer_handler();

        let video_event_stream = VideoDataOutputEventStream::subscribe(&video, 2)?;
        let legacy_stream_error = VideoSampleBufferStream::try_subscribe(&video, 2)
            .expect_err("legacy stream must not replace an event stream");
        assert!(matches!(
            legacy_stream_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        assert_next_item(video_event_stream.next());
        drop(video_event_stream);

        let audio = avcapture::AudioDataOutput::new()?;
        audio.set_sample_buffer_handler(None, |_sample| {})?;
        let audio_error = AudioSampleBufferStream::subscribe(&audio, 2)
            .expect_err("audio stream must not replace a sync delegate");
        assert!(matches!(
            audio_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        audio.clear_sample_buffer_handler();

        let movie = avcapture::MovieFileOutput::new()?;
        movie.set_sample_buffer_boundary_handler(|_sample| {})?;
        let boundary_error = MovieFileSampleBufferBoundaryStream::subscribe(&movie, 2)
            .expect_err("boundary stream must not replace a sync delegate");
        assert!(matches!(
            boundary_error,
            avcapture::AVCaptureError::DelegateSlotOccupied(_)
        ));
        movie.clear_sample_buffer_boundary_handler();

        if let Ok(metadata) = avcapture::MetadataOutput::new() {
            metadata.set_metadata_objects_handler(None, |_event| {})?;
            let metadata_error = MetadataObjectsStream::subscribe(&metadata, 2)
                .expect_err("metadata stream must not replace a sync delegate");
            assert!(matches!(
                metadata_error,
                avcapture::AVCaptureError::DelegateSlotOccupied(_)
            ));
            metadata.clear_metadata_objects_handler();
        }

        Ok(())
    }

    #[test]
    fn legacy_video_sample_event_source_shape_is_preserved() {
        let _: fn(VideoSampleBufferEvent) = assert_legacy_video_event_fields;
        let _: fn(&avcapture::VideoDataOutput) = check_video_sample_stream_api;
    }

    #[test]
    fn async_export_abi_signatures_are_stable_and_additive() {
        let _: avcapture::ffi::VideoSampleCallback = legacy_video_callback;
        let _: avcapture::ffi::VideoDataOutputEventCallback = video_event_callback;

        let _: unsafe extern "C" fn(
            *mut c_void,
            *const c_char,
            Option<avcapture::ffi::VideoSampleCallback>,
            *mut c_void,
        ) -> *mut c_void = avcapture::ffi::async_stream::avcapture_video_sample_subscribe;

        let _: unsafe extern "C" fn(
            *mut c_void,
            *const c_char,
            Option<avcapture::ffi::VideoSampleCallback>,
            *mut c_void,
            Option<avcapture::ffi::RetainCallback>,
            Option<avcapture::ffi::DropCallback>,
            *mut *mut c_char,
        ) -> i32 =
            avcapture::ffi::video_data_output::av_capture_video_output_set_sample_buffer_callback;

        let _: unsafe extern "C" fn(
            *mut c_void,
            *const c_char,
            Option<avcapture::ffi::VideoDataOutputEventCallback>,
            *mut c_void,
            Option<avcapture::ffi::RetainCallback>,
            Option<avcapture::ffi::DropCallback>,
            *mut *mut c_char,
        ) -> i32 = avcapture::ffi::video_data_output::
            av_capture_video_output_set_sample_buffer_event_callback;

        let _: unsafe extern "C" fn(
            *mut c_void,
            *const c_char,
            Option<avcapture::ffi::VideoDataOutputEventCallback>,
            *mut c_void,
            Option<avcapture::ffi::DropCallback>,
            *mut i32,
            *mut *mut c_char,
        ) -> *mut c_void =
            avcapture::ffi::async_stream::avcapture_video_data_output_event_subscribe;

        let mut error = core::ptr::null_mut();
        let output = unsafe {
            avcapture::ffi::video_data_output::av_capture_video_output_create(&mut error)
        };
        assert!(error.is_null());
        assert!(!output.is_null());
        let queue = std::ffi::CString::new("avcapture-legacy-video-abi")
            .expect("queue label should not contain NUL");

        let status = unsafe {
            avcapture::ffi::video_data_output::av_capture_video_output_set_sample_buffer_callback(
                output,
                queue.as_ptr(),
                Some(legacy_video_callback),
                core::ptr::null_mut(),
                None,
                None,
                &mut error,
            )
        };
        assert_eq!(status, avcapture::ffi::status::OK);
        assert!(error.is_null());
        unsafe {
            avcapture::ffi::video_data_output::av_capture_video_output_clear_sample_buffer_callback(
                output,
            );
        }

        fn every_legacy_async_export_matches_the_head_signature_inventory() {
            let rust_source = include_str!("../src/ffi/async_stream.rs");
            let swift_source =
                include_str!("../swift-bridge/Sources/AVCaptureBridge/AsyncStream.swift");
            let inventory = include_str!("fixtures/async_stream_legacy_abi.txt");
            let mut checked = 0_usize;

            for line in inventory
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
            {
                let columns = line.split('|').collect::<Vec<_>>();
                assert_eq!(columns.len(), 5, "invalid ABI inventory row: {line}");
                let symbol = columns[0];
                let (rust_parameters, rust_return, _) =
                    source_signature(rust_source, "pub fn ", symbol, ';');
                assert_eq!(
                    rust_parameters, columns[1],
                    "Rust parameters changed for {symbol}"
                );
                assert_eq!(
                    rust_return, columns[2],
                    "Rust return type changed for {symbol}"
                );

                let (swift_parameters, swift_return, swift_start) =
                    source_signature(swift_source, "public func ", symbol, '{');
                assert_eq!(
                    swift_parameters, columns[3],
                    "Swift parameters changed for {symbol}"
                );
                assert_eq!(
                    swift_return, columns[4],
                    "Swift return type changed for {symbol}"
                );
                let cdecl_start = swift_start.saturating_sub(100);
                assert!(
                    swift_source[cdecl_start..swift_start]
                        .contains(&format!("@_cdecl(\"{symbol}\")")),
                    "Swift export name changed for {symbol}"
                );
                checked += 1;
            }

            assert_eq!(checked, 20);
        }

        fn representative_legacy_async_exports_are_runtime_callable(
        ) -> Result<(), Box<dyn std::error::Error>> {
            let mut error = core::ptr::null_mut();
            let mut context = 0_u8;
            let context = core::ptr::from_mut(&mut context).cast();

            let session = unsafe { avcapture::ffi::session::av_capture_session_create(&mut error) };
            assert!(!session.is_null(), "{}", unsafe { take_error(error) });
            let running = unsafe {
                avcapture::ffi::async_stream::avcapture_session_running_subscribe(
                    session,
                    Some(stream_event_callback),
                    context,
                )
            };
            let errors = unsafe {
                avcapture::ffi::async_stream::avcapture_session_error_subscribe(
                    session,
                    Some(stream_event_callback),
                    context,
                )
            };
            let interruptions = unsafe {
                avcapture::ffi::async_stream::avcapture_session_interruption_subscribe(
                    session,
                    Some(stream_event_callback),
                    context,
                )
            };
            assert!(!running.is_null());
            assert!(!errors.is_null());
            assert!(!interruptions.is_null());
            unsafe {
                avcapture::ffi::async_stream::avcapture_session_running_unsubscribe(running);
                avcapture::ffi::async_stream::avcapture_session_error_unsubscribe(errors);
                avcapture::ffi::async_stream::avcapture_session_interruption_unsubscribe(
                    interruptions,
                );
                avcapture::ffi::session::av_capture_session_release(session);
            }

            let queue = std::ffi::CString::new("avcapture-legacy-async-abi")?;
            let audio = unsafe {
                avcapture::ffi::audio_data_output::av_capture_audio_output_create(&mut error)
            };
            assert!(!audio.is_null(), "{}", unsafe { take_error(error) });
            let audio_stream = unsafe {
                avcapture::ffi::async_stream::avcapture_audio_sample_subscribe(
                    audio,
                    queue.as_ptr(),
                    Some(audio_sample_callback),
                    context,
                )
            };
            assert!(!audio_stream.is_null());
            unsafe {
                avcapture::ffi::async_stream::avcapture_audio_sample_unsubscribe(audio_stream);
                avcapture::ffi::audio_data_output::av_capture_audio_output_release(audio);
            }

            let movie = unsafe {
                avcapture::ffi::movie_file_output::av_capture_movie_file_output_create(&mut error)
            };
            assert!(!movie.is_null(), "{}", unsafe { take_error(error) });
            let movie_boundary = unsafe {
                avcapture::ffi::async_stream::avcapture_movie_file_boundary_subscribe(
                    movie,
                    Some(audio_sample_callback),
                    context,
                )
            };
            assert!(!movie_boundary.is_null());
            unsafe {
                avcapture::ffi::async_stream::avcapture_movie_file_boundary_unsubscribe(
                    movie_boundary,
                );
            }

            let audio_file = unsafe {
                avcapture::ffi::movie_file_output::av_capture_audio_file_output_create(&mut error)
            };
            assert!(!audio_file.is_null(), "{}", unsafe { take_error(error) });
            let audio_boundary = unsafe {
                avcapture::ffi::async_stream::avcapture_audio_file_boundary_subscribe(
                    audio_file,
                    Some(audio_sample_callback),
                    context,
                )
            };
            assert!(!audio_boundary.is_null());
            unsafe {
                avcapture::ffi::async_stream::avcapture_audio_file_boundary_unsubscribe(
                    audio_boundary,
                );
            }

            let artifact_dir = std::env::current_dir()?
                .join("target")
                .join("test-artifacts");
            std::fs::create_dir_all(&artifact_dir)?;
            let movie_path =
                artifact_dir.join(format!("legacy-async-movie-{}.mov", std::process::id()));
            let audio_path =
                artifact_dir.join(format!("legacy-async-audio-{}.caf", std::process::id()));
            for path in [&movie_path, &audio_path] {
                if path.is_file() {
                    std::fs::remove_file(path)?;
                }
            }
            let movie_path = std::ffi::CString::new(movie_path.to_string_lossy().into_owned())?;
            let movie_recording = unsafe {
                avcapture::ffi::async_stream::avcapture_file_recording_stream_start(
                    movie,
                    movie_path.as_ptr(),
                    Some(stream_event_callback),
                    context,
                    &mut error,
                )
            };
            assert!(movie_recording.is_null());
            assert!(!unsafe { take_error(error) }.is_empty());
            error = core::ptr::null_mut();

            let audio_path = std::ffi::CString::new(audio_path.to_string_lossy().into_owned())?;
            let output_type = std::ffi::CString::new("public.caf")?;
            let audio_recording = unsafe {
                avcapture::ffi::async_stream::avcapture_audio_file_recording_stream_start(
                    audio_file,
                    audio_path.as_ptr(),
                    output_type.as_ptr(),
                    Some(stream_event_callback),
                    context,
                    &mut error,
                )
            };
            assert!(audio_recording.is_null());
            assert!(!unsafe { take_error(error) }.is_empty());
            error = core::ptr::null_mut();

            unsafe {
                avcapture::ffi::movie_file_output::av_capture_movie_file_output_release(movie);
                avcapture::ffi::movie_file_output::av_capture_audio_file_output_release(audio_file);
            }

            let metadata = unsafe {
                avcapture::ffi::metadata_output::av_capture_metadata_output_create(&mut error)
            };
            if metadata.is_null() {
                let _ = unsafe { take_error(error) };
            } else {
                let metadata_stream = unsafe {
                    avcapture::ffi::async_stream::avcapture_metadata_objects_subscribe(
                        metadata,
                        queue.as_ptr(),
                        Some(stream_event_callback),
                        context,
                    )
                };
                assert!(!metadata_stream.is_null());
                unsafe {
                    avcapture::ffi::async_stream::avcapture_metadata_objects_unsubscribe(
                        metadata_stream,
                    );
                    avcapture::ffi::metadata_output::av_capture_metadata_output_release(metadata);
                }
            }

            Ok(())
        }

        let status = unsafe {
            avcapture::ffi::video_data_output::
                av_capture_video_output_set_sample_buffer_event_callback(
                    output,
                    queue.as_ptr(),
                    Some(video_event_callback),
                    core::ptr::null_mut(),
                    None,
                    None,
                    &mut error,
                )
        };
        assert_eq!(status, avcapture::ffi::status::OK);
        assert!(error.is_null());
        unsafe {
            avcapture::ffi::video_data_output::
                av_capture_video_output_clear_sample_buffer_event_callback(output);
        }

        let mut legacy_context = 0_u8;
        let legacy_handle = unsafe {
            avcapture::ffi::async_stream::avcapture_video_sample_subscribe(
                output,
                queue.as_ptr(),
                Some(legacy_video_callback),
                core::ptr::from_mut(&mut legacy_context).cast(),
            )
        };
        assert!(!legacy_handle.is_null());
        unsafe {
            avcapture::ffi::async_stream::avcapture_video_sample_unsubscribe(legacy_handle);
        }

        let mut event_context = 0_u8;
        let mut event_status = avcapture::ffi::status::OK;
        let event_handle = unsafe {
            avcapture::ffi::async_stream::avcapture_video_data_output_event_subscribe(
                output,
                queue.as_ptr(),
                Some(video_event_callback),
                core::ptr::from_mut(&mut event_context).cast(),
                None,
                &mut event_status,
                &mut error,
            )
        };
        assert_eq!(event_status, avcapture::ffi::status::OK);
        assert!(!event_handle.is_null());
        assert!(error.is_null());
        unsafe {
            avcapture::ffi::async_stream::avcapture_video_data_output_event_unsubscribe(
                event_handle,
            );
            avcapture::ffi::video_data_output::av_capture_video_output_release(output);
        }

        every_legacy_async_export_matches_the_head_signature_inventory();
        representative_legacy_async_exports_are_runtime_callable()
            .expect("legacy async exports should remain runtime callable");
    }

    #[test]
    fn owned_recording_preconstruction_failures_release_context_once(
    ) -> Result<(), Box<dyn std::error::Error>> {
        fn assert_movie_failure(output: *mut c_void, path: &[u8], path_length: usize, policy: i32) {
            let drops = AtomicUsize::new(0);
            let mut status = avcapture::ffi::status::OK;
            let mut error = core::ptr::null_mut();
            let handle = unsafe {
                avcapture::ffi::async_stream::
                    avcapture_file_recording_stream_start_with_options_owned(
                        output,
                        path.as_ptr(),
                        path_length,
                        policy,
                        Some(stream_event_callback),
                        core::ptr::from_ref(&drops).cast_mut().cast(),
                        Some(count_context_drop),
                        &mut status,
                        &mut error,
                    )
            };
            assert!(handle.is_null());
            assert_eq!(status, avcapture::ffi::status::INVALID_ARGUMENT);
            assert!(!unsafe { take_error(error) }.is_empty());
            assert_eq!(drops.load(Ordering::SeqCst), 1);
        }

        fn assert_audio_failure(
            output: *mut c_void,
            path: &[u8],
            path_length: usize,
            policy: i32,
            output_type: &std::ffi::CStr,
        ) {
            let drops = AtomicUsize::new(0);
            let mut status = avcapture::ffi::status::OK;
            let mut error = core::ptr::null_mut();
            let handle = unsafe {
                avcapture::ffi::async_stream::
                    avcapture_audio_file_recording_stream_start_with_options_owned(
                        output,
                        path.as_ptr(),
                        path_length,
                        output_type.as_ptr(),
                        policy,
                        Some(stream_event_callback),
                        core::ptr::from_ref(&drops).cast_mut().cast(),
                        Some(count_context_drop),
                        &mut status,
                        &mut error,
                    )
            };
            assert!(handle.is_null());
            assert_eq!(status, avcapture::ffi::status::INVALID_ARGUMENT);
            assert!(!unsafe { take_error(error) }.is_empty());
            assert_eq!(drops.load(Ordering::SeqCst), 1);
        }

        let mut error = core::ptr::null_mut();
        let movie = unsafe {
            avcapture::ffi::movie_file_output::av_capture_movie_file_output_create(&mut error)
        };
        assert!(!movie.is_null(), "{}", unsafe { take_error(error) });
        let audio = unsafe {
            avcapture::ffi::movie_file_output::av_capture_audio_file_output_create(&mut error)
        };
        assert!(!audio.is_null(), "{}", unsafe { take_error(error) });

        let valid_path = b"/tmp/avcapture-owned-recording-validation.mov";
        let empty_path = [0_u8];
        let output_type = std::ffi::CString::new("public.caf")?;
        assert_movie_failure(movie, valid_path, valid_path.len(), 99);
        assert_movie_failure(movie, &empty_path, 0, 0);
        assert_audio_failure(audio, valid_path, valid_path.len(), 99, &output_type);
        assert_audio_failure(audio, &empty_path, 0, 0, &output_type);

        unsafe {
            avcapture::ffi::movie_file_output::av_capture_movie_file_output_release(movie);
            avcapture::ffi::movie_file_output::av_capture_audio_file_output_release(audio);
        }
        Ok(())
    }

    #[test]
    fn photo_async_wrappers_require_attached_output() -> Result<(), Box<dyn std::error::Error>> {
        let output = avcapture::PhotoOutput::new()?;
        check_photo_capture_future_api(&output)?;

        let result_future: Result<PhotoCaptureResultFuture, avcapture::AVCaptureError> =
            PhotoCaptureResultFuture::start(&output);
        assert!(matches!(
            result_future,
            Err(avcapture::AVCaptureError::OutputError(_)
                | avcapture::AVCaptureError::OperationFailed(_))
        ));

        let settings = avcapture::PhotoSettings::new()?;
        let event_future: Result<PhotoCaptureEventFuture, avcapture::AVCaptureError> =
            PhotoCaptureEventFuture::start_with_settings(&output, &settings);
        assert!(matches!(
            event_future,
            Err(avcapture::AVCaptureError::OutputError(_)
                | avcapture::AVCaptureError::OperationFailed(_))
        ));

        let readiness_stream: Result<PhotoCaptureReadinessStream, avcapture::AVCaptureError> =
            PhotoCaptureReadinessStream::subscribe(&output, 8);
        assert!(matches!(
            readiness_stream,
            Err(avcapture::AVCaptureError::OutputError(_)
                | avcapture::AVCaptureError::OperationFailed(_))
        ));

        let _: fn(&PhotoCaptureReadinessStream) = check_photo_readiness_stream_api;
        Ok(())
    }
}
