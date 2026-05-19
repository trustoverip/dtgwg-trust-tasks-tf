# trust-tasks-rs

Reference Rust library for the [Trust Tasks](https://trusttasks.org/) framework.

Trust Tasks are self-contained, transport-agnostic, JSON-based descriptions of
the verifiable work that happens between two parties — a KYC handoff, a consent
grant, an access-control change. This crate provides the framework-level
document type and a `TransportHandler` trait that lets concrete transports
(REST, DIDComm, message queues, ...) plug their identity, integrity, and
freshness semantics into a single validation pipeline.

The framework specification this crate implements is [`SPEC.md`](../SPEC.md).

## What's in here

| Module | Purpose | SPEC.md section |
|---|---|---|
| `TrustTask<P>` | The framework document envelope | §4.2 |
| `TypeUri` | Parsed `https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>` + `#request`/`#response` variant | §4.4, §6.1 |
| `Proof` | W3C Data Integrity proof attachment | §4.7 |
| `ErrorPayload`, `StandardCode`, `TrustTaskCode` | The `trust-task-error/0.1` payload + standard codes + extension codes | §8.2, §8.3, §8.5 |
| `RejectReason`, `ErrorResponse` | Typed rejection conditions + `TrustTask<ErrorPayload>` alias, both `?`-propagatable | §7.2, §8 |
| `TransportHandler` | Trait for transport bindings: derive party identity, prepare outbound, cross-check inbound | §4.8.1, §9.2 |
| `handlers::NoopHandler` | Transport contributes nothing; in-band members are authoritative | reference impl |
| `handlers::InMemoryHandler` | Simulated transport with configured local+peer VIDs | reference impl |
| `Payload`, `TrustTask::for_payload` | Ties a Rust struct to its Type URI; auto-fills `type` on construction | trait |
| `Dispatcher<R>` | Type-URI → handler routing for consumers that implement N specs | open-set match |
| `specs::<slug>::<version>` | Generated per-spec payload types (one module per registry entry) | generated |
| `validate` feature | Runtime JSON Schema validation against the embedded `payload.schema.json` | opt-in |

## Quick start

```rust
use trust_tasks_rs::{TrustTask, TypeUri};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct KycHandoff {
    subject: String,
    result: String,
    level: String,
}

let doc: TrustTask<KycHandoff> = serde_json::from_str(r#"{
    "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
    "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
    "issuer": "did:web:verifier.example",
    "recipient": "did:web:bank.example",
    "issuedAt": "2026-04-12T09:31:00Z",
    "payload": { "subject": "did:key:z6Mk...", "result": "passed", "level": "LOA2" }
}"#)?;

assert_eq!(doc.type_uri.slug(), "kyc-handoff");
assert_eq!(doc.type_uri.major(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Plugging in a transport

The `TransportHandler` trait encodes the §4.8.1 precedence rule: in-band
`issuer` / `recipient` values are authoritative; transport-derived identity is
used to fill in absent members or to cross-check present ones — never to
override them.

```rust,ignore
use trust_tasks_rs::{ResolvedParties, TransportHandler, handlers::InMemoryHandler};

let handler = InMemoryHandler::new()
    .with_local("did:web:bank.example")
    .with_peer("did:web:verifier.example");

let resolved: ResolvedParties = handler.resolve_parties(&doc)?;
// resolved.issuer / resolved.recipient now hold the values the consumer
// MUST apply for every subsequent framework rule that references a party.
```

A REST or DIDComm binding implements the same trait — populating
`derive_parties` from the peer certificate, the DIDComm envelope's verified
sender, or whatever the transport authenticates — and the rest of the
validation pipeline stays unchanged.

## Request → response → error

The framework's request/response model (SPEC §4.4.1) and `trust-task-error/0.1`
response (SPEC §8) are first-class. The recommended consumer-side pipeline is
[`consume_inbound`], which runs the full SPEC §7.2 list (items 4–8) for you and
hands the accepted document plus the resolved parties to your handler:

```rust,ignore
use trust_tasks_rs::{consume_inbound, ConsumeOutcome, ProofPolicy};

let outcome = consume_inbound(
    transport,
    ProofPolicy::Verify(verifier),         // or RejectIfPresent / AcceptUnverified
    inbound,                                // TrustTask<KycHandoff>
    MY_VID,
    Utc::now(),
    || format!("urn:uuid:{}", Uuid::new_v4()),
    |req, parties| async move {
        // parties carries the SPEC §4.8.1-resolved issuer/recipient.
        let receipt = run_kyc(&req.payload).map_err(|e| req.reject_with(new_id(), e.into()))?;
        Ok(req.respond_with(new_id(), receipt))
    },
).await;

match outcome {
    ConsumeOutcome::Handled(response) => emit(response),
    ConsumeOutcome::Rejected(error)   => emit(error),
    ConsumeOutcome::Suppressed        => {} // identity_mismatch w/o transport sender
}
```

`Payload::IS_PROOF_REQUIRED` (codegen-emitted from each spec's
`proofRequirement.requirement: REQUIRED` front-matter) is enforced
authoritatively; `ProofPolicy` makes the proof-handling tradeoff explicit at
the call site rather than implicit in an `Option`. On the receiving side,
`ErrorPayload::should_retry_at(now)` applies §8.4 retry semantics in one call,
and `effective_code()` collapses an unrecognized extension code to
`StandardCode::TaskFailed` per §8.5.

For a runnable producer/consumer loop using the framework primitives directly
(no `consume_inbound`), see [`examples/loopback.rs`](examples/loopback.rs):

```sh
cargo run --example loopback
```

## Per-spec payload types

Every spec under `../specs/<slug>/<version>/payload.schema.json` has a
corresponding Rust module under [`src/specs/`](src/specs/), produced by the
sibling [`trust-tasks-codegen`](../trust-tasks-codegen) crate. Each module
exposes:

- A `Payload` struct (the request payload) with an `impl Payload` pinning the
  request Type URI.
- A `Response` struct (when the spec defines a success response, SPEC §4.4.1)
  with a second `impl Payload` carrying the `#response` fragment.
- Any shared `$defs` types — for example, `AclEntry` for the ACL specs.

```rust,ignore
use trust_tasks_rs::{specs::acl::grant::v0_1::*, Payload, TrustTask};

let req = TrustTask::for_payload("req-1", Payload {
    entry: AclEntry { subject: "did:web:alice.example".into(), role: "admin".parse()?, .. },
    reason: None,
});
assert_eq!(req.type_uri, Payload::type_uri());
```

**Regenerate when a `payload.schema.json` changes:**

```sh
cargo run -p trust-tasks-codegen
```

The output is committed (no `OUT_DIR` magic), so PRs that change a schema
should include the regenerated `src/specs/` diff. CI can enforce this with a
`git diff --exit-code src/specs/` after running the generator.

The framework-defined `trust-task-error/0.1` spec is the one exception — its
payload is modelled by hand in `ErrorPayload` because the framework needs the
richer `TrustTaskCode` enum (standard codes + namespaced extension codes) the
codegen can't produce.

## Cargo features

| Feature | What it enables |
|---|---|
| _(default)_ | The framework crate (no extra deps) |
| `validate` | Runtime JSON Schema validation. Pulls in [`jsonschema`](https://crates.io/crates/jsonschema) and exposes a `validate` module + `ValidatedPayload` impls for every generated request payload. Belt-and-suspenders over serde's structural decoding — catches `pattern`, `minItems`, and `additionalProperties` constraints that the typed structs can't always encode. |

## Status

`0.1.0` — tracks `SPEC.md` version `0.1`. The framework spec is itself a
Working Draft; this crate is a reference implementation maintained alongside
it. Breaking changes are expected until the framework reaches `candidate`.
See [`CHANGELOG.md`](CHANGELOG.md) for what's landed.

## License

Apache-2.0. See [`../SOURCE_CODE.md`](../SOURCE_CODE.md) for the source-code
licensing terms of this repository.
