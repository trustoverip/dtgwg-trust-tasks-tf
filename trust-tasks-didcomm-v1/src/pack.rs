//! Placing a Trust Task document into a DIDComm v1 message, and getting it back.
//!
//! # Where the document goes
//!
//! DIDComm v2.1 has an obvious slot: the message `body`. **v1 has none** — the
//! payload is flattened at top level beside `@id`, `@type` and the decorators —
//! and RFC 0095 types `basic-message`'s `content` as a *string* meant for human
//! display, which Credo renders as chat text. A binding therefore has to choose,
//! and choose normatively, or two implementations will pick differently.
//!
//! This binding carries the document in an **`~attach` decorator** (Aries RFC
//! 0017), inline as `data.json`, under the reserved attachment id
//! [`ATTACHMENT_ID`]:
//!
//! ```json
//! {
//!   "@id": "…",
//!   "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/basicmessage/1.0/message",
//!   "~thread": { "thid": "…", "pthid": "…" },
//!   "content": "Trust Task: https://trusttasks.org/spec/acl/grant/0.1",
//!   "~attach": [{
//!     "@id": "trust-task",
//!     "mime-type": "application/json",
//!     "data": { "json": { "id": "…", "type": "…", "payload": {} } }
//!   }]
//! }
//! ```
//!
//! The two alternatives, and why not:
//!
//! - **JSON-in-`content`.** Reaches every v1 agent, but a human-facing wallet
//!   renders each Trust Task as a wall of double-encoded JSON. That is the
//!   primary experience for an Aries wallet user, not an edge case.
//! - **A sibling top-level member.** Keeps `content` readable and nothing
//!   rejects it, but it invents a slot no Aries reader looks in.
//!
//! `~attach` is the idiomatic Aries home for a structured payload, keeps the
//! document as JSON rather than a string inside a string, and leaves `content`
//! free for a human-readable summary — so a wallet that surfaces the message
//! shows something meaningful rather than a blob.
//!
//! This choice is not settled by consensus yet; see the discussion on issue
//! #173. Nothing depends on it, so it can still move.

use affinidi_messaging_didcomm_v1::protocols::basic_message::{self, BasicMessage};
use affinidi_messaging_didcomm_v1::{MessageV1, UnpackResult};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::{Payload, TrustTask};

use crate::error::DidcommV1Error;
use crate::handler::DidcommV1Handler;

/// The `~attach` entry id this binding reserves for the Trust Task document.
///
/// Fixed rather than generated so a consumer can find the document without
/// scanning every attachment, and so a message may carry unrelated attachments
/// alongside it.
pub const ATTACHMENT_ID: &str = "trust-task";

/// Build the human-readable `content` for a message carrying `type_uri`.
///
/// RFC 0095 says `content` is for display, and a wallet will show it. Naming the
/// Type URI is more use to someone reading a chat log than an empty string, and
/// it does not duplicate anything a consumer parses — the document in the
/// attachment is authoritative.
fn summary(type_uri: &str) -> String {
    format!("Trust Task: {type_uri}")
}

/// Place a Trust Task document into a DIDComm v1 `basic-message`.
///
/// Returns the unpacked [`MessageV1`], ready to be authcrypt-packed by the
/// caller's agent. Packing is left to the caller because a v1 agent needs the
/// sender's private key and the recipient's verkey, both of which live in
/// connection state this crate does not model.
///
/// The `~thread` decorator is populated from the document's framework members,
/// never the reverse — see [`crate::THREAD_MAPPING`].
pub fn build_message<P>(doc: &TrustTask<P>) -> Result<MessageV1, DidcommV1Error>
where
    P: Payload + Serialize,
{
    let document = serde_json::to_value(doc).map_err(DidcommV1Error::SerialiseDocument)?;
    let type_uri = doc.type_uri.to_string();

    // §4.9's own fallback: a document with no threadId is named by its id, so
    // the DIDComm thread and the Trust Task exchange end up sharing one value.
    let thid = doc.thread_id.clone().unwrap_or_else(|| doc.id.clone());

    let mut builder = BasicMessage::new(summary(&type_uri))?
        .field(
            "~attach",
            json!([{
                "@id": ATTACHMENT_ID,
                "mime-type": "application/json",
                "data": { "json": document }
            }]),
        )
        .thid(thid);
    if let Some(parent) = doc.parent_thread_id.clone() {
        builder = builder.pthid(parent);
    }
    Ok(builder.finalize())
}

/// Extract a Trust Task document from an unpacked DIDComm v1 message.
///
/// Applies, in order: the authenticated-sender gate, the message-type check, the
/// attachment lookup, and the `~thread` cross-check. Returns the typed document
/// alongside a [`DidcommV1Handler`] carrying the connection's `theirDid`, ready
/// for the framework's §7.2 pipeline.
pub fn unpack_trust_task<P>(
    unpacked: UnpackResult,
) -> Result<(TrustTask<P>, DidcommV1Handler), DidcommV1Error>
where
    P: Payload + DeserializeOwned,
{
    // A v1 envelope authenticates a verkey, not a DID, so there are two ways to
    // arrive without an attributable sender and they are worth telling apart.
    let authenticated = match unpacked {
        UnpackResult::Authcrypt { .. } => unpacked.require_authenticated()?,
        UnpackResult::AuthcryptUnknownSender { sender_verkey, .. } => {
            return Err(DidcommV1Error::UnknownSenderBinding {
                verkey: sender_verkey.to_string(),
            });
        }
        _ => return Err(DidcommV1Error::UnauthenticatedSender),
    };

    let msg = &authenticated.message;
    // `is_basic_message` rather than comparing `typ` — v1 message types have two
    // interchangeable document URIs and Credo emits the `https://didcomm.org`
    // one by default, so exact comparison silently drops conforming peers.
    if !basic_message::is_basic_message(msg) {
        return Err(DidcommV1Error::WrongMessageType(msg.typ.clone()));
    }

    let document = attachment(msg)?;
    let doc: TrustTask<P> =
        serde_json::from_value(document).map_err(DidcommV1Error::InvalidDocument)?;

    // Binding thread mapping — compare only where both sides are explicitly
    // present. v1's `thid` defaults to the message `@id` and the framework's
    // `threadId` falls back to the document's `id`; those are different
    // identifiers, so an unconditional comparison would reject conforming
    // exchanges.
    // `explicit_thid` rather than the effective one: a defaulted `thid` is a
    // value the transport synthesised from `@id`, not one the sender asserted,
    // and comparing against it would manufacture disagreements.
    check_thread(
        "thid",
        "threadId",
        msg.explicit_thid(),
        doc.thread_id.as_deref(),
    )?;
    check_thread(
        "pthid",
        "parentThreadId",
        msg.pthid_value(),
        doc.parent_thread_id.as_deref(),
    )?;

    let handler = DidcommV1Handler::new(
        Some(authenticated.recipient.to_string()),
        Some(authenticated.sender.to_string()),
    );
    Ok((doc, handler))
}

/// Pull the reserved attachment's inline JSON out of a message.
fn attachment(msg: &MessageV1) -> Result<Value, DidcommV1Error> {
    let missing = || DidcommV1Error::MissingAttachment {
        expected: ATTACHMENT_ID,
    };
    msg.body
        .get("~attach")
        .and_then(Value::as_array)
        .ok_or_else(missing)?
        .iter()
        .find(|a| a.get("@id").and_then(Value::as_str) == Some(ATTACHMENT_ID))
        .and_then(|a| a.get("data")?.get("json"))
        .cloned()
        .ok_or_else(missing)
}

/// Compare a `~thread` field against its framework member. Silent unless both
/// are present and differ.
fn check_thread(
    header: &'static str,
    member: &'static str,
    transport: Option<&str>,
    in_band: Option<&str>,
) -> Result<(), DidcommV1Error> {
    match (transport, in_band) {
        (Some(t), Some(b)) if t != b => Err(DidcommV1Error::ThreadMismatch {
            header,
            member,
            transport: t.to_string(),
            in_band: b.to_string(),
        }),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trust_tasks_rs::specs::acl::grant::v0_1 as grant;

    fn entry() -> grant::AclEntry {
        grant::AclEntry {
            allowed_keys: None,
            subject: "did:sov:alice".into(),
            role: "admin".into(),
            scopes: vec![],
            label: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            expires_at: None,
            approve: None,
            step_up: None,
            ext: None,
        }
    }

    fn doc() -> TrustTask<grant::Payload> {
        TrustTask::for_payload(
            "req-1",
            grant::Payload {
                entry: entry(),
                reason: None,
                ext: None,
            },
        )
    }

    #[test]
    fn document_rides_in_the_reserved_attachment() {
        let msg = build_message(&doc()).unwrap();
        let attach = msg.body.get("~attach").unwrap().as_array().unwrap();
        assert_eq!(attach.len(), 1);
        assert_eq!(attach[0]["@id"], ATTACHMENT_ID);
        assert_eq!(attach[0]["mime-type"], "application/json");
        // Structured JSON, not a string inside a string — the whole reason for
        // choosing ~attach over JSON-in-content.
        assert!(attach[0]["data"]["json"].is_object());
        assert_eq!(attach[0]["data"]["json"]["id"], "req-1");
    }

    #[test]
    fn content_carries_a_human_readable_summary() {
        // RFC 0095 says `content` is for display and a wallet will show it.
        // A blob of JSON here is what we are avoiding.
        let msg = build_message(&doc()).unwrap();
        let content = basic_message::content(&msg).unwrap();
        assert!(content.contains("acl/grant"), "content: {content}");
        assert!(
            !content.contains('{'),
            "content must not be JSON: {content}"
        );
    }

    #[test]
    fn thread_is_populated_from_the_framework_members() {
        let mut d = doc();
        d.thread_id = Some("exchange-1".into());
        d.parent_thread_id = Some("outer-1".into());
        let msg = build_message(&d).unwrap();
        assert_eq!(msg.explicit_thid(), Some("exchange-1"));
        assert_eq!(msg.pthid_value(), Some("outer-1"));
    }

    /// §4.9's fallback: a document with no `threadId` is named by its `id`, so
    /// the DIDComm thread and the Trust Task exchange share one value rather
    /// than the transport inventing its own.
    #[test]
    fn thread_falls_back_to_the_document_id() {
        let msg = build_message(&doc()).unwrap();
        assert_eq!(msg.explicit_thid(), Some("req-1"));
        assert_eq!(msg.pthid_value(), None);
    }

    #[test]
    fn absent_on_either_side_is_not_a_mismatch() {
        assert!(check_thread("thid", "threadId", None, Some("a")).is_ok());
        assert!(check_thread("thid", "threadId", Some("a"), None).is_ok());
        assert!(check_thread("thid", "threadId", None, None).is_ok());
    }

    /// A disagreement is `malformedRequest`, never `identityMismatch` — no
    /// party's identity is contested, so §8.1's suppression rules must not be
    /// reached.
    #[test]
    fn thread_disagreement_is_a_malformed_request() {
        let err = check_thread("pthid", "parentThreadId", Some("outer"), Some("other"))
            .expect_err("both present and different must fail");
        match err.into_reject_reason() {
            trust_tasks_rs::RejectReason::MalformedRequest { .. } => {}
            other => panic!("expected malformedRequest, got {other:?}"),
        }
    }

    /// An envelope authenticated by an unbound verkey is cryptographically
    /// sound but attributable to nobody. It must not collapse into
    /// `UnauthenticatedSender`: one means nobody signed it, the other means
    /// somebody did and we cannot name them, and the operator response differs
    /// (a missing connection record versus a hostile message).
    #[test]
    fn unknown_sender_binding_is_distinct_from_unauthenticated() {
        let unknown = DidcommV1Error::UnknownSenderBinding {
            verkey: "8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K".into(),
        };
        let unauth = DidcommV1Error::UnauthenticatedSender;
        assert_ne!(unknown.to_string(), unauth.to_string());
        // Both deny the framework a party to attribute to, so both ask for an
        // in-band proof instead.
        for e in [unknown, unauth] {
            assert!(matches!(
                e.into_reject_reason(),
                trust_tasks_rs::RejectReason::ProofRequired
            ));
        }
    }

    #[test]
    fn a_message_without_the_attachment_is_rejected() {
        let msg = BasicMessage::new("just chat").unwrap().finalize();
        let err = attachment(&msg).unwrap_err();
        assert!(matches!(
            err,
            DidcommV1Error::MissingAttachment {
                expected: ATTACHMENT_ID
            }
        ));
    }

    /// The document survives the round trip through the attachment unchanged.
    #[test]
    fn round_trips_through_the_attachment() {
        let mut original = doc();
        original.thread_id = Some("exchange-1".into());
        original.issuer = Some("did:sov:bob".into());

        let msg = build_message(&original).unwrap();
        let recovered: TrustTask<grant::Payload> =
            serde_json::from_value(attachment(&msg).unwrap()).unwrap();

        assert_eq!(recovered.id, original.id);
        assert_eq!(recovered.thread_id, original.thread_id);
        assert_eq!(recovered.issuer, original.issuer);
        assert_eq!(recovered.payload.entry.subject, "did:sov:alice");
    }
}
