use chrono::{DateTime, SecondsFormat};
use nc_pore_application::synchronization::{
    ArtifactTransferMetadata, PersistentSynchronizationQueue, SynchronizationWorkStore,
};
use nc_pore_application::synchronization_orchestration::SynchronizationProcessOutcome;
use nc_pore_infrastructure::nextcloud::{
    new_nextcloud_synchronization_orchestrator, NextcloudConnection, NextcloudConnectionConfig,
};
use nc_pore_infrastructure::synchronization_work_store::FilesystemSynchronizationWorkStore;
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
            eprintln!("Nextcloud restart recovery check skipped: {error}");
            return None;
        }
    };

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

    eprintln!(
        "Nextcloud restart recovery check skipped: no supported exact recording configuration found."
    );
    None
}

#[test]
fn nextcloud_real_recording_restart_recovery_check() {
    let required = [
        "NC_PORE_NEXTCLOUD_URL",
        "NC_PORE_NEXTCLOUD_USER",
        "NC_PORE_NEXTCLOUD_APP_PASSWORD",
        "NC_PORE_NEXTCLOUD_REMOTE_ROOT",
    ];
    if required.iter().any(|name| env::var(name).is_err()) {
        eprintln!(
            "Nextcloud restart recovery check skipped: required credentials are not configured."
        );
        return;
    }

    let config = NextcloudConnectionConfig::from_environment()
        .expect("Nextcloud runtime configuration must be valid");
    let connection = NextcloudConnection::new(config)
        .expect("Nextcloud connection configuration must be valid");
    let capture = CpalCaptureProvider::new();
    let Some(configuration) = runtime_configuration(&capture) else {
        return;
    };

    let process_id = std::process::id();
    let session_value = format!("session-restart-recovery-{process_id}");
    let participant = RecordingParticipantId::new("participant-restart-recovery");
    let mut workflow = RecorderWorkflow::new(RecordingSession::new(&session_value), capture);
    workflow
        .start(&configuration)
        .expect("real CPAL capture must start");
    let mut start_coordinator = RecordingStartCoordinator::new([participant.clone()]);
    workflow
        .ready_and_maybe_opening_signet(&mut start_coordinator, &participant)
        .expect("real capture must reach READY");
    thread::sleep(Duration::from_secs(9));

    let mut stop_coordinator = RecordingStopCoordinator::new([participant.clone()]);
    let (_closing_signet, capture_result) = workflow
        .stop_with_coordinator(&mut stop_coordinator)
        .expect("real capture must stop");
    stop_coordinator
        .confirm_ok(&participant)
        .expect("stop confirmation must succeed");

    let artifact_id = capture_result.id().to_owned();
    let temp_root = env::temp_dir().join(format!("nc-pore-restart-recovery-{process_id}"));
    let _ = std::fs::remove_dir_all(&temp_root);
    let persistence = FilesystemPersistenceProvider::new(&temp_root);
    let coordinator = ArtifactCoordinator::new(persistence);
    let mut processor = RecordingArtifactProcessor::new(coordinator);
    let artifact = processor
        .process(
            capture_result,
            recorder::session::RecordingSessionId::new(&session_value),
            recorder::artifact::RecordingArtifactAssociation::new(
                "production-restart-recovery-check",
                "recording-restart-recovery-check",
            ),
        )
        .expect("real capture must become a persisted RecordingArtifact");

    let recorded_at: DateTime<chrono::Utc> = SystemTime::now().into();
    let metadata = ArtifactTransferMetadata::new(
        Some("NC-PoRE Restart Recovery Reality Check".to_owned()),
        Some(recorded_at.to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    let work_store_root = temp_root.join("sync-work");
    let mut queue = PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(
        &work_store_root,
    ));
    queue
        .enqueue_with_metadata(
            nc_pore_core::recording::RecordingArtifactId::new(&artifact_id),
            *artifact.manifest_hash().as_bytes(),
            metadata,
        )
        .expect("real artifact must be queued");

    let mut first = new_nextcloud_synchronization_orchestrator(
        queue,
        FilesystemPersistenceProvider::new(&temp_root),
        FilesystemPersistenceProvider::new(&temp_root),
        connection.clone(),
    );
    assert_eq!(
        first
            .process_next()
            .expect("first synchronization must succeed"),
        SynchronizationProcessOutcome::Synchronized
    );
    drop(first);

    let reconstructed_queue = PersistentSynchronizationQueue::new(
        FilesystemSynchronizationWorkStore::new(&work_store_root),
    );
    let persisted_work = reconstructed_queue
        .list()
        .expect("reconstructed queue must be readable");
    assert_eq!(persisted_work.len(), 1);
    assert_eq!(
        persisted_work[0].status(),
        nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Synchronized
    );

    let mut second = new_nextcloud_synchronization_orchestrator(
        reconstructed_queue,
        FilesystemPersistenceProvider::new(&temp_root),
        FilesystemPersistenceProvider::new(&temp_root),
        connection,
    );
    assert_eq!(
        second
            .process_next()
            .expect("restarted synchronization must be readable"),
        SynchronizationProcessOutcome::NoPendingWork
    );

    let final_work = second.queue().list().expect("final queue must be readable");
    assert_eq!(final_work.len(), 1);
    assert_eq!(
        final_work[0].status(),
        nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Synchronized
    );

    let _ = std::fs::remove_dir_all(&temp_root);
    println!(
        "Nextcloud restart recovery check passed: artifact='{artifact_id}' survived synchronization restart as Synchronized."
    );
}
