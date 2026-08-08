//! Error type surfaced by [`pack_trust_task`](crate::pack_trust_task) and
//! [`unpack_trust_task`](crate::unpack_trust_task).

use thiserror::Error;
use trust_tasks_rs::RejectReason;

/// Failure modes the DIDComm binding can produce. Most variants map
/// cleanly onto a SPEC.md §8.3 [`StandardCode`](trust_tasks_rs::StandardCode)
/// when surfaced as an `unpack` failure on the consumer side; the
/// [`Self::into_reject_reason`] convenience does the mapping for callers
/// that want to fold this straight into the framework's
/// [`RejectReason`].
#[derive(Debug, Error)]
pub enum DidcommError {
    /// The underlying `affinidi-messaging-didcomm` call failed.
    #[error("DIDComm error: {0}")]
    Upstream(#[from] affinidi_messaging_didcomm::DIDCommError),

    /// The unpacked envelope arrived as plaintext or as anoncrypt without
    /// a verified sender — neither provides the transport-authenticated
    /// identity the framework's §4.8.1 precedence depends on.
    #[error("envelope lacks an authenticated sender (anoncrypt or plaintext)")]
    UnauthenticatedSender,

    /// The envelope's DIDComm `type` is not the framework's reserved
    /// Trust Tasks envelope type.
    #[error("unexpected DIDComm envelope type: {0}")]
    WrongEnvelopeType(String),

    /// The envelope's `body` did not deserialise into a `TrustTask<P>`.
    #[error("envelope body did not parse as a Trust Task document: {0}")]
    InvalidBody(serde_json::Error),

    /// A DIDComm thread header and its framework member were both present and
    /// disagreed (binding §3.1).
    ///
    /// Maps to `malformedRequest`, **not** `identityMismatch`: a thread
    /// disagreement contests no party's identity, so §8.1's response
    /// suppression rules do not apply. It is a structurally inconsistent
    /// document, and the producer should be told so.
    #[error("DIDComm {header} is {transport:?} but the document's {member} is {in_band:?}")]
    ThreadMismatch {
        /// The DIDComm header name (`thid` or `pthid`).
        header: &'static str,
        /// The framework member name (`threadId` or `parentThreadId`).
        member: &'static str,
        /// Value carried in the DIDComm header.
        transport: String,
        /// Value carried in the Trust Task document.
        in_band: String,
    },

    /// The framework's [`TrustTask`](trust_tasks_rs::TrustTask) failed to
    /// serialise into a JSON value for placement in the DIDComm `body`.
    /// This is effectively impossible for well-formed payload types and
    /// is here to keep the error taxonomy total.
    #[error("could not serialise TrustTask for envelope body: {0}")]
    SerialiseBody(serde_json::Error),
}

impl DidcommError {
    /// Map this transport-level error into a framework
    /// [`RejectReason`] suitable for emitting via
    /// [`TransportHandler::reject`](trust_tasks_rs::TransportHandler::reject).
    ///
    /// The mapping is intentionally conservative: all envelope-level
    /// failures collapse to `malformed_request`, since from the
    /// framework's perspective the bytes simply did not yield a
    /// well-formed Trust Task document.
    pub fn into_reject_reason(self) -> RejectReason {
        match self {
            DidcommError::UnauthenticatedSender => RejectReason::ProofRequired,
            other => RejectReason::MalformedRequest {
                reason: other.to_string(),
            },
        }
    }
}
