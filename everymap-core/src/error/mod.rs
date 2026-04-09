use thiserror::Error;

#[derive(Error, Debug)]
pub enum EveryMapError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Provider-specific error: {0}")]
    ProviderError(String),

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("Unknown error occurred")]
    Unknown,
}

pub type EveryMapResult<T> = Result<T, EveryMapError>;
