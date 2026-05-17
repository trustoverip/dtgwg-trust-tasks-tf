//! End-to-end Trust Tasks exchange over the in-memory loopback transport.
//!
//! Demonstrates, in one file, both branches of SPEC.md §4.4.1:
//!
//!   1. A producer issues a `kyc-handoff/1.0` *request* and lets the
//!      [`InMemoryHandler`] strip its in-band `issuer`/`recipient` because the
//!      transport carries authenticated identity end-to-end (§9.2 item 1).
//!
//!   2. A consumer applies the framework rules — `resolve_parties` for
//!      §4.8.1 precedence and `validate_basic` for §7.2 items 4 + 5 — then
//!      either calls `respond_with` to emit a `#response` document on
//!      success, or `reject_with` to emit a `trust-task-error/0.1` document
//!      on failure.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example loopback
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use trust_tasks_rs::{
    handlers::InMemoryHandler, ErrorPayload, ErrorResponse, RejectReason, StandardCode,
    TransportHandler, TrustTask, TypeUri,
};

#[derive(Debug, Serialize, Deserialize)]
struct KycHandoff {
    subject: String,
    result: String,
    level: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct KycReceipt {
    receipt_id: String,
    accepted_at: chrono::DateTime<Utc>,
}

const VERIFIER: &str = "did:web:verifier.example";
const BANK: &str = "did:web:bank.example";

/// What the responding party — the bank — does when a `kyc-handoff` request
/// arrives over its transport.
///
/// In production code you'd return `Result<_, Box<ErrorResponse>>` to keep
/// the success path lean; the example keeps the unboxed form for clarity.
#[allow(clippy::result_large_err)]
fn handle(
    request: TrustTask<KycHandoff>,
    handler: &impl TransportHandler,
) -> Result<TrustTask<KycReceipt>, ErrorResponse> {
    let new_id = |stem: &str| format!("urn:example:{}-{}", stem, request.id);

    // §4.8.1 + §7.2 item 6 — in-band must agree with transport identity.
    let _resolved = handler
        .resolve_parties(&request)
        .map_err(|e| request.reject_with(new_id("err"), RejectReason::from(e)))?;

    // §7.2 items 4 (expiry) + 5 (recipient).
    request
        .validate_basic(Utc::now(), BANK)
        .map_err(|reason| request.reject_with(new_id("err"), reason))?;

    // Domain-level check — if the verifier reports failure, return a
    // task-specific error code that consumers handling kyc-handoff specs
    // recognize, and fall back to `task_failed` for those that don't.
    if request.payload.result != "passed" {
        return Err(request.reject_with(
            new_id("err"),
            ErrorPayload::new(StandardCode::TaskFailed).with_message("KYC result was not 'passed'"),
        ));
    }

    Ok(request.respond_with(
        new_id("resp"),
        KycReceipt {
            receipt_id: format!("rcpt-{}", request.id),
            accepted_at: Utc::now(),
        },
    ))
}

fn main() {
    // Both ends know who they are; the in-memory handler conveys this as
    // transport-authenticated identity.
    let producer = InMemoryHandler::new().with_local(VERIFIER).with_peer(BANK);
    let consumer = InMemoryHandler::new().with_local(BANK).with_peer(VERIFIER);

    // ─── 1. Producer builds a request and prepares it for the transport.
    let mut request = TrustTask::new(
        "req-001",
        TypeUri::canonical("kyc-handoff", 1, 0).unwrap(),
        KycHandoff {
            subject: "did:key:z6Mk...".into(),
            result: "passed".into(),
            level: "LOA2".into(),
        },
    );
    request.issuer = Some(VERIFIER.into());
    request.recipient = Some(BANK.into());
    request.issued_at = Some(Utc::now());
    producer.prepare_outbound(&mut request);
    println!(
        "REQUEST (wire form, in-band parties stripped since transport conveys them):\n{}",
        serde_json::to_string_pretty(&request).unwrap()
    );

    // ─── 2. Consumer processes the request.
    match handle(request, &consumer) {
        Ok(response) => println!(
            "\nSUCCESS RESPONSE:\n{}",
            serde_json::to_string_pretty(&response).unwrap()
        ),
        Err(err) => println!(
            "\nERROR RESPONSE:\n{}",
            serde_json::to_string_pretty(&err).unwrap()
        ),
    }

    // ─── 3. Same flow with a tampered request — the in-band issuer disagrees
    // with the transport-authenticated sender.
    let mut tampered = TrustTask::new(
        "req-002",
        TypeUri::canonical("kyc-handoff", 1, 0).unwrap(),
        KycHandoff {
            subject: "did:key:z6Mk...".into(),
            result: "passed".into(),
            level: "LOA2".into(),
        },
    );
    tampered.issuer = Some("did:web:attacker.example".into());
    match handle(tampered, &consumer) {
        Ok(_) => unreachable!("identity mismatch must be rejected"),
        Err(err) => println!(
            "\nERROR RESPONSE (identity mismatch):\n{}",
            serde_json::to_string_pretty(&err).unwrap()
        ),
    }
}
