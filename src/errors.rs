use std::time::Duration;
use thiserror::Error;

/// Main error hierarchy for the Aeitron Rust SDK.
#[derive(Debug, Error)]
pub enum AeitronError {
    #[error("aeitron: invalid api key: {0}")]
    InvalidApiKey(String),

    #[error("aeitron: authentication failed (HTTP 401): {message}")]
    AuthenticationError {
        message: String,
        request_id: Option<String>,
    },

    #[error("aeitron: permission denied (HTTP 403): {message}")]
    PermissionDeniedError {
        message: String,
        request_id: Option<String>,
    },

    #[error("aeitron: resource not found (HTTP 404): {message}")]
    NotFoundError {
        message: String,
        request_id: Option<String>,
    },

    #[error("aeitron: conflict error (HTTP 409): {message}")]
    ConflictError {
        message: String,
        request_id: Option<String>,
    },

    #[error("aeitron: rate limit exceeded (HTTP 429): {message}")]
    RateLimitError {
        message: String,
        retry_after: Option<Duration>,
        request_id: Option<String>,
    },

    #[error("aeitron: server error (HTTP {status}): {message}")]
    ServerError {
        status: u16,
        message: String,
        request_id: Option<String>,
    },

    #[error("aeitron: network request error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("aeitron: serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("aeitron: webhook verification error: {0}")]
    WebhookError(#[from] WebhookError),
}

/// Errors occurring during webhook HMAC-SHA256 verification.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WebhookError {
    #[error("invalid webhook signature header format (expected t=...,v1=...)")]
    InvalidHeader(String),

    #[error("webhook timestamp expired (age {age:?} exceeds tolerance {tolerance:?})")]
    TimestampExpired { age: Duration, tolerance: Duration },

    #[error("webhook signature mismatch (invalid secret or tampered payload)")]
    SignatureMismatch,

    #[error("webhook signing secret cannot be empty")]
    MissingSecret,
}
