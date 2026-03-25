use thiserror::Error;

#[derive(Error, Debug)]
pub enum PushOverError {
    #[cfg(feature = "reqwest")]
    #[error("API request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API returned error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Webhook signature verification failed")]
    InvalidSignature,
}
