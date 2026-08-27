use chrono::{DateTime, SecondsFormat};
use nc_pore_application::synchronization::{
    ArtifactTransferMetadata, InMemorySynchronizationWorkStore, PersistentSynchronizationQueue,
};
use nc_pore_application::synchronization_orchestration::SynchronizationProcessOutcome;
use nc_pore_infrastructure::nextcloud::{
    new_nextcloud_synchronization_orchestrator, NextcloudConnection, NextcloudConnectionConfig,
    WebDavClient,
};
use recorder::artifact::coordination::ArtifactCoordinator;
use recorder::artifact::processing::RecordingArtifactProcessor;
use recorder::audio::{
    CpalCaptureProvider, RecordingChunkDuration, RecordingConfiguration, SampleFormat,
};
use recorder::persistence::FilesystemPersistenceProvider;
use recorder::session::RecordingSession;
use recorder::workflow::{
    recording_start::{RecordingParticipantId, RecordingStartCoordinator},
    recording_stop::RecordingStopCoordinator,
    RecorderWorkflow,
};
use std::env;
use std::thread;
use std::time::{Duration, SystemTime};

fn runtime_configuration(provider: &CpalCaptureProvider) -> Option<RecordingConfiguration> {
    let capabilities = match provider.discover_input_configurations() {
        Ok(capabilities) => capabilities,
        Err(error) => {
            eprintln!("Nextcloud real recording reality check skipped: {error}");
            return None;
        }
    };

    for capability in &capabilities {
        let rates = [48_000, 44_100, capability.min_sample_rate_hz()];
        for rate in rates {
            if rate < capability.min_sample_rate_hz() || rate > capability.max_sample_rate_hz() {
                continue;
            }
            for format in [SampleFormat::Pcm24, SampleFormat::F32] {
                let configuration = RecordingConfiguration::with_chunk_duration(
                    rate,
                    capability.channels(),
                    format,
                    RecordingChunkDuration::ThirtySeconds,
                );
                if capability.matches_recording_configuration(&configuration) {
                    return Some(configuration);
                }
            }
        }
    }

    eprintln!(
        "Nextcloud real recording reality check skipped: no supported exact recording configuration found."
    );
    None
}

#[test]
fn nextcloud_real_recording_reality_check() {
    let required = [
        "NC_PORE_NEXTCLOUD_URL",
        "NC_PORE_NEXTCLOUD_USER",
        "NC_PORE_NEXTCLOUD_APP_PASSWORD",
        "NC_PORE_NEXTCLOUD_REMOTE_ROOT",
    ];
    if required.iter().any(|name| env::var(name).is_err()) {
        eprintln!(
            "Nextcloud real recording reality check skipped: required credentials are not configured."
        );
        return;
    }

    let config = NextcloudConnectionConfig::from_environment()
        .expect("Nextcloud runtime configuration must be valid");
    let username = config.username().to_owned();
    let root = config.remote_root().trim_matches('/').to_owned();
    let connection = NextcloudConnection::new(config.clone())
        .expect("Nextcloud connection configuration must be valid");
    let client = WebDavClient::new(&config).expect("Nextcloud client must be constructible");
    client
        .authenticate(&username)
        .expect("Nextcloud authentication must succeed");

    let capture = CpalCaptureProvider::new();
    let Some(configuration) = runtime_configuration(&capture) else {
        return;
    };

    let process_id = std::process::id();
    let session_value = format!("session-real-recording-{}", process_id);
    let participant = RecordingParticipantId::new("participant-real-recording");
    let session = RecordingSession::new(&session_value);
    let mut workflow = RecorderWorkflow::new(session, capture);

    workflow
        .start(&configuration)
        .expect("real CPAL capture must start");

    let mut start_coordinator = RecordingStartCoordinator::new([participant.clone()]);
    workflow
        .ready_and_maybe_opening_signet(&mut start_coordinator, &participant)
        .expect("real capture must reach READY and emit opening signet");

    thread::sleep(Duration::from_secs(9));

    let mut stop_coordinator = RecordingStopCoordinator::new([participant.clone()]);
    let (_closing_signet, capture_result) = workflow
        .stop_with_coordinator(&mut stop_coordinator)
        .expect("real capture must stop with closing signet");
    stop_coordinator
        .confirm_ok(&participant)
        .expect("real capture participant must confirm stop");

    assert!(
        !capture_result.tracks().is_empty(),
        "real capture must produce at least one track"
    );
    assert!(
        capture_result
            .tracks()
            .iter()
            .any(|track| !track.chunks().is_empty()),
        "real capture must produce at least one payload chunk"
    );

    let artifact_id_value = capture_result.id().to_owned();
    let temp_root = env::temp_dir().join(format!("nc-pore-real-recording-{}", process_id));
    let _ = std::fs::remove_dir_all(&temp_root);

    let persistence = FilesystemPersistenceProvider::new(&temp_root);
    let coordinator = ArtifactCoordinator::new(persistence);
    let mut processor = RecordingArtifactProcessor::new(coordinator);
    let artifact = processor
        .process(
            capture_result,
            recorder::session::RecordingSessionId::new(&session_value),
            recorder::artifact::RecordingArtifactAssociation::new(
                "production-real-recording-check",
                "recording-real-recording-check",
            ),
        )
        .expect("real capture must become a persisted RecordingArtifact");

    let payload_bytes: usize = artifact
        .tracks()
        .iter()
        .flat_map(|track| track.chunks())
        .map(|chunk| chunk.payload().data().len())
        .sum();
    assert!(
        payload_bytes > 0,
        "real recording artifact must contain payload bytes"
    );

    let manifest_hash = artifact.manifest_hash();
    let recorded_at: DateTime<chrono::Utc> = SystemTime::now().into();
    let recorded_at = recorded_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let display_name = "NC-PoRE Real Recording Reality Check".to_owned();
    let metadata =
        ArtifactTransferMetadata::new(Some(display_name.clone()), Some(recorded_at.clone()));

    let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
    queue
        .enqueue_with_metadata(
            nc_pore_core::recording::RecordingArtifactId::new(&artifact_id_value),
            *manifest_hash.as_bytes(),
            metadata,
        )
        .expect("real artifact must be queued");

    let orchestration_persistence = FilesystemPersistenceProvider::new(&temp_root);
    let transfer_persistence = FilesystemPersistenceProvider::new(&temp_root);
    let mut orchestrator = new_nextcloud_synchronization_orchestrator(
        queue,
        orchestration_persistence,
        transfer_persistence,
        connection,
    );

    let result = orchestrator
        .process_next()
        .expect("real artifact synchronization orchestration must succeed");
    assert_eq!(result, SynchronizationProcessOutcome::Synchronized);
    let synchronized_work = orchestrator
        .queue()
        .list()
        .expect("synchronization queue must remain readable");
    assert_eq!(synchronized_work.len(), 1);
    assert_eq!(
        synchronized_work[0].status(),
        nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Synchronized
    );

    let recorded_at = recorded_at
        .parse::<DateTime<chrono::FixedOffset>>()
        .expect("recorded_at must remain a valid RFC3339 timestamp");
    let expected_prefix = format!(
        "remote.php/dav/files/{username}/{root}/{}/{} - {} - {}",
        recorded_at.format("%Y/%m/%d"),
        recorded_at.format("%H-%M"),
        display_name,
        artifact_id_value
    );
    let manifest_path = format!("{expected_prefix}/manifest.json");
    let payload_prefix = format!("{expected_prefix}/tracks/track-01-cpal-track/chunks");

    let manifest = client
        .get_optional(&manifest_path)
        .expect("uploaded manifest must be readable")
        .expect("uploaded manifest must exist");
    let manifest_text = String::from_utf8(manifest.body).expect("manifest must be UTF-8 JSON");
    assert!(manifest_text.contains(&artifact_id_value));
    assert!(manifest_text.contains("cpal-track"));
    assert!(manifest_text.contains(&display_name));

    let first_payload_path = format!("{payload_prefix}/chunk-000001.payload");
    let payload = client
        .get_optional(&first_payload_path)
        .expect("uploaded real payload must be readable")
        .expect("uploaded real payload must exist");
    assert_eq!(payload.body.len(), payload_bytes);
    assert_eq!(
        payload.body,
        artifact.tracks()[0].chunks()[0].payload().data()
    );

    let _ = std::fs::remove_dir_all(&temp_root);

    println!(
        "Nextcloud real recording reality check passed: artifact='{}', payload_bytes={}, remote_path='{}/{}'",
        artifact_id_value, payload_bytes, root, expected_prefix
    );
}
