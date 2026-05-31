//! `pack_trust_task` / `unpack_trust_task` — convert a typed
//! [`TrustTask`] into and out of a DIDComm v2.1 envelope.
//!
//! The envelope's `type` is the framework-reserved
//! [`ENVELOPE_TYPE`] URI; the `body` carries the full `TrustTask<P>`
//! JSON. The outer envelope's authcrypt'd verified `sender_kid` (a DID
//! URL like `did:peer:2.Ez6...#key-agreement-1`) is stripped of its
//! fragment and surfaced as the binding's transport-authenticated peer.

use affinidi_messaging_didcomm::{DIDCommAgent, Message, UnpackResult};
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
    let msg = Message::new(ENVELOPE_TYPE, body)
        .from(sender_did.to_string())
        .to(vec![recipient_did.to_string()])
        .thid(doc.id.clone());
    let wire = agent.pack_authcrypt(&msg, sender_did, recipient_did)?;
    Ok(wire)
}

/// Unwrap a DIDComm v2.1 envelope produced by [`pack_trust_task`] into
/// a typed [`TrustTask<P>`] plus a [`DidcommHandler`] populated with
/// the verified peer DID.
///
/// `expected_sender_did` is the DID the consumer expects the envelope
/// to come from. The current `affinidi-messaging-didcomm` (v0.14)
/// `DIDCommAgent::unpack` requires this to look up the sender's public
/// key in its store; pass the DID of the peer you previously called
/// `agent.add_peer(...)` for. Servers receiving from multiple peers
/// iterate over their known senders (per-peer retry on
/// `DIDCommError::IdentityNotFound`).
///
/// **Conformance:** rejects anoncrypt'd and plaintext envelopes — both
/// lack the transport-authenticated sender SPEC.md §4.8.1 needs to
/// cross-check the in-band `issuer`. Returns
/// [`DidcommError::UnauthenticatedSender`] in those cases.
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
        } => (
            message,
            did_from_kid(&sender_kid),
            Some(did_from_kid(&recipient_kid).unwrap_or(recipient_kid)),
        ),
        UnpackResult::Encrypted { .. } | UnpackResult::Plaintext(_) => {
            return Err(DidcommError::UnauthenticatedSender);
        }
        UnpackResult::Signed {
            message,
            signer_kid: Some(signer_kid),
        } => (message, did_from_kid(&signer_kid), None),
        UnpackResult::Signed { .. } => {
            return Err(DidcommError::UnauthenticatedSender);
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
    let handler = DidcommHandler::new(local_did, peer_did);
    Ok((doc, handler))
}

/// `did:peer:2.Ez6...#key-agreement-1` → `did:peer:2.Ez6...`.
/// A DIDComm `kid` is always a fully-qualified DID URL whose fragment
/// names a verification method in the DID document. The framework only
/// cares about the DID portion.
fn did_from_kid(kid: &str) -> Option<String> {
    kid.split_once('#').map(|(did, _)| did.to_string())
}
