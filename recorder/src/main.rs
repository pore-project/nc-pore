use recorder::application::RecorderApplication;
use recorder::artifact::RecordingArtifactAssociation;
use recorder::artifact::coordination::ArtifactCoordinator;
use recorder::artifact::processing::RecordingArtifactProcessor;
use recorder::audio::{CpalCaptureProvider, RecordingConfiguration};
use recorder::persistence::FilesystemPersistenceProvider;
use recorder::session::RecordingSession;

fn main() {
    let mut args = std::env::args();
    let _program = args.next();

    match args.next().as_deref() {
        Some("test-audio-stream") => {
            return recorder::audio::test_input_stream().unwrap_or_else(|error| {
                eprintln!("Audio-Stream-Test fehlgeschlagen: {error}");
                std::process::exit(1);
            });
        }
        Some("inspect-audio") => {
            match recorder::audio::inspect_default_input_device() {
                Ok(()) => {}
                Err(error) => {
                    eprintln!("Audio-Inspektion fehlgeschlagen: {error}");
                    std::process::exit(1);
                }
            }

            return;
        }
        Some("record-test") => run_record_test(args),
        Some(command) => {
            eprintln!("Unbekannter Befehl: {command}");
            eprintln!("Verwendung: recorder [inspect-audio|test-audio-stream|record-test [sekunden]]");
            std::process::exit(2);
        }
        None => {
            println!("NC-PoRe Recorder starting...");
            println!("Kein Aufnahmetest gestartet. Verwende 'record-test [sekunden]'.");
        }
    }
}

fn run_record_test(mut args: impl Iterator<Item = String>) {
    let duration_seconds = args
        .next()
        .map(|value| {
            value.parse::<u64>().unwrap_or_else(|_| {
                eprintln!("Ungültige Aufnahmedauer: {value}");
                std::process::exit(2);
            })
        })
        .unwrap_or(10);

    if duration_seconds == 0 {
        eprintln!("Die Aufnahmedauer muss größer als 0 Sekunden sein.");
        std::process::exit(2);
    }

    let persistence_root = std::env::var_os("NC_PORE_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(".nc-pore-data"));

    println!("NC-PoRe Recording Reality Check");
    println!("Persistenz: {}", persistence_root.display());
    println!("Aufnahmedauer: {duration_seconds} Sekunden");

    let session = RecordingSession::new("reality-check-session");
    let capture = CpalCaptureProvider::new();
    let persistence = FilesystemPersistenceProvider::new(&persistence_root);
    let coordinator = ArtifactCoordinator::new(persistence);
    let processor = RecordingArtifactProcessor::new(coordinator);
    let mut application = RecorderApplication::new(session, capture, processor);
    let configuration = RecordingConfiguration::default();

    if let Err(error) = application.start(&configuration) {
        eprintln!("Aufnahme konnte nicht gestartet werden: {error:?}");
        std::process::exit(1);
    }

    println!("Aufnahme läuft – bitte jetzt sprechen oder Musik abspielen.");
    std::thread::sleep(std::time::Duration::from_secs(duration_seconds));
    println!("Stoppe Aufnahme...");

    let artifact = application.stop(RecordingArtifactAssociation::new(
        "reality-check-production",
        "reality-check-recording",
    ));

    let artifact = artifact.unwrap_or_else(|error| {
        eprintln!("RecordingArtifact konnte nicht erzeugt werden: {error:?}");
        std::process::exit(1);
    });

    println!(
        "RecordingArtifact gespeichert: {} Track(s), {} Chunk(s)",
        artifact.tracks().len(),
        artifact
            .tracks()
            .iter()
            .map(|track| track.chunks().len())
            .sum::<usize>()
    );
    println!("Artifact-ID: {}", artifact.id.value());
    println!("Persistenzpfad: {}", persistence_root.display());
    println!("Reality Check: Aufnahme + Filesystem-Persistenz abgeschlossen.");
}
