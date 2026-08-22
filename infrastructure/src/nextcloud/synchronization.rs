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

use nc_pore_application::synchronization::{
    ArtifactTransferResult, PersistentSynchronizationQueue, SynchronizationOrchestrationError,
    SynchronizationProcessOutcome, SynchronizationWorkStore,
};
use nc_pore_application::synchronization_orchestration::SynchronizationOrchestrator;
use recorder::persistence::PersistenceProvider;

use super::{NextcloudArtifactTransfer, NextcloudConnection};

/// Application orchestrator configured with the Nextcloud v1 provider.
pub type NextcloudSynchronizationOrchestrator<W, OP, TP> = SynchronizationOrchestrator<
    W,
    OP,
    NextcloudArtifactTransfer<TP>,
>;

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
    let transfer = NextcloudArtifactTransfer::new(connection, transfer_persistence);
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
}
