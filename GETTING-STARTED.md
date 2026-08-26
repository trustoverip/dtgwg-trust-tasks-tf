# Getting started with Trust Tasks

A [Trust Task](https://trusttasks.org/) is a self-contained, transport-agnostic
JSON document describing verifiable work between two parties. This page takes
you from nothing to a **signed `acl/grant` round trip** — sender and receiver,
Rust and TypeScript — and then names the four things that reliably cost an
afternoon.

If you are writing a *specification* rather than consuming one, you want
[`CONTRIBUTING-SPECS.md`](./CONTRIBUTING-SPECS.md) and [`SPEC.md`](./SPEC.md)
instead. This page is for consumers.

---

## 1. One dependency line

```toml
[dependencies]
trust-tasks = { version = "0.1", features = ["https", "proof-affinidi"] }
```

```sh
npm install @openvtc/trust-tasks
```

`trust-tasks` is a facade: it re-exports the framework's eight crates behind
Cargo features so you pick a transport rather than a set of version numbers.
Everything in it is a `pub use` — `trust_tasks::TrustTask` **is**
`trust_tasks_rs::TrustTask` — so dropping the facade later is a
find-and-replace, not a migration.

### The crate matrix

Pick one row per thing you want to do; features combine.

| I want to… | facade feature | what you write | underlying crate |
|---|---|---|---|
| model a document, run the SPEC §7.2 consumer checks | *(always on)* | `trust_tasks::{TrustTask, consume_inbound, specs, RejectReason}` | `trust-tasks-rs` |
| **send/receive over HTTPS** | `https` | `trust_tasks::https::{HttpsClient, HttpsServer, BearerAuth}` | `trust-tasks-https` |
| **send/receive over DIDComm v2.1** | `didcomm` | `trust_tasks::didcomm::{DidcommHandler, DidcommConsumer}` | `trust-tasks-didcomm` |
| **reach Aries-lineage agents** (DIDComm v1) | `didcomm-v1` | `trust_tasks::didcomm_v1::{DidcommV1Handler, DidcommV1Consumer}` | `trust-tasks-didcomm-v1` |
| **send/receive over ToIP TSP** | `tsp` | `trust_tasks::tsp::{TspHandler, TspConsumer}` | `trust-tasks-tsp` |
| **sign** a document / **verify** an inbound proof | `proof-affinidi` | `trust_tasks::proof::{ProofExt, affinidi::{SignOptions, Verifier}}` | `trust-tasks-proof` |
| validate payloads against their JSON Schema at runtime | `validate` | `trust_tasks::validate` | `trust-tasks-rs` |
| verify Trust Ceremony receipts / step digests | `ceremony` | `trust_tasks::ceremony::JcsSha256Digester` | `trust-tasks-ceremony` |
| build/parse `governance/capability/*`, `git-trust/*` documents | `capability-client` | `trust_tasks::capability_client::*` | `trust-tasks-capability-client` |
| bridge two bindings in one process | `all-transports` | all four transport modules | — |
| use JWT bearer auth on the HTTPS server | `https-jwt` | `trust_tasks::https::JwtBearerAuth` | `trust-tasks-https` |

**You almost certainly want a transport plus `proof-affinidi`.** A task whose
specification declares `proof` REQUIRED — `acl/grant/0.1` is one — cannot be
produced or consumed without a signer and a verifier.

### When *not* to use the facade

`trust-tasks-rs` has 26 per-spec-family features (`vault`, `acl`, `keys`, …) so
a size-sensitive build compiles only the families it speaks. The facade
forwards only the `all-specs` umbrella, not the 26 — putting 26 more names in
front of a newcomer is the problem the facade exists to remove, and Cargo
unifies features across the graph anyway, so trimming only pays off when
nothing else asks for them.

So: **if you are trimming spec families, or turning off a transport crate's own
defaults (`trust-tasks-https`'s `server`, `trust-tasks-didcomm-v1`'s
`legacy-basic-message`), depend on those crates directly.** That is the
supported answer, not a workaround — you have outgrown the front door.

---

## 2. A signed `acl/grant` round trip, in Rust

Every Rust block below is **extracted from a runnable example**,
[`trust-tasks/examples/acl_grant_roundtrip.rs`](./trust-tasks/examples/acl_grant_roundtrip.rs).
A test (`trust-tasks/tests/getting_started_snippets.rs`) fails the build if this
page and that file drift, so what you copy is code that compiles today.

```sh
cargo run -p trust-tasks --features https,proof-affinidi \
    --example acl_grant_roundtrip
```

### Imports

Everything comes through the facade — one crate name in every path.

<!-- snippet: trust-tasks/examples/acl_grant_roundtrip.rs#imports -->
```rust
use affinidi_secrets_resolver::secrets::Secret;
use trust_tasks::https::{BearerAuth, ClientError, HttpsClient, HttpsServer};
use trust_tasks::proof::affinidi::{SignOptions, Verifier};
use trust_tasks::proof::ProofExt;
use trust_tasks::specs::acl::grant::v0_1 as grant;
use trust_tasks::TrustTask;
```

### Identity

Both ends have to agree on who the sender is. The demo derives a `did:key` from
a fixed seed so there is no key exchange to get wrong; a real deployment loads a
real key.

<!-- snippet: trust-tasks/examples/acl_grant_roundtrip.rs#identity -->
```rust
/// A `did:key` derived from a fixed seed, so both ends agree on the sender's
/// identity with no key exchange. Real deployments load a real key here.
fn demo_identity() -> (Secret, String) {
    const SEED: [u8; 32] = [7u8; 32];
    let throwaway = Secret::generate_ed25519(None, Some(&SEED));
    let pk_mb = throwaway
        .get_public_keymultibase()
        .expect("ed25519 public multikey");
    let vm = format!("did:key:{pk_mb}#{pk_mb}");
    let mut secret = Secret::generate_ed25519(Some(&vm), Some(&SEED));
    secret.id = vm.clone();
    let did = vm.split('#').next().expect("did:key prefix").to_string();
    (secret, did)
}
```

### The receiver

<!-- snippet: trust-tasks/examples/acl_grant_roundtrip.rs#receiver -->
```rust
/// The consuming end: authenticate the peer, verify the proof, handle the task.
async fn receiver(alice_did: String) -> std::io::Result<()> {
    // Bearer token → Verifiable Identifier. This is what the SPEC §4.8.1
    // cross-check compares the document's in-band `issuer` against, so it must
    // be the DID the sender actually signs with.
    let auth = BearerAuth::from_pairs([("alice", alice_did.as_str())]);

    let server = HttpsServer::builder()
        .local_vid("did:web:maintainer.example")
        .with_auth(auth)
        // Without a verifier the server only checks a proof is *present*.
        // `for_did_key()` resolves `did:key` offline — no network, no DID doc.
        .with_verifier(Verifier::for_did_key())
        .on::<grant::Payload, grant::Response, _>(|req, ctx| {
            println!(
                "  [server] acl/grant from {} → subject={} role={}",
                ctx.authenticated_sender.as_deref().unwrap_or("<unauth>"),
                req.payload.entry.subject,
                &*req.payload.entry.role,
            );
            // Generated types are `#[non_exhaustive]`: build with `builder()`.
            Ok(grant::Response::builder()
                .entry(req.payload.entry.clone())
                .try_into()
                .expect("acl/grant response"))
        })
        .build();

    server.serve("127.0.0.1:3000").await?;
    Ok(())
}
```

### The sender

<!-- snippet: trust-tasks/examples/acl_grant_roundtrip.rs#sender -->
```rust
/// The producing end: build the document, sign it, send it, read the reply.
async fn sender(secret: &Secret, my_did: &str) -> Result<(), Box<dyn std::error::Error>> {
    // `my_vid` must be the *signing* identity: the server cross-checks the
    // in-band `issuer` against whatever its bearer token maps to (§4.8.1).
    let client = HttpsClient::builder()
        .server_url("http://127.0.0.1:3000")
        .server_vid("did:web:maintainer.example")
        .my_vid(my_did)
        .my_token("alice")
        .build()?;

    // Generated payload types are `#[non_exhaustive]` as of trust-tasks-rs
    // 0.14, so cross-crate construction goes through `X::builder()`. A member
    // added by a later revision of the spec leaves this call site alone.
    let entry: grant::AclEntry = grant::AclEntry::builder()
        .subject("did:web:carol.example")
        .role("moderator")
        .label("Carol — content moderation".to_string())
        .try_into()?;

    let mut request = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Payload::builder()
            .entry(entry)
            .reason("onboarding moderator".to_string())
            .try_into()?,
    );

    // TRAP: party members and `issuedAt` must be set BEFORE signing.
    // `HttpsClient::send` fills in any that are still unset — and a member
    // filled in after the fact is a member the proof does not cover, which the
    // server rejects as `proofInvalid`.
    request.issuer = Some(my_did.to_string());
    request.recipient = Some("did:web:maintainer.example".into());
    request.issued_at = Some(chrono::Utc::now());
    request.sign(secret, SignOptions::new()).await?;

    println!("  [client] POST /trust-tasks  id={}", request.id);

    match client.send::<grant::Payload>(request).await {
        Ok(resp) => println!(
            "  [client] ← {}\n           threadId={} entry: subject={} role={}",
            resp.type_uri,
            resp.thread_id.as_deref().unwrap_or("<none>"),
            resp.payload.entry.subject,
            &*resp.payload.entry.role,
        ),
        Err(ClientError::TrustTaskError { http_status, error }) => println!(
            "  [client] ← HTTP {} trust-task-error: code={} retryable={}",
            http_status, error.payload.code, error.payload.retryable,
        ),
        Err(other) => return Err(other.into()),
    }
    Ok(())
}
```

### What it prints

Real output, from the command above:

```text
  [server] listening on http://127.0.0.1:3000/trust-tasks (vid did:web:maintainer.example)
  [client] POST /trust-tasks  id=urn:uuid:335c52fd-3f48-4f06-8886-c469f57ef284
  [server] acl/grant from did:key:z6MkvDqGT54cXesYGvABpF1UapVNwjCqRcafi4Px6Thv5T3Z → subject=did:web:carol.example role=moderator
  [client] ← https://trusttasks.org/spec/acl/grant/0.1#response
           threadId=urn:uuid:335c52fd-3f48-4f06-8886-c469f57ef284 entry: subject=did:web:carol.example role=moderator
```

Note the `threadId` on the response: it echoes the request's `id`, which is how
a reply is correlated to what it answers.

---

## 3. The same round trip in TypeScript

The TypeScript package is `@openvtc/trust-tasks`. Producing a document is the
mirror of the Rust; the interesting half is **consuming** one, where
[`consumeInbound`](./trust-tasks-ts/README.md) runs the SPEC §7.2 pipeline
(items 2 and 4–8, the freshness bound, and item 11's duplicate-execution
record) and then calls your handler.

```ts
import {
  consequentialChecks,
  consumeInbound,
  InMemoryReplayGuard,
  respondWith,
  StaticTransport,
  AclGrant_v0_1,
} from "@openvtc/trust-tasks";

// The guard *is* the duplicate-execution record — one per consumer, held for
// the process's lifetime. Back it with a shared store if you run replicas.
const guard = new InMemoryReplayGuard();

const outcome = await consumeInbound<AclGrant_v0_1.Payload, AclGrant_v0_1.Response>({
  transport: new StaticTransport({ issuer: peerVid }), // what the transport authenticated
  spec: AclGrant_v0_1.SPEC,
  proofPolicy: { kind: "verify", verify: myVerifier },
  payloadPolicy: { kind: "validate", validate: myValidator },
  // acl/grant is consequential: a replayed envelope must not grant twice.
  checks: consequentialChecks(guard),
  doc,
  myVid: "did:web:maintainer.example",
  now: Date.now(),
  newErrorId: () => crypto.randomUUID(),
  handler: async (accepted, parties) =>
    respondWith(accepted, crypto.randomUUID(), await applyGrant(accepted.payload, parties)),
});

switch (outcome.kind) {
  case "handled":
    return send(outcome.response);
  case "rejected":
    return send(outcome.error); // already addressed per §8.1
  case "suppressed":
    // §8.1: an `identityMismatch` the transport cannot safely answer. Emitting
    // anything here would be an oracle. Log it — silent is the rule, invisible
    // is a footgun.
    return log(outcome.reason);
  case "duplicate":
    // §7.2 item 11: this document already executed. Return the prior result
    // where there is one, otherwise emit nothing.
    return outcome.priorResponse === undefined ? undefined : send(outcome.priorResponse);
  case "accepted":
    return undefined; // fire-and-forget: nothing to emit
}
```

`consequentialChecks(guard)` is correct for any task whose execution grants
access, moves value, discloses a secret, or is otherwise irreversible — SPEC
§7.2 item 11 makes duplicate-execution protection normative for those, and every
transport binding delegates it to the consumer. `notConsequentialChecks()` keeps
no record and is conformant only where repeated execution is safe and intended.

Items 1 and 3 (framework schema, unknown `type`) belong to your parse and
dispatch and have already succeeded by the time you hold a typed document. Full
detail is in [`trust-tasks-ts/README.md`](./trust-tasks-ts/README.md).

---

## 4. The four traps

Each of these is documented somewhere in the code. Collected here because each
one costs an afternoon the first time.

### 4.1 Set the party members and `issuedAt` *before* signing

```rust,ignore
request.issuer     = Some(my_did.to_string());
request.recipient  = Some("did:web:maintainer.example".into());
request.issued_at  = Some(chrono::Utc::now());
request.sign(&secret, SignOptions::new()).await?;   // ← signing comes LAST
```

`HttpsClient::send` fills in any party member and `issuedAt` still unset when
the document reaches it. That is a convenience for unsigned tasks and a trap for
signed ones: **a member filled in after signing is a member the proof does not
cover**, and the server rejects the document as `proofInvalid`. The failure
looks like a broken signature; it is a field that arrived late.

### 4.2 Sign *and* authenticate — one alone yields `identityMismatch`

The consuming server runs the SPEC §4.8.1 cross-check: the document's in-band
`issuer` must agree with the identity the transport authenticated. Two ways to
get this wrong:

- The bearer token maps to `did:web:alice.example`, but the client signs with a
  `did:key`. In-band and transport disagree → **`identityMismatch`**.
- The document is signed correctly, but the server has no verifier
  (`with_verifier` never called). It then only checks a proof is *present*, and
  a forged one passes.

In the example, `BearerAuth::from_pairs([("alice", alice_did)])` maps the token
to **the same DID the client signs with** — that agreement is the whole point of
the pairing. Note also that §8.1 makes some `identityMismatch` cases
*unanswerable*: a transport that cannot safely address a reply must stay silent
rather than act as an oracle, so a client can see a hang rather than an error.

### 4.3 Generated types are `#[non_exhaustive]` — construct via `builder()`

As of `trust-tasks-rs` **0.14.0**, every generated named-field struct and enum is
`#[non_exhaustive]`. Struct literals still compile *inside* `trust-tasks-rs`, so
this bites only cross-crate — which is to say, it bites you:

```rust,ignore
// won't compile from your crate:
let entry = grant::AclEntry { subject: "…".into(), role: "moderator".into(), .. };

// do this instead:
let entry: grant::AclEntry = grant::AclEntry::builder()
    .subject("did:web:carol.example")
    .role("moderator")
    .try_into()?;
```

`builder()` returns a builder whose `try_into()` validates and yields the type;
`X::default()` also works. This was taken as one deliberate break so that an
additive schema change — a new optional member, a new enum variant — is no
longer a source break. See the `#[non_exhaustive]` section of
[`CLAUDE.md`](./CLAUDE.md) for the full reasoning.

### 4.4 Eight crates, five version numbers — and the facade does not track them

`trust-tasks-rs` is 0.14, `-https` and `-didcomm` are 0.15, `-proof` is 0.13,
`-ceremony` is 0.2. They move independently and that is deliberate: each
binding versions over *its* API. The facade is `0.1.0` and does **not** chase
them — it pins compatible ranges so you do not have to.

Consequence worth knowing: a leading-component bump in `trust-tasks-rs` is a
workspace event that moves six dependent crates with it (see
[`CLAUDE.md`](./CLAUDE.md)). If you pin the individual crates yourself, pin them
as a set.

---

## 5. Where to go next

| | |
|---|---|
| The normative framework | [`SPEC.md`](./SPEC.md) — §4.8.1 identity precedence, §7.2 the consumer pipeline, §8 errors |
| The task registry | <https://trusttasks.org/> — every published spec and its schema |
| Transport bindings | [`bindings/`](./bindings/) — the wire format for https, didcomm, didcomm-v1, tsp |
| Writing a spec | [`CONTRIBUTING-SPECS.md`](./CONTRIBUTING-SPECS.md) |
| Rust API docs | <https://docs.rs/trust-tasks> (facade) · <https://docs.rs/trust-tasks-rs> (core) |
| More Rust examples | [`trust-tasks-https/examples/`](./trust-tasks-https/examples/) — the two-process version of the round trip above |
