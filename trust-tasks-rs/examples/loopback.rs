//! End-to-end Trust Tasks exchange over the in-memory loopback transport.
//!
//! Demonstrates, in one file, both branches of SPEC.md §4.4.1:
//!
//!   1. A producer issues an `acl/change-role/0.1` *request* and lets the
//!      [`InMemoryHandler`] strip its in-band `issuer`/`recipient` because the
//!      transport carries authenticated identity end-to-end (§9.2 item 1).
//!
//!   2. A consumer applies the framework rules — `resolve_parties` for
//!      §4.8.1 precedence and `validate_basic` for §7.2 items 4 + 5 — then
//!      either calls `respond_with` to emit a `#response` document on
//!      success, or `reject_with` to emit a `trust-task-error` document
//!      on failure.
//!
//! The task is a registered one — <https://trusttasks.org/spec/acl/change-role/0.1>
//! — so every URI here dereferences, and the payload members are the ones its
//! schema defines.
//!
//! ## Why this example hand-rolls the pipeline
//!
//! Real consumers should use [`trust_tasks_rs::consume_inbound`], which
//! runs the full SPEC §7.2 list (items 4–8 — expiry, recipient,
//! identity, proof, audience binding) in one call and takes a typed
//! [`ProofPolicy`](trust_tasks_rs::ProofPolicy) for the proof-handling
//! tradeoff. This example exercises the framework primitives directly
//! so a reader can see what `consume_inbound` is composing under the
//! hood; for production code the helper is shorter, applies all eight
//! checks in the spec-prescribed order, and forces the proof policy
//! to be an explicit choice at the call site.
//!
//! The payload types are likewise hand-rolled to keep the framework
//! mechanics in view; `trust_tasks_rs::specs::acl::change_role` carries the
//! generated ones, which is what production code should use.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example loopback
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};
use trust_tasks_rs::{
    handlers::InMemoryHandler, ErrorPayload, ErrorResponse, RejectReason, TransportHandler,
    TrustTask, TrustTaskCode, TypeUri,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AclChangeRole {
    subject: String,
    from_role: String,
    to_role: String,
}

/// The `#response` payload: the entry the maintainer now holds for the
/// subject, whose `role` equals the request's `toRole`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeRoleResponse {
    entry: AclEntry,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AclEntry {
    subject: String,
    role: String,
    updated_at: chrono::DateTime<Utc>,
    updated_by: String,
}

const CHANGING_AUTHORITY: &str = "did:web:org.example";
const MAINTAINER: &str = "did:web:maintainer.example";

/// The role the maintainer's own ACL currently records for the subject. The
/// optimistic concurrency check compares `payload.fromRole` against this.
const CURRENT_ROLE: &str = "member";

/// What the responding party — the ACL maintainer — does when an
/// `acl/change-role` request arrives over its transport.
///
/// In production code you'd return `Result<_, Box<ErrorResponse>>` to keep
/// the success path lean; the example keeps the unboxed form for clarity.
#[allow(clippy::result_large_err)]
fn handle(
    request: TrustTask<AclChangeRole>,
    handler: &impl TransportHandler,
) -> Result<TrustTask<ChangeRoleResponse>, ErrorResponse> {
    let new_id = |stem: &str| format!("urn:example:{}-{}", stem, request.id);

    // §4.8.1 + §7.2 item 6 — in-band must agree with transport identity.
    let _resolved = handler
        .resolve_parties(&request)
        .map_err(|e| request.reject_with(new_id("err"), RejectReason::from(e)))?;

    // §7.2 items 4 (expiry) + 5 (recipient).
    request
        .validate_basic(Utc::now(), MAINTAINER)
        .map_err(|reason| request.reject_with(new_id("err"), reason))?;

    // Domain-level check — the specification's own optimistic concurrency
    // rule. It fails with a §8.5 extended code namespaced under the emitting
    // slug, which consumers implementing this spec recognize and every other
    // consumer degrades to `taskFailed`.
    if request.payload.from_role != CURRENT_ROLE {
        return Err(request.reject_with(
            new_id("err"),
            ErrorPayload::new(
                TrustTaskCode::new_extended("acl/change-role", "stateMismatch")
                    .expect("a well-formed §8.5 extended code"),
            )
            .with_message("subject's current role does not match payload.fromRole")
            .with_retryable(true)
            .with_details(serde_json::json!({ "currentRole": CURRENT_ROLE })),
        ));
    }

    Ok(request.respond_with(
        new_id("resp"),
        ChangeRoleResponse {
            entry: AclEntry {
                subject: request.payload.subject.clone(),
                role: request.payload.to_role.clone(),
                updated_at: Utc::now(),
                updated_by: CHANGING_AUTHORITY.into(),
            },
        },
    ))
}

fn main() {
    // Both ends know who they are; the in-memory handler conveys this as
    // transport-authenticated identity.
    let producer = InMemoryHandler::new()
        .with_local(CHANGING_AUTHORITY)
        .with_peer(MAINTAINER);
    let consumer = InMemoryHandler::new()
        .with_local(MAINTAINER)
        .with_peer(CHANGING_AUTHORITY);

    let change = |from: &str, to: &str| AclChangeRole {
        subject: "did:web:bob.example".into(),
        from_role: from.into(),
        to_role: to.into(),
    };

    // ─── 1. Producer builds a request and prepares it for the transport.
    let mut request = TrustTask::new(
        "req-001",
        TypeUri::canonical("acl/change-role", 0, 1).unwrap(),
        change("member", "moderator"),
    );
    request.issuer = Some(CHANGING_AUTHORITY.into());
    request.recipient = Some(MAINTAINER.into());
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

    // ─── 3. A request built on stale state — the subject has already been
    // moved off the role the producer thinks they hold.
    let mut stale = TrustTask::new(
        "req-002",
        TypeUri::canonical("acl/change-role", 0, 1).unwrap(),
        change("moderator", "admin"),
    );
    stale.issuer = Some(CHANGING_AUTHORITY.into());
    stale.recipient = Some(MAINTAINER.into());
    stale.issued_at = Some(Utc::now());
    match handle(stale, &consumer) {
        Ok(_) => unreachable!("a fromRole that disagrees with the ACL must be rejected"),
        Err(err) => println!(
            "\nERROR RESPONSE (stale state):\n{}",
            serde_json::to_string_pretty(&err).unwrap()
        ),
    }

    // ─── 4. Same flow with a tampered request — the in-band issuer disagrees
    // with the transport-authenticated sender.
    let mut tampered = TrustTask::new(
        "req-003",
        TypeUri::canonical("acl/change-role", 0, 1).unwrap(),
        change("member", "moderator"),
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
