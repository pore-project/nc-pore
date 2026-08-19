use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::participation::Participation;
use nc_pore_core::recording::{Recording, RecordingArtifactId, RecordingId, RecordingStatus};
use nc_pore_core::role::ParticipantRole;
use nc_pore_core::session::{repository::ProductionSessionRepository, ProductionSession};
use nc_pore_infrastructure::FileProductionSessionRepository;

fn temp_root() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("nc-pore-vertical-slice-{nanos}"))
}

fn completed_session() -> ProductionSession {
    let production_id = ProductionId::new("production-001");
    let owner = ParticipantId::new("owner-1");
    let recording_id = RecordingId::new("recording-001");
    let mut session = ProductionSession::new_with_actor(production_id, Some(owner.clone()));

    session
        .add_participation_by(
            &owner,
            Participation::with_roles(
                owner.clone(),
                [
                    ParticipantRole::Owner,
                    ParticipantRole::Producer,
                    ParticipantRole::Participant,
                ],
            ),
        )
        .unwrap();
    session.start_by(&owner).unwrap();
    session
        .add_recording_by(&owner, Recording::new(recording_id.value()))
        .unwrap();
    session.start_recording_by(&owner, &recording_id).unwrap();
    session
        .complete_recording_by(
            &owner,
            &recording_id,
            RecordingArtifactId::new("artifact-001"),
        )
        .unwrap();

    session
}

// TEST-01
//
// Verify: A completed recording survives a repository restart with its
// artifact association and activity history intact.
#[test]
fn completed_recording_survives_repository_restart() {
    let root = temp_root();
    let session = completed_session();
    let production_id = session.id.clone();
    let expected_activity_count = session.activities().len();

    {
        let mut repository = FileProductionSessionRepository::new(&root).unwrap();
        repository.store(&session).unwrap();
    }

    let reloaded = {
        let repository = FileProductionSessionRepository::new(&root).unwrap();
        repository.get(&production_id).unwrap().unwrap()
    };

    let recording = &reloaded.recordings()[0];
    assert_eq!(recording.status(), RecordingStatus::Completed);
    assert_eq!(
        recording.artifact_id().unwrap().value(),
        "artifact-001"
    );
    assert_eq!(reloaded.activities().len(), expected_activity_count);
    assert_eq!(reloaded.activities()[expected_activity_count - 1].activity_type,
               nc_pore_core::activity::ActivityType::RecordingCompleted);

    let _ = std::fs::remove_dir_all(root);
}
