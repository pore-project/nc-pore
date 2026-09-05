use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{RecordingArtifactId, RecordingId};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;
use recorder::artifact::RecordingArtifact;

#[derive(Debug, PartialEq, Eq)]
pub enum CompleteRecordingError<E> {
    SessionNotFound,
    RecordingNotFound,
    ArtifactAssociationMismatch,
    Repository(E),
    Session(ProductionSessionError),
}

/// Completes one Core recording from an already persisted recorder artifact.
///
/// This is the application boundary between technical artifact persistence and
/// the authoritative Core recording lifecycle. It deliberately does not
/// persist the artifact itself and does not create a second recording state.
pub fn complete_recording_from_artifact<R>(
    repository: &mut R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
    artifact: &RecordingArtifact,
) -> Result<(), CompleteRecordingError<R::Error>>
where
    R: ProductionSessionRepository,
{
    if artifact.production_id() != Some(production_id.value())
        || artifact.recording_id() != Some(recording_id.value())
    {
        return Err(CompleteRecordingError::ArtifactAssociationMismatch);
    }

    let mut session = repository
        .get(production_id)
        .map_err(CompleteRecordingError::Repository)?
        .ok_or(CompleteRecordingError::SessionNotFound)?;

    if !session
        .recordings()
        .iter()
        .any(|recording| recording.id() == recording_id)
    {
        return Err(CompleteRecordingError::RecordingNotFound);
    }

    session
        .complete_recording_by(
            actor,
            recording_id,
            RecordingArtifactId::new(artifact.id.value()),
        )
        .map_err(CompleteRecordingError::Session)?;

    repository
        .update(&session)
        .map_err(CompleteRecordingError::Repository)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::{Recording, RecordingStatus};
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::ProductionSession;
    use recorder::artifact::coordination::ArtifactCoordinator;
    use recorder::artifact::processing::RecordingArtifactProcessor;
    use recorder::audio::{CaptureProvider, CaptureResult, RecordingConfiguration};
    use recorder::persistence::InMemoryPersistenceProvider;
    use recorder::session::RecordingSession;
    use recorder::application::RecorderApplication;

    struct InMemorySessions {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemorySessions {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
                .map(|existing| *existing = session.clone())
                .ok_or("session not found")
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
        }
    }

    struct TestCapture;

    impl CaptureProvider for TestCapture {
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), recorder::audio::CaptureStartError> {
            Ok(())
        }

        fn stop_capture(&mut self) -> CaptureResult {
            CaptureResult::new("completion-boundary-artifact")
        }
    }

    fn fixture() -> (InMemorySessions, ProductionId, ParticipantId, RecordingId) {
        let production_id = ProductionId::new("production-001");
        let actor = ParticipantId::new("alice");
        let recording_id = RecordingId::new("recording-001");
        let mut session = ProductionSession::new_with_actor(
            production_id.clone(),
            Some(actor.clone()),
        );
        session
            .add_participation_by(
                &actor,
                Participation::with_roles(
                    actor.clone(),
                    [
                        ParticipantRole::Owner,
                        ParticipantRole::Producer,
                        ParticipantRole::Participant,
                    ],
                ),
            )
            .unwrap();
        session.start_by(&actor).unwrap();
        session
            .add_recording_by(&actor, Recording::new(recording_id.value()))
            .unwrap();
        session.start_recording_by(&actor, &recording_id).unwrap();
        session.stop_recording_by(&actor, &recording_id).unwrap();

        (
            InMemorySessions {
                sessions: vec![session],
            },
            production_id,
            actor,
            recording_id,
        )
    }

    fn persisted_artifact(
        production_id: &ProductionId,
        recording_id: &RecordingId,
    ) -> recorder::artifact::RecordingArtifact {
        let mut recorder = RecorderApplication::new(
            RecordingSession::new(recording_id.value()),
            TestCapture,
            RecordingArtifactProcessor::new(ArtifactCoordinator::new(
                InMemoryPersistenceProvider::new(),
            )),
        );
        recorder.start(&RecordingConfiguration::default()).unwrap();
        recorder.ready().unwrap();
        recorder
            .stop(recorder::artifact::RecordingArtifactAssociation::new(
                production_id.value(),
                recording_id.value(),
            ))
            .unwrap()
    }

    #[test]
    fn persisted_artifact_completes_stopped_core_recording() {
        let (mut repository, production_id, actor, recording_id) = fixture();
        let artifact = persisted_artifact(&production_id, &recording_id);

        complete_recording_from_artifact(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &artifact,
        )
        .unwrap();

        let session = repository.get(&production_id).unwrap().unwrap();
        let recording = &session.recordings()[0];
        assert_eq!(recording.status(), RecordingStatus::Completed);
        assert_eq!(recording.artifact_id().unwrap().value(), artifact.id.value());
    }

    #[test]
    fn completion_is_idempotent_for_same_artifact() {
        let (mut repository, production_id, actor, recording_id) = fixture();
        let artifact = persisted_artifact(&production_id, &recording_id);

        complete_recording_from_artifact(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &artifact,
        )
        .unwrap();
        complete_recording_from_artifact(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &artifact,
        )
        .unwrap();
    }

    #[test]
    fn mismatched_artifact_cannot_complete_recording() {
        let (mut repository, production_id, actor, recording_id) = fixture();
        let artifact = persisted_artifact(&production_id, &recording_id);
        let mismatched = persisted_artifact(&ProductionId::new("other-production"), &recording_id);

        assert_eq!(
            complete_recording_from_artifact(
                &mut repository,
                &production_id,
                &actor,
                &recording_id,
                &mismatched,
            ),
            Err(CompleteRecordingError::ArtifactAssociationMismatch)
        );
        assert_eq!(
            repository
                .get(&production_id)
                .unwrap()
                .unwrap()
                .recordings()[0]
                .status(),
            RecordingStatus::Stopped
        );
        let _ = artifact;
    }
}
