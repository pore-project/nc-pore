use nc_pore_application::synchronization::{ArtifactTransferRequest, ArtifactTransferResult};
use nc_pore_core::recording::RecordingArtifactId;
use nc_pore_infrastructure::nextcloud::{
    NextcloudArtifactTransfer, NextcloudConnection, NextcloudConnectionConfig,
    NextcloudCredentials, WebDavClient, WebDavEntry, WebDavTransport, WebDavTransportError,
};
use recorder::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use recorder::persistence::{FilesystemPersistenceProvider, PersistenceProvider};
use recorder::session::RecordingSessionId;
use reqwest::{Method, Url};

#[derive(Clone, Copy)]
struct StatusTransport {
    status: u16,
}

impl WebDavTransport for StatusTransport {
    fn execute(
        &self,
        _method: Method,
        _url: Url,
        _headers: &[(&str, &str)],
        _body: Option<Vec<u8>>,
    ) -> Result<WebDavEntry, WebDavTransportError> {
        Ok(WebDavEntry {
            status: self.status,
            body: Vec::new(),
        })
    }
}

#[derive(Clone, Copy)]
struct FailingTransport;

impl WebDavTransport for FailingTransport {
    fn execute(
        &self,
        _method: Method,
        _url: Url,
        _headers: &[(&str, &str)],
        _body: Option<Vec<u8>>,
    ) -> Result<WebDavEntry, WebDavTransportError> {
        Err(WebDavTransportError::new("connection reset"))
    }
}

fn persisted_artifact(root: &std::path::Path) -> [u8; 32] {
    let mut persistence = FilesystemPersistenceProvider::new(root);
    let mut artifact = RecordingArtifact::new(
        "artifact-error-classification",
        RecordingSessionId::new("session-error-classification"),
    );
    let mut track = RecordingTrack::new("track-error-classification");
    track.add_chunk(RecordingChunk::with_sample_offset(
        0,
        0,
        "chunk-error-classification",
        b"classification-test-payload".to_vec(),
    ));
    artifact.add_track(track);
    artifact.make_available();
    let hash = *artifact.manifest_hash().as_bytes();
    persistence.store_checked(artifact).unwrap();
    hash
}

fn transfer_with_transport<T: WebDavTransport + 'static>(
    root: &std::path::Path,
    transport: T,
) -> ArtifactTransferResult {
    let hash = persisted_artifact(root);
    let config = NextcloudConnectionConfig::new(
        "https://cloud.example.test",
        NextcloudCredentials::new("test-user", "test-password"),
    );
    let connection = NextcloudConnection::new(config.clone()).unwrap();
    let client = WebDavClient::with_transport(&config, transport).unwrap();
    let persistence = FilesystemPersistenceProvider::new(root);
    let mut transfer = NextcloudArtifactTransfer::new(connection, persistence);
    let request = ArtifactTransferRequest::new(
        RecordingArtifactId::new("artifact-error-classification"),
        hash,
    );
    transfer.transfer_with_client(&client, &request, request.metadata())
}

#[test]
fn nextcloud_provider_errors_have_stable_retry_classification() {
    let cases = [
        (401, "permanent"),
        (403, "permanent"),
        (409, "conflict"),
        (408, "retryable"),
        (429, "retryable"),
        (500, "retryable"),
        (502, "retryable"),
        (503, "retryable"),
    ];

    for (status, expected) in cases {
        let root = std::env::temp_dir().join(format!(
            "nc-pore-error-classification-{status}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let result = transfer_with_transport(&root, StatusTransport { status });

        match expected {
            "permanent" => assert!(matches!(
                result,
                ArtifactTransferResult::PermanentFailure { .. }
            )),
            "conflict" => assert!(matches!(result, ArtifactTransferResult::Conflict { .. })),
            "retryable" => assert!(matches!(
                result,
                ArtifactTransferResult::RetryableFailure { .. }
            )),
            _ => unreachable!(),
        }
        let _ = std::fs::remove_dir_all(root);
    }

    let root = std::env::temp_dir().join(format!(
        "nc-pore-error-classification-transport-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    assert!(matches!(
        transfer_with_transport(&root, FailingTransport),
        ArtifactTransferResult::RetryableFailure { .. }
    ));
    let _ = std::fs::remove_dir_all(root);
}
