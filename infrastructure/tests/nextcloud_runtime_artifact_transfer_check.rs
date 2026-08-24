use nc_pore_application::synchronization::{
    ArtifactTransferMetadata, ArtifactTransferRequest, ArtifactTransferResult,
};
use nc_pore_core::recording::RecordingArtifactId;
use nc_pore_infrastructure::nextcloud::{
    NextcloudArtifactTransfer, NextcloudConnection, NextcloudConnectionConfig, WebDavClient,
};
use recorder::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use recorder::persistence::{FilesystemPersistenceProvider, PersistenceProvider};
use recorder::session::RecordingSessionId;
use std::env;

struct RemoteCleanup<'a> {
    client: &'a WebDavClient,
    artifact_path: String,
    active: bool,
}

impl<'a> RemoteCleanup<'a> {
    fn new(client: &'a WebDavClient, artifact_path: String) -> Self {
        Self {
            client,
            artifact_path,
            active: true,
        }
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for RemoteCleanup<'_> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let paths = [
            format!(
                "{}/tracks/track-01-track-runtime-check/chunks",
                self.artifact_path
            ),
            format!(
                "{}/tracks/track-01-track-runtime-check",
                self.artifact_path
            ),
            format!("{}/tracks", self.artifact_path),
            self.artifact_path.clone(),
        ];
        for path in paths.into_iter().rev() {
            let _ = self.client.delete(&path);
        }
    }
}

#[test]
fn nextcloud_runtime_artifact_transfer_check() {
    let required = [
        "NC_PORE_NEXTCLOUD_URL",
        "NC_PORE_NEXTCLOUD_USER",
        "NC_PORE_NEXTCLOUD_APP_PASSWORD",
        "NC_PORE_NEXTCLOUD_REMOTE_ROOT",
    ];
    if required.iter().any(|name| env::var(name).is_err()) {
        eprintln!(
            "Nextcloud runtime artifact transfer check skipped: required credentials are not configured."
        );
        return;
    }

    let config = NextcloudConnectionConfig::from_environment()
        .expect("Nextcloud runtime configuration must be valid");
    let username = config.username().to_owned();
    let root = config.remote_root().trim_matches('/').to_owned();
    let connection = NextcloudConnection::new(config.clone())
        .expect("Nextcloud connection configuration must be valid");
    let client = WebDavClient::new(&config).expect("Nextcloud client must be constructible");

    client
        .authenticate(&username)
        .expect("Nextcloud authentication must succeed");

    let artifact_id = format!("artifact-runtime-{}", std::process::id());
    let session_id = RecordingSessionId::new(format!("session-runtime-{}", std::process::id()));
    let mut artifact = RecordingArtifact::new(&artifact_id, session_id);
    artifact.set_domain_association("production-runtime-check", "recording-runtime-check");

    let mut track = RecordingTrack::new("track-runtime-check");
    track.add_chunk(RecordingChunk::with_sample_offset(
        0,
        0,
        "runtime-check-chunk-000000",
        b"NC-PoRE real artifact transfer reality check\n".to_vec(),
    ));
    artifact.add_track(track);
    artifact.make_available();

    let manifest_hash = artifact.manifest_hash();
    let temp_root =
        env::temp_dir().join(format!("nc-pore-runtime-transfer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_root);
    let mut persistence = FilesystemPersistenceProvider::new(&temp_root);
    persistence
        .store_checked(artifact)
        .expect("test artifact must be persisted locally");

    let metadata = ArtifactTransferMetadata::new(
        Some("NC-PoRE Runtime Reality Check".to_owned()),
        Some("2026-08-24T08:00:00+02:00".to_owned()),
    );
    let request = ArtifactTransferRequest::new_with_metadata(
        RecordingArtifactId::new(&artifact_id),
        *manifest_hash.as_bytes(),
        metadata,
    );

    let mut transfer = NextcloudArtifactTransfer::new(connection, persistence);
    let result = transfer.transfer_with_metadata(&request, request.metadata());
    assert_eq!(result, ArtifactTransferResult::Succeeded);

    let expected_prefix = format!(
        "remote.php/dav/files/{username}/{root}/2026/08/24/08-00 - NC-PoRE Runtime Reality Check - {artifact_id}"
    );
    let manifest_path = format!("{expected_prefix}/manifest.json");
    let payload_path = format!(
        "{expected_prefix}/tracks/track-01-track-runtime-check/chunks/chunk-000001.payload"
    );
    let mut cleanup = RemoteCleanup::new(&client, expected_prefix.clone());

    let manifest = client
        .get_optional(&manifest_path)
        .expect("uploaded manifest must be readable")
        .expect("uploaded manifest must exist");
    let manifest_text = String::from_utf8(manifest.body).expect("manifest must be UTF-8 JSON");
    assert!(manifest_text.contains(&artifact_id));
    assert!(manifest_text.contains("track-runtime-check"));
    assert!(manifest_text.contains("NC-PoRE Runtime Reality Check"));

    let payload = client
        .get_optional(&payload_path)
        .expect("uploaded payload must be readable")
        .expect("uploaded payload must exist");
    assert_eq!(
        payload.body,
        b"NC-PoRE real artifact transfer reality check\n"
    );

    client
        .delete(&manifest_path)
        .expect("manifest cleanup must succeed");
    client
        .delete(&payload_path)
        .expect("payload cleanup must succeed");
    client
        .delete(&format!(
            "{expected_prefix}/tracks/track-01-track-runtime-check/chunks"
        ))
        .expect("chunk directory cleanup must succeed");
    client
        .delete(&format!(
            "{expected_prefix}/tracks/track-01-track-runtime-check"
        ))
        .expect("track directory cleanup must succeed");
    client
        .delete(&format!("{expected_prefix}/tracks"))
        .expect("tracks directory cleanup must succeed");
    client
        .delete(&expected_prefix)
        .expect("artifact directory cleanup must succeed");
    cleanup.disarm();

    assert_eq!(
        client
            .get_optional(&manifest_path)
            .expect("cleaned artifact must be queryable"),
        None
    );

    let _ = std::fs::remove_dir_all(temp_root);

    println!(
        "Nextcloud runtime artifact transfer check passed: real artifact transfer, verification and cleanup succeeded for '{root}/{artifact_id}'."
    );
}
