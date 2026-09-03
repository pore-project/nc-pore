pub mod browser_recording_handoff;
pub mod client;
#[cfg(test)]
mod client_session_context_tests;
pub mod distributed_recording;
pub mod distributed_recording_stop;
pub mod external_session_context;
pub mod recording;
pub mod recording_stop;
pub mod session;
pub mod session_context;
#[cfg(test)]
mod session_context_contract_tests;
pub mod synchronization;
pub mod synchronization_metadata;
pub mod synchronization_orchestration;
pub mod synchronization_persistence;
