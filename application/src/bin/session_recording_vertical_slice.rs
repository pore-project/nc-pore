//! Browser-to-recorder vertical slice for one local production recording.
//!
//! This proves: browser -> READY -> application recording orchestration ->
//! recorder -> artifact. Authentication, production persistence and WebSocket
//! transport remain outside this slice.

use nc_pore_application::client::{ClientRole, ClientSessionError, ClientSessionService};
use nc_pore_application::recording::execute_recording;
use nc_pore_application::session_context::{
    ProductionSessionContextError, ProductionSessionContextProvider, SessionCapability,
    SessionContext, SessionContextProvider, SessionState,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{Recording, RecordingId};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use recorder::application::RecorderApplication;
use recorder::artifact::coordination::ArtifactCoordinator;
use recorder::artifact::processing::RecordingArtifactProcessor;
use recorder::audio::{
    CaptureProvider, CaptureResult, CaptureStartError, CpalCaptureProvider, RecordingConfiguration,
};
use recorder::persistence::InMemoryPersistenceProvider;
use recorder::session::RecordingSession;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const ADDRESS: &str = "127.0.0.1:8789";
const SESSION_ID: &str = "recording-vertical-slice-session";
const RECORDING_ID: &str = "recording-vertical-slice";
const OWNER_ID: &str = "host-1";
const CAPTURE_DURATION: Duration = Duration::from_secs(3);

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

struct TimedCpalCaptureProvider {
    provider: CpalCaptureProvider,
    duration: Duration,
}

impl CaptureProvider for TimedCpalCaptureProvider {
    fn start_capture(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), CaptureStartError> {
        self.provider.start_capture(configuration)?;
        thread::sleep(self.duration);
        Ok(())
    }

    fn stop_capture(&mut self) -> CaptureResult {
        self.provider.stop_capture()
    }
}

type Recorder = RecorderApplication<TimedCpalCaptureProvider, InMemoryPersistenceProvider>;

struct ServerState {
    repository: InMemoryRepository,
    ready_participants: HashSet<String>,
    recorder: Recorder,
}

fn main() -> std::io::Result<()> {
    let mut repository = InMemoryRepository {
        sessions: Vec::new(),
    };
    let mut client = ClientSessionService::new(&mut repository);
    client
        .create(SESSION_ID, OWNER_ID)
        .expect("vertical-slice session must be creatable");
    drop(client);

    let production_id = ProductionId::new(SESSION_ID);
    let owner = ParticipantId::new(OWNER_ID);
    let recording_id = RecordingId::new(RECORDING_ID);
    let mut session = repository
        .get(&production_id)
        .expect("session lookup must succeed")
        .expect("vertical-slice session must exist");
    session
        .add_recording_by(&owner, Recording::new(RECORDING_ID))
        .expect("vertical-slice recording must be creatable");
    repository
        .update(&session)
        .expect("vertical-slice session update must succeed");

    let recorder = new_recorder();
    let listener = TcpListener::bind(ADDRESS)?;
    println!("NC-PoRe session recording vertical slice: http://{ADDRESS}/");
    println!("Bob can open: http://{ADDRESS}/");

    let mut state = ServerState {
        repository,
        ready_participants: HashSet::new(),
        recorder,
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &mut state),
            Err(error) => eprintln!("connection error: {error}"),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, state: &mut ServerState) {
    let mut buffer = [0_u8; 16 * 1024];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(bytes_read) => bytes_read,
        Err(error) => {
            eprintln!("request read error: {error}");
            return;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut sections = request.splitn(2, "\r\n\r\n");
    let headers = sections.next().unwrap_or_default();
    let body = sections.next().unwrap_or_default();
    let request_line = match headers.lines().next() {
        Some(line) => line,
        None => return,
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    let (status, content_type, body) = match (method, path) {
        ("GET", "/") => (200, "text/html; charset=utf-8", INDEX_HTML.to_owned()),
        ("GET", path) if path.starts_with("/api/sessions/") => {
            get_session_or_context(path, state)
        }
        ("POST", "/api/sessions/recording-vertical-slice-session/join") => {
            join_session(state, body)
        }
        ("POST", "/api/sessions/recording-vertical-slice-session/ready") => {
            mark_ready(state, body)
        }
        ("POST", "/api/sessions/recording-vertical-slice-session/record") => {
            record(state, body)
        }
        _ => response(404, r#"{"error":"not_found"}"#),
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

fn get_session_or_context(path: &str, state: &mut ServerState) -> (u16, &'static str, String) {
    let (route, query) = path.split_once('?').unwrap_or((path, ""));
    let parts: Vec<_> = route.trim_start_matches('/').split('/').collect();
    if parts.len() < 3 || parts[0] != "api" || parts[1] != "sessions" {
        return response(404, r#"{"error":"not_found"}"#);
    }

    let session_id = parts[2];
    let actor_id = query_value(query, "actor").unwrap_or_else(|| "bob-1".to_owned());
    if parts.len() == 4 && parts[3] == "context" {
        let provider = ProductionSessionContextProvider::new(&state.repository);
        return match provider.resolve(session_id, &actor_id) {
            Ok(context) => (200, "application/json; charset=utf-8", context_json(&context)),
            Err(ProductionSessionContextError::SessionNotFound) => {
                response(404, r#"{"error":"session_not_found"}"#)
            }
            Err(ProductionSessionContextError::ActorNotFound) => {
                response(404, r#"{"error":"actor_not_found"}"#)
            }
            Err(ProductionSessionContextError::Repository(_)) => {
                response(500, r#"{"error":"application_error"}"#)
            }
        };
    }

    let client = ClientSessionService::new(&mut state.repository);
    match client.get(session_id) {
        Ok(session) => (200, "application/json; charset=utf-8", session_json(&session)),
        Err(ClientSessionError::SessionNotFound) => {
            response(404, r#"{"error":"session_not_found"}"#)
        }
        Err(_) => response(500, r#"{"error":"application_error"}"#),
    }
}

fn join_session(state: &mut ServerState, body: &str) -> (u16, &'static str, String) {
    let participant_id = match json_field(body, "participant_id") {
        Some(value) if !value.is_empty() => value,
        _ => return response(400, r#"{"error":"invalid_request"}"#),
    };

    let mut client = ClientSessionService::new(&mut state.repository);
    match client.add_participant(
        SESSION_ID,
        OWNER_ID,
        &participant_id,
        [ClientRole::Participant],
    ) {
        Ok(session) => (200, "application/json; charset=utf-8", session_json(&session)),
        Err(ClientSessionError::ParticipantAlreadyExists) => {
            response(409, r#"{"error":"participant_already_exists"}"#)
        }
        Err(ClientSessionError::SessionNotFound) => {
            response(404, r#"{"error":"session_not_found"}"#)
        }
        Err(_) => response(500, r#"{"error":"application_error"}"#),
    }
}

fn mark_ready(state: &mut ServerState, body: &str) -> (u16, &'static str, String) {
    let participant_id = match json_field(body, "participant_id") {
        Some(value) if !value.is_empty() => value,
        _ => return response(400, r#"{"error":"invalid_request"}"#),
    };

    let provider = ProductionSessionContextProvider::new(&state.repository);
    let context = match provider.resolve(SESSION_ID, &participant_id) {
        Ok(context) => context,
        Err(ProductionSessionContextError::ActorNotFound) => {
            return response(404, r#"{"error":"actor_not_found"}"#)
        }
        Err(ProductionSessionContextError::SessionNotFound) => {
            return response(404, r#"{"error":"session_not_found"}"#)
        }
        Err(ProductionSessionContextError::Repository(_)) => {
            return response(500, r#"{"error":"application_error"}"#)
        }
    };

    if context.state != SessionState::Available
        || !context
            .capabilities
            .contains(&SessionCapability::ParticipateInRecording)
    {
        return response(409, r#"{"error":"not_ready_for_recording"}"#);
    }

    state.ready_participants.insert(participant_id.clone());
    (
        200,
        "application/json; charset=utf-8",
        format!(
            "{{\"session_id\":\"{}\",\"actor_id\":\"{}\",\"state\":\"READY\"}}",
            json_escape(SESSION_ID),
            json_escape(&participant_id),
        ),
    )
}

fn record(state: &mut ServerState, body: &str) -> (u16, &'static str, String) {
    let participant_id = match json_field(body, "participant_id") {
        Some(value) if !value.is_empty() => value,
        _ => return response(400, r#"{"error":"invalid_request"}"#),
    };
    if !state.ready_participants.contains(&participant_id) {
        return response(409, r#"{"error":"participant_not_ready"}"#);
    }

    let mut recorder = std::mem::replace(&mut state.recorder, new_recorder());
    let result = execute_recording(
        &mut state.repository,
        &ProductionId::new(SESSION_ID),
        &ParticipantId::new(&participant_id),
        &RecordingId::new(RECORDING_ID),
        &mut recorder,
        &RecordingConfiguration::default(),
    );

    match result {
        Ok(artifact) => {
            let payload_bytes: usize = artifact
                .tracks()
                .iter()
                .flat_map(|track| track.chunks())
                .map(|chunk| chunk.payload().data().len())
                .sum();
            (
                200,
                "application/json; charset=utf-8",
                format!(
                    "{{\"recording_id\":\"{}\",\"artifact_id\":\"{}\",\"tracks\":{},\"payload_bytes\":{},\"state\":\"COMPLETED\"}}",
                    RECORDING_ID,
                    json_escape(artifact.id.value()),
                    artifact.tracks().len(),
                    payload_bytes,
                ),
            )
        }
        Err(error) => response(
            500,
            format!(
                "{{\"error\":\"{}\"}}",
                json_escape(&format!("{error:?}"))
            ),
        ),
    }
}

fn new_recorder() -> Recorder {
    let persistence = InMemoryPersistenceProvider::new();
    let processor = RecordingArtifactProcessor::new(ArtifactCoordinator::new(persistence));
    let capture = TimedCpalCaptureProvider {
        provider: CpalCaptureProvider::new(),
        duration: CAPTURE_DURATION,
    };
    RecorderApplication::new(RecordingSession::new(RECORDING_ID), capture, processor)
}

fn response(status: u16, body: impl Into<String>) -> (u16, &'static str, String) {
    (status, "application/json; charset=utf-8", body.into())
}

fn query_value(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|item| {
        let (name, value) = item.split_once('=')?;
        (name == key).then(|| value.to_owned())
    })
}

fn json_field(body: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\":\"");
    let start = body.find(&marker)? + marker.len();
    let rest = &body[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}

fn session_json(session: &nc_pore_application::client::ClientProductionSession) -> String {
    let participants = session
        .participants
        .iter()
        .map(|participant| {
            let roles = participant
                .roles
                .iter()
                .map(|role| format!("\"{}\"", role_name(*role)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"roles\":[{}]}}",
                json_escape(&participant.id),
                roles
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"id\":\"{}\",\"status\":\"{:?}\",\"participants\":[{}]}}",
        json_escape(&session.id),
        session.status,
        participants
    )
}

fn context_json(context: &SessionContext) -> String {
    let capabilities = context
        .capabilities
        .iter()
        .map(|capability| format!("\"{}\"", capability_name(*capability)))
        .collect::<Vec<_>>()
        .join(",");
    let participants = context
        .participants
        .iter()
        .map(|participant| format!("\"{}\"", json_escape(&participant.id)))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"session_id\":\"{}\",\"state\":\"{:?}\",\"actor_id\":\"{}\",\"participants\":[{}],\"capabilities\":[{}]}}",
        json_escape(&context.session_id),
        context.state,
        json_escape(&context.actor_id),
        participants,
        capabilities
    )
}

fn role_name(role: ClientRole) -> &'static str {
    match role {
        ClientRole::Owner => "Owner",
        ClientRole::Producer => "Producer",
        ClientRole::Participant => "Participant",
        ClientRole::Guest => "Guest",
    }
}

fn capability_name(capability: SessionCapability) -> &'static str {
    match capability {
        SessionCapability::StartSession => "StartSession",
        SessionCapability::CompleteSession => "CompleteSession",
        SessionCapability::ManageParticipants => "ManageParticipants",
        SessionCapability::ManageRecordings => "ManageRecordings",
        SessionCapability::ParticipateInRecording => "ParticipateInRecording",
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
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>NC-PoRe Session Recording</title>
</head>
<body>
  <h1>NC-PoRe session recording vertical slice</h1>
  <p id="status">Opening session…</p>
  <button id="join">Join as Bob</button>
  <button id="ready" disabled>Ready</button>
  <button id="record" disabled>Start Recording</button>
  <pre id="result"></pre>
<script>
const session = 'recording-vertical-slice-session';
const actor = `bob-${crypto.randomUUID()}`;
const status = document.getElementById('status');
const result = document.getElementById('result');
const join = document.getElementById('join');
const ready = document.getElementById('ready');
const record = document.getElementById('record');
async function api(path, options) {
  const response = await fetch(path, options);
  const data = await response.json();
  if (!response.ok) throw new Error(data.error || `HTTP ${response.status}`);
  return data;
}
async function refresh() {
  try {
    const context = await api(`/api/sessions/${session}/context?actor=${encodeURIComponent(actor)}`);
    result.textContent = JSON.stringify(context, null, 2);
    status.textContent = `Session ${context.state}; actor ${context.actor_id}`;
    ready.disabled = !context.capabilities.includes('ParticipateInRecording');
  } catch (error) {
    status.textContent = `Session not joined: ${error}`;
  }
}
join.onclick = async () => {
  try {
    await api(`/api/sessions/${session}/join`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({participant_id: actor})
    });
    join.disabled = true;
    await refresh();
  } catch (error) {
    status.textContent = `Join failed: ${error}`;
  }
};
ready.onclick = async () => {
  try {
    const value = await api(`/api/sessions/${session}/ready`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({participant_id: actor})
    });
    result.textContent = JSON.stringify(value, null, 2);
    status.textContent = 'Bob = READY';
    ready.disabled = true;
    record.disabled = false;
  } catch (error) {
    status.textContent = `Ready failed: ${error}`;
  }
};
record.onclick = async () => {
  try {
    record.disabled = true;
    status.textContent = 'Recording…';
    const value = await api(`/api/sessions/${session}/record`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({participant_id: actor})
    });
    result.textContent = JSON.stringify(value, null, 2);
    status.textContent = 'Recording COMPLETED';
  } catch (error) {
    status.textContent = `Recording failed: ${error}`;
    record.disabled = false;
  }
};
refresh();
</script>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ServerState {
        let mut repository = InMemoryRepository {
            sessions: Vec::new(),
        };
        let mut client = ClientSessionService::new(&mut repository);
        client.create(SESSION_ID, OWNER_ID).unwrap();
        drop(client);

        let production_id = ProductionId::new(SESSION_ID);
        let owner = ParticipantId::new(OWNER_ID);
        let mut session = repository.get(&production_id).unwrap().unwrap();
        session
            .add_recording_by(&owner, Recording::new(RECORDING_ID))
            .unwrap();
        repository.update(&session).unwrap();

        ServerState {
            repository,
            ready_participants: HashSet::new(),
            recorder: new_recorder(),
        }
    }

    #[test]
    fn participant_must_be_ready_before_recording() {
        let mut state = state();
        let result = record(&mut state, r#"{"participant_id":"bob-1"}"#);
        assert_eq!(result.0, 409);
        assert!(result.2.contains("participant_not_ready"));
    }

    #[test]
    fn participant_can_cross_ready_boundary() {
        let mut state = state();
        assert_eq!(
            join_session(&mut state, r#"{"participant_id":"bob-1"}"#).0,
            200
        );
        let ready = mark_ready(&mut state, r#"{"participant_id":"bob-1"}"#);
        assert_eq!(ready.0, 200);
        assert!(state.ready_participants.contains("bob-1"));
    }
}
