mod audio;
mod export;
mod metadata;
mod session;
mod storage;
mod workflow;

use session::RecordingSession;

fn main() {
    println!("NC-PoRe Recorder starting...");

    let session = RecordingSession::new("test-session-001");

    println!("{:?}", session);
}
