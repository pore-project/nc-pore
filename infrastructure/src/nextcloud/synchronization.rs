//! Concrete composition of the vendor-neutral synchronization queue with the
//! Nextcloud artifact-transfer provider.
//!
//! The application orchestrator remains provider-neutral. This module is the
//! infrastructure composition point that selects Nextcloud for v1 while
//! keeping the provider-specific type out of the application boundary.
//!
//! The orchestrator and the transfer provider receive separate persistence
//! handles. They may point at the same underlying local store; the separation
//! keeps ownership explicit and avoids coupling the application queue to the
//! provider implementation.

use chrono::DateTime;
use nc_pore_application::synchronization::{
    ArtifactTransfer, ArtifactTransferRequest, ArtifactTransferResult,
    PersistentSynchronizationQueue, SynchronizationWorkStore,
};
use nc_pore_application::synchronization_orchestration::{
    SynchronizationOrchestrationError, SynchronizationOrchestrator, SynchronizationProcessOutcome,
};
use recorder::persistence::PersistenceProvider;

use super::{NextcloudArtifactTransfer, NextcloudConnection, NextcloudTransferMetadata};

/// Application orchestrator configured with the Nextcloud v1 provider.
pub type NextcloudSynchronizationOrchestrator<W, OP, TP> =
    SynchronizationOrchestrator<W, OP, NextcloudMetadataTransfer<TP>>;

/// Adapter that translates provider-neutral transfer metadata into the
/// representation required by the Nextcloud connector.
///
/// Timestamp parsing and the concrete `NextcloudTransferMetadata` type stay in
/// the infrastructure layer. The application only transports semantic values.
pub struct NextcloudMetadataTransfer<P> {
    inner: NextcloudArtifactTransfer<P>,
}

impl<P> NextcloudMetadataTransfer<P>
where
    P: PersistenceProvider,
{
    fn new(connection: NextcloudConnection, persistence: P) -> Self {
        Self {
            inner: NextcloudArtifactTransfer::new(connection, persistence),
        }
    }
}

impl<P> ArtifactTransfer for NextcloudMetadataTransfer<P>
where
    P: PersistenceProvider,
{
    fn transfer(&mut self, request: &ArtifactTransferRequest) -> ArtifactTransferResult {
        let recorded_at = match request.metadata().recorded_at() {
            Some(value) => match DateTime::parse_from_rfc3339(value) {
                Ok(timestamp) => Some(timestamp),
                Err(error) => {
                    return ArtifactTransferResult::PermanentFailure {
                        reason: format!("invalid recording timestamp metadata: {error}"),
                    };
                }
            },
            None => None,
        };

        let metadata = NextcloudTransferMetadata {
            recording_started_at: recorded_at,
            display_name: request.metadata().display_name().map(str::to_owned),
        };

        self.inner.transfer_with_metadata(request, &metadata)
    }
}

/// Builds the concrete v1 synchronization composition.
///
/// `orchestration_persistence` and `transfer_persistence` are intentionally
/// separate values. In production they can be two providers backed by the
/// same local artifact store.
pub fn new_nextcloud_synchronization_orchestrator<W, OP, TP>(
    queue: PersistentSynchronizationQueue<W>,
    orchestration_persistence: OP,
    transfer_persistence: TP,
    connection: NextcloudConnection,
) -> NextcloudSynchronizationOrchestrator<W, OP, TP>
where
    W: SynchronizationWorkStore,
    OP: PersistenceProvider,
    TP: PersistenceProvider,
{
    let transfer = NextcloudMetadataTransfer::new(connection, transfer_persistence);
    SynchronizationOrchestrator::new(queue, orchestration_persistence, transfer)
}

/// Re-export the application outcome type at the concrete provider boundary.
pub type NextcloudSynchronizationOutcome = SynchronizationProcessOutcome;

/// Re-export the application error type at the concrete provider boundary.
pub type NextcloudSynchronizationError = SynchronizationOrchestrationError;

/// Re-export the provider-neutral transfer result for integration callers.
pub type NextcloudTransferResult = ArtifactTransferResult;

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_application::synchronization::InMemorySynchronizationWorkStore;
    use recorder::persistence::InMemoryPersistenceProvider;

    // TEST-01: Nextcloud composition uses the existing vendor-neutral orchestrator.
    #[test]
    fn nextcloud_composition_constructs_without_provider_types_in_application_queue() {
        let config = super::super::NextcloudConnectionConfig::new(
            "https://cloud.example.test",
            super::super::NextcloudCredentials::new("host", "app-password"),
        );
        let connection = NextcloudConnection::new(config).unwrap();
        let queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());

        let _orchestrator = new_nextcloud_synchronization_orchestrator(
            queue,
            InMemoryPersistenceProvider::new(),
            InMemoryPersistenceProvider::new(),
            connection,
        );
    }

    // TEST-02: provider-neutral metadata is interpreted only at the connector boundary.
    #[test]
    fn nextcloud_transfer_rejects_invalid_recording_timestamp_metadata() {
        let request = ArtifactTransferRequest::new_with_metadata(
            nc_pore_core::recording::RecordingArtifactId::new("artifact-1"),
            [0; 32],
            nc_pore_application::synchronization_metadata::ArtifactTransferMetadata::new(
                Some("Test recording".to_owned()),
                Some("not-a-timestamp".to_owned()),
            ),
        );
        let connection = NextcloudConnection::new(super::super::NextcloudConnectionConfig::new(
            "https://cloud.example.test",
            super::super::NextcloudCredentials::new("host", "app-password"),
        ))
        .unwrap();
        let mut transfer = NextcloudMetadataTransfer::new(
            connection,
            InMemoryPersistenceProvider::new(),
        );

        assert!(matches!(
            transfer.transfer(&request),
            ArtifactTransferResult::PermanentFailure { .. }
        ));
    }
}
