//! Fail-closed regressions for the inbound gate.
//!
//! `bindings/didcomm/0.2` §2 makes authcrypt a **MUST** and §4 states that an
//! envelope without an authenticated sender "MUST NOT enter the framework
//! pipeline". Every test here is a shape that reached the pipeline through
//! `trust-tasks-didcomm` 0.10 and must not from 0.11 on.
//!
//! Runs entirely in-process on `PrivateIdentity::generate` — no mediator, no
//! network.

use affinidi_messaging_didcomm::{identity::PrivateIdentity, DIDCommAgent, Message, UnpackResult};
use serde::{Deserialize, Serialize};
use serde_json::json;
use trust_tasks_didcomm::{
    advertised_sender_did, pack_trust_task, unpack_trust_task, unpack_trust_task_from,
    DidcommError, SenderAllowlist, BINDING_URI, ENVELOPE_TYPE,
};
use trust_tasks_rs::{Payload, RejectReason, TrustTask};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GrantPayload {
    subject: String,
    role: String,
}

impl Payload for GrantPayload {
    const TYPE_URI: &'static str = "https://example.com/spec/grant/0.1";
}

fn grant_doc(issuer: &str, recipient: &str) -> TrustTask<GrantPayload> {
    let mut doc = TrustTask::for_payload(
        "urn:uuid:test-grant",
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

/// An agent holding `me` and knowing each of `peers`.
fn agent_for(me: &PrivateIdentity, peers: &[&PrivateIdentity]) -> DIDCommAgent {
    let mut agent = DIDCommAgent::new();
    agent.add_identity(clone_identity(me));
    for peer in peers {
        agent.add_peer(peer.to_resolved());
    }
    agent
}

/// `PrivateIdentity` is not `Clone`, and every field is public, so a test that
/// needs the same identity in two agents copies it by hand.
fn clone_identity(id: &PrivateIdentity) -> PrivateIdentity {
    PrivateIdentity {
        did: id.did.clone(),
        key_agreement_kid: id.key_agreement_kid.clone(),
        key_agreement_private: id.key_agreement_private.clone(),
        signing_kid: id.signing_kid.clone(),
        signing_private: id.signing_private,
    }
}

// ---------------------------------------------------------------------------
// 1. A bare JWS is not an authenticated sender.
// ---------------------------------------------------------------------------

/// A signed-only envelope is signed but sealed to **nobody**: it carries no
/// recipient binding and no confidentiality. Through 0.10 this binding
/// accepted `UnpackResult::Signed` and set the local DID to `None`, which did
/// not merely weaken the SPEC.md §4.8.1 recipient cross-check — it *skipped*
/// it, because there was no transport-authenticated recipient left to compare
/// against.
///
/// The fan-out that follows is asserted rather than described: the same bytes
/// verify for every party that holds the signer's key, so one document could
/// be delivered to a whole deployment and each consumer would take it as
/// addressed to itself.
#[test]
fn signed_only_envelopes_are_rejected() {
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");
    let carol = PrivateIdentity::generate("did:peer:carol");

    let alice_agent = agent_for(&alice, &[&bob]);
    let bob_agent = agent_for(&bob, &[&alice]);
    // Carol is not the addressee. She merely knows alice.
    let carol_agent = agent_for(&carol, &[&alice]);

    let doc = grant_doc(&alice.did, &bob.did);
    let msg = Message::new(ENVELOPE_TYPE, serde_json::to_value(&doc).unwrap())
        .from(alice.did.clone())
        .to(vec![bob.did.clone()]);
    let wire = alice_agent
        .pack_signed(&msg, &alice.did)
        .expect("pack_signed");

    for (who, agent) in [("bob", &bob_agent), ("carol", &carol_agent)] {
        // The transport itself is happy to hand this to anyone: nothing in a
        // JWS names bob.
        assert!(
            matches!(
                agent.unpack(&wire, Some(&alice.did)),
                Ok(UnpackResult::Signed { .. })
            ),
            "{who}: a bare JWS verifies for every holder of the signer's key"
        );

        // The binding gate closes it, for the addressee and the bystander
        // alike.
        let err = unpack_trust_task::<GrantPayload>(&wire, agent, Some(&alice.did))
            .expect_err("a signed-only envelope must not enter the pipeline");
        assert!(
            matches!(err, DidcommError::SignedNotAuthcrypted),
            "{who}: expected SignedNotAuthcrypted, got {err}"
        );
        // §4: with no authenticated sender the only thing that could still
        // attribute the document is an in-band proof.
        assert!(matches!(
            err.into_reject_reason(),
            RejectReason::ProofRequired
        ));
    }
}

/// The positive control for the test above: the same document, authcrypt'd,
/// still round-trips and still yields both parties. Closing the JWS path must
/// not close the conforming one.
#[test]
fn authcrypt_still_round_trips() {
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");
    let alice_agent = agent_for(&alice, &[&bob]);
    let bob_agent = agent_for(&bob, &[&alice]);

    let doc = grant_doc(&alice.did, &bob.did);
    let wire = pack_trust_task(&doc, &alice_agent, &alice.did, &bob.did).expect("pack");
    let (received, handler) =
        unpack_trust_task::<GrantPayload>(&wire, &bob_agent, Some(&alice.did)).expect("unpack");

    assert_eq!(received.payload, doc.payload);
    assert_eq!(handler.peer(), Some(alice.did.as_str()));
    assert_eq!(handler.local(), Some(bob.did.as_str()));
    // Binding §1: the identifier moved to 0.2 with the spec; the envelope type
    // deliberately did not (§7.1).
    assert_eq!(BINDING_URI, "https://trusttasks.org/binding/didcomm/0.2");
    assert_eq!(
        ENVELOPE_TYPE,
        "https://trusttasks.org/binding/didcomm/0.1/envelope"
    );
}

// ---------------------------------------------------------------------------
// 2. A fragment-less `kid` is an error, not an absent sender.
// ---------------------------------------------------------------------------

/// A DIDComm `kid` is a DID **URL** — `did:…#verification-method`. One with no
/// fragment names no verification method and cannot be reduced to a party.
///
/// Through 0.10 that case produced `None`, which is not "reject": it is
/// "treat this authenticated sender as absent, and fall back to the in-band
/// `issuer` with the §4.8.1 cross-check skipped". So a sender with a
/// malformed `kid` could name any issuer it liked and be believed. The
/// downgrade must be an error.
#[test]
fn a_fragment_less_sender_kid_is_an_error_not_an_absent_sender() {
    let bob = PrivateIdentity::generate("did:peer:bob");

    // A sender whose key-agreement kid is a bare DID rather than a DID URL.
    let mut mallory = PrivateIdentity::generate("did:peer:mallory");
    mallory.key_agreement_kid = "did:peer:mallory".to_string();

    let mallory_agent = agent_for(&mallory, &[&bob]);
    let bob_agent = agent_for(&bob, &[&mallory]);

    // Mallory claims in-band to be an authority she is not. Before the fix the
    // malformed kid erased the transport identity and left this unchallenged.
    let doc = grant_doc("did:web:trusted-authority.example", &bob.did);
    let wire = pack_trust_task(&doc, &mallory_agent, &mallory.did, &bob.did).expect("pack");

    let err = unpack_trust_task::<GrantPayload>(&wire, &bob_agent, Some(&mallory.did))
        .expect_err("a fragment-less sender kid must be rejected");
    assert!(
        matches!(&err, DidcommError::UnqualifiedSenderKid { kid } if kid == "did:peer:mallory"),
        "expected UnqualifiedSenderKid, got {err}"
    );
    assert!(matches!(
        err.into_reject_reason(),
        RejectReason::ProofRequired
    ));
}

/// The `skid`/`apu` is written by the sender, so the DID it carries is a claim
/// — the DID that actually authenticated is the one whose public key opened
/// the ECDH-1PU wrap. A peer that authenticates as itself while labelling the
/// envelope with somebody else's DID must not have that label handed to
/// §4.8.1 as the transport-authenticated sender.
#[test]
fn a_skid_naming_a_different_did_than_the_key_that_opened_it_is_rejected() {
    let bob = PrivateIdentity::generate("did:peer:bob");

    // Mallory's own key, labelled as the victim's verification method.
    let mut mallory = PrivateIdentity::generate("did:peer:mallory");
    mallory.key_agreement_kid = "did:web:victim.example#key-agreement-1".to_string();

    let mallory_agent = agent_for(&mallory, &[&bob]);
    let bob_agent = agent_for(&bob, &[&mallory]);

    let doc = grant_doc("did:web:victim.example", &bob.did);
    let wire = pack_trust_task(&doc, &mallory_agent, &mallory.did, &bob.did).expect("pack");

    let err = unpack_trust_task::<GrantPayload>(&wire, &bob_agent, Some(&mallory.did))
        .expect_err("the skid must agree with the key that opened the envelope");
    assert!(
        matches!(
            &err,
            DidcommError::SenderKidMismatch { expected, advertised }
                if expected == "did:peer:mallory" && advertised == "did:web:victim.example"
        ),
        "expected SenderKidMismatch, got {err}"
    );
}

// ---------------------------------------------------------------------------
// 3. The sender allowlist is explicit, and costs one decrypt.
// ---------------------------------------------------------------------------

/// The old advice was to loop over known senders retrying `unpack_trust_task`
/// on `IdentityNotFound` — O(known peers) ECDH-1PU decrypts per message, with
/// the allowlist expressed only as a side effect of which peers the agent
/// happened to hold. `unpack_trust_task_from` reads the envelope's `skid`,
/// checks it against a declared list, and decrypts once.
#[test]
fn the_sender_allowlist_gates_before_any_decryption() {
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");
    let alice_agent = agent_for(&alice, &[&bob]);
    let bob_agent = agent_for(&bob, &[&alice]);

    let doc = grant_doc(&alice.did, &bob.did);
    let wire = pack_trust_task(&doc, &alice_agent, &alice.did, &bob.did).expect("pack");

    // The `skid` is readable without decrypting anything.
    assert_eq!(advertised_sender_did(&wire).unwrap(), alice.did);

    // On the list: accepted, and the verified sender is the one the list
    // matched.
    let allow = SenderAllowlist::new([alice.did.clone()]);
    let (received, handler) =
        unpack_trust_task_from::<GrantPayload>(&wire, &bob_agent, &allow).expect("unpack");
    assert_eq!(received.id, doc.id);
    assert_eq!(handler.peer(), Some(alice.did.as_str()));

    // Off the list: rejected by name, before any cryptography.
    let deny = SenderAllowlist::new(["did:peer:someone-else"]);
    let err = unpack_trust_task_from::<GrantPayload>(&wire, &bob_agent, &deny)
        .expect_err("a sender off the allowlist must be rejected");
    assert!(
        matches!(&err, DidcommError::SenderNotAllowed { did } if did == &alice.did),
        "expected SenderNotAllowed, got {err}"
    );

    // An empty allowlist permits nothing — the fail-closed default.
    let empty = SenderAllowlist::default();
    assert!(empty.is_empty());
    assert!(!empty.permits(&alice.did));
    assert!(matches!(
        unpack_trust_task_from::<GrantPayload>(&wire, &bob_agent, &empty),
        Err(DidcommError::SenderNotAllowed { .. })
    ));

    // `from_agent_peers` reproduces exactly the set the old retry loop could
    // have accepted, so the migration is behaviour-preserving.
    let from_peers = SenderAllowlist::from_agent_peers(&bob_agent);
    assert!(from_peers.permits(&alice.did));
    assert!(unpack_trust_task_from::<GrantPayload>(&wire, &bob_agent, &from_peers).is_ok());
}

/// An anoncrypt envelope names no sender at all, so there is nothing for the
/// allowlist to check and the pre-decrypt read says so rather than guessing.
#[test]
fn anoncrypt_and_non_jwe_bytes_have_no_advertised_sender() {
    let alice = PrivateIdentity::generate("did:peer:alice");
    let bob = PrivateIdentity::generate("did:peer:bob");
    let alice_agent = agent_for(&alice, &[&bob]);
    let bob_agent = agent_for(&bob, &[&alice]);

    let doc = grant_doc(&alice.did, &bob.did);
    let msg = Message::new(ENVELOPE_TYPE, serde_json::to_value(&doc).unwrap())
        .from(alice.did.clone())
        .to(vec![bob.did.clone()]);
    let anon = alice_agent
        .pack_anoncrypt(&msg, &bob.did)
        .expect("pack_anoncrypt");

    assert!(matches!(
        advertised_sender_did(&anon),
        Err(DidcommError::NotAuthcryptJwe(_))
    ));
    assert!(matches!(
        advertised_sender_did(&json!({"not": "a jwe"}).to_string()),
        Err(DidcommError::NotAuthcryptJwe(_))
    ));
    assert!(matches!(
        advertised_sender_did("}{"),
        Err(DidcommError::NotAuthcryptJwe(_))
    ));

    // And an anoncrypt envelope is still refused by the unpack gate itself —
    // §4, no authenticated sender.
    let allow = SenderAllowlist::new([alice.did.clone()]);
    assert!(matches!(
        unpack_trust_task_from::<GrantPayload>(&anon, &bob_agent, &allow),
        Err(DidcommError::NotAuthcryptJwe(_))
    ));
    assert!(matches!(
        unpack_trust_task::<GrantPayload>(&anon, &bob_agent, Some(&alice.did)),
        Err(DidcommError::UnauthenticatedSender)
    ));
}
