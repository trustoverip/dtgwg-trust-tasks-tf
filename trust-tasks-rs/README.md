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
| `ErrorPayload`, `StandardCode`, `TrustTaskCode` | The `trust-task-error` payload + standard codes + extension codes | §8.2, §8.3, §8.5 |
| `trust_task_error_type_uri()` | The one definition of the `trust-task-error` version this library emits | §8.1 |
| `ReplayGuard`, `InMemoryReplayGuard` | Duplicate-execution record: absorbs a bit-for-bit retry, rejects a reused `id` with `idConflict` | §7.2 item 11, §8.4 |
| `FreshnessPolicy` | Acceptance window over `issuedAt` / `expiresAt` — the bound that makes the replay record droppable | §4.2, §7.2 |
| `RejectReason`, `ErrorResponse` | Typed rejection conditions + `TrustTask<ErrorPayload>` alias, both `?`-propagatable | §7.2, §8 |
| `TransportHandler` | Trait for transport bindings: derive party identity, prepare outbound, cross-check inbound | §4.8.1, §9.2 |
| `handlers::NoopHandler` | Transport contributes nothing; in-band members are authoritative | reference impl |
| `handlers::InMemoryHandler` | Simulated transport with configured local+peer VIDs | reference impl |
| `Payload`, `TrustTask::for_payload` | Ties a Rust struct to its Type URI; auto-fills `type` on construction | trait |
| `Dispatcher<R>` | Type-URI → handler routing for consumers that implement N specs | open-set match |
| `AsyncDispatcher<Ctx, R>` | The same routing for `async` handlers, carrying a request-scoped context | open-set match |
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

## Routing to a handler

A consumer that implements several specs registers one handler per Type URI
rather than writing an `if doc.type_uri == …` chain. `Dispatcher<R>` is the
synchronous form; `AsyncDispatcher<Ctx, R>` is the same routing for handlers
that need to `await` and for the request-scoped context they need to do it:

```rust,ignore
use trust_tasks_rs::{specs::acl, AsyncDispatcher};

let dispatcher = AsyncDispatcher::<Arc<AppState>, Outcome>::new()
    .on_async::<acl::grant::v0_1::Payload, _, _>(|req, ctx| async move {
        ctx.db.record_grant(&req.payload.entry).await;      // a real await
        Outcome::Granted
    })
    .on_async::<acl::revoke::v0_1::Payload, _, _>(|req, ctx| async move {
        ctx.db.revoke(&req.payload.subject).await;
        Outcome::Revoked
    });

// `dispatch_or_reject` returns the §8.1-routed `trust-task-error` document
// for every routing-time failure, so the caller emits one or the other.
match dispatcher.dispatch_or_reject(inbound, state.clone(), new_id()).await {
    Ok(outcome) => emit(outcome),
    Err(error)  => emit(error),
}
```

Both dispatchers downcast `Value → P` **once**, and both distinguish
`unsupportedType` (slug not registered) from `unsupportedVersion` (slug
registered at a different `MAJOR.MINOR`, SPEC §5.2 / §8.3) — the answer a
`match` on the whole URI string cannot produce. `AsyncDispatcher` additionally
applies `TrustTask::enforce_spec_policy` to request documents after the
downcast, which is where §7.2 items 5b / 7A / 8 (`recipient` REQUIRED, `proof`
REQUIRED, audience binding) become checkable at all.

## Request → response → error

The framework's request/response model (SPEC §4.4.1) and `trust-task-error`
response (SPEC §8) are first-class. The recommended consumer-side pipeline is
[`consume_inbound`], which runs the SPEC §7.2 checks (item 2 and items 4–8, plus
the freshness bound and item 11's duplicate-execution record) and hands the
accepted document plus the resolved parties to your handler:

```rust,ignore
use trust_tasks_rs::{
    consume_inbound, ConsumeChecks, ConsumeOutcome, InMemoryReplayGuard,
    NoValidator, PayloadPolicy, ProofPolicy,
};

// The guard *is* the duplicate-execution record — one per consumer, held for
// the process's lifetime. Back it with a shared store if you run replicas.
static GUARD: LazyLock<InMemoryReplayGuard> = LazyLock::new(Default::default);

let outcome = consume_inbound(
    transport,
    ProofPolicy::Verify(verifier),         // or RejectIfPresent / AcceptUnverified
    PayloadPolicy::Validate(validator),    // or ::<NoValidator>::AcceptUnvalidated
    ConsumeChecks::consequential(&*GUARD), // or ::not_consequential()
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
    // §7.2 item 11: this document already executed. Not an error — return the
    // prior result where the spec defines one, otherwise emit nothing.
    ConsumeOutcome::Duplicate { prior_response, .. } => {
        if let Some(prior) = prior_response { emit_json(prior) }
    }
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
  request Type URI, plus an `impl RequestPayload` naming the response type
  where the spec defines one.
- A `Response` struct (when the spec defines a success response, SPEC §4.4.1)
  with a second `impl Payload` carrying the `#response` fragment.
- Any shared `$defs` types — for example, `AclEntry` for the ACL specs.
- A `builder` module with one builder per struct.

Every generated struct is `#[non_exhaustive]` and is constructed through its
builder, so **a member added to a schema is not a source break**: it arrives as
one more optional setter and every existing call site keeps compiling. Reading
is unchanged — the fields are `pub`.

```rust,ignore
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, Payload, TrustTask};

let entry: grant::AclEntry = grant::AclEntry::builder()
    .subject("did:web:alice.example")
    .role("admin")
    .try_into()?;
let payload: grant::Payload = grant::Payload::builder()
    .entry(entry)
    .try_into()?;

let req = TrustTask::for_payload("req-1", payload);
assert_eq!(req.type_uri, grant::Payload::type_uri());
```

A struct whose members are all optional also implements `Default`, so
`acl::list::v0_1::Payload::default()` is the whole construction.

`RequestPayload` pairs a request with its response type, which is how a
transport infers the response instead of being told it — see
`HttpsClient::send` in `trust-tasks-https`. A spec that defines no success
response implements no `RequestPayload`.

**Regenerate when a `payload.schema.json` changes:**

```sh
cargo run -p trust-tasks-codegen
```

The output is committed (no `OUT_DIR` magic), so PRs that change a schema
should include the regenerated `src/specs/` diff. CI can enforce this with a
`git diff --exit-code src/specs/` after running the generator.

The framework-defined `trust-task-error` spec is the one exception — its
payload is modelled by hand in `ErrorPayload` because the framework needs the
richer `TrustTaskCode` enum (standard codes + namespaced extension codes) the
codegen can't produce.

## Cargo features

| Feature | What it enables |
|---|---|
| _(default)_ | `all-specs` — the framework crate plus every generated spec family, which is what the crate has always shipped |
| `all-specs` | Every top-level spec family |
| one per family | `acl`, `audit`, `auth`, `chat`, `config`, `confirm`, `consent`, `credential-exchange`, `device`, `did-management`, `git-trust`, `governance`, `keys`, `messaging`, `policy`, `provision`, `push`, `registry`, `sync`, `task-consent`, `vault`, `vrc`, `vta`, `vtc`, `webvh`, `witness` — the `specs::<family>` module tree and nothing else |
| `validate` | Runtime JSON Schema validation. Pulls in [`jsonschema`](https://crates.io/crates/jsonschema) and exposes a `validate` module + `ValidatedPayload` impls for every generated request payload. Belt-and-suspenders over serde's structural decoding — catches `pattern`, `minItems`, and `additionalProperties` constraints that the typed structs can't always encode. |

### Compiling only the families you use

`specs/` is 344 generated modules and ~15 MB of source. Depending on the crate
with default features compiles all of it. If you use three tasks, say so:

```toml
[dependencies]
trust-tasks-rs = { version = "0.13", default-features = false, features = ["vault"] }
```

Measured on one machine with dependencies cached and `CARGO_INCREMENTAL=0`:
**22 s** for the default feature set, **4.0 s** for `vault` alone, **1.2 s**
for `acl` alone.

The framework surface — `TrustTask`, the §7.2 consume pipeline, the transport
traits, `ErrorPayload`, discovery — is always compiled, along with the five
framework-reserved slugs of SPEC §6.1 (`trust-task-control`,
`trust-task-discovery`, `trust-task-next-step`, `trust-task-ok`,
`trust-ceremony-receipt`) that it depends on. Only the task families are
selectable, and each is self-contained: no family's types reference another's,
so any subset compiles.

`schema_index::schema_for` returns `None` for a Type URI whose family you did
not select. That is what `None` already means — "this build knows no spec for
it" — but if you dispatch on Type URIs across the whole registry, take
`all-specs`.

## Status

The crate version is semver over **this library's own API**, deliberately
decoupled from the `SPEC.md` framework version — the two move for different
reasons, and one number cannot answer both questions. A document's framework
version is carried by its specification's `targetFrameworkVersion` declaration
(SPEC §7.3 item 3).

The framework spec is itself a Working Draft; this crate is a reference
implementation maintained alongside it. Breaking changes are expected until the
framework reaches `candidate`. See [`CHANGELOG.md`](CHANGELOG.md) for what has
landed and for the versioning rules.

## License

Apache-2.0. See [`../SOURCE_CODE.md`](../SOURCE_CODE.md) for the source-code
licensing terms of this repository.
