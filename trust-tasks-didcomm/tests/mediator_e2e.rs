//! Integration test against `affinidi-messaging-test-mediator`.
//!
//! Spawns the embedded mediator + ATM client, generates two
//! did:peer users (alice and bob), and proves that:
//!
//! 1. A DIDComm v2.1 [`Message`] carrying our framework-reserved
//!    [`ENVELOPE_TYPE`] and a serialised [`TrustTask`] body round-
//!    trips through `ATM::pack_encrypted` → `ATM::unpack`. This uses
//!    the SDK's real DID resolver and secrets resolver — backed by
//!    the running mediator's identity store — not the bare
//!    `DIDCommAgent` of the local-roundtrip test.
//! 2. The `UnpackMetadata` reported by the SDK contains a verified
//!    sender DID that maps correctly into a [`DidcommHandler`], so
//!    the framework's `TransportHandler::resolve_parties` (SPEC.md
//!    §4.8.1) operates on the same value the production stack
//!    surfaces.
//!
//! What this test does **not** yet cover: routing the packed JWE
//! through the mediator's message-pickup protocol. The mediator's
//! HTTP `send`/`fetch` endpoints aren't the supported wire surface;
//! a follow-up will wire up the message-pickup WebSocket protocol
//! via `affinidi_messaging_sdk::protocols::message_pickup`.
//!
//! Gated `#[ignore]` because the mediator stack has a long cold
//! compile (~3 minutes after the workspace deps are warm; ~10
//! minutes from clean). Run locally with:
//!
//! ```sh
//! cargo test -p trust-tasks-didcomm --test mediator_e2e -- --ignored
//! ```

#![allow(clippy::needless_pass_by_value)]

use affinidi_messaging_didcomm::Message;
use affinidi_messaging_test_mediator::TestEnvironment;
use serde::{Deserialize, Serialize};
use trust_tasks_didcomm::{DidcommHandler, BINDING_URI, ENVELOPE_TYPE};
use trust_tasks_rs::{Payload, TransportHandler, TrustTask};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GrantPayload {
    subject: String,
    role: String,
}

impl Payload for GrantPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/grant/0.1";
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "spawns affinidi-messaging-test-mediator (heavy compile); run with --ignored"]
async fn framework_envelope_round_trips_through_sdk_and_mediator_identity_store() {
    // ─── 1. Spawn the mediator + SDK + a shared secrets/DID resolver.
    //        TestEnvironment.atm is the live ATM client; alice and bob
    //        get real did:peer DIDs registered as LOCAL on the
    //        mediator's identity store.
    let env = TestEnvironment::spawn().await.expect("spawn test mediator");
    let alice = env.add_user("alice").await.expect("add user: alice");
    let bob = env.add_user("bob").await.expect("add user: bob");

    // ─── 2. Compose a Trust Task and wrap it in the framework
    //        envelope type.
    let request = build_trust_task(&alice.did, &bob.did);
    let trust_task_body = serde_json::to_value(&request).expect("serialise TrustTask");
    let didcomm_message = Message::new(ENVELOPE_TYPE, trust_task_body)
        .from(alice.did.clone())
        .to(vec![bob.did.clone()])
        .thid(request.id.clone());

    // ─── 3. Alice authcrypts via the SDK. `from = Some(alice)` →
    //        authcrypt so bob's unpack reports a verified sender.
    let (packed, _meta) = env
        .atm
        .pack_encrypted(&didcomm_message, &bob.did, Some(&alice.did), None)
        .await
        .expect("ATM::pack_encrypted");

    // ─── 4. Bob unpacks via the SAME ATM (they share the resolver
    //        and secrets backend that the mediator manages). This is
    //        the proof of compatibility: any consumer that holds
    //        bob's keys via the SDK can decrypt + verify what alice
    //        produced.
    let (unpacked, meta) = env.atm.unpack(&packed).await.expect("ATM::unpack");

    assert!(meta.encrypted, "envelope should be encrypted");
    assert!(meta.authenticated, "envelope should be authenticated");
    assert_eq!(
        unpacked.typ, ENVELOPE_TYPE,
        "framework envelope type survives the SDK pipe"
    );

    // The SDK reports the verified sender via UnpackMetadata. The
    // value is the sender's key id (DID URL); strip the fragment to
    // get the DID itself.
    let verified_sender_kid = meta
        .encrypted_from_kid
        .as_deref()
        .or(meta.sign_from.as_deref())
        .expect("authcrypt should expose a verified sender kid");
    let verified_sender_did = verified_sender_kid
        .split_once('#')
        .map(|(d, _)| d.to_string())
        .unwrap_or_else(|| verified_sender_kid.to_string());
    assert_eq!(verified_sender_did, alice.did, "verified sender is alice");

    // ─── 5. Reconstruct the typed TrustTask and the DidcommHandler.
    let received: TrustTask<GrantPayload> =
        serde_json::from_value(unpacked.body).expect("deserialise TrustTask body");
    let handler = DidcommHandler::new(Some(bob.did.clone()), Some(verified_sender_did.clone()));

    assert_eq!(received.payload, request.payload);
    assert_eq!(received.issuer.as_deref(), Some(alice.did.as_str()));
    assert_eq!(received.recipient.as_deref(), Some(bob.did.as_str()));
    assert_eq!(handler.binding_uri(), BINDING_URI);

    // ─── 6. Framework §4.8.1 — in-band issuer matches transport
    //        peer; resolve_parties accepts.
    let resolved = handler
        .resolve_parties(&received)
        .expect("identity consistent");
    assert_eq!(resolved.issuer.as_deref(), Some(alice.did.as_str()));
    assert_eq!(resolved.recipient.as_deref(), Some(bob.did.as_str()));

    drop(env);
}

fn build_trust_task(issuer: &str, recipient: &str) -> TrustTask<GrantPayload> {
    let mut doc = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        GrantPayload {
            subject: "did:web:carol.example".into(),
            role: "moderator".into(),
        },
    );
    doc.issuer = Some(issuer.to_string());
    doc.recipient = Some(recipient.to_string());
    doc.issued_at = Some(chrono::Utc::now());
    doc
}
