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
    /// The mapping is conservative: most failures collapse to
    /// `malformedRequest`, since from the framework's perspective the bytes
    /// simply did not yield a well-formed, authenticated Trust Task document.
    ///
    /// The exception is [`Self::SenderMismatch`], which maps to
    /// `proofRequired` as the DIDComm bindings map their sender failures: it
    /// leaves no transport-authenticated sender the consumer can name, so only
    /// an in-band `proof` could still attribute the document. Binding §4 puts
    /// an authentication failure outside the framework pipeline altogether, so
    /// a consumer should normally drop such a message rather than answer it.
    ///
    /// The reason is a category, never this error's `Display`: that names
    /// VIDs, upstream failures and the deserializer's offsets, which SPEC §10.4
    /// keeps off the wire. Log the error itself.
    pub fn into_reject_reason(self) -> RejectReason {
        match self {
            TspError::InvalidBody(e) => RejectReason::malformed_from_serde(&e),
            TspError::SenderMismatch { .. } => RejectReason::ProofRequired,
            _ => RejectReason::MalformedRequest {
                reason: ENVELOPE_MALFORMED.to_string(),
            },
        }
    }
}

/// Wire reason for a `malformedRequest` this binding cannot describe more
/// precisely without echoing an upstream error. The error's `Display` keeps the
/// detail for the log; SPEC §10.4 keeps it off the wire.
const ENVELOPE_MALFORMED: &str = "message did not yield a well-formed Trust Task document";

#[cfg(test)]
mod tests {
    use super::*;

    fn reason(err: TspError) -> String {
        match err.into_reject_reason() {
            RejectReason::MalformedRequest { reason } => reason,
            other => panic!("expected malformedRequest, got {other:?}"),
        }
    }

    #[test]
    fn the_wire_reason_carries_no_detail_from_the_error() {
        let serde = serde_json::from_str::<serde_json::Value>("{\"subject\": ").unwrap_err();
        let parsed = reason(TspError::InvalidBody(serde));
        assert!(
            !parsed.contains("line"),
            "deserializer offset on the wire: {parsed}"
        );

        let other = reason(TspError::WrongEnvelopeType("internal/type".into()));
        assert!(
            !other.contains("internal/type"),
            "envelope detail on the wire: {other}"
        );
    }

    #[test]
    fn a_sender_mismatch_asks_for_a_proof_as_the_didcomm_bindings_do() {
        let mismatch = TspError::SenderMismatch {
            claimed: "did:web:claimed.example".into(),
            verified: "did:web:verified.example".into(),
        };
        assert!(matches!(
            mismatch.into_reject_reason(),
            RejectReason::ProofRequired
        ));
    }
}
