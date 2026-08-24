mod artifact_transfer;
mod config;
mod error;
mod synchronization;
mod webdav;

pub use artifact_transfer::NextcloudArtifactTransfer;
pub use config::{NextcloudConnectionConfig, NextcloudCredentials};
pub use error::{NextcloudProviderError, WebDavTransportError};
pub use synchronization::{
    new_nextcloud_synchronization_orchestrator, NextcloudSynchronizationError,
    NextcloudSynchronizationOrchestrator, NextcloudSynchronizationOutcome, NextcloudTransferResult,
};
pub use webdav::{HttpWebDavTransport, WebDavClient, WebDavEntry, WebDavTransport};

/// A configured Nextcloud connection for the v1 provider.
///
/// Credentials remain in the infrastructure layer and are never part of the
/// vendor-neutral synchronization domain.
#[derive(Clone, Debug)]
pub struct NextcloudConnection {
    config: NextcloudConnectionConfig,
}

impl NextcloudConnection {
    pub fn new(config: NextcloudConnectionConfig) -> Result<Self, NextcloudProviderError> {
        config.validate()?;
        Ok(Self { config })
    }

    pub fn config(&self) -> &NextcloudConnectionConfig {
        &self.config
    }

    pub fn client(&self) -> Result<WebDavClient, NextcloudProviderError> {
        WebDavClient::new(&self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_accepts_valid_app_password_configuration() {
        let config = NextcloudConnectionConfig::new(
            "https://cloud.example.test",
            NextcloudCredentials::new("host-user", "app-password"),
        );
        let connection = NextcloudConnection::new(config).unwrap();
        assert_eq!(connection.config().username(), "host-user");
    }

    #[test]
    fn connection_rejects_missing_endpoint() {
        let config = NextcloudConnectionConfig::new(
            "",
            NextcloudCredentials::new("host-user", "app-password"),
        );
        assert!(matches!(
            NextcloudConnection::new(config),
            Err(NextcloudProviderError::InvalidConfiguration(_))
        ));
    }
}
