//! Shared error type used by both the client and server.
//!
//! Most error responses *that the framework defines* are themselves
//! Trust Task documents (`trust-task-error/0.1`); the consumer-side
//! pipeline assembles them via [`trust_tasks_rs::TransportHandler::reject`].
//! [`TransportError`] is reserved for failures that don't fit the
//! framework's payload model — connectivity issues, malformed JSON
//! arriving at the transport layer, etc.

use thiserror::Error;

/// Transport-level failures: I/O, broken HTTP framing, body that doesn't
/// even parse as JSON. Distinct from `trust-task-error/0.1` documents,
/// which a well-behaved peer returns inside an HTTP response body and
/// which this crate surfaces as typed values via [`crate::ClientError`].
#[derive(Debug, Error)]
pub enum TransportError {
    /// The request or response body could not be deserialised as JSON.
    #[error("transport body is not valid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// The transport-authenticated identity was required but unavailable
    /// (no bearer header, no recognised token).
    #[error("transport authentication required but not provided")]
    AuthRequired,

    /// Any other transport-level failure surfaced by the underlying
    /// implementation (axum / reqwest / tokio).
    #[error("transport failure: {0}")]
    Other(String),
}
