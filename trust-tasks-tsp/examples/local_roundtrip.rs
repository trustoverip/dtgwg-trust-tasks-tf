//! Local TSP roundtrip — alice seals a Trust Task to bob, in-process.
//!
//! Demonstrates the full `pack_trust_task` / `unpack_trust_task` cycle plus the
//! framework's `TransportHandler` pipeline on the receiving end. No mediator, no
//! network — both VIDs live in the same process.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p trust-tasks-tsp --example local_roundtrip
//! ```

use affinidi_tsp::PrivateVid;
use serde::{Deserialize, Serialize};
use trust_tasks_rs::{Payload, TransportHandler, TrustTask};
use trust_tasks_tsp::{pack_trust_task, unpack_trust_task};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Grant {
    subject: String,
    role: String,
}

impl Payload for Grant {
    const TYPE_URI: &'static str = "https://example.com/spec/grant/0.1";
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Two parties, each with their own VID + keys.
    let alice = PrivateVid::generate("did:example:alice");
    let bob = PrivateVid::generate("did:example:bob");

    // Alice builds a Trust Task and seals it for bob.
    let mut doc = TrustTask::for_payload(
        "urn:uuid:grant-1",
        Grant {
            subject: "did:web:carol.example".into(),
            role: "moderator".into(),
        },
    );
    doc.issuer = Some(alice.id.clone());
    doc.recipient = Some(bob.id.clone());

    let wire = pack_trust_task(&doc, &alice, &bob.to_resolved())?;
    println!("alice sealed {} bytes of TSP", wire.len());

    // Bob opens it (alice's VID resolved for verification).
    let (received, handler) = unpack_trust_task::<Grant>(&wire, &bob, &alice.to_resolved())?;
    println!(
        "bob opened a `{}` from {:?}",
        Grant::TYPE_URI,
        handler.peer()
    );

    // The framework cross-checks the in-band issuer against the authenticated
    // sender (SPEC §4.8.1) and yields the resolved parties.
    let resolved = handler.resolve_parties(&received)?;
    println!(
        "verified  issuer={:?}  recipient={:?}",
        resolved.issuer, resolved.recipient
    );
    Ok(())
}
