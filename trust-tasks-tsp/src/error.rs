//! Error type surfaced by [`pack_trust_task`](crate::pack_trust_task) and
//! [`unpack_trust_task`](crate::unpack_trust_task).

use affinidi_tsp::MessageType;
use thiserror::Error;
use trust_tasks_rs::{RejectReason, UnanswerableBody};

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

    /// The message is framed in a TSP MAJOR version other than 1 — a peer still
    /// on a TSP draft (Rev 3 `YTSP-AAC` or Rev 2 `YTSP-AAB`, both MAJOR 0) or
    /// on a later MAJOR. Binding `tsp/0.3` §2.2: it is told apart from the
    /// version marker alone, before any key is used or the sender is resolved
    /// (see [`advertised_sender`](crate::advertised_sender)), and must not be
    /// answered — nothing in it is authenticated. Report it to the operator as
    /// a version mismatch: TSP has no version negotiation, so the peer has to
    /// upgrade.
    ///
    /// A TSP MINOR never gates a message (TSP 1.0 §3.1): a 1.x message that then
    /// fails to open is [`Self::Upstream`], as any other unopenable one is.
    #[error(
        "unsupported TSP version: the message is TSP MAJOR {major}, this binding speaks TSP {} only (the sender must upgrade; TSP has no version negotiation)",
        crate::TSP_VERSION
    )]
    UnsupportedTspVersion {
        /// The TSP MAJOR version the message's version marker declares.
        major: u16,
    },

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

    /// A refusal before the pipeline that could not be answered — see
    /// [`UnanswerableBody`]. Not produced through this crate's own unpack,
    /// which always names the verified sender.
    #[error("{0}")]
    Unanswerable(UnanswerableBody),

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
    /// a consumer should normally drop such a message rather than answer it —
    /// and [`Self::UnsupportedTspVersion`] must never be answered at all.
    ///
    /// The reason is a category, never this error's `Display`: that names
    /// VIDs, upstream failures and the deserializer's offsets, which SPEC §10.4
    /// keeps off the wire. Log the error itself.
    pub fn into_reject_reason(self) -> RejectReason {
        match self {
            TspError::InvalidBody(e) => RejectReason::malformed_from_serde(&e),
            TspError::SenderMismatch { .. } => RejectReason::ProofRequired,
            // No authenticated sender: only an in-band proof could attribute it.
            TspError::Unanswerable(UnanswerableBody::NoAuthenticatedSender(_)) => {
                RejectReason::ProofRequired
            }
            _ => RejectReason::MalformedRequest {
                reason: ENVELOPE_MALFORMED.to_string(),
            },
        }
    }
}

/// What [`TspConsumer`](crate::TspConsumer) reports for a `document` it could
/// not answer: one that is not a document at all is
/// [`InvalidBody`](TspError::InvalidBody), as from
/// [`unpack_trust_task`](crate::unpack_trust_task). This crate's unpack always
/// names the verified sender, so the other case does not arise from it; it is
/// reported as the sender failure it would be.
impl From<UnanswerableBody> for TspError {
    fn from(unanswerable: UnanswerableBody) -> Self {
        match unanswerable {
            UnanswerableBody::Unparsable(e) => TspError::InvalidBody(e),
            other => TspError::Unanswerable(other),
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
    fn a_draft_version_is_its_own_failure_and_names_the_version() {
        let draft = TspError::UnsupportedTspVersion { major: 0 };
        let shown = draft.to_string();
        assert!(shown.contains("MAJOR 0"), "{shown}");
        assert!(shown.contains("TSP 1.0"), "{shown}");
        // Never answered, but if a caller folds it anyway nothing leaks.
        assert!(!reason(draft).contains("MAJOR"));
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
