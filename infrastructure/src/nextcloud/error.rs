#[derive(Debug)]
pub enum NextcloudProviderError {
    InvalidConfiguration(String),
    Transport(WebDavTransportError),
    Authentication,
    Remote {
        status: u16,
        operation: &'static str,
    },
}

impl std::fmt::Display for NextcloudProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfiguration(message) => {
                write!(formatter, "invalid Nextcloud configuration: {message}")
            }
            Self::Transport(error) => write!(formatter, "Nextcloud transport error: {error}"),
            Self::Authentication => write!(formatter, "Nextcloud authentication failed"),
            Self::Remote { status, operation } => {
                write!(
                    formatter,
                    "Nextcloud {operation} request failed with HTTP status {status}"
                )
            }
        }
    }
}

impl std::error::Error for NextcloudProviderError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebDavTransportError {
    message: String,
}

impl WebDavTransportError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WebDavTransportError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WebDavTransportError {}
