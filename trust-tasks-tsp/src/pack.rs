//! `pack_trust_task` / `unpack_trust_task` — convert a typed [`TrustTask`] into
//! and out of a TSP-sealed envelope.
//!
//! The TSP message payload is the JSON envelope object `{ "type":
//! ENVELOPE_TYPE, "document": <TrustTask<P>> }`; TSP seals it with HPKE
//! authenticated encryption and signs it from the sender's VID. On unwrap the
//! authenticated `VID_sndr` is surfaced verbatim as the binding's
//! transport-authenticated peer (a TSP VID is a framework VID — no
//! transformation).

use affinidi_tsp::message::direct;
use affinidi_tsp::{MessageType, PrivateVid, ResolvedVid};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use trust_tasks_rs::{Payload, TrustTask};

use crate::error::TspError;
use crate::handler::TspHandler;

/// Envelope `type` URI for Trust Tasks carried over TSP.
///
/// Conforming consumers reject payloads with any other `type` via
/// [`TspError::WrongEnvelopeType`].
pub const ENVELOPE_TYPE: &str = "https://trusttasks.org/binding/tsp/0.1/envelope";

/// Wrap a Trust Task document in the binding envelope and TSP-seal it
/// (`Direct`) from `sender` to `recipient`.
///
/// `sender` is the producer's own [`PrivateVid`] (its signing + decryption
/// keys); `recipient` is the consumer's [`ResolvedVid`] (its public encryption
/// key), as obtained from a TSP VID resolver. Returns the raw TSP message bytes
/// (CESR qb2) ready for transport.
pub fn pack_trust_task<P>(
    doc: &TrustTask<P>,
    sender: &PrivateVid,
    recipient: &ResolvedVid,
) -> Result<Vec<u8>, TspError>
where
    P: Payload + Serialize,
{
    let document = serde_json::to_value(doc).map_err(TspError::SerialiseBody)?;
    let envelope = json!({ "type": ENVELOPE_TYPE, "document": document });
    let payload = serde_json::to_vec(&envelope).map_err(TspError::SerialiseBody)?;

    let packed = direct::pack(
        &payload,
        MessageType::Direct,
        &sender.id,
        &recipient.id,
        &sender.signing_key,
        &sender.decryption_key,
        &recipient.encryption_key,
    )?;
    Ok(packed.bytes)
}

/// Unwrap a TSP message produced by [`pack_trust_task`] into a typed
/// [`TrustTask<P>`] plus a [`TspHandler`] populated with the authenticated peer
/// VID.
///
/// `recipient` is the consumer's own [`PrivateVid`]; `sender` is the
/// [`ResolvedVid`] of the VID the message is expected from (its public keys
/// verify the signature and authenticate the HPKE seal). A server receiving from
/// unknown senders reads the cleartext `VID_sndr` first (via
/// [`affinidi_tsp::MetaEnvelope::parse`]), resolves it, then calls this.
///
/// Rejects non-`Direct` carriage and any envelope whose cleartext sender does
/// not match the verified `sender`, and verifies the envelope `type`.
pub fn unpack_trust_task<P>(
    wire: &[u8],
    recipient: &PrivateVid,
    sender: &ResolvedVid,
) -> Result<(TrustTask<P>, TspHandler), TspError>
where
    P: Payload + DeserializeOwned,
{
    let unpacked = direct::unpack(
        wire,
        &recipient.decryption_key,
        &sender.encryption_key,
        &sender.signing_key,
    )?;

    if unpacked.message_type != MessageType::Direct {
        return Err(TspError::UnsupportedCarriage(unpacked.message_type));
    }

    // The signature verified against `sender`'s key; the cleartext envelope must
    // also name that VID, or it is claiming an identity it cannot prove.
    if unpacked.sender != sender.id {
        return Err(TspError::SenderMismatch {
            claimed: unpacked.sender,
            verified: sender.id.clone(),
        });
    }

    let envelope: Envelope =
        serde_json::from_slice(&unpacked.payload).map_err(TspError::InvalidBody)?;
    if envelope.type_ != ENVELOPE_TYPE {
        return Err(TspError::WrongEnvelopeType(envelope.type_));
    }

    let doc: TrustTask<P> =
        serde_json::from_value(envelope.document).map_err(TspError::InvalidBody)?;

    // A TSP VID is a framework VID verbatim — no normalisation.
    let handler = TspHandler::new(Some(unpacked.receiver), Some(sender.id.clone()));
    Ok((doc, handler))
}

/// The binding envelope object: a `type` tag plus the carried Trust Task.
#[derive(serde::Deserialize)]
struct Envelope {
    #[serde(rename = "type")]
    type_: String,
    document: serde_json::Value,
}
