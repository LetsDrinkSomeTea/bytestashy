use thiserror::Error;

/// Application-specific error types
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ByteStashyError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication failed: {message}")]
    Auth { message: String },

    #[error("File operation failed: {path} - {source}")]
    FileOperation {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("API error: HTTP {status} - {message}")]
    Api { status: u16, message: String },

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Dialog interaction failed: {0}")]
    Dialog(#[from] dialoguer::Error),

    #[error("Invalid URL: {0}, make sure it starts with 'http://' or 'https://'")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Progress bar template error: {0}")]
    ProgressTemplate(#[from] indicatif::style::TemplateError),
}

/// Convenience type alias for Results with ByteStashyError
pub type Result<T> = std::result::Result<T, ByteStashyError>;

/// Constructors for common error scenarios
impl ByteStashyError {
    #[allow(dead_code)]
    /// Create authentication error
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    /// Create API error with HTTP status
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create file operation error
    pub fn file_operation(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::FileOperation {
            path: path.into(),
            source,
        }
    }

    /// Create input validation error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}
