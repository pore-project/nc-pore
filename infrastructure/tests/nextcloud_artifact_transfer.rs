use std::sync::{Arc, Mutex};

use nc_pore_application::synchronization::{
    ArtifactTransferMetadata, ArtifactTransferRequest, ArtifactTransferResult,
};
use nc_pore_core::recording::RecordingArtifactId;
use nc_pore_infrastructure::nextcloud::{
    NextcloudArtifactTransfer, NextcloudConnection, NextcloudConnectionConfig,
    NextcloudCredentials, WebDavClient, WebDavEntry, WebDavTransport,
};
use recorder::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use recorder::persistence::{FilesystemPersistenceProvider, PersistenceProvider};
use recorder::session::RecordingSessionId;
use reqwest::{Method, Url};

#[derive(Clone, Default)]
struct RecordingTransport {
    state: Arc<Mutex<TransportState>>,
}

#[derive(Default)]
struct TransportState {
    requests: Vec<RequestRecord>,
    manifest: Option<Vec<u8>>,
}

struct RequestRecord {
    method: Method,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl WebDavTransport for RecordingTransport {
    fn execute(
        &self,
        method: Method,
        url: Url,
        headers: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<WebDavEntry, nc_pore_infrastructure::nextcloud::WebDavTransportError> {
        let mut state = self.state.lock().unwrap();
        state.requests.push(RequestRecord {
            method: method.clone(),
            url: url.to_string(),
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
            body: body.clone(),
        });

        if method == Method::GET && url.path().ends_with("/manifest.json") {
            return Ok(match state.manifest.clone() {
                Some(manifest) => WebDavEntry {
                    status: 200,
                    body: manifest,
                },
                None => WebDavEntry {
                    status: 404,
                    body: Vec::new(),
                },
            });
        }

        if method == Method::PUT && url.path().ends_with("/manifest.json") {
            state.manifest = body;
            return Ok(WebDavEntry {
                status: 201,
                body: Vec::new(),
            });
        }

        if method == Method::from_bytes(b"MKCOL").unwrap() {
            return Ok(WebDavEntry {
                status: 201,
                body: Vec::new(),
            });
        }

        if method == Method::PUT {
            return Ok(WebDavEntry {
                status: 201,
                body: Vec::new(),
            });
        }

        Ok(WebDavEntry {
            status: 200,
            body: Vec::new(),
        })
    }
}

fn test_artifact() -> RecordingArtifact {
    let mut artifact = RecordingArtifact::new(
        "artifact-e2e-001",
        RecordingSessionId::new("session-e2e-001"),
    );
    artifact.set_domain_association("production-e2e-001", "recording-e2e-001");

    let mut track = RecordingTrack::new("track-host");
    track.add_chunk(RecordingChunk::with_sample_offset(
        0,
        0,
        "host-chunk-000000",
        b"deterministic-audio-payload".to_vec(),
    ));
    artifact.add_track(track);
    artifact.make_available();
    artifact
}

fn test_client(transport: RecordingTransport) -> WebDavClient<RecordingTransport> {
    let config = NextcloudConnectionConfig::new(
        "https://cloud.example.test",
        NextcloudCredentials::new("home-user", "test-app-password"),
    )
    .with_remote_root("recordings");

    WebDavClient::with_transport(&config, transport).unwrap()
}

#[test]
fn complete_artifact_transfer_is_deterministic_and_idempotent() {
    let root = std::env::temp_dir().join("nc-pore-nextcloud-transfer-e2e");
    let _ = std::fs::remove_dir_all(&root);

    let mut persistence = FilesystemPersistenceProvider::new(&root);
    let artifact = test_artifact();
    let manifest_hash = artifact.manifest_hash();
    persistence.store_checked(artifact).unwrap();

    let connection = NextcloudConnection::new(NextcloudConnectionConfig::new(
        "https://cloud.example.test",
        NextcloudCredentials::new("home-user", "test-app-password"),
    ))
    .unwrap();

    let transport = RecordingTransport::default();
    let client = test_client(transport.clone());
    let mut transfer = NextcloudArtifactTransfer::new(connection, persistence);

    let metadata = ArtifactTransferMetadata::new(
        Some("Frizz Feick / Help the man".to_owned()),
        Some("2026-08-23T14:37:52+02:00".to_owned()),
    );
    let request = ArtifactTransferRequest::new_with_metadata(
        RecordingArtifactId::new("artifact-e2e-001"),
        *manifest_hash.as_bytes(),
        metadata,
    );

    let first = transfer.transfer_with_client(&client, &request, request.metadata());
    assert_eq!(first, ArtifactTransferResult::Succeeded);

    let state = transport.state.lock().unwrap();
    let first_request_count = state.requests.len();
    assert!(first_request_count > 4);

    let payload_request = state
        .requests
        .iter()
        .find(|request| {
            request.method == Method::PUT && request.url.ends_with("/chunk-000001.payload")
        })
        .expect("payload upload request");
    assert_eq!(
        payload_request.body.as_deref(),
        Some(b"deterministic-audio-payload".as_slice())
    );
    assert!(payload_request
        .headers
        .iter()
        .any(|(name, value)| name == "OC-Checksum" && value.starts_with("sha256:")));

    let manifest_request = state
        .requests
        .iter()
        .find(|request| request.method == Method::PUT && request.url.ends_with("/manifest.json"))
        .expect("manifest upload request");
    let manifest_body = manifest_request.body.as_ref().expect("manifest body");
    let manifest_text = String::from_utf8_lossy(manifest_body);
    assert!(manifest_text.contains("artifact-e2e-001"));
    assert!(manifest_text.contains("Frizz Feick / Help the man"));
    assert!(manifest_text.contains("2026-08-23T14:37:52+02:00"));
    assert!(manifest_text.contains("host-chunk-000000"));
    drop(state);

    let second = transfer.transfer_with_client(&client, &request, request.metadata());
    assert_eq!(second, ArtifactTransferResult::Succeeded);

    let state = transport.state.lock().unwrap();
    assert_eq!(state.requests.len(), first_request_count + 1);
    assert_eq!(
        state.requests.last().map(|request| &request.method),
        Some(&Method::GET)
    );
    assert!(state
        .requests
        .last()
        .map(|request| request.url.ends_with("/manifest.json"))
        .unwrap_or(false));

    drop(state);
    let _ = std::fs::remove_dir_all(root);
}
