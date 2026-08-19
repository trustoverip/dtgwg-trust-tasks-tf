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
//! [`ATTACHMENT_ID`], on a message of the binding's **own type**
//! ([`ENVELOPE_TYPE`]) — `bindings/didcomm-v1/0.2` §2, which settled the
//! carriage question 0.1 left open, on measurement (see its §2.1):
//!
//! ```json
//! {
//!   "@id": "…",
//!   "@type": "https://trusttasks.org/binding/didcomm-v1/0.2/trust-task/1.0/task",
//!   "~thread": { "thid": "…", "pthid": "…" },
//!   "comment": "Trust Task: https://trusttasks.org/spec/acl/grant/0.1",
//!   "~attach": [{
//!     "@id": "trust-task",
//!     "mime-type": "application/json",
//!     "data": { "json": { "id": "…", "type": "…", "payload": {} } }
//!   }]
//! }
//! ```
//!
//! Per §2.3 receivers move first: a consumer built from this crate **accepts
//! both** carriages — the dedicated type above and 0.1's `basic-message` —
//! while the producer emits only the dedicated type.
//!
//! # The omit rule
//!
//! `~thread` values must satisfy RFC 0008's shape (`[-_./a-zA-Z0-9]{8,64}`) —
//! Credo enforces it on receipt. Where a document's `threadId`,
//! `parentThreadId`, or fallback `id` cannot be represented, the producer
//! **omits the field rather than rewriting the value** (binding §4): a
//! rewritten correlator would contradict the in-band member and draw a
//! guaranteed `malformedRequest` from the consumer's §3.1 comparison. An
//! omitted `thid` defaults to the message `@id` on the wire; a consumer
//! **must not** infer exchange continuation from that defaulted value, which
//! is why the comparison below reads `explicit_thid()`.

use affinidi_messaging_didcomm_v1::protocols::basic_message;
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

/// The binding's message `@type` (`bindings/didcomm-v1/0.2` §2): an RFC 0020
/// message type URI, protocol `trust-task` 1.0, message `task`.
pub const ENVELOPE_TYPE: &str = "https://trusttasks.org/binding/didcomm-v1/0.2/trust-task/1.0/task";

/// Whether a correlator value satisfies RFC 0008's `thid` shape
/// (`[-_./a-zA-Z0-9]{8,64}`). Values that do not — `urn:uuid:` ids most
/// prominently — are **omitted** from `~thread`, never rewritten (binding §4).
fn representable(value: &str) -> bool {
    (8..=64).contains(&value.len())
        && value.bytes().all(
            |b| matches!(b, b'-' | b'_' | b'.' | b'/' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z'),
        )
}

/// Build the human-readable `content` for a message carrying `type_uri`.
///
/// RFC 0095 says `content` is for display, and a wallet will show it. Naming the
/// Type URI is more use to someone reading a chat log than an empty string, and
/// it does not duplicate anything a consumer parses — the document in the
/// attachment is authoritative.
fn summary(type_uri: &str) -> String {
    format!("Trust Task: {type_uri}")
}

/// Place a Trust Task document into a DIDComm v1 message of the binding's
/// dedicated type ([`ENVELOPE_TYPE`], binding 0.2 §2).
///
/// Returns the unpacked [`MessageV1`], ready to be authcrypt-packed by the
/// caller's agent. Packing is left to the caller because a v1 agent needs the
/// sender's private key and the recipient's verkey, both of which live in
/// connection state this crate does not model.
///
/// The `~thread` decorator is populated from the document's framework members,
/// never the reverse — see [`crate::THREAD_MAPPING`] — and only where the
/// value satisfies RFC 0008's shape: a non-representable correlator is
/// omitted, never rewritten (the omit rule, binding §4).
pub fn build_message<P>(doc: &TrustTask<P>) -> Result<MessageV1, DidcommV1Error>
where
    P: Payload + Serialize,
{
    let document = serde_json::to_value(doc).map_err(DidcommV1Error::SerialiseDocument)?;
    let type_uri = doc.type_uri.to_string();

    let mut msg = MessageV1::new(
        ENVELOPE_TYPE,
        json!({
            // §2: a SHOULD, advisory only — a consumer MUST NOT parse it.
            "comment": summary(&type_uri),
            "~attach": [{
                "@id": ATTACHMENT_ID,
                "mime-type": "application/json",
                "data": { "json": document }
            }],
        }),
    )?;

    // §4.9's own fallback: a document with no threadId is named by its id, so
    // the DIDComm thread and the Trust Task exchange end up sharing one value —
    // but only where that value is representable at all (the omit rule).
    let thid = doc.thread_id.clone().unwrap_or_else(|| doc.id.clone());
    if representable(&thid) {
        msg = msg.thid(thid);
    }
    if let Some(parent) = doc.parent_thread_id.clone() {
        if representable(&parent) {
            msg = msg.pthid(parent);
        }
    }
    Ok(msg)
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
    // §2.3, receivers move first: accept the dedicated type AND 0.1's
    // basic-message carriage. `is_basic_message` rather than comparing `typ`
    // for the legacy half — v1 message types have two interchangeable document
    // URIs and Credo emits the `https://didcomm.org` one by default, so exact
    // comparison silently drops conforming peers.
    if msg.typ != ENVELOPE_TYPE && !basic_message::is_basic_message(msg) {
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
            "req-0001",
            grant::Payload {
                entry: entry(),
                reason: None,
                ext: None,
            },
        )
    }

    #[test]
    fn document_rides_in_the_reserved_attachment_on_the_dedicated_type() {
        let msg = build_message(&doc()).unwrap();
        // §2 item 1: the binding's own message type, not basic-message.
        assert_eq!(msg.typ, ENVELOPE_TYPE);
        assert!(!basic_message::is_basic_message(&msg));
        let attach = msg.body.get("~attach").unwrap().as_array().unwrap();
        assert_eq!(attach.len(), 1);
        assert_eq!(attach[0]["@id"], ATTACHMENT_ID);
        assert_eq!(attach[0]["mime-type"], "application/json");
        // Structured JSON, not a string inside a string — the whole reason for
        // choosing ~attach over JSON-in-content.
        assert!(attach[0]["data"]["json"].is_object());
        assert_eq!(attach[0]["data"]["json"]["id"], "req-0001");
    }

    #[test]
    fn comment_carries_a_human_readable_summary() {
        // §2: `comment` is the advisory human-readable line on a structured
        // message (the Aries convention); the document never travels in it.
        let msg = build_message(&doc()).unwrap();
        let comment = msg.body.get("comment").unwrap().as_str().unwrap();
        assert!(comment.contains("acl/grant"), "comment: {comment}");
        assert!(
            !comment.contains('{'),
            "comment must not be JSON: {comment}"
        );
    }

    /// The omit rule (binding §4): a correlator that cannot satisfy RFC 0008's
    /// shape is omitted, never rewritten. `urn:uuid:` ids are the prominent
    /// real-world case — colons are outside the allowed alphabet.
    #[test]
    fn unrepresentable_thread_ids_are_omitted_never_rewritten() {
        let mut d = doc();
        d.thread_id = Some("urn:uuid:4a0e2b77-88c1-4d55-9f2a-6c3d1e5b7a92".into());
        d.parent_thread_id = Some("urn:uuid:2c7f5d19-6e0b-4c3d-8a41-9b2e6f0d4c88".into());
        let msg = build_message(&d).unwrap();
        assert_eq!(
            msg.explicit_thid(),
            None,
            "thid must be omitted, not rewritten"
        );
        assert_eq!(
            msg.pthid_value(),
            None,
            "pthid must be omitted, not rewritten"
        );
        // The in-band members are untouched — the document is authoritative.
        let attach = msg.body.get("~attach").unwrap().as_array().unwrap();
        assert_eq!(
            attach[0]["data"]["json"]["threadId"],
            "urn:uuid:4a0e2b77-88c1-4d55-9f2a-6c3d1e5b7a92"
        );
    }

    /// The fallback id obeys the same rule: a document with no threadId and an
    /// unrepresentable id produces a message with no thid at all.
    #[test]
    fn unrepresentable_fallback_id_is_also_omitted() {
        let d = TrustTask::for_payload(
            "urn:uuid:11e6c7a2-53d4-4a10-9b6e-2f01c3a9d201",
            grant::Payload {
                entry: entry(),
                reason: None,
                ext: None,
            },
        );
        let msg = build_message(&d).unwrap();
        assert_eq!(msg.explicit_thid(), None);
    }

    #[test]
    fn representable_boundaries() {
        assert!(!representable("short-7")); // 7 chars
        assert!(representable("eight--8")); // 8 chars
        assert!(representable(&"a".repeat(64)));
        assert!(!representable(&"a".repeat(65)));
        assert!(!representable("has:a:colon-in-it"));
        assert!(representable("-_./AZaz09-_./AZaz09"));
    }

    /// §2.3, receivers move first: the consumer-side type gate accepts the 0.1
    /// basic-message carriage alongside the dedicated type.
    #[test]
    fn legacy_basic_message_carriage_still_passes_the_type_gate() {
        use affinidi_messaging_didcomm_v1::protocols::basic_message::BasicMessage;
        let legacy = BasicMessage::new("Trust Task: legacy")
            .unwrap()
            .field(
                "~attach",
                json!([{
                    "@id": ATTACHMENT_ID,
                    "mime-type": "application/json",
                    "data": { "json": serde_json::to_value(doc()).unwrap() }
                }]),
            )
            .finalize();
        // The gate in unpack_trust_task: dedicated type OR basic-message.
        assert!(basic_message::is_basic_message(&legacy));
        let recovered: TrustTask<grant::Payload> =
            serde_json::from_value(attachment(&legacy).unwrap()).unwrap();
        assert_eq!(recovered.id, "req-0001");
    }

    #[test]
    fn thread_is_populated_from_the_framework_members() {
        let mut d = doc();
        d.thread_id = Some("exchange-0001".into());
        d.parent_thread_id = Some("outer-0001".into());
        let msg = build_message(&d).unwrap();
        assert_eq!(msg.explicit_thid(), Some("exchange-0001"));
        assert_eq!(msg.pthid_value(), Some("outer-0001"));
    }

    /// §4.9's fallback: a document with no `threadId` is named by its `id`, so
    /// the DIDComm thread and the Trust Task exchange share one value rather
    /// than the transport inventing its own.
    #[test]
    fn thread_falls_back_to_the_document_id() {
        let msg = build_message(&doc()).unwrap();
        assert_eq!(msg.explicit_thid(), Some("req-0001"));
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
        use affinidi_messaging_didcomm_v1::protocols::basic_message::BasicMessage;
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
        original.thread_id = Some("exchange-0001".into());
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
