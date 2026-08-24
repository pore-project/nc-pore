use nc_pore_infrastructure::nextcloud::{NextcloudConnectionConfig, WebDavClient};
use std::env;

fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("missing required environment variable: {name}"))
}

#[test]
fn nextcloud_runtime_reality_check() {
    let url = required("NC_PORE_NEXTCLOUD_URL");
    let username = required("NC_PORE_NEXTCLOUD_USER");
    let app_password = required("NC_PORE_NEXTCLOUD_APP_PASSWORD");
    let remote_root = required("NC_PORE_NEXTCLOUD_REMOTE_ROOT");

    let config = NextcloudConnectionConfig::new(
        url,
        nc_pore_infrastructure::nextcloud::NextcloudCredentials::new(username.clone(), app_password),
    );
    let client = WebDavClient::new(&config).expect("Nextcloud configuration must be valid");

    client
        .authenticate(&username)
        .expect("Nextcloud authentication must succeed");

    let root = format!("remote.php/dav/files/{username}/{remote_root}/");
    let entry = client
        .propfind(&root, 0)
        .expect("Pore-Test remote root must be accessible");

    assert!((200..300).contains(&entry.status), "unexpected PROPFIND status: {}", entry.status);

    println!("Nextcloud runtime reality check: authentication and PROPFIND succeeded for remote root '{remote_root}'.");
}
