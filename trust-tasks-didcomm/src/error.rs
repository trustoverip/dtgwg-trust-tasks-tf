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

    /// The envelope arrived as a bare JWS — signed but not encrypted.
    ///
    /// Binding §2 makes authcrypt a **MUST** and §4 puts a non-authcrypt
    /// envelope outside the framework pipeline entirely. A signed-only
    /// envelope is not merely weaker confidentiality: it carries **no
    /// recipient binding at all**, so a single JWS can be replayed to every
    /// party in a deployment and each will verify it. Accepting one also
    /// leaves the consumer with no transport-authenticated *recipient*,
    /// which silently disables SPEC.md §4.8.1's `recipient` cross-check
    /// rather than failing it.
    #[error("envelope arrived signed-only (JWS); this binding requires authcrypt (binding §2)")]
    SignedNotAuthcrypted,

    /// A verified authcrypt `sender_kid` carried no `#fragment`.
    ///
    /// A DIDComm `kid` is a DID **URL** whose fragment names a verification
    /// method. A bare DID is not one, and the binding cannot reduce it to a
    /// transport-authenticated party without guessing. Previously this
    /// yielded `None`, which downgraded an authenticated identity to an
    /// unauthenticated one and skipped the §4.8.1 cross-check — a silent
    /// reclassification, not a rejection.
    #[error("authcrypt sender kid {kid:?} carries no verification-method fragment")]
    UnqualifiedSenderKid {
        /// The `kid` as it arrived.
        kid: String,
    },

    /// The sender the envelope *names* is not the sender it was
    /// *authenticated as*.
    ///
    /// The authcrypt `skid`/`apu` is chosen by the sender, so the DID it
    /// carries is a claim; the DID that actually authenticated is the one
    /// whose public key opened the ECDH-1PU key wrap. Where a caller told
    /// [`unpack_trust_task`](crate::unpack_trust_task) which sender to
    /// expect, the two must agree — otherwise a peer could authenticate as
    /// itself while labelling the envelope with somebody else's DID, and the
    /// binding would hand that label to §4.8.1 as the authenticated sender.
    #[error("envelope names sender {advertised:?} but authenticated as {expected:?}")]
    SenderKidMismatch {
        /// The DID the envelope was unpacked against — the key that opened it.
        expected: String,
        /// The DID the envelope's own `skid`/`apu` claims.
        advertised: String,
    },

    /// The envelope's sender is not on this consumer's inbound allowlist.
    ///
    /// See [`SenderAllowlist`](crate::SenderAllowlist).
    #[error("sender {did:?} is not on this consumer's inbound allowlist")]
    SenderNotAllowed {
        /// The DID named by the envelope's `skid`.
        did: String,
    },

    /// The wire bytes are not an authcrypt JWE naming a sender, so the
    /// pre-decrypt allowlist check has nothing to check.
    #[error("not an authcrypt JWE with a sender kid: {0}")]
    NotAuthcryptJwe(String),

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
    ///
    /// The exception is the family of failures that leave the consumer with
    /// **no transport-authenticated sender it can name** — anoncrypt or
    /// plaintext, a signed-only envelope, a fragment-less `sender_kid`, a
    /// `skid` that disagrees with the key that opened the envelope, and a
    /// sender outside the allowlist. Each maps to `proofRequired`: the only
    /// thing that could still attribute such a document is an in-band
    /// `proof`.
    ///
    /// Note that binding §4 puts these *outside* the framework pipeline
    /// altogether — there is no authenticated party to route a
    /// `trust-task-error` response to, so a consumer should normally drop
    /// the message rather than answer it. This mapping exists for callers
    /// that fold every transport failure into one taxonomy for logging.
    pub fn into_reject_reason(self) -> RejectReason {
        match self {
            DidcommError::UnauthenticatedSender
            | DidcommError::SignedNotAuthcrypted
            | DidcommError::UnqualifiedSenderKid { .. }
            | DidcommError::SenderKidMismatch { .. }
            | DidcommError::SenderNotAllowed { .. }
            | DidcommError::NotAuthcryptJwe(_) => RejectReason::ProofRequired,
            other => RejectReason::MalformedRequest {
                reason: other.to_string(),
            },
        }
    }
}
