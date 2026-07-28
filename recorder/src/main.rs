mod audio;
mod export;
mod metadata;
mod session;
mod storage;

use session::{RecordingSession, SessionStatus};

fn main() {
    println!("NC-PoRe Recorder starting...");

    let session = RecordingSession {
        id: String::from("test-session-001"),
        status: SessionStatus::Created,
    };

    println!("{:?}", session);
}
