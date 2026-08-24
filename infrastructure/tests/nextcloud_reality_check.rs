use nc_pore_infrastructure::nextcloud::{NextcloudConnectionConfig, WebDavClient};
use std::env;

#[test]
fn nextcloud_runtime_reality_check() {
    let required = [
        "NC_PORE_NEXTCLOUD_URL",
        "NC_PORE_NEXTCLOUD_USER",
        "NC_PORE_NEXTCLOUD_APP_PASSWORD",
    ];
    if required.iter().any(|name| env::var(name).is_err()) {
        eprintln!("Nextcloud runtime reality check skipped: required credentials are not configured.");
        return;
    }

    let config = NextcloudConnectionConfig::from_environment()
        .expect("Nextcloud runtime environment must form a valid configuration");
    let username = config.username().to_owned();
    let remote_root = config.remote_root().to_owned();
    let client = WebDavClient::new(&config).expect("Nextcloud configuration must be usable");

    client
        .authenticate(&username)
        .expect("Nextcloud authentication must succeed");

    let root = format!("remote.php/dav/files/{username}/{remote_root}/");
    let entry = client
        .propfind(&root, 0)
        .expect("configured Nextcloud remote root must be accessible");

    assert!(
        (200..300).contains(&entry.status),
        "unexpected PROPFIND status: {}",
        entry.status
    );

    println!(
        "Nextcloud runtime reality check passed: authentication and PROPFIND succeeded for remote root '{remote_root}'."
    );
}
