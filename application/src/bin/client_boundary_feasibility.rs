//! Feasibility harness for the external client boundary.
//!
//! This is intentionally not a production HTTP server. It tests the smallest
//! real external path: browser -> HTTP -> application client facade -> JSON.
//! Transport, wire format and development identity remain experimental here.

use nc_pore_application::client::{
    ClientProductionSession, ClientRole, ClientSessionError, ClientSessionService,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8787";
const SESSION_ID: &str = "feasibility-session";
const OWNER_ID: &str = "owner-1";

struct InMemoryRepository {
    sessions: Vec<ProductionSession>,
}

impl ProductionSessionRepository for InMemoryRepository {
    type Error = &'static str;

    fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
        if self.sessions.iter().any(|existing| existing.id == session.id) {
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
    let mut repository = InMemoryRepository {
        sessions: Vec::new(),
    };
    let mut client = ClientSessionService::new(&mut repository);
    client
        .create(SESSION_ID, OWNER_ID)
        .expect("feasibility session must be creatable");
    drop(client);

    let listener = TcpListener::bind(ADDRESS)?;
    println!("NC-PoRe external-client feasibility harness: http://{ADDRESS}/");
    println!("GET http://{ADDRESS}/api/sessions/{SESSION_ID}");
    println!("POST http://{ADDRESS}/api/sessions");
    println!("Development identity: {OWNER_ID}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream, &mut repository),
            Err(error) => eprintln!("connection error: {error}"),
        }
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream, repository: &mut InMemoryRepository) {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let bytes_read = match stream.read(&mut chunk) {
        Ok(bytes_read) => bytes_read,
        Err(error) => {
            eprintln!("request read error: {error}");
            return;
        }
    };
    buffer.extend_from_slice(&chunk[..bytes_read]);

    let request = String::from_utf8_lossy(&buffer);
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
        ("GET", "/api/sessions/feasibility-session") => {
            let client = ClientSessionService::new(repository);
            match client.get(SESSION_ID) {
                Ok(session) => (
                    200,
                    "application/json; charset=utf-8",
                    session_json(&session),
                ),
                Err(ClientSessionError::SessionNotFound) => (
                    404,
                    "application/json; charset=utf-8",
                    r#"{"error":"session_not_found"}"#.to_owned(),
                ),
                Err(_) => (
                    500,
                    "application/json; charset=utf-8",
                    r#"{"error":"application_error"}"#.to_owned(),
                ),
            }
        }
        ("POST", "/api/sessions") => create_session(repository, body),
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

fn create_session(
    repository: &mut InMemoryRepository,
    body: &str,
) -> (u16, &'static str, String) {
    let id = match json_field(body, "id") {
        Some(value) if !value.is_empty() => value,
        _ => {
            return (
                400,
                "application/json; charset=utf-8",
                r#"{"error":"invalid_request"}"#.to_owned(),
            )
        }
    };
    let owner = match json_field(body, "owner") {
        Some(value) if !value.is_empty() => value,
        _ => {
            return (
                400,
                "application/json; charset=utf-8",
                r#"{"error":"invalid_request"}"#.to_owned(),
            )
        }
    };

    let mut client = ClientSessionService::new(repository);
    match client.create(&id, &owner) {
        Ok(session) => (
            201,
            "application/json; charset=utf-8",
            session_json(&session),
        ),
        Err(ClientSessionError::Repository(_)) => (
            409,
            "application/json; charset=utf-8",
            r#"{"error":"session_already_exists"}"#.to_owned(),
        ),
        Err(_) => (
            500,
            "application/json; charset=utf-8",
            r#"{"error":"application_error"}"#.to_owned(),
        ),
    }
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
        201 => "Created",
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

    let recordings = session
        .recordings
        .iter()
        .map(|recording| {
            let artifact_id = recording
                .artifact_id
                .as_ref()
                .map(|id| format!("\"{}\"", json_escape(id)))
                .unwrap_or_else(|| "null".to_owned());
            format!(
                "{{\"id\":\"{}\",\"status\":\"{}\",\"artifact_id\":{artifact_id}}}",
                json_escape(&recording.id),
                format!("{:?}", recording.status),
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"id\":\"{}\",\"status\":\"{:?}\",\"participants\":[{}],\"recordings\":[{}]}}",
        json_escape(&session.id),
        session.status,
        participants,
        recordings,
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
  <title>NC-PoRe Client Boundary Feasibility</title>
</head>
<body>
  <h1>NC-PoRe external client boundary</h1>
  <p>This page is a feasibility test, not a production client.</p>
  <button id="create">Create session</button>
  <pre id="result">Loading…</pre>
  <script>
    const result = document.getElementById('result');

    function showResponse(response) {
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      return response.json();
    }

    fetch('/api/sessions/feasibility-session')
      .then(showResponse)
      .then(data => {
        result.textContent = JSON.stringify(data, null, 2);
      })
      .catch(error => {
        result.textContent = `Client boundary failed: ${error}`;
      });

    document.getElementById('create').addEventListener('click', () => {
      const id = `browser-session-${Date.now()}`;
      const owner = 'browser-owner';
      fetch('/api/sessions', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({id, owner})
      })
        .then(showResponse)
        .then(data => {
          result.textContent = JSON.stringify(data, null, 2);
        })
        .catch(error => {
          result.textContent = `Create session failed: ${error}`;
        });
    });
  </script>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn TEST_01_json_escaping_preserves_valid_string_content() {
        assert_eq!(json_escape("guest\"one"), "guest\\\"one");
        assert_eq!(json_escape("line\nnext"), "line\\nnext");
    }

    #[test]
    fn TEST_02_reason_phrase_maps_expected_http_statuses() {
        assert_eq!(reason_phrase(200), "OK");
        assert_eq!(reason_phrase(201), "Created");
        assert_eq!(reason_phrase(400), "Bad Request");
        assert_eq!(reason_phrase(404), "Not Found");
        assert_eq!(reason_phrase(409), "Conflict");
        assert_eq!(reason_phrase(500), "Internal Server Error");
    }

    #[test]
    fn TEST_03_json_field_reads_simple_request_fields() {
        let body = r#"{"id":"session-1","owner":"owner-1"}"#;
        assert_eq!(json_field(body, "id"), Some("session-1".to_owned()));
        assert_eq!(json_field(body, "owner"), Some("owner-1".to_owned()));
    }
}
