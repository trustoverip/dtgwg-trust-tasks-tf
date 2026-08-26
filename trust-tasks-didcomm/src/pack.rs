//! `pack_trust_task` / `unpack_trust_task` — convert a typed
//! [`TrustTask`] into and out of a DIDComm v2.1 envelope.
//!
//! The envelope's `type` is the framework-reserved
//! [`ENVELOPE_TYPE`] URI; the `body` carries the full `TrustTask<P>`
//! JSON. The outer envelope's authcrypt'd verified `sender_kid` (a DID
//! URL like `did:peer:2.Ez6...#key-agreement-1`) is stripped of its
//! fragment and surfaced as the binding's transport-authenticated peer.

use std::collections::BTreeSet;

use affinidi_messaging_didcomm::{DIDCommAgent, Message, UnpackResult};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde::{de::DeserializeOwned, Serialize};
use trust_tasks_rs::{Payload, TrustTask};

use crate::error::DidcommError;
use crate::handler::DidcommHandler;

/// DIDComm `type` URI for Trust Tasks envelopes.
///
/// Conforming consumers reject DIDComm messages with any other `type`
/// via [`DidcommError::WrongEnvelopeType`].
pub const ENVELOPE_TYPE: &str = "https://trusttasks.org/binding/didcomm/0.1/envelope";

/// Wrap a Trust Task document in a DIDComm v2.1 envelope and authcrypt
/// it for `recipient_did`.
///
/// `agent` must have `sender_did` registered as a local
/// [`PrivateIdentity`](affinidi_messaging_didcomm::identity::PrivateIdentity)
/// and `recipient_did` registered as a remote
/// [`ResolvedIdentity`](affinidi_messaging_didcomm::identity::ResolvedIdentity)
/// (via `agent.add_identity` / `agent.add_peer` respectively).
///
/// Returns the JWE-encoded string ready for transport.
pub fn pack_trust_task<P>(
    doc: &TrustTask<P>,
    agent: &DIDCommAgent,
    sender_did: &str,
    recipient_did: &str,
) -> Result<String, DidcommError>
where
    P: Payload + Serialize,
{
    let body = serde_json::to_value(doc).map_err(DidcommError::SerialiseBody)?;
    // Binding §3.1 — populate the DIDComm thread headers *from* the framework
    // members, never the reverse. `thid` previously took `doc.id`, which starts
    // a fresh DIDComm thread for every document: correct only for a document
    // that opens an exchange, and wrong for every response, which carries the
    // originating `threadId`. §4.9's own fallback is the same one used here, so
    // the DIDComm thread and the Trust Task exchange end up named by one value.
    let thid = doc.thread_id.clone().unwrap_or_else(|| doc.id.clone());
    let mut msg = Message::new(ENVELOPE_TYPE, body)
        .from(sender_did.to_string())
        .to(vec![recipient_did.to_string()])
        .thid(thid);
    if let Some(parent) = doc.parent_thread_id.clone() {
        msg = msg.pthid(parent);
    }
    let wire = agent.pack_authcrypt(&msg, sender_did, recipient_did)?;
    Ok(wire)
}

/// Unwrap a DIDComm v2.1 envelope produced by [`pack_trust_task`] into
/// a typed [`TrustTask<P>`] plus a [`DidcommHandler`] populated with
/// the verified peer DID.
///
/// `expected_sender_did` is the DID the consumer expects the envelope
/// to come from. The current `affinidi-messaging-didcomm` (v0.15)
/// `DIDCommAgent::unpack` requires this to look up the sender's public
/// key in its store; pass the DID of the peer you previously called
/// `agent.add_peer(...)` for. A server receiving from many peers should
/// **not** loop over its known senders retrying this call — use
/// [`unpack_trust_task_from`], which reads the envelope's own `skid`
/// and looks the sender up once against an explicit
/// [`SenderAllowlist`].
///
/// Where `expected_sender_did` is supplied it is enforced, not merely
/// used as a lookup key: the envelope's `skid`-derived sender must equal
/// it, or [`DidcommError::SenderKidMismatch`] is returned. The `skid` is
/// sender-chosen, so without that check a peer could authenticate with
/// its own key while labelling the envelope with another party's DID and
/// have the label handed to §4.8.1 as the authenticated sender.
///
/// **Conformance:** binding §2 makes authcrypt a **MUST** and §4 keeps
/// every other envelope shape out of the framework pipeline. This
/// function therefore rejects:
///
/// * anoncrypt'd and plaintext envelopes —
///   [`DidcommError::UnauthenticatedSender`];
/// * signed-only (bare JWS) envelopes —
///   [`DidcommError::SignedNotAuthcrypted`]. A JWS has no recipient
///   binding, so one message can be delivered to every party and each
///   accepts it, and it leaves no transport-authenticated *recipient* to
///   cross-check the in-band `recipient` against;
/// * an authenticated `sender_kid` with no `#fragment` —
///   [`DidcommError::UnqualifiedSenderKid`].
///
/// The returned [`DidcommHandler`] is ready to feed into
/// [`TransportHandler::resolve_parties`](trust_tasks_rs::TransportHandler::resolve_parties),
/// [`TrustTask::validate_basic`], and the rest of the §7.2 pipeline.
pub fn unpack_trust_task<P>(
    wire: &str,
    agent: &DIDCommAgent,
    expected_sender_did: Option<&str>,
) -> Result<(TrustTask<P>, DidcommHandler), DidcommError>
where
    P: Payload + DeserializeOwned,
{
    let (message, peer_did, local_did) = match agent.unpack(wire, expected_sender_did)? {
        UnpackResult::Encrypted {
            message,
            authenticated: true,
            sender_kid: Some(sender_kid),
            recipient_kid,
            // didcomm 0.14 adds `legacy_kek_used` (pre-0.14 ECDH-1PU KEK
            // migration signal), `non_repudiation`, and inner-JWS
            // `signer_kid`. The §4.8.1 transport-authenticated sender is
            // the authcrypt `sender_kid`; surfacing the inner signer or
            // gating on the legacy KEK would be a behaviour change beyond
            // this binding's current contract, so they're ignored here.
            ..
        } => {
            // A fragment-less kid is an error, never a `None`. Returning
            // `None` here used to downgrade an authenticated identity into
            // an unauthenticated one: the pipeline then fell back to the
            // in-band `issuer` with the transport cross-check skipped.
            let peer = sender_did_from_kid(&sender_kid)?;
            // The `skid`/`apu` is sender-chosen; the DID that actually
            // authenticated is the one whose public key opened the
            // ECDH-1PU wrap, which is `expected_sender_did`. Where the
            // caller named one, the two must agree.
            if let Some(expected) = expected_sender_did {
                if expected != peer {
                    return Err(DidcommError::SenderKidMismatch {
                        expected: expected.to_string(),
                        advertised: peer,
                    });
                }
            }
            (
                message,
                Some(peer),
                // A fragment-less *recipient* kid is left as-is rather than
                // rejected: unlike the sender case this fails closed. The
                // raw kid becomes the transport-authenticated recipient and
                // a document whose in-band `recipient` disagrees is caught
                // by §4.8.1 as an `identityMismatch`.
                Some(did_from_kid(&recipient_kid).unwrap_or(recipient_kid)),
            )
        }
        UnpackResult::Encrypted { .. } | UnpackResult::Plaintext(_) => {
            return Err(DidcommError::UnauthenticatedSender);
        }
        // Binding §2: authcrypt is a MUST, and §4 keeps plaintext out of
        // the pipeline. A bare JWS is signed but not sealed to anyone, so
        // it has no recipient binding and no confidentiality; accepting it
        // (as this binding did through 0.10) meant one signed message could
        // be delivered to every recipient in a deployment and each would
        // take it, with the §4.8.1 recipient cross-check disabled because
        // there was no local DID to check against. Fail closed.
        UnpackResult::Signed { .. } => {
            return Err(DidcommError::SignedNotAuthcrypted);
        }
        // `UnpackResult` is `#[non_exhaustive]` as of didcomm 0.14. Any
        // future variant won't carry the transport-authenticated sender
        // the §4.8.1 pipeline relies on, so fail closed.
        _ => return Err(DidcommError::UnauthenticatedSender),
    };

    if message.typ != ENVELOPE_TYPE {
        return Err(DidcommError::WrongEnvelopeType(message.typ.clone()));
    }

    let doc: TrustTask<P> =
        serde_json::from_value(message.body).map_err(DidcommError::InvalidBody)?;

    // Binding §3.1 — where both a DIDComm thread header and its framework
    // member are explicitly present they MUST agree. Scoped to both-present on
    // purpose: DIDComm's `thid` defaults to the DIDComm message `id` and the
    // framework's `threadId` falls back to the document's `id`, and those are
    // different identifier spaces (§2), so an unconditional comparison would
    // reject exchanges that conform on both layers.
    check_thread(
        "thid",
        "threadId",
        message.thid.as_deref(),
        doc.thread_id.as_deref(),
    )?;
    check_thread(
        "pthid",
        "parentThreadId",
        message.pthid.as_deref(),
        doc.parent_thread_id.as_deref(),
    )?;

    let handler = DidcommHandler::new(local_did, peer_did);
    Ok((doc, handler))
}

/// Compare a DIDComm thread header against its framework member, per binding
/// §3.1. Silent unless both are present and differ.
fn check_thread(
    header: &'static str,
    member: &'static str,
    transport: Option<&str>,
    in_band: Option<&str>,
) -> Result<(), DidcommError> {
    match (transport, in_band) {
        (Some(t), Some(b)) if t != b => Err(DidcommError::ThreadMismatch {
            header,
            member,
            transport: t.to_string(),
            in_band: b.to_string(),
        }),
        _ => Ok(()),
    }
}

/// `did:peer:2.Ez6...#key-agreement-1` → `did:peer:2.Ez6...`.
/// A DIDComm `kid` is always a fully-qualified DID URL whose fragment
/// names a verification method in the DID document. The framework only
/// cares about the DID portion.
fn did_from_kid(kid: &str) -> Option<String> {
    kid.split_once('#').map(|(did, _)| did.to_string())
}

/// The same reduction for a `kid` that is supposed to name the
/// *transport-authenticated sender*, where the absence of a fragment is a
/// hard error rather than an absent identity.
///
/// Downgrading it to `None` — which is what this binding did through
/// 0.10 — reclassified an authenticated party as an unauthenticated one
/// and skipped the SPEC.md §4.8.1 cross-check entirely, so a document
/// arriving with a malformed `kid` had its in-band `issuer` believed
/// without challenge.
fn sender_did_from_kid(kid: &str) -> Result<String, DidcommError> {
    did_from_kid(kid).ok_or_else(|| DidcommError::UnqualifiedSenderKid {
        kid: kid.to_string(),
    })
}

/// The set of sender DIDs a consumer will accept an inbound Trust Task
/// envelope from.
///
/// Before 0.11 this crate's callers were told to loop over their known
/// peers, retrying [`unpack_trust_task`] on
/// `DIDCommError::IdentityNotFound`. That loop was O(known peers)
/// ECDH-1PU decrypts per inbound message, and — more to the point — it
/// *was* the sender allowlist: a peer the agent had never been given
/// could not be unpacked, so it could not be accepted. That property was
/// real but incidental, invisible in the type signature and easy to lose
/// to a refactor that made unpacking cheaper.
///
/// [`unpack_trust_task_from`] makes it explicit: the envelope's own
/// `skid` is read from the JWE protected header, the DID it names is
/// looked up **once**, and a sender outside the list is rejected with
/// [`DidcommError::SenderNotAllowed`] before any decryption is
/// attempted.
///
/// An empty allowlist permits nothing.
///
/// The `skid` is sender-chosen and unauthenticated at the point it is
/// read — it selects which key to unpack against, it does not prove
/// anything. Authentication still comes from the ECDH-1PU wrap opening,
/// and [`unpack_trust_task`] additionally re-checks the verified sender
/// against the DID that was looked up.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SenderAllowlist {
    allowed: BTreeSet<String>,
}

impl SenderAllowlist {
    /// An allowlist of exactly these sender DIDs.
    pub fn new<I, S>(dids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            allowed: dids.into_iter().map(Into::into).collect(),
        }
    }

    /// Every peer the agent has been given a resolved identity for.
    ///
    /// This reproduces exactly the set the old retry loop could accept —
    /// use it when migrating off that loop and tighten later.
    pub fn from_agent_peers(agent: &DIDCommAgent) -> Self {
        Self::new(agent.store().resolved_dids())
    }

    /// Add one permitted sender DID.
    pub fn allow(mut self, did: impl Into<String>) -> Self {
        self.allowed.insert(did.into());
        self
    }

    /// Whether `did` may send this consumer a Trust Task.
    pub fn permits(&self, did: &str) -> bool {
        self.allowed.contains(did)
    }

    /// The permitted sender DIDs.
    pub fn allowed(&self) -> impl Iterator<Item = &str> {
        self.allowed.iter().map(String::as_str)
    }

    /// Whether the list is empty — in which case it permits nothing.
    pub fn is_empty(&self) -> bool {
        self.allowed.is_empty()
    }
}

/// The sender DID an authcrypt envelope *names*, read from the JWE
/// protected header's `skid` without decrypting anything.
///
/// **This is a routing hint, not an authenticated identity.** The `skid`
/// is written by whoever produced the envelope. Its only safe use is to
/// choose which known sender to unpack against — which is precisely what
/// [`unpack_trust_task_from`] uses it for, re-checking the value against
/// the key that actually opened the envelope afterwards.
///
/// Returns [`DidcommError::NotAuthcryptJwe`] for bytes that are not a JWE
/// or carry no `skid` (an anoncrypt envelope carries none), and
/// [`DidcommError::UnqualifiedSenderKid`] where the `skid` has no
/// `#fragment`.
pub fn advertised_sender_did(wire: &str) -> Result<String, DidcommError> {
    let envelope: serde_json::Value = serde_json::from_str(wire)
        .map_err(|e| DidcommError::NotAuthcryptJwe(format!("not JSON: {e}")))?;
    let protected = envelope
        .get("protected")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| DidcommError::NotAuthcryptJwe("no `protected` header".into()))?;
    let decoded = URL_SAFE_NO_PAD
        .decode(protected)
        .map_err(|e| DidcommError::NotAuthcryptJwe(format!("`protected` is not base64url: {e}")))?;
    let header: serde_json::Value = serde_json::from_slice(&decoded)
        .map_err(|e| DidcommError::NotAuthcryptJwe(format!("`protected` is not JSON: {e}")))?;
    let skid = header
        .get("skid")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| DidcommError::NotAuthcryptJwe("no `skid` (not authcrypt)".into()))?;
    sender_did_from_kid(skid)
}

/// Unwrap an inbound envelope from any sender on an explicit
/// [`SenderAllowlist`], with **one** decrypt attempt.
///
/// The envelope's `skid` names the sender it claims to be from; that DID
/// is checked against `allowlist` before anything is decrypted, and the
/// unpack then runs once against that one sender. A sender outside the
/// list is rejected with [`DidcommError::SenderNotAllowed`] having cost
/// no cryptography at all.
///
/// This replaces the "iterate over known senders, retry on
/// `IdentityNotFound`" pattern the crate previously documented, which was
/// O(known peers) ECDH-1PU decrypts per inbound message and expressed the
/// allowlist only as a side effect of which peers the agent happened to
/// hold. To keep the old behaviour exactly while dropping the cost, pass
/// [`SenderAllowlist::from_agent_peers`].
///
/// Every conformance rule of [`unpack_trust_task`] still applies —
/// including the re-check that the verified sender matches the `skid`
/// that selected it.
pub fn unpack_trust_task_from<P>(
    wire: &str,
    agent: &DIDCommAgent,
    allowlist: &SenderAllowlist,
) -> Result<(TrustTask<P>, DidcommHandler), DidcommError>
where
    P: Payload + DeserializeOwned,
{
    let advertised = advertised_sender_did(wire)?;
    if !allowlist.permits(&advertised) {
        return Err(DidcommError::SenderNotAllowed { did: advertised });
    }
    unpack_trust_task(wire, agent, Some(&advertised))
}

#[cfg(test)]
mod thread_tests {
    use super::*;

    /// Binding §3.1 — the comparison engages only when both sides are present.
    /// DIDComm's `thid` defaults to the DIDComm message id and the framework's
    /// `threadId` falls back to the document's id, and those are different
    /// identifier spaces (§2), so comparing unconditionally would reject
    /// exchanges that conform on both layers.
    #[test]
    fn absent_on_either_side_is_not_a_mismatch() {
        assert!(check_thread("thid", "threadId", None, Some("a")).is_ok());
        assert!(check_thread("thid", "threadId", Some("a"), None).is_ok());
        assert!(check_thread("thid", "threadId", None, None).is_ok());
    }

    #[test]
    fn equal_values_pass() {
        assert!(check_thread("thid", "threadId", Some("a"), Some("a")).is_ok());
    }

    /// A disagreement is `malformedRequest`, never `identityMismatch`: no
    /// party's identity is contested, so §8.1's suppression rules must not be
    /// reached.
    #[test]
    fn disagreement_is_a_malformed_request() {
        let err = check_thread("pthid", "parentThreadId", Some("outer"), Some("other"))
            .expect_err("both present and different must fail");
        assert!(matches!(
            err,
            DidcommError::ThreadMismatch {
                header: "pthid",
                member: "parentThreadId",
                ..
            }
        ));
        match err.into_reject_reason() {
            trust_tasks_rs::RejectReason::MalformedRequest { .. } => {}
            other => panic!("expected malformedRequest, got {other:?}"),
        }
    }
}
