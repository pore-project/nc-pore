use nc_pore_infrastructure::nextcloud::{NextcloudConnectionConfig, NextcloudCredentials, WebDavClient};
use std::env;

fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("missing required environment variable: {name}"))
}

#[test]
fn nextcloud_runtime_write_check() {
    let config = NextcloudConnectionConfig::from_environment()
        .expect("Nextcloud runtime configuration must be present and valid");
    let client = WebDavClient::new(&config).expect("Nextcloud client must be constructible");
    let username = config.username().to_owned();
    let remote_root = config.remote_root().to_owned();

    client
        .authenticate(&username)
        .expect("Nextcloud authentication must succeed");

    let marker = format!(".nc-pore-reality-check-{}", std::process::id());
    let path = format!("remote.php/dav/files/{username}/{remote_root}/{marker}");
    let body = b"NC-PoRE runtime reality check\n".to_vec();

    client
        .put(&path, body.clone())
        .expect("Nextcloud must accept a test PUT");

    let status = client.head(&path).expect("test object must be addressable");
    assert!((200..300).contains(&status), "unexpected HEAD status: {status}");

    let fetched = client
        .get_optional(&path)
        .expect("test object must be readable")
        .expect("test object must exist after PUT");
    assert_eq!(fetched.body, body);

    client
        .delete(&path)
        .expect("Nextcloud must accept deletion of the test object");

    assert_eq!(
        client
            .get_optional(&path)
            .expect("deleted test object must be queryable"),
        None
    );

    println!("Nextcloud runtime write check passed: PUT, HEAD, GET and DELETE succeeded for '{remote_root}/{marker}'.");
}
