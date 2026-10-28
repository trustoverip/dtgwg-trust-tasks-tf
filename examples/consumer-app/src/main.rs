//! An application that speaks four Trust Tasks and compiles only those.
//!
//! `trust-tasks.lock` pins `acl/grant/0.2`, `keys/list/0.2` and `vault/get/0.3`
//! from the registry, and `acme:vault/get/0.3` from a third-party source —
//! acme's registry, `[sources.acme]` in `trust-tasks.toml`, here the fixture
//! under `fixture-registry/`. acme's `vault/get` 0.3 has the registry's slug
//! and version and is a different specification: a slug is unique only within
//! its authority (SPEC §6.1), and its Type URI is
//! `https://tasks.example.com/spec/vault/get/0.3`.
//!
//! The bundles are vendored under `trust-tasks/` (acme's under
//! `trust-tasks/@acme/`). `build.rs` runs `trust-tasks-build`, which checks
//! each bundle against its locked digest and writes their Rust types to
//! `$OUT_DIR`, included below as `tasks` — acme's under `tasks::acme`. The
//! application depends on `trust-tasks-rs` with `default-features = false`, so
//! none of the registry's prebuilt specification modules is compiled.
//!
//! ```sh
//! cargo run -p trust-tasks-example-consumer-app
//! ```
//!
//! It plays both parties in one process: a producer builds requests and
//! serializes them; a consumer parses the bytes, routes them with a
//! `Dispatcher`, applies each version's policy, and answers.

use std::num::NonZeroU64;

use serde_json::Value;
use trust_tasks_rs::{Dispatcher, Payload as _, TrustTask};

mod tasks {
    include!(concat!(env!("OUT_DIR"), "/trust_tasks_generated.rs"));
}

use tasks::acl::grant::v0_2 as grant;
use tasks::acme::vault::get::v0_3 as acme_vault_get;
use tasks::keys::list::v0_2 as keys_list;
use tasks::vault::get::v0_3 as vault_get;

const CONSUMER: &str = "did:web:vta.example";
const PRODUCER: &str = "did:web:root-admin.example";

type Error = Box<dyn std::error::Error>;

/// A reply document as JSON.
macro_rules! json {
    ($document:expr) => {
        serde_json::to_value($document).expect("a Trust Task document serializes")
    };
}

fn main() -> Result<(), Error> {
    // The locked versions' SPEC §5.2 `extends` chains, for forward-minor
    // routing. None of these three declares one; the call is what an
    // application does at start-up regardless.
    tasks::declare_extends();

    // The policy each bundle declares is on the generated types.
    assert_eq!(
        keys_list::Payload::TYPE_URI,
        "https://trusttasks.org/spec/keys/list/0.2"
    );
    const { assert!(grant::Payload::IS_PROOF_REQUIRED) };
    const { assert!(!keys_list::Payload::IS_PROOF_REQUIRED) };
    assert_eq!(vault_get::error_codes::NOT_FOUND.code, "vault/get:notFound");
    // One slug and version, two authorities, two specifications.
    assert_eq!(
        acme_vault_get::Payload::TYPE_URI,
        "https://tasks.example.com/spec/vault/get/0.3"
    );
    assert_ne!(
        acme_vault_get::Payload::TYPE_URI,
        vault_get::Payload::TYPE_URI
    );

    let consumer = consumer();

    // 1. keys/list — built with the generated builder, answered with a
    //    generated `Response`.
    let mut request: TrustTask<keys_list::Payload> = TrustTask::for_payload(
        "urn:uuid:00000000-0000-4000-8000-000000000001",
        keys_list::Payload::builder()
            .status(keys_list::KeyStatus::Active)
            .limit(NonZeroU64::new(10).expect("non-zero"))
            .try_into()?,
    );
    // Both specifications REQUIRE the recipient party in-band.
    request.issuer = Some(PRODUCER.into());
    request.recipient = Some(CONSUMER.into());
    let reply = round_trip(&consumer, serde_json::to_string(&request)?)?;
    println!("keys/list  → {}", reply["type"]);
    assert_eq!(
        reply["type"],
        "https://trusttasks.org/spec/keys/list/0.2#response"
    );
    let response: TrustTask<keys_list::Response> = serde_json::from_value(reply)?;
    assert_eq!(response.thread_id.as_deref(), Some(request.id.as_str()));
    assert!(response.payload.keys.is_empty());

    // 2. vault/get — refused with the error code the specification declares.
    let mut request: TrustTask<vault_get::Payload> = TrustTask::for_payload(
        "urn:uuid:00000000-0000-4000-8000-000000000002",
        vault_get::Payload::builder().id("entry-42").try_into()?,
    );
    request.issuer = Some(PRODUCER.into());
    request.recipient = Some(CONSUMER.into());
    let reply = round_trip(&consumer, serde_json::to_string(&request)?)?;
    println!(
        "vault/get  → {} code={}",
        reply["type"], reply["payload"]["code"]
    );
    assert_eq!(reply["payload"]["code"], "vault/get:notFound");

    // 3. acl/grant — the specification REQUIRES a proof; a document without
    //    one is refused by the policy the bundle carried.
    let unsigned: Value = serde_json::json!({
        "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
        "type": "https://trusttasks.org/spec/acl/grant/0.2#request",
        "issuer": PRODUCER,
        "recipient": CONSUMER,
        "issuedAt": "2026-10-02T10:00:00Z",
        "payload": {
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "act": { "scope": "contexts", "contexts": ["ctx/payments"] },
                "keys": { "scope": "all" },
                "capabilities": { "scope": "ceiling" },
                "label": "Alice — payments context admin"
            },
            "reason": "Owns the payments integration."
        }
    });
    let reply = round_trip(&consumer, unsigned.to_string())?;
    println!(
        "acl/grant  → {} code={}",
        reply["type"], reply["payload"]["code"]
    );
    assert_eq!(reply["payload"]["code"], "proofRequired");

    // 4. A task this application did not lock is not a type it knows.
    let unknown = serde_json::json!({
        "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
        "type": "https://trusttasks.org/spec/vault/upsert/0.4",
        "payload": {}
    });
    let reply = round_trip(&consumer, unknown.to_string())?;
    println!(
        "vault/upsert → {} code={}",
        reply["type"], reply["payload"]["code"]
    );
    assert_eq!(reply["payload"]["code"], "unsupportedType");

    // 5. acme's vault/get — the same slug and version as (2), under acme's
    //    authority: routed to its own handler by its whole Type URI, and
    //    answered; never to the registry's vault/get handler.
    let mut request: TrustTask<acme_vault_get::Payload> = TrustTask::for_payload(
        "urn:uuid:00000000-0000-4000-8000-000000000005",
        acme_vault_get::Payload::builder()
            .path("/payments/endpoint")
            .try_into()?,
    );
    request.issuer = Some(PRODUCER.into());
    request.recipient = Some(CONSUMER.into());
    let reply = round_trip(&consumer, serde_json::to_string(&request)?)?;
    println!("acme:vault/get → {}", reply["type"]);
    assert_eq!(
        reply["type"],
        "https://tasks.example.com/spec/vault/get/0.3#response"
    );
    let response: TrustTask<acme_vault_get::Response> = serde_json::from_value(reply)?;
    assert_eq!(response.payload.value, "https://payments.acme.example");

    // 6. A registry vault/get payload sent under acme's Type URI is acme's to
    //    judge — malformed for acme's schema — not quietly the registry's.
    let crossed = serde_json::json!({
        "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
        "type": "https://tasks.example.com/spec/vault/get/0.3",
        "issuer": PRODUCER,
        "recipient": CONSUMER,
        "payload": { "id": "entry-42" }
    });
    let reply = round_trip(&consumer, crossed.to_string())?;
    println!(
        "acme:vault/get (registry payload) → code={}",
        reply["payload"]["code"]
    );
    assert_eq!(reply["payload"]["code"], "malformedRequest");

    println!("round trip ok: 4 generated tasks from two authorities, nothing else compiled");
    Ok(())
}

/// The consumer's half of a round trip: parse the bytes the producer put on
/// the wire, route them, and return the reply document.
fn round_trip(consumer: &Dispatcher<Value>, wire: String) -> Result<Value, Error> {
    let received: TrustTask<Value> = serde_json::from_str(&wire)?;
    let reply = consumer
        .dispatch_or_reject(received, "urn:uuid:00000000-0000-4000-8000-0000000000ee")
        .unwrap_or_else(|rejected| serde_json::to_value(rejected).expect("serializable"));
    Ok(reply)
}

/// The consuming party: one handler per locked task, each returning the reply
/// document as JSON.
fn consumer() -> Dispatcher<Value> {
    Dispatcher::new()
        .on::<keys_list::Payload, _>(|doc| {
            if let Err(reason) = doc.enforce_spec_policy() {
                return json!(&doc.reject_with("urn:uuid:reply-keys", reason));
            }
            let response: keys_list::Response = keys_list::Response::builder()
                .keys(Vec::<keys_list::KeyRecord>::new())
                .try_into()
                .expect("keys/list response");
            json!(&doc.respond_with("urn:uuid:reply-keys", response))
        })
        .on::<vault_get::Payload, _>(|doc| {
            if let Err(reason) = doc.enforce_spec_policy() {
                return json!(&doc.reject_with("urn:uuid:reply-vault", reason));
            }
            println!("  [consumer] vault/get id={}", *doc.payload.id);
            json!(&doc.reject_with("urn:uuid:reply-vault", vault_get::error_codes::NOT_FOUND))
        })
        .on::<acme_vault_get::Payload, _>(|doc| {
            if let Err(reason) = doc.enforce_spec_policy() {
                return json!(&doc.reject_with("urn:uuid:reply-acme", reason));
            }
            println!("  [consumer] acme:vault/get path={}", *doc.payload.path);
            let response: acme_vault_get::Response = acme_vault_get::Response::builder()
                .path(doc.payload.path.to_string())
                .value("https://payments.acme.example")
                .try_into()
                .expect("acme vault/get response");
            json!(&doc.respond_with("urn:uuid:reply-acme", response))
        })
        .on::<grant::Payload, _>(|doc| {
            if let Err(reason) = doc.enforce_spec_policy() {
                return json!(&doc.reject_with("urn:uuid:reply-grant", reason));
            }
            let entry = doc.payload.entry.clone();
            let response: grant::Response = grant::Response::builder()
                .entry(entry)
                .try_into()
                .expect("acl/grant response");
            json!(&doc.respond_with("urn:uuid:reply-grant", response))
        })
}
