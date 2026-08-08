//! Transport-level failures, and how they map into the framework's §8.3
//! vocabulary.

use thiserror::Error;
use trust_tasks_rs::RejectReason;

/// A DIDComm v1 transport failure.
#[derive(Debug, Error)]
pub enum DidcommV1Error {
    /// The underlying `affinidi-messaging-didcomm-v1` call failed.
    #[error("DIDComm v1 error: {0}")]
    Upstream(#[from] affinidi_messaging_didcomm_v1::DIDCommV1Error),

    /// The envelope arrived as anoncrypt or plaintext — neither carries the
    /// transport-authenticated sender §4.8.1 depends on.
    #[error("envelope lacks an authenticated sender (anoncrypt or plaintext)")]
    UnauthenticatedSender,

    /// The envelope was authcrypt'd, and cryptographically sound, but the
    /// authenticating verkey is bound to no DID this agent knows.
    ///
    /// This case has no v2.1 counterpart. A v1 envelope carries no DID — only a
    /// bare verkey — so attribution depends on connection state. Someone holds
    /// the secret half of that key, but this agent cannot say who, and §4.8.1
    /// needs a party identity rather than a key.
    ///
    /// Deliberately distinct from [`UnauthenticatedSender`](Self::UnauthenticatedSender):
    /// that one means *nobody* authenticated the envelope, this one means
    /// somebody did and we cannot name them. Collapsing the two would hide a
    /// missing connection record behind what looks like a hostile message.
    #[error("authenticated by verkey {verkey}, which is bound to no known DID")]
    UnknownSenderBinding {
        /// The base58 verkey that authenticated the envelope.
        verkey: String,
    },

    /// The message's `@type` is not one the binding carries Trust Tasks under.
    #[error("unexpected DIDComm v1 message type: {0}")]
    WrongMessageType(String),

    /// No `~attach` entry carried a Trust Task document.
    #[error("message has no Trust Task attachment (expected ~attach id {expected})")]
    MissingAttachment {
        /// The attachment `@id` the binding reserves.
        expected: &'static str,
    },

    /// The attachment was present but did not deserialise into a `TrustTask<P>`.
    #[error("attachment did not parse as a Trust Task document: {0}")]
    InvalidDocument(serde_json::Error),

    /// A DIDComm thread header and its framework member were both present and
    /// disagreed.
    ///
    /// Maps to `malformedRequest`, **not** `identityMismatch`: a thread
    /// disagreement contests no party's identity, so §8.1's suppression rules
    /// do not apply.
    #[error("~thread {header} is {transport:?} but the document's {member} is {in_band:?}")]
    ThreadMismatch {
        /// The `~thread` field name (`thid` or `pthid`).
        header: &'static str,
        /// The framework member name (`threadId` or `parentThreadId`).
        member: &'static str,
        /// Value carried in the `~thread` decorator.
        transport: String,
        /// Value carried in the Trust Task document.
        in_band: String,
    },

    /// The framework document failed to serialise for placement in the
    /// attachment. Effectively impossible for well-formed payload types; here
    /// to keep the taxonomy total.
    #[error("could not serialise TrustTask for the attachment: {0}")]
    SerialiseDocument(serde_json::Error),
}

impl DidcommV1Error {
    /// Map this transport-level error into a framework [`RejectReason`].
    ///
    /// Conservative, and matching the v2.1 binding: envelope-level failures
    /// collapse to `malformedRequest`, because from the framework's
    /// perspective the bytes simply did not yield a well-formed Trust Task
    /// document.
    ///
    /// The two sender cases map to `proofRequired` instead. Neither yields a
    /// party the framework can attribute the document to, so the only thing
    /// that could still carry attribution is an in-band `proof`.
    pub fn into_reject_reason(self) -> RejectReason {
        match self {
            DidcommV1Error::UnauthenticatedSender | DidcommV1Error::UnknownSenderBinding { .. } => {
                RejectReason::ProofRequired
            }
            other => RejectReason::MalformedRequest {
                reason: other.to_string(),
            },
        }
    }
}
