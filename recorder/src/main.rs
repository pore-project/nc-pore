use recorder::application::RecorderApplication;
use recorder::artifact::RecordingArtifactAssociation;
use recorder::artifact::coordination::ArtifactCoordinator;
use recorder::artifact::processing::RecordingArtifactProcessor;
use recorder::audio::{CpalCaptureProvider, RecordingConfiguration};
use recorder::persistence::InMemoryPersistenceProvider;
use recorder::session::RecordingSession;

fn main() {
    if std::env::args().nth(1).as_deref() == Some("test-audio-stream") {
        return recorder::audio::test_input_stream().unwrap_or_else(|error| {
            eprintln!("Audio-Stream-Test fehlgeschlagen: {error}");
            std::process::exit(1);
        });
    }

    if std::env::args().nth(1).as_deref() == Some("inspect-audio") {
        match recorder::audio::inspect_default_input_device() {
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

    let capture = CpalCaptureProvider::new();

    let persistence = InMemoryPersistenceProvider::new();

    let coordinator = ArtifactCoordinator::new(persistence);

    let processor = RecordingArtifactProcessor::new(coordinator);

    let mut application = RecorderApplication::new(session, capture, processor);
    let configuration = RecordingConfiguration::default();

    if let Err(error) = application.start(&configuration) {
        eprintln!("Aufnahme konnte nicht gestartet werden: {error:?}");
        std::process::exit(1);
    }

    let _ = application.session();

    let artifact = application.stop(RecordingArtifactAssociation::new(
        "production-test-001",
        "recording-test-001",
    ));

    let artifact = artifact.expect("RecordingArtifact konnte nicht erzeugt werden.");

    println!(
        "RecordingArtifact erzeugt: {} Track(s)",
        artifact.tracks().len()
    );

    println!("NC-PoRe Recorder flow completed.");
}
