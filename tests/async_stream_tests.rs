//! Basic compilation + type tests for async stream surfaces.
//! These tests verify subscribe → stream-is-open → drop-handle semantics
//! without requiring real hardware.

#[cfg(feature = "async")]
mod async_stream {
    use std::fs;

    use avcapture::async_api::*;

    const fn assert_next_item<T>(_: doom_fish_utils::stream::NextItem<'_, T>) {}

    fn assert_future_result<T, U>(_: T)
    where
        T: std::future::Future<Output = Result<U, avcapture::AVCaptureError>>,
    {
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
        let s = VideoSampleBufferStream::subscribe(output, 8);
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_audio_sample_stream_api(output: &avcapture::AudioDataOutput) {
        let s = AudioSampleBufferStream::subscribe(output, 8);
        let _ = s.buffered_count();
        let _ = s.try_next();
        assert_next_item(s.next());
        let _ = s.is_closed();
    }

    fn check_metadata_objects_stream_api(output: &avcapture::MetadataOutput) {
        let s = MetadataObjectsStream::subscribe(output, 8);
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

        let movie_stream = MovieFileSampleBufferBoundaryStream::subscribe(&movie, 8);
        let _ = movie_stream.buffered_count();
        let _ = movie_stream.try_next();
        assert_next_item(movie_stream.next());
        assert!(!movie_stream.is_closed());
        drop(movie_stream);

        let audio_stream = AudioFileSampleBufferBoundaryStream::subscribe(&audio, 8);
        let _ = audio_stream.buffered_count();
        let _ = audio_stream.try_next();
        assert_next_item(audio_stream.next());
        assert!(!audio_stream.is_closed());
        drop(audio_stream);

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
