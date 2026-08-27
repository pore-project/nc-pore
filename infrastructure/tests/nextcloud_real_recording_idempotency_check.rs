use chrono::{DateTime, FixedOffset, SecondsFormat, Utc};
use nc_pore_application::synchronization::{
    ArtifactTransferMetadata, InMemorySynchronizationWorkStore, PersistentSynchronizationQueue,
};
use nc_pore_application::synchronization_orchestration::SynchronizationProcessOutcome;
use nc_pore_infrastructure::nextcloud::{
    new_nextcloud_synchronization_orchestrator, NextcloudConnection, NextcloudConnectionConfig,
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
    let capabilities = provider.discover_input_configurations().ok()?;
    for capability in &capabilities {
        for rate in [48_000, 44_100, capability.min_sample_rate_hz()] {
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
    None
}

#[test]
fn nextcloud_real_recording_idempotency_check() {
    let required = [
        "NC_PORE_NEXTCLOUD_URL",
        "NC_PORE_NEXTCLOUD_USER",
        "NC_PORE_NEXTCLOUD_APP_PASSWORD",
        "NC_PORE_NEXTCLOUD_REMOTE_ROOT",
    ];
    if required.iter().any(|name| env::var(name).is_err()) {
        eprintln!(
            "Nextcloud idempotency check skipped: credentials are not configured."
        );
        return;
    }
    let config =
        NextcloudConnectionConfig::from_environment().expect("valid Nextcloud config");
    let capture = CpalCaptureProvider::new();
    let Some(configuration) = runtime_configuration(&capture) else {
        return;
    };
    let process_id = std::process::id();
    let session_value = format!("session-idempotency-{}", process_id);
    let participant = RecordingParticipantId::new("participant-idempotency");
    let mut workflow = RecorderWorkflow::new(RecordingSession::new(&session_value), capture);
    workflow.start(&configuration).expect("capture must start");
    let mut start = RecordingStartCoordinator::new([participant.clone()]);
    workflow
        .ready_and_maybe_opening_signet(&mut start, &participant)
        .expect("capture must become READY");
    thread::sleep(Duration::from_secs(9));
    let mut stop = RecordingStopCoordinator::new([participant.clone()]);
    let (_closing, capture_result) = workflow
        .stop_with_coordinator(&mut stop)
        .expect("capture must stop");
    stop.confirm_ok(&participant)
        .expect("stop confirmation must succeed");

    let artifact_id = capture_result.id().to_owned();
    let temp_root = env::temp_dir().join(format!("nc-pore-idempotency-{}", process_id));
    let _ = std::fs::remove_dir_all(&temp_root);
    let persistence = FilesystemPersistenceProvider::new(&temp_root);
    let mut processor = RecordingArtifactProcessor::new(ArtifactCoordinator::new(persistence));
    let artifact = processor
        .process(
            capture_result,
            recorder::session::RecordingSessionId::new(&session_value),
            recorder::artifact::RecordingArtifactAssociation::new(
                "production-idempotency-check",
                "recording-idempotency-check",
            ),
        )
        .expect("capture must become an artifact");

    let manifest_hash = artifact.manifest_hash();
    let recorded_at: DateTime<Utc> = SystemTime::now().into();
    let recorded_at = recorded_at.to_rfc3339_opts(SecondsFormat::Secs, true);
    let metadata = ArtifactTransferMetadata::new(
        Some("NC-PoRE Real Recording Idempotency Check".to_owned()),
        Some(recorded_at.clone()),
    );

    let mut first_queue =
        PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
    first_queue
        .enqueue_with_metadata(
            nc_pore_core::recording::RecordingArtifactId::new(&artifact_id),
            *manifest_hash.as_bytes(),
            metadata.clone(),
        )
        .expect("first queue entry must be created");
    let mut first_orchestrator = new_nextcloud_synchronization_orchestrator(
        first_queue,
        FilesystemPersistenceProvider::new(&temp_root),
        FilesystemPersistenceProvider::new(&temp_root),
        NextcloudConnection::new(config.clone()).expect("first connection must be valid"),
    );
    assert_eq!(
        first_orchestrator
            .process_next()
            .expect("first synchronization must succeed"),
        SynchronizationProcessOutcome::Synchronized
    );

    let mut second_queue =
        PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
    second_queue
        .enqueue_with_metadata(
            nc_pore_core::recording::RecordingArtifactId::new(&artifact_id),
            *manifest_hash.as_bytes(),
            metadata,
        )
        .expect("second queue entry must be created");
    let mut second_orchestrator = new_nextcloud_synchronization_orchestrator(
        second_queue,
        FilesystemPersistenceProvider::new(&temp_root),
        FilesystemPersistenceProvider::new(&temp_root),
        NextcloudConnection::new(config).expect("second connection must be valid"),
    );
    assert_eq!(
        second_orchestrator
            .process_next()
            .expect("repeated synchronization must succeed"),
        SynchronizationProcessOutcome::Synchronized
    );
    let work = second_orchestrator
        .queue()
        .list()
        .expect("queue must remain readable");
    assert_eq!(work.len(), 1);
    assert_eq!(
        work[0].status(),
        nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Synchronized
    );

    let recorded_at = recorded_at
        .parse::<DateTime<FixedOffset>>()
        .expect("valid timestamp");
    println!(
        "Nextcloud real recording idempotency check passed: artifact='{}', remote minute='{}'",
        artifact_id,
        recorded_at.format("%H-%M")
    );
    let _ = std::fs::remove_dir_all(&temp_root);
}
