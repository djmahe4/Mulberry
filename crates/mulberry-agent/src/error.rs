use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Agent not found: {0}")]
    NotFound(String),

    #[error("Agent endpoint not configured")]
    NoEndpoint,

    #[error("Invalid response from agent: {0}")]
    InvalidResponse(String),

    #[error("Agent task was cancelled")]
    Cancelled,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
