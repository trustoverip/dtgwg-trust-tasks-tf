//! Local DIDComm roundtrip — alice sends a Trust Task to bob, in-process.
//!
//! Demonstrates the full `pack_trust_task` / `unpack_trust_task` cycle
//! plus the framework's `TransportHandler` pipeline on the receiving
//! end. No mediator, no network — both `DIDCommAgent`s live in the
//! same process.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p trust-tasks-didcomm --example local_roundtrip
//! ```

use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent};
use trust_tasks_didcomm::{pack_trust_task, unpack_trust_task};
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, RejectReason, TransportHandler, TrustTask};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Generate two fresh peer identities (X25519 key agreement +
    //    Ed25519 signing) and cross-register them.
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");
    let alice_did = alice.did.clone();
    let bob_did = bob.did.clone();
    let alice_resolved = alice.to_resolved();
    let bob_resolved = bob.to_resolved();

    let mut alice_agent = DIDCommAgent::new();
    alice_agent.add_identity(alice);
    alice_agent.add_peer(bob_resolved);

    let mut bob_agent = DIDCommAgent::new();
    bob_agent.add_identity(bob);
    bob_agent.add_peer(alice_resolved);

    println!("alice = {alice_did}\nbob   = {bob_did}\n");

    // 2. Alice composes an acl/grant request and packs it.
    let mut request = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Payload {
            entry: grant::AclEntry {
                subject: "did:web:carol.example".into(),
                role: "moderator".into(),
                scopes: vec![],
                allowed_keys: None,
                label: Some("Carol — content moderation".into()),
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
                expires_at: None,
                approve: None,
                step_up: None,
                ext: None,
            },
            reason: Some("onboarding".into()),
            ext: None,
        },
    );
    request.issuer = Some(alice_did.clone());
    request.recipient = Some(bob_did.clone());
    request.issued_at = Some(chrono::Utc::now());

    let wire = pack_trust_task(&request, &alice_agent, &alice_did, &bob_did)?;
    println!(
        "→ packed JWE ({} bytes, opaque to non-recipients)\n",
        wire.len()
    );

    // 3. Bob unpacks. The returned handler already carries the
    //    transport-authenticated peer DID.
    let (received, handler) =
        unpack_trust_task::<grant::Payload>(&wire, &bob_agent, Some(&alice_did))?;
    println!(
        "← unpacked request:\n  id: {}\n  type: {}\n  in-band issuer: {}\n  in-band recipient: {}\n  transport peer (authenticated): {}\n",
        received.id,
        received.type_uri,
        received.issuer.as_deref().unwrap_or("<none>"),
        received.recipient.as_deref().unwrap_or("<none>"),
        handler.peer().unwrap_or("<none>"),
    );

    // 4. Apply the framework's §7.2 + §4.8.1 pipeline.
    let resolved = handler.resolve_parties(&received)?;
    received.validate_basic(chrono::Utc::now(), &bob_did)?;
    received.enforce_audience_binding()?;
    println!(
        "✓ §4.8.1 resolved parties: issuer={} recipient={}\n",
        resolved.issuer.as_deref().unwrap_or("<none>"),
        resolved.recipient.as_deref().unwrap_or("<none>"),
    );

    // 5. Bob's domain handler accepts and crafts a response.
    let response_doc = received.respond_with(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Response {
            entry: received.payload.entry.clone(),
            ext: None,
        },
    );

    // 6. Bob packs the response back to alice.
    let response_wire = pack_trust_task(&response_doc, &bob_agent, &bob_did, &alice_did)?;
    let (alice_received, _alice_handler) =
        unpack_trust_task::<grant::Response>(&response_wire, &alice_agent, Some(&bob_did))?;

    println!(
        "← alice received response:\n  id: {}\n  type: {}\n  threadId: {}\n  role: {}\n",
        alice_received.id,
        alice_received.type_uri,
        alice_received.thread_id.as_deref().unwrap_or("<none>"),
        &*alice_received.payload.entry.role,
    );

    // 7. Show a failure path: bob rejects a request with PermissionDenied.
    let reject = bob_agent;
    let _ = reject;
    let err: RejectReason = RejectReason::PermissionDenied {
        reason: "demo: bob refuses this grant".into(),
    };
    println!(
        "Failure path would produce a trust-task-error/0.1 with code={}, packed authcrypt'd back to alice.",
        err.code()
    );
    Ok(())
}
