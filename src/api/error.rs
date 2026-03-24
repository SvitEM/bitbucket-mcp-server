use std::fmt;

/// API error types for Bitbucket client operations
#[derive(Debug)]
pub enum ApiError {
    /// HTTP request/response errors
    HttpError(reqwest::Error),
    /// JSON parsing errors
    ParseError(serde_json::Error),
    /// Authentication failures
    AuthError(String),
    /// Configuration errors
    ConfigError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::HttpError(e) => write!(f, "HTTP error: {}", e),
            ApiError::ParseError(e) => write!(f, "Parse error: {}", e),
            ApiError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            ApiError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ApiError::HttpError(e) => Some(e),
            ApiError::ParseError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::HttpError(err)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::ParseError(err)
    }
}
