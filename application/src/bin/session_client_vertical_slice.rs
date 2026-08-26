//! Minimal browser/client vertical slice for the first real session path.
//!
//! This deliberately proves only the boundary:
//! browser -> HTTP -> application client facade -> Core -> browser state.
//! Authentication, production persistence, WebSocket transport and recording
//! remain outside this slice.

use nc_pore_application::client::{
    ClientProductionSession, ClientRole, ClientSessionError, ClientSessionService,
};
use nc_pore_application::session_context::{
    ProductionSessionContextError, ProductionSessionContextProvider, SessionCapability,
    SessionContext, SessionContextProvider, SessionState,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8788";
const SESSION_ID: &str = "vertical-slice-session";
const OWNER_ID: &str = "host-1";

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

struct ServerState {
    repository: InMemoryRepository,
    ready_participants: HashSet<String>,
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

    let listener = TcpListener::bind(ADDRESS)?;
    println!("NC-PoRe session/client vertical slice: http://{ADDRESS}/");
    println!("Bob can open: http://{ADDRESS}/?session={SESSION_ID}");

    let mut state = ServerState {
        repository,
        ready_participants: HashSet::new(),
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
        ("GET", path) if path.starts_with("/api/sessions/") => get_session_or_context(path, state),
        ("POST", "/api/sessions/vertical-slice-session/join") => join_session(state, body),
        ("POST", "/api/sessions/vertical-slice-session/ready") => mark_ready(state, body),
        _ => (
            404,
            "application/json; charset=utf-8",
            r#"{"error":"not_found"}"#.to_owned(),
        ),
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
            Ok(context) => (
                200,
                "application/json; charset=utf-8",
                context_json(&context),
            ),
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

    let client = ClientSessionService::new(&state.repository);
    match client.get(session_id) {
        Ok(session) => (
            200,
            "application/json; charset=utf-8",
            session_json(&session),
        ),
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
        Ok(session) => (
            200,
            "application/json; charset=utf-8",
            session_json(&session),
        ),
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

fn response(status: u16, body: &'static str) -> (u16, &'static str, String) {
    (status, "application/json; charset=utf-8", body.to_owned())
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

fn session_json(session: &ClientProductionSession) -> String {
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
        participants,
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
        capabilities,
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
  <title>NC-PoRe Session Client</title>
</head>
<body>
  <h1>NC-PoRe session client</h1>
  <p id="status">Opening session…</p>
  <button id="join">Join as Bob</button>
  <button id="ready" disabled>Ready</button>
  <pre id="result"></pre>
<script>
const params = new URLSearchParams(location.search);
const session = params.get('session') || 'vertical-slice-session';
const actor = `bob-${crypto.randomUUID()}`;
const status = document.getElementById('status');
const result = document.getElementById('result');
const join = document.getElementById('join');
const ready = document.getElementById('ready');

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
  } catch (error) {
    status.textContent = `Ready failed: ${error}`;
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
        client
            .create(SESSION_ID, OWNER_ID)
            .expect("session must be creatable");
        drop(client);
        ServerState {
            repository,
            ready_participants: HashSet::new(),
        }
    }

    // TEST-01: The browser boundary can read the Core-backed session state.
    #[test]
    fn session_can_be_created_and_read_through_application_boundary() {
        let state = state();
        let response = get_session_or_context("/api/sessions/vertical-slice-session", &state);
        assert_eq!(response.0, 200);
        assert!(response.2.contains(r#""status":"Created""#));
    }

    // TEST-02: Bob joins as a recording participant and receives the recording capability.
    #[test]
    fn participant_can_join_and_resolve_recording_capability() {
        let mut state = state();
        assert_eq!(
            join_session(&mut state, r#"{"participant_id":"bob-1"}"#).0,
            200
        );

        let response = get_session_or_context(
            "/api/sessions/vertical-slice-session/context?actor=bob-1",
            &state,
        );
        assert_eq!(response.0, 200);
        assert!(response.2.contains("ParticipateInRecording"));
        assert!(response.2.contains(r#""state":"Available""#));
    }

    // TEST-03: Bob crosses the readiness boundary only after capability resolution.
    #[test]
    fn participant_can_be_marked_ready() {
        let mut state = state();
        assert_eq!(
            join_session(&mut state, r#"{"participant_id":"bob-1"}"#).0,
            200
        );

        let ready = mark_ready(&mut state, r#"{"participant_id":"bob-1"}"#);

        assert_eq!(ready.0, 200);
        assert!(ready.2.contains(r#""state":"READY""#));
        assert!(state.ready_participants.contains("bob-1"));
    }

    // TEST-04: An unknown actor cannot cross the readiness boundary.
    #[test]
    fn unknown_actor_cannot_be_marked_ready() {
        let mut state = state();
        let ready = mark_ready(&mut state, r#"{"participant_id":"bob-1"}"#);

        assert_eq!(ready.0, 404);
        assert_eq!(ready.2, r#"{"error":"actor_not_found"}"#);
    }
}
