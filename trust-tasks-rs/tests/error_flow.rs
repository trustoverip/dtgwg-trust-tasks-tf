//! End-to-end consumer-side error handling.
//!
//! Walks the path a real consumer would take:
//!
//!   1. Resolve party identity through the [`TransportHandler`].
//!   2. Run framework-level checks (§7.2 items 4 and 5).
//!   3. On failure, mint a `trust-task-error/0.2` response that satisfies the
//!      spec's "Reporting consumer" conformance rules.
//!
//! Each step uses the public surface only.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use trust_tasks_rs::{
    handlers::InMemoryHandler, ConsistencyError, ErrorPayload, ErrorResponse, RejectReason,
    StandardCode, TransportHandler, TrustTask, TrustTaskCode, TypeUri,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct KycHandoff {
    subject: String,
    result: String,
    level: String,
}

fn request(issuer: Option<&str>, recipient: Option<&str>) -> TrustTask<KycHandoff> {
    let mut req = TrustTask::new(
        "req-1",
        TypeUri::canonical("kyc-handoff", 1, 0).unwrap(),
        KycHandoff {
            subject: "did:key:z6Mk".into(),
            result: "passed".into(),
            level: "LOA2".into(),
        },
    );
    req.issuer = issuer.map(str::to_string);
    req.recipient = recipient.map(str::to_string);
    req
}

/// SPEC §8 "Reporting consumer" — every framework-required field on the error
/// response is wired correctly by `reject_with`.
#[test]
fn reject_with_wires_framework_members() {
    let req = request(
        Some("did:web:verifier.example"),
        Some("did:web:bank.example"),
    );

    let err: ErrorResponse = req.reject_with("err-1", RejectReason::ProofRequired);

    assert_eq!(err.id, "err-1");
    assert_eq!(
        err.type_uri,
        "https://trusttasks.org/spec/trust-task-error/0.2"
            .parse()
            .unwrap()
    );
    // threadId falls back to the request's id when no threadId was carried.
    assert_eq!(err.thread_id.as_deref(), Some("req-1"));
    // Issuer / recipient swap.
    assert_eq!(err.issuer.as_deref(), Some("did:web:bank.example"));
    assert_eq!(err.recipient.as_deref(), Some("did:web:verifier.example"));
    assert!(err.issued_at.is_some());
    assert_eq!(err.payload.code, StandardCode::ProofRequired.into());
    assert!(!err.payload.retryable);
}

/// SPEC §4.9 — when the request carries a threadId, the error preserves it.
#[test]
fn reject_with_preserves_thread_id() {
    let mut req = request(
        Some("did:web:verifier.example"),
        Some("did:web:bank.example"),
    );
    req.thread_id = Some("thread-42".into());

    let err = req.reject_with("err-1", StandardCode::Expired);
    assert_eq!(err.thread_id.as_deref(), Some("thread-42"));
}

/// SPEC §7.2 item 4 — expired documents short-circuit `validate_basic`.
#[test]
fn validate_basic_rejects_expired() {
    let mut req = request(
        Some("did:web:verifier.example"),
        Some("did:web:bank.example"),
    );
    req.expires_at = Some("2026-04-12T09:31:00Z".parse().unwrap());

    let now: DateTime<Utc> = "2026-05-17T00:00:00Z".parse().unwrap();
    let reason = req.validate_basic(now, "did:web:bank.example").unwrap_err();
    assert_eq!(reason.code(), StandardCode::Expired);
}

/// SPEC §7.2 item 5 — `recipient` set to someone else short-circuits.
#[test]
fn validate_basic_rejects_wrong_recipient() {
    let req = request(
        Some("did:web:verifier.example"),
        Some("did:web:not-me.example"),
    );

    let now: DateTime<Utc> = "2026-05-17T00:00:00Z".parse().unwrap();
    let reason = req.validate_basic(now, "did:web:bank.example").unwrap_err();
    assert!(matches!(
        reason,
        RejectReason::WrongRecipient { ref in_band, ref expected }
        if in_band == "did:web:not-me.example" && expected == "did:web:bank.example"
    ));
}

/// SPEC §7.2 item 5 — an absent in-band `recipient` is fine; the transport
/// (or the consumer's policy) fills the gap.
#[test]
fn validate_basic_passes_when_recipient_absent() {
    let req = request(Some("did:web:verifier.example"), None);
    let now: DateTime<Utc> = "2026-05-17T00:00:00Z".parse().unwrap();
    assert!(req.validate_basic(now, "did:web:bank.example").is_ok());
}

/// SPEC §4.8.1 / §7.2 item 6 — a transport handler that disagrees with the
/// in-band issuer feeds back through `?` into a `RejectReason::IdentityMismatch`.
#[test]
fn full_consumer_flow_emits_well_formed_error_response() {
    // Inbound request claims to be from an attacker; transport authenticated
    // a different sender.
    let req = request(Some("did:web:attacker.example"), None);

    let handler = InMemoryHandler::new()
        .with_local("did:web:bank.example")
        .with_peer("did:web:verifier.example");

    let resolved = handler.resolve_parties(&req);
    let reason: RejectReason = resolved.unwrap_err().into();
    assert_eq!(reason.code(), StandardCode::IdentityMismatch);

    let err = req.reject_with("err-99", reason);
    // The response itself satisfies §7.2 / §8.1 — try round-tripping it.
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(
        json["type"],
        serde_json::json!("https://trusttasks.org/spec/trust-task-error/0.2")
    );
    assert_eq!(
        json["payload"]["code"],
        serde_json::json!("identityMismatch")
    );
    assert_eq!(json["payload"]["retryable"], serde_json::json!(false));

    let back: ErrorResponse = serde_json::from_value(json).unwrap();
    assert_eq!(back.payload.code, StandardCode::IdentityMismatch.into());
}

/// SPEC §8.4 — retry semantics on the receiving side.
#[test]
fn receiver_honors_retry_after() {
    let now: DateTime<Utc> = "2026-05-17T12:00:00Z".parse().unwrap();
    let later: DateTime<Utc> = "2026-05-17T13:00:00Z".parse().unwrap();

    let err: ErrorResponse = request(None, None).reject_with(
        "err-1",
        ErrorPayload::new(StandardCode::Unavailable).with_retry_after(later),
    );

    assert!(!err.payload.should_retry_at(now));
    assert!(err.payload.should_retry_at(later));
}

/// SPEC §8.5 — a consumer that doesn't recognize the extension code MUST
/// still honor retryable/retryAfter; `effective_code` lets it route the error
/// like a `task_failed`.
#[test]
fn unknown_extension_code_collapses_to_task_failed() {
    let json = r#"{
        "id": "err-2",
        "type": "https://trusttasks.org/spec/trust-task-error/0.1",
        "threadId": "req-1",
        "payload": {
            "code": "unknown-spec:weird_failure",
            "retryable": false,
            "details": { "trace": "abc" }
        }
    }"#;

    let err: ErrorResponse = serde_json::from_str(json).unwrap();
    assert_eq!(err.payload.effective_code(), StandardCode::TaskFailed);
    // But the original code is preserved on the wire.
    assert_eq!(
        err.payload.code,
        TrustTaskCode::Extended {
            slug: "unknown-spec".into(),
            local: "weird_failure".into(),
        }
    );
}

/// SPEC.md §10.4 — wire error messages MUST NOT leak consumer-side
/// authentication context. `RejectReason::wire_message` returns sanitised
/// strings for the identity-bearing variants; `From<RejectReason> for
/// ErrorPayload` uses them.
#[test]
fn wire_error_message_does_not_leak_consumer_vid() {
    let req = request(Some("did:web:attacker.example"), None);

    let handler = InMemoryHandler::new()
        .with_local("did:web:bank.example")
        .with_peer("did:web:verifier.example");

    let reason: RejectReason = handler.resolve_parties(&req).unwrap_err().into();
    let response = req.reject_with("err-1", reason);

    let wire = response.payload.message.as_deref().unwrap();
    assert!(
        !wire.contains("bank.example"),
        "wire message leaks consumer's authenticated VID: {wire}"
    );
    assert!(
        !wire.contains("attacker.example"),
        "wire message leaks contested in-band VID: {wire}"
    );
    assert!(!wire.contains("verifier.example"));

    // Same for wrong-recipient.
    let reason = req
        .validate_basic(
            "2026-05-17T00:00:00Z".parse().unwrap(),
            "did:web:bank.example",
        )
        .err();
    if let Some(r) = reason {
        let resp = req.reject_with("err-2", r);
        let wire = resp.payload.message.as_deref().unwrap();
        assert!(!wire.contains("bank.example"));
    }
}

/// SPEC.md §8.1 — under `identity_mismatch`, the error response MUST be
/// addressed to the transport-authenticated sender, not the contested
/// in-band issuer. [`TransportHandler::reject`] applies this policy.
#[test]
fn handler_reject_routes_identity_mismatch_to_transport_peer() {
    let req = request(Some("did:web:attacker.example"), None);

    let handler = InMemoryHandler::new()
        .with_local("did:web:bank.example")
        .with_peer("did:web:verifier.example");

    let reason: RejectReason = handler.resolve_parties(&req).unwrap_err().into();
    let response = handler.reject(&req, "err-1", reason).unwrap();

    assert_eq!(
        response.recipient.as_deref(),
        Some("did:web:verifier.example"),
        "identity_mismatch error MUST address the transport-authenticated sender, \
         not the contested in-band issuer"
    );
}

/// SPEC.md §8.1 — when the transport authenticates no sender and the
/// rejection is `identity_mismatch`, the consumer SHOULD NOT emit a
/// response. The handler API surfaces this by returning `None`.
#[test]
fn handler_reject_suppresses_response_when_no_transport_sender() {
    use trust_tasks_rs::handlers::NoopHandler;
    let req = request(Some("did:web:attacker.example"), None);

    let handler = NoopHandler::new();
    let reason: RejectReason = ConsistencyError::IssuerMismatch {
        in_band: "did:web:attacker.example".into(),
        transport: "did:web:somewhere.example".into(),
    }
    .into();
    let response = handler.reject(&req, "err-1", reason);
    assert!(
        response.is_none(),
        "identity_mismatch with no transport sender MUST suppress the response"
    );
}

/// SPEC.md §8.1 — for non-mismatch rejections, the in-band issuer is the
/// trusted "original producer" (resolve_parties already accepted it) and
/// is the right recipient. The handler-mediated `reject` preserves that.
#[test]
fn handler_reject_routes_non_mismatch_to_in_band_issuer() {
    let req = request(
        Some("did:web:verifier.example"),
        Some("did:web:bank.example"),
    );

    let handler = InMemoryHandler::new()
        .with_local("did:web:bank.example")
        .with_peer("did:web:verifier.example");

    let response = handler
        .reject(&req, "err-1", RejectReason::ProofRequired)
        .unwrap();
    assert_eq!(
        response.recipient.as_deref(),
        Some("did:web:verifier.example")
    );
}

/// `ErrorResponse` is a real `std::error::Error` — it `?`-propagates and
/// surfaces the payload as its source.
#[test]
fn error_response_implements_std_error() {
    use std::error::Error;

    let err = request(None, None).reject_with(
        "err-1",
        ErrorPayload::new(StandardCode::Expired).with_message("expired"),
    );

    let as_dyn: &dyn Error = &err;
    assert!(as_dyn.source().is_some());
    assert!(format!("{err}").contains("expired"));
}
