//! Basic compilation + type tests for async stream surfaces.
//! These tests verify subscribe → stream-is-open → drop-handle semantics
//! without requiring real hardware.

#[cfg(feature = "async")]
mod async_stream {
    use std::fs;

    use avcapture::async_api::*;

    const fn assert_next_item<T>(_: doom_fish_utils::stream::NextItem<'_, T>) {}

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
}
