use crate::client::{ClientProductionSession, ClientRole, ClientSessionError, ClientSessionService};
use nc_pore_core::session::repository::ProductionSessionRepository;

/// Materialize the provider-selected Production identity without activating it.
///
/// Creation is restricted to the provider-selected owner. Once materialized,
/// the owner may reconcile the provider participant list; other participants
/// only observe the existing Production.
pub fn ensure_production<R>(
    repository: &mut R,
    production_id: &str,
    actor_id: &str,
    owner_id: &str,
    participant_ids: &[String],
) -> Result<ClientProductionSession, ClientSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut client = ClientSessionService::new(repository);

    match client.get(production_id) {
        Ok(existing) => {
            if actor_id != owner_id {
                return Ok(existing);
            }

            let existing_ids = existing
                .participants
                .iter()
                .map(|participant| participant.id.as_str())
                .collect::<Vec<_>>();
            for participant_id in participant_ids {
                if participant_id != owner_id && !existing_ids.contains(&participant_id.as_str()) {
                    client.add_participant(
                        production_id,
                        actor_id,
                        participant_id,
                        [ClientRole::Participant],
                    )?;
                }
            }

            client.get(production_id)
        }
        Err(ClientSessionError::SessionNotFound) => {
            if actor_id != owner_id {
                return Err(ClientSessionError::Unauthorized);
            }

            client.create(production_id, owner_id)?;
            for participant_id in participant_ids {
                if participant_id != owner_id {
                    client.add_participant(
                        production_id,
                        owner_id,
                        participant_id,
                        [ClientRole::Participant],
                    )?;
                }
            }

            client.get(production_id)
        }
        Err(error) => Err(error),
    }
}

pub fn start_production<R>(
    repository: &mut R,
    production_id: &str,
    actor_id: &str,
) -> Result<ClientProductionSession, ClientSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    ClientSessionService::new(repository).start(production_id, actor_id)
}
