mod audio;
mod session;
mod metadata;
mod storage;
mod export;

use session::{RecordingSession, SessionStatus};

fn main() {
    println!("NC-PoRe Recorder starting...");

    let session = RecordingSession {
        id: String::from("test-session-001"),
        status: SessionStatus::Created,
    };

    println!("{:?}", session);
}
