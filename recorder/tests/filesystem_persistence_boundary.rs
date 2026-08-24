use recorder::persistence::{FilesystemPersistenceProvider, PersistenceProvider};

#[test]
fn filesystem_provider_is_available_to_recorder_binary_boundary() {
    let path = std::env::temp_dir().join("nc-pore-filesystem-boundary-test");
    let _ = std::fs::remove_dir_all(&path);

    let provider = FilesystemPersistenceProvider::new(&path);
    drop(provider);

    assert!(path.is_dir());
    let _ = std::fs::remove_dir_all(path);
}
