//! Minimal local capture reality check for the first real recorder path.
//!
//! This deliberately proves only the technical boundary:
//! discover the default input device -> select the exact NC-PoRe recording
//! configuration -> start local capture -> receive real samples -> stop.
//!
//! It does not persist artifacts, emit synchronization signets, or connect
//! the recorder to a Production Session. Those remain separate boundaries.

use recorder::audio::{
    CaptureProvider, CaptureStatus, CpalCaptureProvider, RecordingConfiguration,
};
use std::thread;
use std::time::Duration;

const CAPTURE_DURATION: Duration = Duration::from_secs(3);

fn main() {
    let configuration = RecordingConfiguration::default();
    let mut capture = CpalCaptureProvider::new();

    println!("NC-PoRe local capture vertical slice");
    println!(
        "Requested configuration: {} Hz, {} channel, {:?}",
        configuration.sample_rate_hz(),
        configuration.channels(),
        configuration.sample_format()
    );

    let capabilities = match capture.discover_input_configurations() {
        Ok(capabilities) => capabilities,
        Err(error) => fail(format!("Input device discovery failed: {error}")),
    };

    println!("Discovered input configurations: {}", capabilities.len());

    if !capabilities
        .iter()
        .any(|capability| capability.matches_recording_configuration(&configuration))
    {
        fail(
            "No exact native input configuration matches the requested recording configuration."
                .to_owned(),
        );
    }

    println!("Exact native recording configuration: available");

    if let Err(error) = capture.start_capture(&configuration) {
        fail(format!("Local capture could not be started: {error:?}"));
    }

    println!("Local capture: ACTIVE");
    println!(
        "Recording real input for {} seconds...",
        CAPTURE_DURATION.as_secs()
    );

    thread::sleep(CAPTURE_DURATION);

    let result = capture.stop_capture();

    match result.status() {
        CaptureStatus::Failed(error) => fail(format!("Local capture failed: {error}")),
        CaptureStatus::Completed => {}
    }

    let payload_bytes: usize = result
        .tracks()
        .iter()
        .flat_map(|track| track.chunks())
        .map(|chunk| chunk.payload().len())
        .sum();

    println!("Capture result: COMPLETED");
    println!("Tracks: {}", result.tracks().len());
    println!("Captured payload bytes: {payload_bytes}");

    if payload_bytes == 0 {
        fail("Capture completed without receiving audio payload bytes.".to_owned());
    }

    println!("RESULT: REAL AUDIO CAPTURED");
}

fn fail(message: String) -> ! {
    eprintln!("ERROR: {message}");
    std::process::exit(1);
}
