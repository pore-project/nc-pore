use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::RecordingWorkflowError;

use crate::distributed_recording::DistributedRecording;

/// Records the technical stop acknowledgement of one selected recorder.
///
/// The local recorder must stop independently; this operation only advances
/// the distributed workflow barrier. In particular, a remote participant
/// does not need to have a local `RecorderApplication` instance in the host
/// process. The workflow remains incomplete until every selected participant
/// has acknowledged the stop.
pub fn acknowledge_distributed_recording_stop(
    recording: &mut DistributedRecording,
    participant: &ParticipantId,
) -> Result<bool, RecordingWorkflowError> {
    recording.workflow_mut().acknowledge_stop(participant)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed_recording::begin_distributed_recording;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::{Recording, RecordingId};
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::repository::ProductionSessionRepository;
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

    #[test]
    fn remote_stop_acknowledgement_is_independent_from_host_process() {
        let (mut repository, production_id, alice, bob, recording_id) = fixture();
        let mut recording =
            begin_distributed_recording(&mut repository, &production_id, &alice, &recording_id)
                .unwrap();

        recording.workflow_mut().mark_ready(&alice).unwrap();
        recording.workflow_mut().mark_ready(&bob).unwrap();
        recording
            .workflow_mut()
            .start_recording_with_signet()
            .unwrap();
        recording.workflow_mut().confirm_opening(&alice).unwrap();
        recording.workflow_mut().confirm_opening(&bob).unwrap();
        recording.workflow_mut().request_stop().unwrap();
        recording
            .workflow_mut()
            .confirm_core_stop_persisted()
            .unwrap();

        assert!(!acknowledge_distributed_recording_stop(&mut recording, &bob).unwrap());
        assert!(acknowledge_distributed_recording_stop(&mut recording, &alice).unwrap());
    }
}
