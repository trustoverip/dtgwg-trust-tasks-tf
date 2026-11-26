//! A generated payload type keeps a member's presence: absent, present and
//! empty, and an explicit `null` stay three different things on decode and
//! through a decode-then-encode round trip.
//!
//! typify models an optional array as a bare `Vec` that is skipped when empty
//! and an optional nullable member as an `Option` that is skipped when `None`,
//! so `[]` and `null` used to decode as, and re-encode to, an absent member.
//! Where a schema gives the states different meanings that changes the
//! document's meaning: `acl/update` reads an explicit `null` `label` as
//! "clear the label" and an absent one as "leave it unchanged", and an
//! `expiresAt: null` makes the entry permanent. `trust-tasks-build`'s
//! `presence` pass rewrites those fields (`Option<Vec<T>>`,
//! `Option<Option<T>>`). Its own tests cover the field shapes on a fixture
//! schema (a `{"type": "null"}` member, an intrinsic `default`, `true`), and
//! `specs::__presence`'s unit tests run the helper on each shape.
//!
//! A `null` on an optional member whose schema has no null form is **not**
//! refused by decoding: it reads as absent, as in Go and Dart (TypeScript
//! keeps it). Refusing it is payload-schema validation's job, in every
//! runtime (`docs/runtime-parity.md`). The `schema_validation` module below
//! pins where the Rust runtime does that, and where it does not.
#![cfg(feature = "acl")]
// Exercises the prebuilt specification modules on purpose; they are deprecated
// in favour of generating the tasks an application uses (trust-tasks-build).
#![allow(deprecated)]

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use trust_tasks_rs::specs::acl::show::v0_3 as show;
use trust_tasks_rs::specs::acl::update::v0_3 as update;

/// Decode `value` into `T`, re-encode it, and require the same JSON back.
fn round_trips<T: DeserializeOwned + Serialize>(value: Value) -> T {
    let parsed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("deserialize {value}: {e}"));
    let rendered = serde_json::to_value(&parsed).expect("re-serialize");
    assert_eq!(rendered, value, "round-trip changed the payload");
    parsed
}

#[test]
fn an_empty_optional_array_stays_present_and_empty() {
    let r: show::Response = round_trips(json!({ "entry": null, "redactedFields": [] }));
    assert_eq!(r.redacted_fields.as_deref().map(<[_]>::len), Some(0));
}

#[test]
fn an_absent_optional_array_stays_absent() {
    let r: show::Response = round_trips(json!({ "entry": null }));
    assert!(r.redacted_fields.is_none());
}

#[test]
fn a_non_empty_optional_array_round_trips() {
    let r: show::Response = round_trips(json!({ "entry": null, "redactedFields": ["label"] }));
    assert_eq!(r.redacted_fields.map(|v| v.len()), Some(1));
}

#[test]
fn an_explicit_null_stays_null() {
    // "Explicit `null` clears it." / "Explicit `null` makes the entry permanent."
    let p: update::Payload = round_trips(json!({
        "subject": "did:example:alice",
        "label": null,
        "expiresAt": null
    }));
    assert!(
        matches!(p.label, Some(None)),
        "label: null decoded as {:?}",
        p.label
    );
    assert!(matches!(p.expires_at, Some(None)));
}

#[test]
fn an_absent_nullable_member_stays_absent() {
    // "Omitted leaves it unchanged."
    let p: update::Payload = round_trips(json!({ "subject": "did:example:alice" }));
    assert!(p.label.is_none());
    assert!(p.expires_at.is_none());
}

#[test]
fn a_present_nullable_member_round_trips() {
    let p: update::Payload = round_trips(json!({
        "subject": "did:example:alice",
        "label": "Alice",
        "expiresAt": "2030-01-01T00:00:00Z"
    }));
    assert_eq!(
        p.label.flatten().as_deref().map(String::as_str),
        Some("Alice")
    );
}

/// Decode `value` into `T` and return what it re-encodes to.
fn re_encoded<T: DeserializeOwned + Serialize>(value: Value) -> Value {
    let parsed: T = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("deserialize {value}: {e}"));
    serde_json::to_value(&parsed).expect("re-serialize")
}

#[test]
fn a_null_on_an_optional_array_without_a_null_form_reads_as_absent() {
    // `redactedFields` admits no `null`. Decoding reads it as absent; schema
    // validation refuses it (`schema_validation` below).
    let r: show::Response = serde_json::from_value(json!({
        "entry": null,
        "redactedFields": null
    }))
    .expect("decode");
    assert!(r.redacted_fields.is_none());
    assert_eq!(
        re_encoded::<show::Response>(json!({ "entry": null, "redactedFields": null })),
        json!({ "entry": null })
    );
}

#[test]
fn a_null_on_an_optional_member_without_a_null_form_reads_as_absent() {
    // acl/update 0.3 `purposes`: "There is no null form." A `null` decodes
    // as absent ("leave unchanged", the conservative reading) and re-encodes
    // without the member.
    for member in ["purposes", "approve", "keys", "ext"] {
        assert_eq!(
            re_encoded::<update::Payload>(json!({
                "subject": "did:example:alice",
                member: null
            })),
            json!({ "subject": "did:example:alice" }),
            "{member}"
        );
    }
}

#[test]
fn the_builder_writes_each_state() {
    let clear = update::Payload::builder()
        .subject("did:example:alice")
        .label(None::<update::PayloadLabel>)
        .expires_at(None::<chrono::DateTime<chrono::Utc>>);
    let clear: update::Payload = clear.try_into().expect("build");
    assert_eq!(
        serde_json::to_value(&clear).unwrap(),
        json!({ "subject": "did:example:alice", "label": null, "expiresAt": null })
    );

    let set: update::Payload = update::Payload::builder()
        .subject("did:example:alice")
        .label(Some(update::PayloadLabel::try_from("Alice").unwrap()))
        .try_into()
        .expect("build");
    assert_eq!(
        serde_json::to_value(&set).unwrap(),
        json!({ "subject": "did:example:alice", "label": "Alice" })
    );

    let untouched: update::Payload = update::Payload::builder()
        .subject("did:example:alice")
        .try_into()
        .expect("build");
    assert_eq!(
        serde_json::to_value(&untouched).unwrap(),
        json!({ "subject": "did:example:alice" })
    );

    let none_redacted: show::Response = show::Response::builder()
        // `entry` is *required* and nullable, so it is a single `Option`:
        // `None` here is the member's `null` (always written), not the
        // optional-member rule above where `None::<T>` sends `null` and leaving
        // the setter out omits the member.
        .entry(None::<show::AclEntry>)
        .redacted_fields(Vec::<show::ResponseRedactedFieldsItem>::new())
        .try_into()
        .expect("build");
    assert_eq!(
        serde_json::to_value(&none_redacted).unwrap(),
        json!({ "entry": null, "redactedFields": [] })
    );
}

/// An optional map member: `{}` stays present and empty, absent stays absent,
/// and `null` (no null form) reads as absent.
#[cfg(feature = "vta")]
mod map_member {
    use super::*;
    use trust_tasks_rs::specs::vta::did_templates::update::v3_2 as templates;

    fn payload(defaults: Option<Value>) -> Value {
        let mut template = json!({
            "schemaVersion": 1,
            "kind": "agent",
            "name": "example",
            "document": { "id": "did:webvh:{SCID}:example.com:{DID}" }
        });
        if let Some(d) = defaults {
            template["defaults"] = d;
        }
        json!({ "name": "example", "template": template })
    }

    #[test]
    fn an_empty_map_stays_present_and_empty() {
        let p: templates::Payload = round_trips(payload(Some(json!({}))));
        assert_eq!(p.template.defaults.map(|m| m.len()), Some(0));
    }

    #[test]
    fn an_absent_map_stays_absent() {
        let p: templates::Payload = round_trips(payload(None));
        assert!(p.template.defaults.is_none());
    }

    #[test]
    fn a_null_map_reads_as_absent() {
        let p: templates::Payload =
            serde_json::from_value(payload(Some(Value::Null))).expect("decode");
        assert!(p.template.defaults.is_none());
    }
}

/// Where the Rust runtime refuses a `null` the schema forbids: payload-schema
/// validation of the payload **as received**.
#[cfg(feature = "validate")]
mod schema_validation {
    use super::*;
    use trust_tasks_rs::handlers::NoopHandler;
    use trust_tasks_rs::validate::{SchemaValidator, ValidatedPayload};
    use trust_tasks_rs::Payload as _;
    use trust_tasks_rs::{
        consume_dispatched, consume_inbound, AsyncDispatcher, ConsumeChecks, ConsumeOutcome,
        ErrorResponse, PayloadPolicy, ProofPolicy, ProofVerifier, ResolvedParties, StandardCode,
        TrustTask, VerificationError,
    };

    /// The documents carry no proof; this only names a verifier type.
    struct NoVerifier;

    #[async_trait::async_trait]
    impl ProofVerifier for NoVerifier {
        async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
        where
            P: serde::Serialize + Send + Sync,
        {
            Ok(())
        }
    }

    static VALIDATOR: SchemaValidator = SchemaValidator::new();
    const CONSUMER: &str = "did:web:consumer.example";

    /// Each `null` the acl/update 0.3 and acl/show 0.3 schemas forbid.
    fn forbidden_nulls() -> Vec<(&'static str, Value)> {
        let mut cases: Vec<(&'static str, Value)> = ["purposes", "approve", "keys", "ext"]
            .into_iter()
            .map(|member| {
                (
                    update::Payload::TYPE_URI,
                    json!({ "subject": "did:example:alice", member: null }),
                )
            })
            .collect();
        cases.push((
            show::Response::TYPE_URI,
            json!({ "entry": null, "redactedFields": null }),
        ));
        cases
    }

    #[test]
    fn validating_the_received_payload_refuses_a_forbidden_null() {
        for (type_uri, value) in forbidden_nulls() {
            let verdict = if type_uri == update::Payload::TYPE_URI {
                update::Payload::validate_value(&value)
            } else {
                show::Response::validate_value(&value)
            };
            assert!(verdict.is_err(), "{value} passed validation");
        }
        // The same members' `null`, where the schema admits it, passes.
        update::Payload::validate_value(&json!({
            "subject": "did:example:alice",
            "label": null,
            "expiresAt": null
        }))
        .expect("nullable members");
    }

    fn document(payload: Value) -> Value {
        json!({
            "id": "urn:uuid:5f0c2a4e-8d1b-4c3a-9e2f-00000000a1c1",
            "type": update::Payload::TYPE_URI,
            "issuer": "did:web:producer.example",
            "recipient": CONSUMER,
            "issuedAt": "2026-11-01T00:00:00Z",
            "payload": payload
        })
    }

    fn now() -> chrono::DateTime<chrono::Utc> {
        "2026-11-01T00:00:30Z".parse().unwrap()
    }

    /// The refusal code, or `"accepted"`.
    fn verdict<R: std::fmt::Debug>(outcome: &ConsumeOutcome<R>) -> String {
        match outcome {
            ConsumeOutcome::Rejected(err) => format!("{:?}", err.payload.code),
            ConsumeOutcome::Handled(_) => "accepted".into(),
            other => format!("{other:?}"),
        }
    }

    fn malformed() -> String {
        format!(
            "{:?}",
            trust_tasks_rs::TrustTaskCode::from(StandardCode::MalformedRequest)
        )
    }

    /// The document with the forbidden `null`, and the same document without
    /// the member. acl/update requires a proof and these carry none, so a
    /// document that clears schema validation is refused `proofRequired`.
    fn with_and_without_the_null() -> [Value; 2] {
        [
            document(json!({ "subject": "did:example:alice", "purposes": null })),
            document(json!({ "subject": "did:example:alice" })),
        ]
    }

    async fn dispatched(document: Value) -> String {
        let dispatcher =
            AsyncDispatcher::<ResolvedParties, Result<TrustTask<Value>, ErrorResponse>>::new()
                .on_async::<update::Payload, _, _>(|req, _| async move {
                Ok(req.respond_with("resp-1", json!({})))
            });
        let doc: TrustTask<Value> = serde_json::from_value(document).unwrap();
        let outcome = consume_dispatched(
            &NoopHandler::new(),
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::Validate(&VALIDATOR),
            ConsumeChecks::not_consequential(),
            &dispatcher,
            doc,
            CONSUMER,
            now(),
            || "err-1".to_string(),
            |parties| parties,
        )
        .await;
        verdict(&outcome)
    }

    async fn typed(document: Value) -> String {
        let doc: TrustTask<update::Payload> = serde_json::from_value(document).unwrap();
        let outcome = consume_inbound(
            &NoopHandler::new(),
            ProofPolicy::<NoVerifier>::AcceptUnverified,
            PayloadPolicy::Validate(&VALIDATOR),
            ConsumeChecks::not_consequential(),
            doc,
            CONSUMER,
            now(),
            || "err-1".to_string(),
            |req, _| async move { Ok::<_, ErrorResponse>(req.respond_with("resp-1", json!({}))) },
        )
        .await;
        verdict(&outcome)
    }

    #[tokio::test]
    async fn the_dispatched_path_refuses_a_forbidden_null() {
        // `consume_dispatched` holds the payload as received (a `Value`) and
        // validates it before anything reads its meaning.
        let [with, without] = with_and_without_the_null();
        assert_eq!(dispatched(with).await, malformed());
        assert_ne!(dispatched(without).await, malformed());
    }

    #[tokio::test]
    async fn the_typed_path_does_not_see_a_forbidden_null() {
        // `consume_inbound::<P>` takes a document already decoded into `P`
        // and validates `P` re-encoded, from which the forbidden `null`, read
        // as absent, is gone: both documents get the same verdict. A typed
        // consumer that must refuse it validates the received payload itself
        // (`ValidatedPayload::validate_value`) before decoding, or consumes
        // through `consume_dispatched`.
        let [with, without] = with_and_without_the_null();
        let verdict = typed(with).await;
        assert_ne!(verdict, malformed());
        assert_eq!(verdict, typed(without).await);
    }
}
