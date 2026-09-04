//! Feasibility harness for the recording-state external boundary.
//!
//! This is intentionally not a production HTTP server. It proves the smallest
//! concrete adapter path: browser -> HTTP -> RecordingStateSource -> Application
//! read model -> Core-owned state -> browser.

use nc_pore_application::client::ClientSessionService;
use nc_pore_application::recording_state::{
    ClientRecordingParticipant, ClientRecordingPhase, ClientRecordingRole, ClientRecordingState,
};
use nc_pore_application::recording_state_source::RecordingStateSource;
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::participation::Participation;
use nc_pore_core::recording::{Recording, RecordingId};
use nc_pore_core::role::ParticipantRole;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8788";
const SESSION_ID: &str = "recording-state-feasibility-session";
const RECORDING_ID: &str = "recording-001";
const OWNER_ID: &str = "owner-1";

struct InMemoryRepository {
    sessions: Vec<ProductionSession>,
}

impl ProductionSessionRepository for InMemoryRepository {
    type Error = &'static str;

    fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
        if self
            .sessions
            .iter()
            .any(|existing| existing.id == session.id)
        {
            return Err("session already exists");
        }
        self.sessions.push(session.clone());
        Ok(())
    }

    fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
        let existing = self
            .sessions
            .iter_mut()
            .find(|existing| existing.id == session.id)
            .ok_or("session not found")?;
        *existing = session.clone();
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

fn main() -> std::io::Result<()> {
    let mut repository = seeded_repository();
    let listener = TcpListener::bind(ADDRESS)?;
    println!("NC-PoRe recording-state feasibility harness: http://{ADDRESS}/");
    println!("GET http://{ADDRESS}/api/sessions/{SESSION_ID}/recordings/{RECORDING_ID}/state");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &mut repository),
            Err(error) => eprintln!("connection error: {error}"),
        }
    }

    Ok(())
}

fn seeded_repository() -> InMemoryRepository {
    let owner = ParticipantId::new(OWNER_ID);
    let mut session =
        ProductionSession::new_with_actor(ProductionId::new(SESSION_ID), Some(owner.clone()));
    session
        .add_participation_by(
            &owner,
            Participation::with_roles(
                owner.clone(),
                [ParticipantRole::Owner, ParticipantRole::Producer],
            ),
        )
        .expect("owner participation must be valid");
    session.start_by(&owner).expect("session must start");
    session
        .add_recording_by(&owner, Recording::new(RECORDING_ID))
        .expect("recording must be addable");
    session
        .begin_recording_by(&owner, &RecordingId::new(RECORDING_ID), [owner.clone()])
        .expect("recording must be prepared");

    InMemoryRepository {
        sessions: vec![session],
    }
}

fn handle_connection(mut stream: TcpStream, repository: &mut InMemoryRepository) {
    let mut buffer = [0_u8; 4096];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(bytes_read) => bytes_read,
        Err(error) => {
            eprintln!("request read error: {error}");
            return;
        }
    };
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request_line = match request.lines().next() {
        Some(line) => line,
        None => return,
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    let (status, body) = match (method, path) {
        ("GET", "/") => (200, INDEX_HTML.to_owned()),
        (
            "GET",
            "/api/sessions/recording-state-feasibility-session/recordings/recording-001/state",
        ) => {
            let client = ClientSessionService::new(repository);
            match client.read_recording_state(SESSION_ID, OWNER_ID, RECORDING_ID) {
                Ok(state) => (200, recording_state_json(&state)),
                Err(_) => (500, r#"{"error":"application_error"}"#.to_owned()),
            }
        }
        _ => (404, r#"{"error":"not_found"}"#.to_owned()),
    };

    let content_type = if path == "/" {
        "text/html; charset=utf-8"
    } else {
        "application/json; charset=utf-8"
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
        reason = reason_phrase(status),
    );
    if let Err(error) = stream.write_all(response.as_bytes()) {
        eprintln!("response write error: {error}");
    }
}

fn recording_state_json(state: &ClientRecordingState) -> String {
    let participants = state
        .participants
        .iter()
        .map(participant_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"recording_id\":\"{}\",\"phase\":\"{}\",\"role\":\"{}\",\"participants\":[{}],\"confirmed\":{},\"artifact_id\":{}}}",
        json_escape(&state.recording_id),
        phase_name(state.phase),
        role_name(state.role),
        participants,
        state.confirmed,
        state
            .artifact_id
            .as_ref()
            .map(|id| format!("\"{}\"", json_escape(id)))
            .unwrap_or_else(|| "null".to_owned()),
    )
}

fn participant_json(participant: &ClientRecordingParticipant) -> String {
    format!(
        "{{\"id\":\"{}\",\"ready\":{}}}",
        json_escape(&participant.id),
        participant.ready
    )
}

fn phase_name(phase: ClientRecordingPhase) -> &'static str {
    match phase {
        ClientRecordingPhase::Preparing => "Preparing",
        ClientRecordingPhase::Ready => "Ready",
        ClientRecordingPhase::Recording => "Recording",
        ClientRecordingPhase::Stopped => "Stopped",
        ClientRecordingPhase::Completed => "Completed",
    }
}

fn role_name(role: ClientRecordingRole) -> &'static str {
    match role {
        ClientRecordingRole::Host => "Host",
        ClientRecordingRole::Participant => "Participant",
        ClientRecordingRole::Listener => "Listener",
    }
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>NC-PoRe recording state boundary</title></head>
<body>
  <h1>NC-PoRe recording-state external boundary</h1>
  <p>This page proves the concrete feasibility adapter. It is not a production client.</p>
  <pre id="result">Loading…</pre>
  <script>
    fetch('/api/sessions/recording-state-feasibility-session/recordings/recording-001/state')
      .then(response => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return response.json();
      })
      .then(state => {
        document.getElementById('result').textContent = JSON.stringify(state, null, 2);
        window.dispatchEvent(new CustomEvent('pore:recording-state', {detail: state}));
      })
      .catch(error => {
        document.getElementById('result').textContent = `Recording-state boundary failed: ${error}`;
      });
  </script>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn TEST_01_seeded_state_is_authoritative_preparing_state() {
        let mut repository = seeded_repository();
        let client = ClientSessionService::new(&mut repository);
        let state = client
            .read_recording_state(SESSION_ID, OWNER_ID, RECORDING_ID)
            .unwrap();

        assert_eq!(state.phase, ClientRecordingPhase::Preparing);
        assert_eq!(state.role, ClientRecordingRole::Host);
        assert!(!state.confirmed);
        assert_eq!(state.recording_id, RECORDING_ID);
    }

    #[test]
    fn TEST_02_wire_projection_contains_no_transport_state_machine() {
        let state = ClientRecordingState {
            recording_id: RECORDING_ID.to_owned(),
            phase: ClientRecordingPhase::Stopped,
            role: ClientRecordingRole::Participant,
            participants: vec![ClientRecordingParticipant {
                id: "guest-1".to_owned(),
                ready: true,
            }],
            confirmed: false,
            artifact_id: None,
        };

        let json = recording_state_json(&state);
        assert!(json.contains("\"phase\":\"Stopped\""));
        assert!(json.contains("\"ready\":true"));
        assert!(json.contains("\"confirmed\":false"));
    }
}
