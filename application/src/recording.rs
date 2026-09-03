/// ADR-068 signet semantics come from the core domain, while the concrete
/// signet description is supplied by the technical recording configuration.
/// The Opening Signet is emitted while local capture is active and before the
/// workflow enters the stable Recording state.
pub fn execute_recording<R, C, P>(
    repository: &mut R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
    recorder: &mut RecorderApplication<C, P>,
    configuration: &RecordingConfiguration,
) -> Result<RecordingArtifact, ExecuteRecordingError<R::Error>>
where
    R: ProductionSessionRepository,
    C: CaptureProvider,
    P: PersistenceProvider,
{
    let mut distributed =
        begin_distributed_recording(repository, production_id, actor, recording_id)
            .map_err(map_distributed_error)?;

    // Local technical readiness must be established before this participant
    // can contribute READY to Core's distributed barrier.
    distributed
        .prepare_local_recorder(recorder, configuration)
        .map_err(|error| match error {
            DistributedRecordingError::RecorderStart(error) => {
                ExecuteRecordingError::RecorderStart(error)
            }
            DistributedRecordingError::Recorder(error) => ExecuteRecordingError::Recorder(error),
            _ => unreachable!("local recorder preparation cannot return this error"),
        })?;