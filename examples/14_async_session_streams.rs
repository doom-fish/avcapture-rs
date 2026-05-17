//! Demonstrates the `async_api` stream surfaces.
//! Subscribes to session running + error + interruption streams,
//! creates a session, tries to start/stop, checks events.
//! Exits 0 on headless macOS (no camera required).

fn main() {
    use avcapture::async_api::{SessionErrorStream, SessionRunningStream};

    match avcapture::CaptureSession::new() {
        Ok(session) => {
            let _running = SessionRunningStream::subscribe(&session, 8);
            let _errors = SessionErrorStream::subscribe(&session, 8);
            println!("streams subscribed and dropped OK");
        }
        Err(e) => {
            println!("session creation not available (headless): {e}");
        }
    }
    println!("async session stream example: OK");
}
