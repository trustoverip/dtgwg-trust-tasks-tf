# trust-tasks-didcomm

DIDComm v2.1 transport binding for the [Trust Tasks](https://trusttasks.org/) framework. Wraps [`affinidi-messaging-didcomm`](https://crates.io/crates/affinidi-messaging-didcomm) so Trust Task documents can ride inside an authcrypt'd JWE envelope, gaining sender authentication, recipient encryption, and out-of-band routing semantics over any DIDComm-aware transport (mediator pickup, raw HTTPS POST, message queue, even pen-and-paper).

## Binding URI

`https://trusttasks.org/binding/didcomm/0.2`

## Envelope `type`

`https://trusttasks.org/binding/didcomm/0.1/envelope`

The two versions differ on purpose. The binding identifier tracks the binding specification; the envelope `type` is pinned at `0.1` by binding §1 and §7.1 so that a `0.1` and a `0.2` implementation remain mutually intelligible on the wire.

The crate's `pack_trust_task` always emits this exact DIDComm `type` for outbound Trust Task envelopes; `unpack_trust_task` rejects DIDComm messages with any other `type` via [`DidcommError::WrongEnvelopeType`](src/error.rs).

## Quickstart

```rust,ignore
use affinidi_messaging_didcomm::{DIDCommAgent, identity::PrivateIdentity};
use trust_tasks_didcomm::{pack_trust_task, unpack_trust_task};

// 1. Identities.
let alice = PrivateIdentity::generate("did:peer:alice");
let bob   = PrivateIdentity::generate("did:peer:bob");
let alice_did = alice.did.clone();
let bob_did   = bob.did.clone();

// 2. Agents — alice has alice's keys + knows bob; bob has bob's keys + knows alice.
let mut alice_agent = DIDCommAgent::new();
alice_agent.add_identity(alice);
alice_agent.add_peer(bob.to_resolved());

let mut bob_agent = DIDCommAgent::new();
bob_agent.add_identity(bob);
bob_agent.add_peer(alice.to_resolved());

// 3. Alice packs a Trust Task doc.
let wire: String = pack_trust_task(&request, &alice_agent, &alice_did, &bob_did)?;

// 4. Bob unpacks. Handler is populated with the verified peer DID.
let (doc, handler) = unpack_trust_task::<MyPayload>(&wire, &bob_agent, Some(&alice_did))?;

// 5. §7.2 pipeline (unchanged from any other transport).
handler.resolve_parties(&doc)?;
doc.validate_basic(Utc::now(), &bob_did)?;
doc.enforce_audience_binding()?;
```

A full runnable example lives at [`examples/local_roundtrip.rs`](examples/local_roundtrip.rs):

```sh
cargo run -p trust-tasks-didcomm --example local_roundtrip
```

## What the binding authenticates

[`pack_trust_task`](src/pack.rs) emits **authcrypt** JWEs (sender-authenticated + recipient-encrypted). [`unpack_trust_task`](src/pack.rs):

* Accepts `UnpackResult::Encrypted { authenticated: true, sender_kid: Some(_), .. }` — the verified sender DID becomes the framework's transport-authenticated peer for SPEC §4.8.1 cross-check.
* Rejects anoncrypt-only envelopes (no verified sender) with `DidcommError::UnauthenticatedSender`.
* Rejects plaintext envelopes with the same error.
* Rejects **signed-only (bare JWS) envelopes** with `DidcommError::SignedNotAuthcrypted`. Binding §2 makes authcrypt a MUST and §4 keeps everything else out of the pipeline; a JWS is signed but sealed to nobody, so it carries no recipient binding — one message can be delivered to every party in a deployment and each will verify it.
* Rejects a verified `sender_kid` carrying no `#fragment` with `DidcommError::UnqualifiedSenderKid`. A bare DID is not a DID URL, and reducing it to "no sender" would downgrade an authenticated identity to an unauthenticated one rather than rejecting it.
* Rejects an envelope whose `skid` names a different DID than the key that opened it, with `DidcommError::SenderKidMismatch`. The `skid` is sender-chosen; the key that unwrapped the CEK is what authenticated.

The DID is derived from the verified `sender_kid` by stripping the `#fragment` (the key ID); the framework's `issuer` field uses the bare DID.

## Multi-peer servers

`DIDCommAgent::unpack` needs the expected sender DID to look up that sender's public key, so `unpack_trust_task` takes an `expected_sender_did: Option<&str>`.

A server receiving from many peers should **declare its senders** rather than guess them:

```rust,ignore
use trust_tasks_didcomm::{SenderAllowlist, unpack_trust_task_from};

let allow = SenderAllowlist::new(["did:peer:alice", "did:peer:carol"]);
let (doc, handler) = unpack_trust_task_from::<MyPayload>(&wire, &agent, &allow)?;
```

`unpack_trust_task_from` reads the `skid` from the JWE protected header, checks the DID it names against the allowlist **before decrypting anything**, and then unpacks once against that one sender. An empty allowlist permits nothing.

This replaces the "iterate over known senders, retry on `DIDCommError::IdentityNotFound`" pattern earlier versions of this README recommended, which cost O(known peers) ECDH-1PU decrypts per inbound message and expressed the allowlist only as a side effect of which peers the agent happened to hold. `SenderAllowlist::from_agent_peers(&agent)` reproduces exactly that set if you want the old behaviour while dropping the cost.

The `skid` is sender-chosen and proves nothing on its own — it only selects which key to unpack against. Authentication comes from the ECDH-1PU wrap opening, and the verified sender is re-checked against the `skid` that selected it (`DidcommError::SenderKidMismatch`).

## MSRV

1.95, matching the workspace.

## Tests

| File | What it proves | Run cost |
|---|---|---|
| `tests/end_to_end.rs` | Local pack/unpack roundtrip via the bare `DIDCommAgent`; happy path, forged in-band issuer, wrong envelope type, JWE-on-wire | seconds |
| `tests/fail_closed.rs` | The inbound gate refuses every non-authcrypt shape binding §2/§4 excludes: bare JWS (asserting the fan-out it enables), fragment-less `sender_kid`, spoofed `skid`, anoncrypt; plus the `SenderAllowlist` path | seconds |
| `tests/mediator_e2e.rs` | Real `affinidi-messaging-test-mediator` spawned, two `did:peer` users registered as LOCAL on the mediator, framework `ENVELOPE_TYPE` round-trips through `ATM::pack_encrypted` → `ATM::unpack`, verified sender from `UnpackMetadata` slots into `DidcommHandler::peer()` correctly, framework §4.8.1 still honored | minutes (cold compile of full mediator + SDK) |

The mediator test is gated `#[ignore]` so the default `cargo test` skips it. Opt in with:

```sh
cargo test -p trust-tasks-didcomm --test mediator_e2e -- --ignored
```

Currently the test proves compatibility through the SDK's pack/unpack pipeline against a live mediator's identity store and resolver — it does not yet route the packed envelope through the mediator's message-pickup protocol. Wiring up `affinidi_messaging_sdk::protocols::message_pickup` for full mediator-routed delivery is a future PR.

## Status

`0.11.0`, implementing binding [`didcomm/0.2`](../bindings/didcomm/0.2/spec.md) against framework `0.3`+.
