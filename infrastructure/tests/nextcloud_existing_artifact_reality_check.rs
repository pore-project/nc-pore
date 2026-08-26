use nc_pore_application::synchronization::{ArtifactTransferMetadata, ArtifactTransferRequest, ArtifactTransferResult};
use nc_pore_core::recording::RecordingArtifactId;
use nc_pore_infrastructure::nextcloud::{NextcloudArtifactTransfer, NextcloudConnection, NextcloudConnectionConfig, WebDavClient};
use recorder::persistence::{FilesystemPersistenceProvider, PersistenceLoadResult, PersistenceProvider};
use std::{env, path::PathBuf};

#[test]
fn nextcloud_existing_artifact_reality_check() {
    let required = [
        "NC_PORE_NEXTCLOUD_URL",
        "NC_PORE_NEXTCLOUD_USER",
        "NC_PORE_NEXTCLOUD_APP_PASSWORD",
        "NC_PORE_NEXTCLOUD_REMOTE_ROOT",
    ];
    if required.iter().any(|name| env::var(name).is_err()) {
        eprintln!("Nextcloud existing artifact reality check skipped: required credentials are not configured.");
        return;
    }

    let root = env::var("NC_PORE_EXISTING_ARTIFACT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".nc-pore-data"));
    let artifact_id = env::var("NC_PORE_EXISTING_ARTIFACT_ID").unwrap_or_else(|_| "cpal-capture".to_owned());

    let config = NextcloudConnectionConfig::from_environment()
        .expect("Nextcloud runtime configuration must be valid");
    let username = config.username().to_owned();
    let remote_root = config.remote_root().trim_matches('/').to_owned();
    let connection = NextcloudConnection::new(config.clone())
        .expect("Nextcloud connection configuration must be valid");
    let client = WebDavClient::new(&config).expect("Nextcloud client must be constructible");
    client.authenticate(&username).expect("Nextcloud authentication must succeed");

    let persistence = FilesystemPersistenceProvider::new(root);
    let artifact = match persistence.load(&artifact_id) {
        PersistenceLoadResult::Valid(artifact) => artifact,
        other => panic!("existing artifact '{artifact_id}' could not be loaded: {other:?}"),
    };

    let payload_bytes: usize = artifact
        .tracks()
        .iter()
        .flat_map(|track| track.chunks())
        .map(|chunk| chunk.payload().data().len())
        .sum();
    assert!(payload_bytes > 0, "existing artifact must contain payload bytes");

    let manifest_hash = artifact.manifest_hash();
    let recorded_at = "2026-08-22T23:38:00+02:00";
    let display_name = "NC-PoRE Local CPAL Capture";
    let metadata = ArtifactTransferMetadata::new(
        Some(display_name.to_owned()),
        Some(recorded_at.to_owned()),
    );
    let request = ArtifactTransferRequest::new_with_metadata(
        RecordingArtifactId::new(artifact.id.value()),
        *manifest_hash.as_bytes(),
        metadata.clone(),
    );

    let mut transfer = NextcloudArtifactTransfer::new(connection, persistence);
    assert_eq!(
        transfer.transfer_with_metadata(&request, &metadata),
        ArtifactTransferResult::Succeeded
    );

    let recorded_at = recorded_at.parse::<chrono::DateTime<chrono::FixedOffset>>().unwrap();
    let expected_prefix = format!(
        "remote.php/dav/files/{username}/{remote_root}/{}/{} - {} - {}",
        recorded_at.format("%Y/%m/%d"),
        recorded_at.format("%H-%M"),
        display_name,
        artifact.id.value()
    );
    let manifest_path = format!("{expected_prefix}/manifest.json");
    let manifest = client
        .get_optional(&manifest_path)
        .expect("uploaded manifest must be readable")
        .expect("uploaded manifest must exist");
    let manifest_text = String::from_utf8(manifest.body).expect("manifest must be UTF-8");
    assert!(manifest_text.contains(artifact.id.value()));

    let mut remote_payload_bytes = 0usize;
    for track in artifact.tracks() {
        for chunk in track.chunks() {
            let payload_path = format!(
                "{expected_prefix}/tracks/track-{:02}-{}/chunks/chunk-{:06}.payload",
                artifact.tracks().iter().position(|candidate| candidate.id == track.id).unwrap() + 1,
                track.id.value(),
                chunk.sequence
            );
            let payload = client
                .get_optional(&payload_path)
                .expect("uploaded payload must be readable")
                .expect("uploaded payload must exist");
            assert_eq!(payload.body.len(), chunk.payload().data().len());
            assert_eq!(payload.body, chunk.payload().data());
            remote_payload_bytes += payload.body.len();
        }
    }

    println!(
        "Nextcloud existing artifact reality check passed: artifact='{}', local_payload_bytes={}, remote_payload_bytes={}, remote_path='{}'",
        artifact.id.value(), payload_bytes, remote_payload_bytes, expected_prefix
    );
}
