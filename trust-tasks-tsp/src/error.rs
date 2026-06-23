//! Error type surfaced by [`pack_trust_task`](crate::pack_trust_task) and
//! [`unpack_trust_task`](crate::unpack_trust_task).

use affinidi_tsp::MessageType;
use thiserror::Error;
use trust_tasks_rs::RejectReason;

/// Failure modes the TSP binding can produce. When surfaced as an `unpack`
/// failure on the consumer side, [`Self::into_reject_reason`] folds each into a
/// framework [`RejectReason`] for callers that want to emit it via
/// [`TransportHandler::reject`](trust_tasks_rs::TransportHandler::reject).
#[derive(Debug, Error)]
pub enum TspError {
    /// The underlying `affinidi-tsp` seal/unseal call failed (bad keys, a
    /// signature that didn't verify, a malformed TSP message, …).
    #[error("TSP error: {0}")]
    Upstream(#[from] affinidi_tsp::TspError),

    /// The message opened as a non-`Direct` carriage. Routed/Nested relaying is
    /// the mediator's job; the consumer only ever opens the innermost `Direct`
    /// message, so anything else here is unexpected.
    #[error("unsupported TSP carriage: only Direct is handled (got {0:?})")]
    UnsupportedCarriage(MessageType),

    /// The message's cleartext sender VID does not match the VID whose key
    /// actually verified the signature — the envelope claims an identity it
    /// cannot prove.
    #[error("claimed sender VID `{claimed}` does not match the verified sender `{verified}`")]
    SenderMismatch {
        /// The `VID_sndr` claimed in the cleartext envelope.
        claimed: String,
        /// The VID whose key verified the signature.
        verified: String,
    },

    /// The decrypted payload's `type` member is not the framework's reserved
    /// Trust Tasks envelope type. A valid TSP message that simply isn't a Trust
    /// Task envelope lands here.
    #[error("unexpected envelope type: {0}")]
    WrongEnvelopeType(String),

    /// The decrypted payload did not parse as a Trust Tasks envelope, or its
    /// `document` did not deserialise into a `TrustTask<P>`.
    #[error("payload did not parse as a Trust Task envelope: {0}")]
    InvalidBody(serde_json::Error),

    /// The framework's [`TrustTask`](trust_tasks_rs::TrustTask) failed to
    /// serialise into JSON for placement in the envelope. Effectively impossible
    /// for well-formed payload types; here to keep the taxonomy total.
    #[error("could not serialise TrustTask for the envelope: {0}")]
    SerialiseBody(serde_json::Error),
}

impl TspError {
    /// Map this transport-level error into a framework [`RejectReason`].
    ///
    /// The mapping is conservative: every failure collapses to
    /// `malformedRequest`, since from the framework's perspective the bytes
    /// simply did not yield a well-formed, authenticated Trust Task document.
    pub fn into_reject_reason(self) -> RejectReason {
        RejectReason::MalformedRequest {
            reason: self.to_string(),
        }
    }
}
