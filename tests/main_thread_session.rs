use std::time::{Duration, Instant};

use avcapture::prelude::*;

fn wait_for_running_state(session: &CaptureSession, expected: bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if session.is_running().expect("session info should decode") == expected {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false
}

fn main() {
    let session = CaptureSession::new().expect("session should be created");

    session
        .start_running()
        .expect("scheduling a start from the main thread should succeed");
    assert!(
        wait_for_running_state(&session, true),
        "the start scheduled from the main thread never ran"
    );

    session.begin_configuration();
    assert!(matches!(
        session.start_running(),
        Err(AVCaptureError::InvalidState(_))
    ));
    assert!(matches!(
        session.stop_running(),
        Err(AVCaptureError::InvalidState(_))
    ));
    session
        .commit_configuration()
        .expect("the matching commit should succeed");

    session
        .stop_running()
        .expect("scheduling a stop from the main thread should succeed");
    assert!(
        wait_for_running_state(&session, false),
        "the stop scheduled from the main thread never ran"
    );

    session
        .start_running()
        .expect("scheduling a second start should succeed");
    drop(session);

    println!("main_thread_session: 1 passed");
}
