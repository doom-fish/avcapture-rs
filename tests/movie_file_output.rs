mod common;

use std::fs;

use avcapture::prelude::*;

#[test]
fn movie_file_output_smoke() -> common::TestResult {
    let output = MovieFileOutput::new()?;
    output.set_max_recorded_file_size(8 * 1024 * 1024);
    output.set_min_free_disk_space_limit(0);
    output.set_max_recorded_duration(output.max_recorded_duration()?);
    output.set_movie_fragment_interval(output.movie_fragment_interval()?);
    let info = output.info()?;
    assert!(!info.is_recording);
    assert_eq!(output.output_info()?.connection_count, info.connection_count);
    assert!(!output.callback_installed()?);

    let artifact_dir = std::env::current_dir()?.join("target").join("test-artifacts");
    fs::create_dir_all(&artifact_dir)?;
    let artifact_path = artifact_dir.join("movie-file-output-smoke.mov");
    let err = output
        .start_recording_with_handler(&artifact_path, |event| {
            eprintln!("unexpected movie recording callback: {event:?}");
        })
        .expect_err("disconnected movie output should refuse recording requests");
    assert!(matches!(
        err,
        AVCaptureError::OutputError(_) | AVCaptureError::OperationFailed(_)
    ));
    Ok(())
}
