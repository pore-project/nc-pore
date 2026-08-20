    #[test]
    fn completed_session_rejects_participant_mutations() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();
        session.complete_by(&owner).unwrap();

        assert_eq!(
            session.add_participation_by(
                &owner,
                create_participation("participant-1", ParticipantRole::Participant),
            ),
            Err(ProductionSessionError::InvalidStateTransition)
        );
    }

    #[test]
    fn starting_recording_binds_actor_as_participant() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();

        session
            .add_recording_by(&owner, Recording::new("recording-001"))
            .unwrap();
        session
            .start_recording_by(&owner, &RecordingId::new("recording-001"))
            .unwrap();

        assert_eq!(session.recordings()[0].participant_id(), Some(&owner));
    }
}
