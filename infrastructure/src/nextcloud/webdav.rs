use crate::nextcloud::{NextcloudConnectionConfig, NextcloudProviderError, WebDavTransportError};
use reqwest::{blocking::Client, Method, StatusCode, Url};
use std::sync::Arc;

const DAV_ROOT: &str = "remote.php/dav/files/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDavEntry {
    pub status: u16,
    pub body: Vec<u8>,
}

pub trait WebDavTransport: Send + Sync {
    fn execute(
        &self,
        method: Method,
        url: Url,
        headers: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<WebDavEntry, WebDavTransportError>;
}

pub struct HttpWebDavTransport {
    client: Client,
    username: String,
    app_password: String,
}

impl HttpWebDavTransport {
    fn new(config: &NextcloudConnectionConfig) -> Result<Self, NextcloudProviderError> {
        let client = Client::builder().build().map_err(|error| {
            NextcloudProviderError::Transport(WebDavTransportError::new(error.to_string()))
        })?;
        Ok(Self {
            client,
            username: config.username().to_owned(),
            app_password: config.credentials().app_password().to_owned(),
        })
    }
}

impl WebDavTransport for HttpWebDavTransport {
    fn execute(
        &self,
        method: Method,
        url: Url,
        headers: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<WebDavEntry, WebDavTransportError> {
        let mut request = self
            .client
            .request(method, url)
            .basic_auth(&self.username, Some(&self.app_password));
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        if let Some(body) = body {
            request = request.body(body);
        }
        let response = request
            .send()
            .map_err(|error| WebDavTransportError::new(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .bytes()
            .map_err(|error| WebDavTransportError::new(error.to_string()))?
            .to_vec();
        Ok(WebDavEntry { status, body })
    }
}

pub struct WebDavClient<T = HttpWebDavTransport> {
    base_url: Url,
    transport: Arc<T>,
}

impl WebDavClient<HttpWebDavTransport> {
    pub fn new(config: &NextcloudConnectionConfig) -> Result<Self, NextcloudProviderError> {
        config.validate()?;
        let base_url = config.base_url()?;
        let transport = HttpWebDavTransport::new(config)?;
        Ok(Self {
            base_url,
            transport: Arc::new(transport),
        })
    }
}

impl<T: WebDavTransport> WebDavClient<T> {
    pub fn with_transport(
        config: &NextcloudConnectionConfig,
        transport: T,
    ) -> Result<Self, NextcloudProviderError> {
        config.validate()?;
        Ok(Self {
            base_url: config.base_url()?,
            transport: Arc::new(transport),
        })
    }

    pub(crate) fn url_for(&self, path: &str) -> Result<Url, NextcloudProviderError> {
        self.base_url
            .join(path.trim_start_matches('/'))
            .map_err(|error| NextcloudProviderError::InvalidConfiguration(error.to_string()))
    }

    pub fn authenticate(&self, username: &str) -> Result<(), NextcloudProviderError> {
        let path = format!("{DAV_ROOT}{username}/");
        let response = self.request(
            Method::from_bytes(b"PROPFIND").expect("PROPFIND is a valid HTTP method"),
            &path,
            &[("Depth", "0")],
            None,
            "authentication",
        )?;
        if (200..=299).contains(&response.status) {
            Ok(())
        } else {
            Err(NextcloudProviderError::Remote {
                status: response.status,
                operation: "authentication",
            })
        }
    }

    pub fn propfind(&self, path: &str, depth: u8) -> Result<WebDavEntry, NextcloudProviderError> {
        let method = Method::from_bytes(b"PROPFIND").expect("PROPFIND is a valid HTTP method");
        let depth = depth.to_string();
        self.request(method, path, &[("Depth", &depth)], None, "PROPFIND")
    }

    pub fn mkcol(&self, path: &str) -> Result<(), NextcloudProviderError> {
        self.mkcol_with_headers(path, &[])
    }

    pub fn mkcol_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<(), NextcloudProviderError> {
        let method = Method::from_bytes(b"MKCOL").expect("MKCOL is a valid HTTP method");
        let response = self.request(method, path, headers, None, "MKCOL")?;
        if response.status == StatusCode::CREATED.as_u16()
            || response.status == StatusCode::NO_CONTENT.as_u16()
        {
            return Ok(());
        }
        Err(NextcloudProviderError::Remote {
            status: response.status,
            operation: "MKCOL",
        })
    }

    pub fn put(&self, path: &str, body: Vec<u8>) -> Result<(), NextcloudProviderError> {
        self.put_with_headers(path, body, &[])
    }

    pub fn put_with_headers(
        &self,
        path: &str,
        body: Vec<u8>,
        headers: &[(&str, &str)],
    ) -> Result<(), NextcloudProviderError> {
        let response = self.request(Method::PUT, path, headers, Some(body), "PUT")?;
        if matches!(response.status, 200 | 201 | 204) {
            return Ok(());
        }
        Err(NextcloudProviderError::Remote {
            status: response.status,
            operation: "PUT",
        })
    }

    pub fn head(&self, path: &str) -> Result<u16, NextcloudProviderError> {
        let response = self.request(Method::HEAD, path, &[], None, "HEAD")?;
        Ok(response.status)
    }

    pub fn get_optional(&self, path: &str) -> Result<Option<WebDavEntry>, NextcloudProviderError> {
        let response = self.request_raw(Method::GET, path, &[], None)?;
        match response.status {
            404 => Ok(None),
            401 | 403 => Err(NextcloudProviderError::Authentication),
            status if status >= 400 => Err(NextcloudProviderError::Remote {
                status,
                operation: "GET",
            }),
            _ => Ok(Some(response)),
        }
    }

    pub fn move_with_headers(
        &self,
        path: &str,
        headers: &[(&str, &str)],
    ) -> Result<(), NextcloudProviderError> {
        let method = Method::from_bytes(b"MOVE").expect("MOVE is a valid HTTP method");
        let response = self.request(method, path, headers, None, "MOVE")?;
        if matches!(response.status, 200 | 201 | 204) {
            return Ok(());
        }
        Err(NextcloudProviderError::Remote {
            status: response.status,
            operation: "MOVE",
        })
    }

    fn request(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, &str)],
        body: Option<Vec<u8>>,
        operation: &'static str,
    ) -> Result<WebDavEntry, NextcloudProviderError> {
        let response = self.request_raw(method, path, headers, body)?;
        if response.status == 401 || response.status == 403 {
            Err(NextcloudProviderError::Authentication)
        } else if response.status >= 400 {
            Err(NextcloudProviderError::Remote {
                status: response.status,
                operation,
            })
        } else {
            Ok(response)
        }
    }

    fn request_raw(
        &self,
        method: Method,
        path: &str,
        headers: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<WebDavEntry, NextcloudProviderError> {
        let url = self.url_for(path)?;
        self.transport
            .execute(method, url, headers, body)
            .map_err(NextcloudProviderError::Transport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Clone)]
    struct FakeTransport {
        requests: Arc<Mutex<Vec<(Method, String, Vec<(String, String)>, Option<Vec<u8>>)>>>,
        response: WebDavEntry,
    }

    impl FakeTransport {
        fn new(status: u16) -> Self {
            Self {
                requests: Arc::new(Mutex::new(Vec::new())),
                response: WebDavEntry {
                    status,
                    body: b"ok".to_vec(),
                },
            }
        }
    }

    impl WebDavTransport for FakeTransport {
        fn execute(
            &self,
            method: Method,
            url: Url,
            headers: &[(&str, &str)],
            body: Option<Vec<u8>>,
        ) -> Result<WebDavEntry, WebDavTransportError> {
            self.requests.lock().unwrap().push((
                method,
                url.to_string(),
                headers
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                    .collect(),
                body,
            ));
            Ok(self.response.clone())
        }
    }

    fn config() -> NextcloudConnectionConfig {
        NextcloudConnectionConfig::new(
            "https://cloud.example.test/",
            crate::nextcloud::NextcloudCredentials::new("host-user", "app-password"),
        )
    }

    #[test]
    fn propfind_uses_requested_depth_and_path() {
        let transport = FakeTransport::new(207);
        let requests = Arc::clone(&transport.requests);
        let client = WebDavClient::with_transport(&config(), transport).unwrap();
        let response = client
            .propfind("remote.php/dav/files/host-user/", 0)
            .unwrap();
        assert_eq!(response.status, 207);
        let request = requests.lock().unwrap().last().unwrap().clone();
        assert_eq!(request.0, Method::from_bytes(b"PROPFIND").unwrap());
        assert_eq!(
            request.1,
            "https://cloud.example.test/remote.php/dav/files/host-user/"
        );
        assert_eq!(request.2, vec![("Depth".into(), "0".into())]);
    }

    #[test]
    fn authentication_failure_is_mapped_without_leaking_http_details_to_core() {
        let client = WebDavClient::with_transport(&config(), FakeTransport::new(401)).unwrap();
        assert!(matches!(
            client.authenticate("host-user"),
            Err(NextcloudProviderError::Authentication)
        ));
    }

    #[test]
    fn put_accepts_created_response() {
        let client = WebDavClient::with_transport(&config(), FakeTransport::new(201)).unwrap();
        assert_eq!(client.put("audio/test.bin", b"data".to_vec()), Ok(()));
    }

    #[test]
    fn optional_get_maps_not_found_to_none() {
        let client = WebDavClient::with_transport(&config(), FakeTransport::new(404)).unwrap();
        assert_eq!(client.get_optional("audio/manifest.json").unwrap(), None);
    }

    #[test]
    fn move_accepts_created_response_and_preserves_headers() {
        let transport = FakeTransport::new(201);
        let requests = Arc::clone(&transport.requests);
        let client = WebDavClient::with_transport(&config(), transport).unwrap();
        client
            .move_with_headers(
                "remote.php/dav/uploads/host-user/upload/.file",
                &[("Destination", "https://cloud.example.test/dest")],
            )
            .unwrap();
        let request = requests.lock().unwrap().last().unwrap().clone();
        assert_eq!(request.0, Method::from_bytes(b"MOVE").unwrap());
        assert_eq!(
            request.2,
            vec![(
                "Destination".into(),
                "https://cloud.example.test/dest".into()
            )]
        );
    }

    #[test]
    fn url_for_resolves_absolute_webdav_paths() {
        let client = WebDavClient::with_transport(&config(), FakeTransport::new(200)).unwrap();
        assert_eq!(
            client
                .url_for("remote.php/dav/files/host-user/audio/test.bin")
                .unwrap()
                .as_str(),
            "https://cloud.example.test/remote.php/dav/files/host-user/audio/test.bin"
        );
    }

    #[test]
    fn transport_failure_is_mapped_to_provider_error() {
        #[derive(Clone)]
        struct FailingTransport;

        impl WebDavTransport for FailingTransport {
            fn execute(
                &self,
                _method: Method,
                _url: Url,
                _headers: &[(&str, &str)],
                _body: Option<Vec<u8>>,
            ) -> Result<WebDavEntry, WebDavTransportError> {
                Err(WebDavTransportError::new("network unavailable"))
            }
        }

        let client = WebDavClient::with_transport(&config(), FailingTransport).unwrap();
        assert!(matches!(
            client.head("audio/test.bin"),
            Err(NextcloudProviderError::Transport(_))
        ));
    }
}
