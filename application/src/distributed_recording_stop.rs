use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::RecordingWorkflowError;
use nc_pore_core::session::repository::ProductionSessionRepository;

use crate::distributed_recording::{DistributedRecording, DistributedRecordingError};

pub fn acknowledge_distributed_recording_stop(
    recording: &mut DistributedRecording,
    participant: &ParticipantId,
) -> Result<bool, RecordingWorkflowError> {
    recording.workflow_mut().acknowledge_stop(participant)
}

pub fn acknowledge_distributed_recording_stop_in_core<R>(
    repository: &mut R,
    recording: &mut DistributedRecording,
    participant: &ParticipantId,
) -> Result<bool, DistributedRecordingError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(recording.production_id())
        .map_err(DistributedRecordingError::Repository)?
        .ok_or(DistributedRecordingError::SessionNotFound)?;

    let core_acknowledged = session
        .acknowledge_recording_stop_by(participant, recording.recording_id())
        .map_err(DistributedRecordingError::Session)?;
    repository
        .update(&session)
        .map_err(DistributedRecordingError::Repository)?;

    let workflow_acknowledged = recording
        .workflow_mut()
        .acknowledge_stop(participant)
        .map_err(DistributedRecordingError::Workflow)?;

    if core_acknowledged != workflow_acknowledged {
        return Err(DistributedRecordingError::CoordinationDiverged);
    }

    Ok(core_acknowledged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed_recording::begin_distributed_recording;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::{Recording, RecordingId, RecordingStatus};
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::ProductionSession;

    struct InMemorySessions {
        session: ProductionSession,
    }

    impl ProductionSessionRepository for InMemorySessions {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.session = session.clone();
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.session = session.clone();
            Ok(())
        }

        fn get(&self, _id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(Some(self.session.clone()))
        }
    }

    fn fixture() -> (
        InMemorySessions,
        ProductionId,
        ParticipantId,
        ParticipantId,
        RecordingId,
    ) {
        let production_id = ProductionId::new("production-001");
        let alice = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let recording_id = RecordingId::new("recording-001");
        let mut session =
            ProductionSession::new_with_actor(production_id.clone(), Some(alice.clone()));
        session
            .add_participation_by(
                &alice,
                Participation::with_roles(
                    alice.clone(),
                    [
                        ParticipantRole::Owner,
                        ParticipantRole::Producer,
                        ParticipantRole::Participant,
                    ],
                ),
            )
            .unwrap();
        session
            .add_participation_by(
                &alice,
                Participation::with_roles(bob.clone(), [ParticipantRole::Participant]),
            )
            .unwrap();
        session.start_by(&alice).unwrap();
        session
            .add_recording_by(&alice, Recording::new(recording_id.value()))
            .unwrap();
        (
            InMemorySessions { session },
            production_id,
            alice,
            bob,
            recording_id,
        )
    }

    fn reach_stopping(
        repository: &mut InMemorySessions,
        recording: &mut DistributedRecording,
        alice: &ParticipantId,
        bob: &ParticipantId,
        recording_id: &RecordingId,
    ) {
        recording.workflow_mut().mark_ready(alice).unwrap();
        recording.workflow_mut().mark_ready(bob).unwrap();
        recording
            .workflow_mut()
            .start_recording_with_signet()
            .unwrap();
        recording.workflow_mut().confirm_opening(alice).unwrap();
        recording.workflow_mut().confirm_opening(bob).unwrap();

        let mut session = repository.session.clone();
        session
            .mark_recording_ready_by(alice, recording_id)
            .unwrap();
        session.mark_recording_ready_by(bob, recording_id).unwrap();
        session
            .confirm_recording_opening_by(alice, recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(bob, recording_id)
            .unwrap();
        session.start_recording_by(alice, recording_id).unwrap();
        session.stop_recording_by(alice, recording_id).unwrap();
        *repository = InMemorySessions { session };
        recording
            .workflow_mut()
            .confirm_core_stop_persisted()
            .unwrap();
    }

    #[test]
    fn remote_stop_acknowledgement_is_independent_from_host_process() {
        let (mut repository, production_id, alice, bob, recording_id) = fixture();
        let mut recording =
            begin_distributed_recording(&mut repository, &production_id, &alice, &recording_id)
                .unwrap();

        reach_stopping(&mut repository, &mut recording, &alice, &bob, &recording_id);

        assert!(!acknowledge_distributed_recording_stop(&mut recording, &bob).unwrap());
        assert!(acknowledge_distributed_recording_stop(&mut recording, &alice).unwrap());
    }

    #[test]
    fn remote_stop_acknowledgement_is_persisted_in_core() {
        let (mut repository, production_id, alice, bob, recording_id) = fixture();
        let mut recording =
            begin_distributed_recording(&mut repository, &production_id, &alice, &recording_id)
                .unwrap();

        reach_stopping(&mut repository, &mut recording, &alice, &bob, &recording_id);

        assert!(!acknowledge_distributed_recording_stop_in_core(
            &mut repository,
            &mut recording,
            &bob,
        )
        .unwrap());
        assert_eq!(
            repository
                .session
                .recording_coordination()
                .unwrap()
                .stop_acknowledged_participants(),
            &[bob.clone()]
        );

        assert!(acknowledge_distributed_recording_stop_in_core(
            &mut repository,
            &mut recording,
            &alice,
        )
        .unwrap());
        assert!(repository
            .session
            .recording_coordination()
            .unwrap()
            .is_stop_acknowledged());
        assert_eq!(
            repository.session.recordings()[0].status(),
            RecordingStatus::Stopped
        );
    }
}
