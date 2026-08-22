use crate::nextcloud::NextcloudProviderError;
use reqwest::Url;

#[derive(Clone, PartialEq, Eq)]
pub struct NextcloudCredentials {
    username: String,
    app_password: String,
}

impl std::fmt::Debug for NextcloudCredentials {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NextcloudCredentials")
            .field("username", &self.username)
            .field("app_password", &"<redacted>")
            .finish()
    }
}

impl NextcloudCredentials {
    pub fn new(username: impl Into<String>, app_password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            app_password: app_password.into(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub(crate) fn app_password(&self) -> &str {
        &self.app_password
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NextcloudConnectionConfig {
    endpoint: String,
    credentials: NextcloudCredentials,
}

impl NextcloudConnectionConfig {
    pub fn new(endpoint: impl Into<String>, credentials: NextcloudCredentials) -> Self {
        Self {
            endpoint: endpoint.into(),
            credentials,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn username(&self) -> &str {
        self.credentials.username()
    }

    pub(crate) fn credentials(&self) -> &NextcloudCredentials {
        &self.credentials
    }

    pub(crate) fn base_url(&self) -> Result<Url, NextcloudProviderError> {
        let mut url = Url::parse(&self.endpoint)
            .map_err(|error| NextcloudProviderError::InvalidConfiguration(error.to_string()))?;
        if !url.path().ends_with('/') {
            let path = format!("{}/", url.path());
            url.set_path(&path);
        }
        Ok(url)
    }

    pub(crate) fn validate(&self) -> Result<(), NextcloudProviderError> {
        let url = self.base_url()?;
        if !matches!(url.scheme(), "https" | "http") {
            return Err(NextcloudProviderError::InvalidConfiguration(
                "Nextcloud endpoint must use http or https".into(),
            ));
        }
        if self.credentials.username().trim().is_empty() {
            return Err(NextcloudProviderError::InvalidConfiguration(
                "Nextcloud username must not be empty".into(),
            ));
        }
        if self.credentials.app_password().is_empty() {
            return Err(NextcloudProviderError::InvalidConfiguration(
                "Nextcloud app password must not be empty".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credentials() -> NextcloudCredentials {
        NextcloudCredentials::new("user", "password")
    }

    #[test]
    fn endpoint_path_is_normalized() {
        let config = NextcloudConnectionConfig::new("https://cloud.example.test/", credentials());
        assert_eq!(
            config.base_url().unwrap().as_str(),
            "https://cloud.example.test/"
        );
    }

    #[test]
    fn empty_password_is_rejected() {
        let config = NextcloudConnectionConfig::new(
            "https://cloud.example.test",
            NextcloudCredentials::new("user", ""),
        );
        assert!(config.validate().is_err());
    }

    #[test]
    fn credentials_debug_output_redacts_password() {
        let credentials = credentials();
        let output = format!("{credentials:?}");
        assert!(!output.contains("app-password"));
        assert!(output.contains("<redacted>"));
    }
}
