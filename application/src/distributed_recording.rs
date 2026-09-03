use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{
    RecordingId, RecordingSyncSignet, RecordingWorkflow, RecordingWorkflowError,
};
use nc_pore_core::role::ProductionAction;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::{ProductionSession, ProductionSessionError};
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::audio::{CaptureProvider, CaptureStartError, RecordingConfiguration, SyncSignet};
use recorder::persistence::PersistenceProvider;

#[derive(Debug, PartialEq, Eq)]
pub enum DistributedRecordingError<E> {
    SessionNotFound,
    RecordingNotFound,
    RecordingCoordinationNotFound,
    Repository(E),
    Session(ProductionSessionError),
    Workflow(RecordingWorkflowError),
    RecorderStart(CaptureStartError),
    Recorder(RecorderApplicationError),
    CoordinationDiverged,
}

/// Application-level handle for one distributed recording start sequence.
///
/// The participant set is selected and frozen by Core via
/// `ProductionSession::begin_recording_by`. The workflow mirrors that same
/// set for its local state machine; it does not invent a second participant
/// set. READY remains a barrier: Opening can only be triggered after every
/// selected recorder has reported READY.
#[derive(Debug)]
pub struct DistributedRecording {
    production_id: ProductionId,
    recording_id: RecordingId,
    actor: ParticipantId,
    workflow: RecordingWorkflow,
}

impl DistributedRecording {
    pub fn production_id(&self) -> &ProductionId {
        &self.production_id
    }

    pub fn recording_id(&self) -> &RecordingId {
        &self.recording_id
    }

    pub fn actor(&self) -> &ParticipantId {
        &self.actor
    }

    pub fn workflow(&self) -> &RecordingWorkflow {
        &self.workflow
    }

    pub fn workflow_mut(&mut self) -> &mut RecordingWorkflow {
        &mut self.workflow
    }

    /// Prepares one local recorder and only then records that participant as
    /// READY in Core. A failed local start/READY transition therefore cannot
    /// advance the distributed READY barrier.
    pub fn prepare_local_recorder<R, C, P>(
        &mut self,
        repository: &mut R,
        participant: &ParticipantId,
        recorder: &mut RecorderApplication<C, P>,
        configuration: &RecordingConfiguration,
    ) -> Result<bool, DistributedRecordingError<R::Error>>
    where
        R: ProductionSessionRepository,
        C: CaptureProvider,
        P: PersistenceProvider,
    {
        recorder
            .start(configuration)
            .map_err(DistributedRecordingError::RecorderStart)?;
        recorder.ready().map_err(|error| {
            DistributedRecordingError::Recorder(RecorderApplicationError::Capture(format!(
                "recorder ready transition failed: {error:?}"
            )))
        })?;

        mark_distributed_recording_ready(repository, self, participant)
    }

    pub fn trigger_opening(&mut self) -> Result<RecordingSyncSignet, RecordingWorkflowError> {
        self.workflow.start_recording_with_signet()
    }

    /// Emits Opening on the local recorder and persists this participant's
    /// Opening confirmation in Core. Stable Recording is persisted only when
    /// the complete selected participant set has confirmed Opening.
    pub fn confirm_opening<R, C, P>(
        &mut self,
        repository: &mut R,
        recorder: &mut RecorderApplication<C, P>,
        opening: &SyncSignet,
    ) -> Result<bool, DistributedRecordingError<R::Error>>
    where
        R: ProductionSessionRepository,
        C: CaptureProvider,
        P: PersistenceProvider,
    {
        recorder
            .emit_sync_signet(opening)
            .map_err(DistributedRecordingError::Recorder)?;

        confirm_distributed_recording_opening(repository, self, &self.actor.clone())
    }

    /// Confirms Opening for a recorder that has already emitted/captured the
    /// signet and persists that confirmation in Core. The returned value is
    /// true only at the aggregate Opening barrier.
    pub fn confirm_opening_for_participant(
        &mut self,
        participant: &ParticipantId,
    ) -> Result<bool, RecordingWorkflowError> {
        self.workflow.confirm_opening(participant)
    }
}

/// Creates the distributed recording coordination from Core's current
/// recording-eligible participants and persists that coordination before any
/// recorder is asked to capture audio.
pub fn begin_distributed_recording<R>(
    repository: &mut R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
) -> Result<DistributedRecording, DistributedRecordingError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(production_id)
        .map_err(DistributedRecordingError::Repository)?
        .ok_or(DistributedRecordingError::SessionNotFound)?;

    let recording = session
        .recordings()
        .iter()
        .find(|recording| recording.id() == recording_id)
        .cloned()
        .ok_or(DistributedRecordingError::RecordingNotFound)?;

    let participants: Vec<_> = session
        .participations()
        .iter()
        .filter(|participation| participation.allows(ProductionAction::ParticipateInRecording))
        .map(|participation| participation.participant_id.clone())
        .collect();

    session
        .begin_recording_by(actor, recording_id, participants.clone())
        .map_err(DistributedRecordingError::Session)?;
    repository
        .update(&session)
        .map_err(DistributedRecordingError::Repository)?;

    let mut workflow = RecordingWorkflow::from_recording(recording, participants)
        .map_err(DistributedRecordingError::Workflow)?;
    workflow
        .begin_ready_phase()
        .map_err(DistributedRecordingError::Workflow)?;

    Ok(DistributedRecording {
        production_id: production_id.clone(),
        recording_id: recording_id.clone(),
        actor: actor.clone(),
        workflow,
    })
}

/// Reconstitutes the application workflow from Core's persisted recording
/// and coordination state. Core remains the source of truth; the workflow is
/// rebuilt from that state rather than inventing a fresh participant set.
pub fn reconstitute_distributed_recording<R>(
    repository: &R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
) -> Result<DistributedRecording, DistributedRecordingError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let session = repository
        .get(production_id)
        .map_err(DistributedRecordingError::Repository)?
        .ok_or(DistributedRecordingError::SessionNotFound)?;

    let recording = session
        .recordings()
        .iter()
        .find(|recording| recording.id() == recording_id)
        .cloned()
        .ok_or(DistributedRecordingError::RecordingNotFound)?;

    let coordination = session
        .recording_coordination()
        .cloned()
        .ok_or(DistributedRecordingError::RecordingCoordinationNotFound)?;

    let workflow = RecordingWorkflow::from_persisted_state(recording, coordination)
        .map_err(DistributedRecordingError::Workflow)?;

    Ok(DistributedRecording {
        production_id: production_id.clone(),
        recording_id: recording_id.clone(),
        actor: actor.clone(),
        workflow,
    })
}

/// Records one participant's local READY state in Core and mirrors the same
/// transition in the application workflow. The returned boolean is true only
/// when the complete frozen participant set is READY.
pub fn mark_distributed_recording_ready<R>(
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

    let core_ready = session
        .mark_recording_ready_by(participant, recording.recording_id())
        .map_err(DistributedRecordingError::Session)?;
    repository
        .update(&session)
        .map_err(DistributedRecordingError::Repository)?;

    let workflow_ready = recording
        .workflow_mut()
        .mark_ready(participant)
        .map_err(DistributedRecordingError::Workflow)?;

    if core_ready != workflow_ready {
        return Err(DistributedRecordingError::CoordinationDiverged);
    }

    Ok(core_ready)
}

/// Persists one participant's Opening confirmation in Core and mirrors the
/// same barrier in the local application workflow. Once the aggregate Opening
/// barrier completes, Core advances the authoritative recording to Recording.
pub fn confirm_distributed_recording_opening<R>(
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

    let core_opening_confirmed = session
        .confirm_recording_opening_by(participant, recording.recording_id())
        .map_err(DistributedRecordingError::Session)?;
    repository
        .update(&session)
        .map_err(DistributedRecordingError::Repository)?;

    let workflow_opening_confirmed = recording
        .workflow_mut()
        .confirm_opening(participant)
        .map_err(DistributedRecordingError::Workflow)?;

    if core_opening_confirmed != workflow_opening_confirmed {
        return Err(DistributedRecordingError::CoordinationDiverged);
    }

    if !core_opening_confirmed {
        return Ok(false);
    }

    let mut session = repository
        .get(recording.production_id())
        .map_err(DistributedRecordingError::Repository)?
        .ok_or(DistributedRecordingError::SessionNotFound)?;
    session
        .start_recording_by(participant, recording.recording_id())
        .map_err(DistributedRecordingError::Session)?;
    repository
        .update(&session)
        .map_err(DistributedRecordingError::Repository)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::Recording;
    use nc_pore_core::role::ParticipantRole;

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
            let stored = self
                .sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
                .ok_or("session not found")?;
            *stored = session.clone();
            Ok(())
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
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

        let mut repository = InMemorySessions {
            sessions: Vec::new(),
        };
        repository.store(&session).unwrap();
        (repository, production_id, alice, bob, recording_id)
    }

    #[test]
    fn begin_uses_core_participant_set_for_distributed_recording() {
        let (mut repository, production_id, alice, bob, recording_id) = fixture();

        let recording =
            begin_distributed_recording(&mut repository, &production_id, &alice, &recording_id)
                .unwrap();

        assert_eq!(
            recording.workflow().coordination().participants(),
            &[alice.clone(), bob.clone()]
        );
        assert_eq!(
            repository
                .get(&production_id)
                .unwrap()
                .unwrap()
                .recording_coordination()
                .unwrap()
                .participants(),
            &[alice, bob]
        );
    }

    #[test]
    fn opening_waits_for_bob_ready_and_opening_confirmation() {
        let (mut repository, production_id, alice, bob, recording_id) = fixture();
        let mut recording =
            begin_distributed_recording(&mut repository, &production_id, &alice, &recording_id)
                .unwrap();

        assert!(
            !mark_distributed_recording_ready(&mut repository, &mut recording, &alice).unwrap()
        );
        assert_eq!(
            recording.trigger_opening(),
            Err(RecordingWorkflowError::InvalidState)
        );

        assert!(mark_distributed_recording_ready(&mut repository, &mut recording, &bob).unwrap());
        assert_eq!(
            recording.trigger_opening(),
            Ok(RecordingSyncSignet::Opening)
        );
        assert!(
            !confirm_distributed_recording_opening(&mut repository, &mut recording, &alice)
                .unwrap()
        );
        assert!(
            confirm_distributed_recording_opening(&mut repository, &mut recording, &bob).unwrap()
        );
        assert_eq!(
            recording.workflow().status(),
            nc_pore_core::recording::RecordingWorkflowStatus::Recording
        );
        assert_eq!(
            repository
                .get(&production_id)
                .unwrap()
                .unwrap()
                .recording_coordination()
                .unwrap()
                .opening_confirmed_participants(),
            &[alice, bob]
        );
        assert_eq!(
            repository
                .get(&production_id)
                .unwrap()
                .unwrap()
                .recordings()[0]
                .status(),
            nc_pore_core::recording::RecordingStatus::Recording
        );
    }

    #[test]
    fn reconstitute_uses_persisted_coordination_state() {
        let (mut repository, production_id, alice, bob, recording_id) = fixture();
        let mut recording =
            begin_distributed_recording(&mut repository, &production_id, &alice, &recording_id)
                .unwrap();

        mark_distributed_recording_ready(&mut repository, &mut recording, &alice).unwrap();
        mark_distributed_recording_ready(&mut repository, &mut recording, &bob).unwrap();
        recording.trigger_opening().unwrap();
        confirm_distributed_recording_opening(&mut repository, &mut recording, &alice).unwrap();

        let restored =
            reconstitute_distributed_recording(&repository, &production_id, &alice, &recording_id)
                .unwrap();

        assert_eq!(
            restored.workflow().status(),
            nc_pore_core::recording::RecordingWorkflowStatus::Opening
        );
        assert_eq!(
            restored
                .workflow()
                .coordination()
                .ready_participants()
                .len(),
            2
        );
        assert_eq!(
            restored
                .workflow()
                .coordination()
                .opening_confirmed_participants(),
            &[alice]
        );
        assert_eq!(restored.actor(), &alice);
        assert_eq!(restored.recording_id(), &recording_id);
    }

    #[test]
    fn reconstitute_requires_persisted_coordination() {
        let (repository, production_id, alice, _, recording_id) = fixture();

        assert_eq!(
            reconstitute_distributed_recording(&repository, &production_id, &alice, &recording_id,),
            Err(DistributedRecordingError::RecordingCoordinationNotFound)
        );
    }
}
