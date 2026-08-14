mod application;
mod artifact;
mod audio;
mod export;
mod metadata;
mod persistence;
mod session;
mod storage;
mod workflow;

use application::RecorderApplication;
use artifact::RecordingArtifactAssociation;
use artifact::coordination::ArtifactCoordinator;
use artifact::processing::RecordingArtifactProcessor;
use audio::{CaptureProvider, CaptureResult};
use persistence::InMemoryPersistenceProvider;
use session::RecordingSession;

struct TestCaptureProvider {
    active: bool,
}

impl TestCaptureProvider {
    fn new() -> Self {
        Self { active: false }
    }
}

impl CaptureProvider for TestCaptureProvider {
    fn start_capture(&mut self) {
        self.active = true;
    }

    fn stop_capture(&mut self) -> CaptureResult {
        self.active = false;

        CaptureResult::new("application-test-capture")
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("inspect-audio") {
        match audio::inspect_default_input_device() {
            Ok(()) => {}
            Err(error) => {
                eprintln!("Audio-Inspektion fehlgeschlagen: {error}");
                std::process::exit(1);
            }
        }

        return;
    }

    println!("NC-PoRe Recorder starting...");

    let session = RecordingSession::new("test-session-001");

    let capture = TestCaptureProvider::new();

    let persistence = InMemoryPersistenceProvider::new();

    let coordinator = ArtifactCoordinator::new(persistence);

    let processor = RecordingArtifactProcessor::new(coordinator);

    let mut application = RecorderApplication::new(session, capture, processor);

    application.start();

    let _ = application.session();

    let _artifact = application.stop(RecordingArtifactAssociation::new(
        "production-test-001",
        "recording-test-001",
    ));

    println!("NC-PoRe Recorder flow completed.");
}
