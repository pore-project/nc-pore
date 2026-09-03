//! Repository boundary for ProductionSession.
//!
//! The repository defines the domain-facing capability to store and
//! retrieve Production Sessions without depending on a concrete
//! persistence technology.
//!
//! See ADR-036 Persistence Boundary and Storage Strategy.

use crate::activity::ActivityEvent;
use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::{Recording, RecordingCoordinationError, RecordingId, RecordingLifecycleError, RecordingStatus};
use crate::role::{ParticipantRole, ProductionAction};

use super::{ProductionSession, ProductionSessionError, ProductionStatus};

/// Domain-facing repository contract for Production Sessions.
///
/// The Core defines which persistence capabilities are required.
/// Concrete storage implementations remain outside the domain model.
///
/// See ADR-036.
pub trait ProductionSessionRepository {
    type Error;

    /// Stores a new Production Session.
    fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error>;

    /// Updates an existing Production Session.
    ///
    /// The session must already exist.
    fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error>;

    /// Retrieves a Production Session by its Production Identifier.
    ///
    /// `Ok(None)` means that no session with the given identifier exists.
    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error>;
}

/// Reconstitutes a domain session from already-decoded domain state.
///
/// Serialization and storage remain outside Core. This function only restores
/// the aggregate from domain values supplied by an outer persistence adapter.
pub fn reconstitute_production_session(
    id: ProductionId,
    status: ProductionStatus,
    participations: Vec<Participation>,
    recordings: Vec<Recording>,
    activities: Vec<ActivityEvent>,
) -> ProductionSession {
    ProductionSession {
        id,
        status,
        participations,
        recordings,
        recording_coordination: None,
        activities,
    }
}

/// Persists one selected participant's Opening confirmation in Core. The
/// complete READY barrier must have been reached before Opening can be
/// confirmed; stable Recording remains a separate aggregate transition.
impl ProductionSession {
    pub fn confirm_recording_opening_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<bool, ProductionSessionError> {
        self.authorize(actor, ProductionAction::ParticipateInRecording)?;

        let coordination = self
            .recording_coordination
            .as_mut()
            .ok_or(ProductionSessionError::RecordingCoordinationNotFound)?;

        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::RecordingCoordinationNotFound);
        }

        coordination
            .confirm_opening(actor)
            .map_err(ProductionSessionError::RecordingCoordination)
    }

    /// Persists one selected participant's technical stop acknowledgement in
    /// Core. The authoritative recording stop must already have been applied;
    /// this method never stops another participant's local recorder.
    pub fn acknowledge_recording_stop_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<bool, ProductionSessionError> {
        self.authorize(actor, ProductionAction::ParticipateInRecording)?;

        let recording = self
            .recordings
            .iter()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;

        if recording.status() != RecordingStatus::Stopped {
            return Err(ProductionSessionError::RecordingLifecycle(
                RecordingLifecycleError::InvalidTransition {
                    from: recording.status(),
                    to: RecordingStatus::Stopped,
                },
            ));
        }

        let coordination = self
            .recording_coordination
            .as_mut()
            .ok_or(ProductionSessionError::RecordingCoordinationNotFound)?;

        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::RecordingCoordinationNotFound);
        }

        coordination
            .acknowledge_stop(actor)
            .map_err(ProductionSessionError::RecordingCoordination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::participant::ParticipantId;
    use crate::role::ParticipantRole;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            if self.sessions.iter().any(|s| s.id == session.id) {
                return Err("session already exists");
            }
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            let existing = self.sessions.iter_mut().find(|s| s.id == session.id);
            match existing {
                Some(existing) => {
                    *existing = session.clone();
                    Ok(())
                }
                None => Err("session not found"),
            }
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
        }
    }

    fn recording_fixture() -> (ProductionSession, ParticipantId, ParticipantId, RecordingId) {
        let owner = ParticipantId::new("owner-1");
        let participant = ParticipantId::new("participant-1");
        let recording_id = RecordingId::new("recording-001");
        let mut session = ProductionSession::new_with_actor(
            ProductionId::new("session-001"),
            Some(owner.clone()),
        );
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
        session
            .add_participation_by(
                &owner,
                Participation::with_roles(participant.clone(), [ParticipantRole::Participant]),
            )
            .unwrap();
        session.start_by(&owner).unwrap();
        session
            .add_recording_by(&owner, Recording::new(recording_id.value()))
            .unwrap();
        session
            .begin_recording_by(
                &owner,
                &recording_id,
                [owner.clone(), participant.clone()],
            )
            .unwrap();
        session
            .mark_recording_ready_by(&owner, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&participant, &recording_id)
            .unwrap();
        (session, owner, participant, recording_id)
    }

    #[test]
    fn repository_can_store_and_get_session() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();
        assert!(repo.get(&id).unwrap().is_some());
    }

    #[test]
    fn repository_rejects_duplicate_session_id() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();
        let result = repo.store(&ProductionSession::new(id));
        assert_eq!(result, Err("session already exists"));
    }

    #[test]
    fn repository_returns_none_for_unknown_session() {
        let repo = InMemory { sessions: vec![] };
        assert!(repo.get(&ProductionId::new("unknown")).unwrap().is_none());
    }

    #[test]
    fn repository_can_update_existing_session() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();

        let mut updated = ProductionSession::new(id.clone());
        let owner = ParticipantId::new("owner-1");
        updated
            .add_participation_by(
                &owner,
                Participation::new(owner.clone(), ParticipantRole::Owner),
            )
            .unwrap();
        updated.start_by(&owner).unwrap();
        repo.update(&updated).unwrap();
        assert_eq!(repo.get(&id).unwrap().unwrap().status(), updated.status());
    }

    #[test]
    fn opening_confirmation_is_persisted_in_session_coordination() {
        let (mut session, owner, participant, recording_id) = recording_fixture();

        assert_eq!(
            session.confirm_recording_opening_by(&owner, &recording_id),
            Ok(false)
        );
        assert_eq!(
            session.confirm_recording_opening_by(&participant, &recording_id),
            Ok(true)
        );
        assert!(
            session
                .recording_coordination()
                .unwrap()
                .is_opening_confirmed()
        );
    }

    #[test]
    fn stop_acknowledgement_requires_authoritative_stop_and_is_persisted() {
        let (mut session, owner, participant, recording_id) = recording_fixture();
        session.start_recording_by(&owner, &recording_id).unwrap();

        assert_eq!(
            session.acknowledge_recording_stop_by(&participant, &recording_id),
            Err(ProductionSessionError::RecordingLifecycle(
                RecordingLifecycleError::InvalidTransition {
                    from: RecordingStatus::Recording,
                    to: RecordingStatus::Stopped,
                }
            ))
        );

        session.stop_recording_by(&owner, &recording_id).unwrap();
        assert_eq!(
            session.acknowledge_recording_stop_by(&participant, &recording_id),
            Ok(false)
        );
        assert_eq!(
            session.acknowledge_recording_stop_by(&owner, &recording_id),
            Ok(true)
        );
        assert!(
            session
                .recording_coordination()
                .unwrap()
                .is_stop_acknowledged()
        );
    }

    #[test]
    fn unselected_participant_cannot_confirm_opening_or_acknowledge_stop() {
        let (mut session, owner, participant, recording_id) = recording_fixture();
        let outsider = ParticipantId::new("outsider");

        assert_eq!(
            session.confirm_recording_opening_by(&outsider, &recording_id),
            Err(ProductionSessionError::Unauthorized)
        );
        session
            .confirm_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&participant, &recording_id)
            .unwrap();
        session.start_recording_by(&owner, &recording_id).unwrap();
        session.stop_recording_by(&owner, &recording_id).unwrap();
        assert_eq!(
            session.acknowledge_recording_stop_by(&outsider, &recording_id),
            Err(ProductionSessionError::Unauthorized)
        );
    }
}
