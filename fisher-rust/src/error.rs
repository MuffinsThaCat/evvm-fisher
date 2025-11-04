//! Error types for Fisher relayer

use thiserror::Error;

/// Fisher error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Batch too large
    #[error("Batch too large: {0}")]
    BatchTooLarge(String),
    
    /// RPC error
    #[error("RPC error: {0}")]
    Rpc(String),
    
    /// Contract error
    #[error("Contract error: {0}")]
    Contract(String),
    
    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,
    
    /// Batch processing error
    #[error("Batch processing error: {0}")]
    BatchProcessing(String),
    
    /// Attestation error
    #[error("Attestation error: {0}")]
    Attestation(String),
    
    /// Invalid intent
    #[error("Invalid intent: {0}")]
    InvalidIntent(String),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type for Fisher operations
pub type Result<T> = std::result::Result<T, Error>;

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}
