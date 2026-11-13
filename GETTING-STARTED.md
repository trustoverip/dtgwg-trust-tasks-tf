# Getting started with Trust Tasks

A [Trust Task](https://trusttasks.org/) is a self-contained, transport-agnostic
JSON document describing verifiable work between two parties. This page takes
you from nothing to a **signed `acl/grant` round trip** — sender and receiver
in one process, in Rust, TypeScript, Go and Dart, each a runnable example —
and then names the four things that reliably cost an afternoon.

If you are writing a *specification* rather than consuming one, you want
[`CONTRIBUTING-SPECS.md`](./CONTRIBUTING-SPECS.md) and [`SPEC.md`](./SPEC.md)
instead. This page is for consumers.

> **You generate the tasks you use.** Every library still ships a prebuilt
> module for every registry specification, but those are deprecated and will be
> removed in a future release. This page pins the one task it uses,
> `acl/grant/0.3`, and generates it. The framework's own specifications
> (`trust-task-*`) come with the runtime in every language.

---

## 1. Pin the tasks you use, then one dependency line

Trust Task specifications are versioned documents published at trusttasks.org.
An application names the versions it speaks, and generates their types:

```sh
trust-tasks add acl/grant/0.3      # writes trust-tasks.toml, trust-tasks.lock, trust-tasks/
```

`trust-tasks` is one command for every language. You do **not** need a Rust
toolchain to get it — see [Installing the `trust-tasks` command](#installing-the-trust-tasks-command)
below.

Commit all three. The lock pins each version by digest and the bundles under
`trust-tasks/` are verified against it; `trust-tasks check` does that offline,
so run it in CI, and `trust-tasks update` moves pins, saying whether a move needs
code changes. The same three files drive every language.

```sh
cargo add trust-tasks --features https,proof-affinidi
cargo add --build trust-tasks-build
```

That writes the current versions into `Cargo.toml`:

```toml
[dependencies]
trust-tasks = { version = "…", features = ["https", "proof-affinidi"] }

[build-dependencies]
trust-tasks-build = "…"
```

```rust
// build.rs — checks each bundle's digest, then generates the pinned tasks
fn main() -> Result<(), trust_tasks_build::Error> {
    trust_tasks_build::Config::new()
        .runtime_path("::trust_tasks") // the generated code reaches the runtime through the facade
        .compile()?;
    Ok(())
}
```

```sh
npm install @openvtc/trust-tasks && npm install -D @openvtc/trust-tasks-gen
trust-tasks generate --lang ts --out src/trust-tasks
```

```sh
go get github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go
trust-tasks generate --lang go --out internal/tasks
```

```sh
dart pub add trust_tasks
trust-tasks generate --lang dart --out lib/trust_tasks
```

Leave out `all-specs`: it is every prebuilt module (~90 MB of generated Rust,
minutes and over 16 GB of memory for a cold build, and deprecated), and it is
off by default (on by default in 0.38 and earlier, where the line above needs
`default-features = false`). Generating three tasks instead builds in about 8
seconds and 0.4 GB. `trust-tasks generate` verifies the lock, then runs each
language's own generator — it installs nothing, and Rust needs no command at
all because `build.rs` is the generator.

**Dispatching by Type URI?** If your consumer looks a received document's
schema or policy up by its `type` string (`trust_tasks_rs::schema_index`) —
a validating gate in front of the handlers, say — add `.index(true)` to the
`Config` in `build.rs` and use the generated `tasks::schema_index` instead.
The runtime's index only knows the prebuilt families, so without it every task
you generated answers `None`, and a gate that reads `None` as "skip
validation" fails open. See
[`trust-tasks-build`](./trust-tasks-build/README.md#the-type-uri-index--indextrue).

`trust-tasks` is a facade: it re-exports the framework's crates behind Cargo
features so you pick a transport rather than a set of version numbers.
Everything in it is a `pub use` — `trust_tasks::TrustTask` **is**
`trust_tasks_rs::TrustTask` — so dropping the facade later is a
find-and-replace, not a migration: depend on `trust-tasks-rs` and leave out
`.runtime_path`.

### The crate matrix

Pick one row per thing you want to do; features combine.

| I want to… | facade feature | what you write | underlying crate |
|---|---|---|---|
| model a document, run the SPEC §7.2 consumer checks | *(always on)* | `trust_tasks::{TrustTask, consume_inbound, RejectReason}`, and the tasks `build.rs` generated | `trust-tasks-rs` |
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
specification declares `proof` REQUIRED — `acl/grant/0.3` is one — cannot be
produced or consumed without a signer and a verifier.

**To turn off a transport crate's own defaults** (`trust-tasks-https`'s
`server`, `trust-tasks-didcomm-v1`'s `legacy-basic-message`), depend on that
crate directly rather than through the facade. That is the supported answer, not
a workaround — you have outgrown the front door.

An ecosystem's own tasks, published outside trusttasks.org, are pinned with
`[sources.<name>]` in `trust-tasks.toml` and generated beside the registry's.
See [`trust-tasks-cli`](./trust-tasks-cli/README.md),
[`trust-tasks-build`](./trust-tasks-build/README.md) and
[consumer-side generation](./docs/design/consumer-side-generation.md).

### Installing the `trust-tasks` command

Pick one; each puts the same `trust-tasks` binary on your `PATH`.

- **Download a prebuilt binary** (no Rust needed). Every
  [`trust-tasks-cli-v*` release](https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases?q=trust-tasks-cli&expanded=true)
  carries `trust-tasks-cli-<version>-<target>.tar.gz` (`.zip` for Windows) for
  `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (glibc 2.35 or
  newer), `x86_64-apple-darwin`, `aarch64-apple-darwin` and
  `x86_64-pc-windows-msvc`, each with a `.sha256` file and a combined
  `SHA256SUMS`. The archive holds the binary alone:

  ```sh
  v=X.Y.Z t=aarch64-apple-darwin   # set: the CLI version, your platform
  curl -fsSLO "https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/download/trust-tasks-cli-v$v/trust-tasks-cli-$v-$t.tar.gz"
  curl -fsSL "https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/download/trust-tasks-cli-v$v/trust-tasks-cli-$v-$t.tar.gz.sha256" | shasum -a 256 -c
  tar -xzf "trust-tasks-cli-$v-$t.tar.gz" && sudo mv trust-tasks /usr/local/bin/
  ```

  On macOS a binary downloaded by a browser is quarantined; one fetched with
  `curl` as above is not.
- **[`cargo binstall`](https://github.com/cargo-bins/cargo-binstall)
  `trust-tasks-cli`** — fetches the same prebuilt archive, if you already have
  cargo but not the time to compile.
- **`cargo install trust-tasks-cli`** — builds from source; needs Rust 1.95 or
  later.

Releases from before the binaries were published (`0.2.0` and earlier) may
carry none, and `cargo binstall` builds those from source.

### What you bring yourself

The runtime in each language runs the SPEC §7.2 pipeline, but leaves three
pieces pluggable: the JSON Schema **validator** for item 2, the proof
**verifier** for item 6 (and a signer to produce), and the **transport**. This
is where each comes from:

| | Validator | Proof verifier / signer | Transport |
|---|---|---|---|
| **Rust** | built in: `SchemaValidator`, the `validate` feature | [`trust-tasks-proof`](./trust-tasks-proof/) (`proof-affinidi` on the facade) | [`trust-tasks-https`](./trust-tasks-https/), [`-didcomm`](./trust-tasks-didcomm/), [`-didcomm-v1`](./trust-tasks-didcomm-v1/), [`-tsp`](./trust-tasks-tsp/) |
| **TypeScript** | yours: a function over [`ajv`](https://www.npmjs.com/package/ajv) (Draft 2020-12) or any engine | [`@openvtc/trust-tasks-proof`](./trust-tasks-ts-proof/) | [`@openvtc/trust-tasks-tsp`](./trust-tasks-ts-tsp/); HTTPS and DIDComm are yours (e.g. `fetch` with the [HTTPS binding](./bindings/https/)'s wire format) |
| **Go** | yours: any JSON Schema engine behind `PayloadValidator` | [`trust-tasks-go/proof`](./trust-tasks-go/proof/) | [`trust-tasks-go/didcomm`](./trust-tasks-go/didcomm/), [`trust-tasks-go/tsp`](./trust-tasks-go/tsp/) (not yet tagged); HTTPS is yours (`net/http`) |
| **Dart** | yours: e.g. [`json_schema`](https://pub.dev/packages/json_schema) behind `PayloadValidator` | [`trust_tasks_proof`](./trust-tasks-dart-proof/) | [`trust_tasks_https`](./trust-tasks-dart-https/), [`trust_tasks_didcomm`](./trust-tasks-dart-didcomm/), [`trust_tasks_tsp`](./trust-tasks-dart-tsp/) (not yet on pub.dev) |

The cores bundle no validator on purpose, to stay dependency-light;
`{ kind: "acceptUnvalidated" }` (and its Go and Dart equivalents) says so
explicitly instead. The examples in §2–§3b wire up all three in each language;
their validators are a dependency-free stand-in for the engine you would use.

---

## 2. A signed `acl/grant` round trip, in Rust

Every Rust block below is **extracted from a runnable example application**,
[`examples/getting-started`](./examples/getting-started): the `Cargo.toml`,
`build.rs` and `trust-tasks.lock` of §1, and
[`src/main.rs`](./examples/getting-started/src/main.rs). A test
(`trust-tasks/tests/getting_started_snippets.rs`) fails the build if this page
and that file drift, so what you copy is code that compiles today.

```sh
cargo run -p trust-tasks-example-getting-started
```

### Imports

Everything comes through the facade — one crate name in every path — except
the task itself, which `build.rs` generated into `tasks`. `tasks::acl::grant::v0_3`
is the path the prebuilt `trust_tasks::specs` module gave the same version, so
moving code over is a change to its `use` line.

<!-- snippet: examples/getting-started/src/main.rs#imports -->
```rust
use affinidi_secrets_resolver::secrets::Secret;
use trust_tasks::https::{BearerAuth, ClientError, HttpsClient, HttpsServer};
use trust_tasks::proof::affinidi::{SignOptions, Verifier};
use trust_tasks::proof::ProofExt;
use trust_tasks::TrustTask;

// The tasks trust-tasks.lock pins, generated by build.rs.
mod tasks {
    include!(concat!(env!("OUT_DIR"), "/trust_tasks_generated.rs"));
}
use tasks::acl::grant::v0_3 as grant;
```

### Identity

Both ends have to agree on who the sender is. The demo derives a `did:key` from
a fixed seed so there is no key exchange to get wrong; a real deployment loads a
real key.

<!-- snippet: examples/getting-started/src/main.rs#identity -->
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

<!-- snippet: examples/getting-started/src/main.rs#receiver -->
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
        .on::<grant::Payload, _>(|req, ctx| {
            println!(
                "  [server] acl/grant from {} → subject={} role={}",
                ctx.authenticated_sender.as_deref().unwrap_or("<unauth>"),
                req.payload.entry.subject.as_str(),
                req.payload.entry.role.as_str(),
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

<!-- snippet: examples/getting-started/src/main.rs#sender -->
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
    // A free-text member declaring a `maxLength` (SPEC §7.3) generates a
    // validating newtype rather than a bare `String`, so the bound is checked
    // where the value is authored instead of at the far end of the wire.
    //
    // acl/grant/0.3 states every authority axis explicitly — there is no
    // default and no implicit form. Carol may act everywhere, may use no
    // keys, holds the full ceiling of the `moderator` role, and may use what
    // she reads for any purpose (`any` never includes `modelTraining`).
    let act = grant::AuthorityScopeAll::try_from(grant::AuthorityScopeAll::builder().scope("all"))?;
    let keys = grant::KeyScopeNone::try_from(grant::KeyScopeNone::builder().scope("none"))?;
    let capabilities = grant::CapabilityScopeCeiling::try_from(
        grant::CapabilityScopeCeiling::builder().scope("ceiling"),
    )?;
    let purposes =
        grant::PurposeScopeAny::try_from(grant::PurposeScopeAny::builder().scope("any"))?;
    let entry: grant::AclEntry = grant::AclEntry::builder()
        .subject("did:web:carol.example")
        .role("moderator")
        .act(act)
        .keys(keys)
        .capabilities(capabilities)
        .purposes(purposes)
        .label(grant::AclEntryLabel::try_from(
            "Carol — content moderation",
        )?)
        .try_into()?;

    let mut request = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Payload::builder()
            .entry(entry)
            .reason(grant::PayloadReason::try_from("onboarding moderator")?)
            .try_into()?,
    );

    // TRAP: party members and `issuedAt` must be set BEFORE signing.
    // `HttpsClient::send` fills in any that are still unset — and a member
    // filled in after the fact is a member the proof does not cover, which the
    // server rejects as `proofInvalid`.
    request.issuer = Some(my_did.to_string());
    request.recipient = Some("did:web:maintainer.example".into());
    request.issued_at = Some(trust_tasks::issued_at_now()); // whole seconds, UTC
    request.sign(secret, SignOptions::new()).await?;

    println!("  [client] POST /trust-tasks  id={}", request.id);

    match client.send::<grant::Payload>(request).await {
        Ok(resp) => println!(
            "  [client] ← {}\n           threadId={} entry: subject={} role={}",
            resp.type_uri,
            resp.thread_id.as_deref().unwrap_or("<none>"),
            resp.payload.entry.subject.as_str(),
            resp.payload.entry.role.as_str(),
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
  [client] POST /trust-tasks  id=urn:uuid:e7df506b-7994-42ba-94d4-3ff4dda46bd0
  [server] acl/grant from did:key:z6MkvDqGT54cXesYGvABpF1UapVNwjCqRcafi4Px6Thv5T3Z → subject=did:web:carol.example role=moderator
  [client] ← https://trusttasks.org/spec/acl/grant/0.3#response
           threadId=urn:uuid:e7df506b-7994-42ba-94d4-3ff4dda46bd0 entry: subject=did:web:carol.example role=moderator
```

Note the `threadId` on the response: it echoes the request's `id`, which is how
a reply is correlated to what it answers.

---

## 3. The same round trip in TypeScript

The TypeScript package is `@openvtc/trust-tasks`; import its runtime from
`@openvtc/trust-tasks/runtime`, and the task from the modules
`trust-tasks generate --lang ts` wrote (§1). The interesting half is
**consuming** a document, where
[`consumeInbound`](./trust-tasks-ts/README.md) runs the SPEC §7.2 pipeline
(items 2–8, item 11's duplicate-execution record and item 13's freshness
bound) and then calls your handler.

Every TypeScript block below is **extracted from a runnable example
application**, [`examples/getting-started-ts`](./examples/getting-started-ts):
the same `trust-tasks.toml`, `trust-tasks.lock` and vendored bundle as the Rust
one, and [`src/main.ts`](./examples/getting-started-ts/src/main.ts).
`scripts/check-getting-started-snippets.mjs` fails CI if this page and that
file drift, and CI generates, builds and runs the example on every change.

```sh
cd examples/getting-started-ts
npm install && npm run all        # generate from the lock, build, run
```

It prints the same round trip as the Rust example — the same `did:key`, since
both derive it from the same seed — then checks that a retry is absorbed, a
tampered copy is refused `proofInvalid`, and a payload missing a REQUIRED
member is refused `malformedRequest`.

### Imports

The proof backend is [`@openvtc/trust-tasks-proof`](./trust-tasks-ts-proof/).
There is no TypeScript HTTPS package, so the receiver is `node:http` speaking
the [HTTPS binding](./bindings/https/)'s wire format, and the sender is
`fetch`. The validator is a stand-in so the example needs no dependency:
[`src/validator.ts`](./examples/getting-started-ts/src/validator.ts) checks
the structural subset of JSON Schema the generated schemas use. A real
application passes a full Draft 2020-12 engine — `ajv` is the usual one.

<!-- snippet: examples/getting-started-ts/src/main.ts#imports -->
```ts
import { randomUUID } from "node:crypto";
import { createServer, type Server } from "node:http";

import {
  consequentialChecks,
  consumeInbound,
  InMemoryReplayGuard,
  issuedAtNow,
  respondWith,
  StaticTransport,
  type TrustTaskDocument,
} from "@openvtc/trust-tasks/runtime";
import { DataIntegrityProofVerifier, signerFromPrivateKey, signTrustTask, type Signer } from "@openvtc/trust-tasks-proof";

// The tasks trust-tasks.lock pins, generated by `npm run generate`.
import { AclGrant_v0_3 as grant, declareExtends } from "./trust-tasks/index.js";
// Stands in for a JSON Schema engine such as ajv — see src/validator.ts.
import { structuralValidator } from "./validator.js";
```

### Identity

<!-- snippet: examples/getting-started-ts/src/main.ts#identity -->
```ts
/**
 * A `did:key` derived from a fixed seed, so both ends agree on the sender's
 * identity with no key exchange. Real deployments load a real key here.
 */
function demoIdentity(): Signer {
  return signerFromPrivateKey("ed25519", new Uint8Array(32).fill(7));
}
```

### The receiver

<!-- snippet: examples/getting-started-ts/src/main.ts#receiver -->
```ts
/** The binding's HTTP status for a framework error code (bindings/https §4). */
function statusFor(code: string): number {
  const status: Record<string, number> = { malformedRequest: 400, permissionDenied: 403, idConflict: 409 };
  return status[code] ?? (code === "unavailable" ? 503 : code === "internalError" ? 500 : 422);
}

/** The consuming end: authenticate the peer, run the §7.2 pipeline, handle the task. */
function receiver(aliceDid: string): Server {
  // Bearer token → Verifiable Identifier. This is what the SPEC §4.8.1
  // cross-check compares the document's in-band `issuer` against, so it must
  // be the DID the sender actually signs with.
  const tokens = new Map([["alice", aliceDid]]);

  // The guard *is* the duplicate-execution record — one per consumer, held for
  // the process's lifetime. Back it with a shared store if you run replicas.
  const guard = new InMemoryReplayGuard();
  // `did:key` only — no network, no DID document.
  const verifier = new DataIntegrityProofVerifier();

  return createServer(async (req, res) => {
    const reply = (status: number, body?: unknown) => {
      res.writeHead(status, body === undefined ? {} : { "content-type": "application/json" });
      res.end(body === undefined ? undefined : JSON.stringify(body));
    };
    if (req.url !== "/trust-tasks") return reply(404);
    if (req.method !== "POST") return reply(405);

    const token = /^Bearer (.+)$/.exec(req.headers.authorization ?? "")?.[1];
    const peerVid = token === undefined ? undefined : tokens.get(token);
    let doc: TrustTaskDocument<grant.Payload>;
    try {
      const chunks: Buffer[] = [];
      for await (const chunk of req) chunks.push(chunk as Buffer);
      doc = JSON.parse(Buffer.concat(chunks).toString("utf8"));
    } catch {
      return reply(400);
    }

    const outcome = await consumeInbound<grant.Payload, grant.Response>({
      transport: new StaticTransport({ issuer: peerVid }), // what the transport authenticated
      spec: grant.SPEC,
      proofPolicy: { kind: "verify", verify: verifier },
      payloadPolicy: { kind: "validate", validate: structuralValidator },
      // acl/grant is consequential: a replayed envelope must not grant twice.
      checks: consequentialChecks(guard),
      doc,
      myVid: SERVER_VID,
      now: Date.now(),
      newErrorId: () => `urn:uuid:${randomUUID()}`,
      handler: (accepted, parties) => {
        const { subject, role } = accepted.payload.entry;
        console.log(`  [server] acl/grant from ${parties.issuer} → subject=${subject} role=${role}`);
        return respondWith<grant.Payload, grant.Response>(accepted, `urn:uuid:${randomUUID()}`, {
          entry: accepted.payload.entry,
        });
      },
    });

    switch (outcome.kind) {
      case "handled":
        return reply(200, outcome.response);
      case "rejected":
        return reply(statusFor(outcome.error.payload.code), outcome.error); // already addressed per §8.1
      case "suppressed":
        // §8.1: an `identityMismatch` the transport cannot safely answer. HTTP
        // must answer something, so answer as for an unreadable body, and log
        // it — silent is the rule, invisible is a footgun.
        console.log(`  [server] suppressed: ${outcome.reason.code}`);
        return reply(400);
      case "duplicate": {
        // §7.2 item 11: this document already executed. Return the prior
        // result where there is one (bindings/https §4: else 202 or 204).
        const prior = outcome.priorResponse as TrustTaskDocument<{ code?: string }> | undefined;
        if (prior === undefined) return reply(outcome.inFlight ? 202 : 204);
        const isError = !prior.type.endsWith("#response");
        return reply(isError ? statusFor(prior.payload.code ?? "") : 200, prior);
      }
      case "accepted":
        return reply(204); // fire-and-forget: nothing to emit
    }
  });
}
```

A consumer may keep its record for less time than a producer's `expiresAt`
asks, provided it stops accepting the document when it stops remembering it
(SPEC §7.2, *Bounding the record*): the consequential policies in all four
libraries honour an `expiresAt` up to 30 days after `issuedAt` and treat the
document as expiring there. The in-memory guard refuses new documents
`unavailable` when full of live records, and lets no single issuer hold more
than a tenth of it; size it for your retention window, or use a shared store.

`consequentialChecks(guard)` is correct for any task whose execution grants
access, moves value, discloses a secret, or is otherwise irreversible — SPEC
§7.2 item 11 makes duplicate-execution protection normative for those, and every
transport binding delegates it to the consumer. `notConsequentialChecks()` keeps
no record and is conformant only where repeated execution is safe and intended.

Item 1 (framework schema) is checked for you: `consumeInbound` refuses a parsed
value that is not a well-formed document envelope before anything else. Item 3
is too: a document whose `type` is not the `spec` it is consumed under (or a
later version extending it) is refused `unsupportedType` / `unsupportedVersion`
before the handler runs. Full detail is in
[`trust-tasks-ts/README.md`](./trust-tasks-ts/README.md).

### The sender

<!-- snippet: examples/getting-started-ts/src/main.ts#sender -->
```ts
/** The producing end: build the document, sign it, send it, read the reply. */
async function sender(signer: Signer, serverUrl: string): Promise<TrustTaskDocument<grant.Payload>> {
  // acl/grant/0.3 states every authority axis explicitly — there is no default
  // and no implicit form. Carol may act everywhere, may use no keys, holds the
  // full ceiling of the `moderator` role, and may use what she reads for any
  // purpose (`any` never includes `modelTraining`).
  const payload: grant.Payload = {
    entry: {
      subject: "did:web:carol.example",
      role: "moderator",
      act: { scope: "all" },
      keys: { scope: "none" },
      capabilities: { scope: "ceiling" },
      purposes: { scope: "any" },
      label: "Carol — content moderation",
    },
    reason: "onboarding moderator",
  };

  // TRAP: party members and `issuedAt` must be set BEFORE signing — a member
  // added after the fact is one the proof does not cover, and the receiver
  // refuses the document as `proofInvalid`.
  const request = signTrustTask<grant.Payload>(
    {
      id: `urn:uuid:${randomUUID()}`,
      type: grant.TYPE_URI,
      issuer: signer.did, // the *signing* identity, which the bearer token maps to
      recipient: SERVER_VID,
      issuedAt: issuedAtNow(), // whole seconds, UTC
      payload,
    },
    signer,
  );

  console.log(`  [client] POST /trust-tasks  id=${request.id}`);
  const res = await fetch(`${serverUrl}/trust-tasks`, {
    method: "POST",
    headers: { "content-type": "application/json", authorization: "Bearer alice" },
    body: JSON.stringify(request),
  });
  const body = await res.json();
  if (res.ok) {
    const resp = body as TrustTaskDocument<grant.Response>;
    console.log(
      `  [client] ← ${resp.type}\n           threadId=${resp.threadId} ` +
        `entry: subject=${resp.payload.entry.subject} role=${resp.payload.entry.role}`,
    );
  } else {
    const error = body as TrustTaskDocument<{ code: string; retryable: boolean }>;
    console.log(
      `  [client] ← HTTP ${res.status} trust-task-error: code=${error.payload.code} retryable=${error.payload.retryable}`,
    );
  }
  return request;
}
```

`signTrustTask` refuses a document whose `issuer` is not the signer's `did:key`,
so the most common signing mistake fails where it is made rather than at the
far end as `proofInvalid`. `issuedAtNow()` stamps whole seconds, UTC — never
`toISOString()` (§4.1).

---

## 3a. The same round trip in Go

The Go module is
`github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go`. It mirrors the
TypeScript shape — generated types plus a hand-written §7.2 pipeline — with the
differences Go's type system forces. The task is the package
`trust-tasks generate --lang go --out internal/tasks` wrote into your own
module (§1).

Every Go block below is **extracted from a runnable example application**,
[`examples/getting-started-go`](./examples/getting-started-go): the same lock
and bundle, a `//go:generate` line that runs the generator, the generated
package committed under `internal/tasks`, and
[`main.go`](./examples/getting-started-go/main.go). The same check holds this
page to it, and CI regenerates, vets and runs it on the module's Go floor and
on the latest Go.

```sh
cd examples/getting-started-go
go generate ./... && go run .
```

### Imports

The proof backend is the [`trust-tasks-go/proof`](./trust-tasks-go/proof/)
module (standard library only). There is no Go HTTPS package, so the receiver
is a `net/http` handler speaking the [HTTPS binding](./bindings/https/)'s wire
format. The validator is a stand-in, as in TypeScript —
[`validator.go`](./examples/getting-started-go/validator.go) — where a real
application puts a full JSON Schema engine behind `PayloadValidator`.

<!-- snippet: examples/getting-started-go/main.go#imports -->
```go
import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"strings"
	"time"

	"github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof"
	tt "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"

	// The task trust-tasks.lock pins, generated by `go generate`.
	"example.com/trust-tasks-getting-started-go/internal/tasks"
	grant "example.com/trust-tasks-getting-started-go/internal/tasks/acl/grant/v0_3"
)
```

### Identity

<!-- snippet: examples/getting-started-go/main.go#identity -->
```go
// demoIdentity is a did:key derived from a fixed seed, so both ends agree on
// the sender's identity with no key exchange. Real deployments load a real key.
func demoIdentity() *proof.Signer {
	seed := bytes.Repeat([]byte{7}, 32)
	signer, err := proof.SignerFromPrivateKey(proof.Ed25519, seed)
	if err != nil {
		panic(err)
	}
	return signer
}

// newID is a urn:uuid (version 4), from crypto/rand.
func newID() string {
	var b [16]byte
	_, _ = rand.Read(b[:])
	b[6], b[8] = b[6]&0x0f|0x40, b[8]&0x3f|0x80
	return fmt.Sprintf("urn:uuid:%x-%x-%x-%x-%x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:])
}
```

### The receiver

<!-- snippet: examples/getting-started-go/main.go#receiver -->
```go
// statusFor is the binding's HTTP status for a framework error code
// (bindings/https §4).
func statusFor(code string) int {
	switch code {
	case "malformedRequest":
		return http.StatusBadRequest
	case "permissionDenied":
		return http.StatusForbidden
	case "idConflict":
		return http.StatusConflict
	case "unavailable":
		return http.StatusServiceUnavailable
	case "internalError":
		return http.StatusInternalServerError
	}
	return http.StatusUnprocessableEntity
}

// receiver is the consuming end: authenticate the peer, run the §7.2
// pipeline, handle the task.
func receiver(aliceDID string) http.Handler {
	// Bearer token → Verifiable Identifier. This is what the SPEC §4.8.1
	// cross-check compares the document's in-band issuer against, so it must be
	// the DID the sender actually signs with.
	tokens := map[string]string{"alice": aliceDID}

	// The guard IS the duplicate-execution record — one per consumer, held for
	// the process's lifetime. Back it with a shared store if you run replicas.
	guard := tt.NewInMemoryReplayGuard(0)
	// did:key only — no network, no DID document.
	verifier := proof.NewVerifier()

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		reply := func(status int, body any) {
			if body != nil {
				w.Header().Set("Content-Type", "application/json")
			}
			w.WriteHeader(status)
			if body != nil {
				_ = json.NewEncoder(w).Encode(body)
			}
		}
		if r.URL.Path != "/trust-tasks" {
			reply(http.StatusNotFound, nil)
			return
		}
		if r.Method != http.MethodPost {
			reply(http.StatusMethodNotAllowed, nil)
			return
		}

		var transport tt.TransportContext // what the transport authenticated
		if token, ok := strings.CutPrefix(r.Header.Get("Authorization"), "Bearer "); ok {
			if vid, known := tokens[token]; known {
				transport.Issuer = &vid
			}
		}
		received, err := io.ReadAll(io.LimitReader(r.Body, 1<<20))
		var doc tt.Document[grant.Payload]
		if err != nil || json.Unmarshal(received, &doc) != nil {
			reply(http.StatusBadRequest, nil)
			return
		}

		outcome, err := tt.ConsumeInbound(r.Context(), tt.ConsumeOptions[grant.Payload, grant.Response]{
			Transport:     tt.StaticTransport{Context: transport},
			Spec:          grant.Spec,
			ProofPolicy:   tt.ProofPolicy{Kind: tt.ProofVerify, Verifier: verifier},
			PayloadPolicy: tt.PayloadPolicy{Kind: tt.PayloadValidate, Validator: structuralValidator{}},
			// acl/grant is consequential: a replayed envelope must not grant twice.
			Checks:     tt.ConsequentialChecks(guard),
			Doc:        &doc,
			Received:   received, // verify the proof over the bytes as they arrived
			Extends:    tasks.ExtendsOf,
			MyVID:      serverVID,
			Now:        time.Now(),
			NewErrorID: newID,
			Handler: func(
				_ context.Context,
				accepted *tt.Document[grant.Payload],
				parties tt.ResolvedParties,
			) (*tt.Document[grant.Response], error) {
				entry := accepted.Payload.Entry
				fmt.Printf("  [server] acl/grant from %s → subject=%s role=%s\n", *parties.Issuer, entry.Subject, entry.Role)
				return tt.RespondWith[grant.Payload, grant.Response](
					accepted, newID(), grant.Response{Entry: entry}, nil), nil
			},
		})
		if err != nil { // a caller mistake or a handler error — never a framework rejection
			reply(http.StatusInternalServerError, nil)
			return
		}

		switch outcome.Kind {
		case tt.OutcomeHandled:
			reply(http.StatusOK, outcome.Response)
		case tt.OutcomeRejected:
			reply(statusFor(outcome.Error.Payload.Code), outcome.Error) // already addressed per §8.1
		case tt.OutcomeSuppressed:
			// §8.1: an identityMismatch the transport cannot safely answer. HTTP
			// must answer something, so answer as for an unreadable body, and log
			// it — silent is the rule, invisible is a footgun.
			fmt.Printf("  [server] suppressed: %s\n", outcome.Reason.Code)
			reply(http.StatusBadRequest, nil)
		case tt.OutcomeDuplicate:
			// §7.2 item 11: this document already executed. Return the prior
			// result where there is one (bindings/https §4: else 202 or 204).
			var prior tt.ErrorResponse
			switch {
			case outcome.PriorResponse == nil && outcome.InFlight:
				reply(http.StatusAccepted, nil)
			case outcome.PriorResponse == nil:
				reply(http.StatusNoContent, nil)
			case json.Unmarshal(outcome.PriorResponse, &prior) == nil && !strings.HasSuffix(prior.Type, "#response"):
				reply(statusFor(prior.Payload.Code), outcome.PriorResponse)
			default:
				reply(http.StatusOK, outcome.PriorResponse)
			}
		case tt.OutcomeAccepted:
			reply(http.StatusNoContent, nil) // fire-and-forget: nothing to emit
		}
	})
}
```

`err` is non-nil only for a caller mistake or an error your handler returned —
every *framework* rejection is an `outcome`, switched on as above.

Three Go-specific things worth knowing before you write the handler:

- **`encoding/json` enforces less than it looks like it does.** It ignores
  unknown members and does not require a single REQUIRED one, so a payload
  missing everything unmarshals cleanly. Pass a real `PayloadValidator`; the
  schema ships as `grant.PayloadSchemaJSON`.
- **Optional members are pointers, including slices** — `*[]string`. That is how
  "present and empty" stays distinguishable from "absent", which several specs
  make load-bearing.
- **`ProofVerifier` takes the document as JSON**, not as a `Document[P]`: a Go
  interface cannot carry a method that is generic over the payload, and a Data
  Integrity verifier works over the serialization anyway. Pass the bytes as
  `Received`, as above, so the proof is checked over what was signed rather
  than a re-encoding that dropped the members the type does not model.

### The sender

<!-- snippet: examples/getting-started-go/main.go#sender -->
```go
// sender is the producing end: build the document, sign it, send it, read the
// reply. It returns the signed document it sent.
func sender(signer *proof.Signer, serverURL string) (json.RawMessage, error) {
	// acl/grant/0.3 states every authority axis explicitly — there is no
	// default and no implicit form. Carol may act everywhere, may use no keys,
	// holds the full ceiling of the `moderator` role, and may use what she
	// reads for any purpose (`any` never includes `modelTraining`). The scope
	// unions are oneOf in the schema, so they generate as json.RawMessage.
	label, reason := "Carol — content moderation", "onboarding moderator"
	payload := grant.Payload{
		Entry: grant.ACLEntry{
			Subject:      "did:web:carol.example",
			Role:         "moderator",
			Act:          json.RawMessage(`{"scope":"all"}`),
			Keys:         json.RawMessage(`{"scope":"none"}`),
			Capabilities: json.RawMessage(`{"scope":"ceiling"}`),
			Purposes:     json.RawMessage(`{"scope":"any"}`),
			Label:        &label,
		},
		Reason: &reason,
	}

	// TRAP: party members and issuedAt must be set BEFORE signing — a member
	// added after the fact is one the proof does not cover, and the receiver
	// refuses the document as proofInvalid.
	recipient, issuedAt := serverVID, tt.SystemClock() // whole seconds, UTC
	unsigned, err := json.Marshal(tt.Document[grant.Payload]{
		ID:        newID(),
		Type:      grant.TypeURI,
		Issuer:    &signer.DID, // the *signing* identity, which the bearer token maps to
		Recipient: &recipient,
		IssuedAt:  &issuedAt,
		Payload:   payload,
	})
	if err != nil {
		return nil, err
	}
	request, err := signer.Sign(unsigned, proof.SignOptions{})
	if err != nil {
		return nil, err
	}

	var sent tt.Document[grant.Payload]
	_ = json.Unmarshal(request, &sent)
	fmt.Printf("  [client] POST /trust-tasks  id=%s\n", sent.ID)

	req, _ := http.NewRequest(http.MethodPost, serverURL+"/trust-tasks", bytes.NewReader(request))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer alice")
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()
	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}

	if res.StatusCode == http.StatusOK {
		var resp tt.Document[grant.Response]
		if err := json.Unmarshal(body, &resp); err != nil {
			return nil, err
		}
		fmt.Printf("  [client] ← %s\n           threadId=%s entry: subject=%s role=%s\n",
			resp.Type, *resp.ThreadID, resp.Payload.Entry.Subject, resp.Payload.Entry.Role)
	} else {
		var refusal tt.ErrorResponse
		if err := json.Unmarshal(body, &refusal); err != nil {
			return nil, fmt.Errorf("HTTP %d", res.StatusCode)
		}
		fmt.Printf("  [client] ← HTTP %d trust-task-error: code=%s retryable=%t\n",
			res.StatusCode, refusal.Payload.Code, refusal.Payload.Retryable)
	}
	return request, nil
}
```

Full detail is in [`trust-tasks-go/README.md`](./trust-tasks-go/README.md) and
[`trust-tasks-go/proof/README.md`](./trust-tasks-go/proof/README.md).

---

## 3b. The same round trip in Dart

The Dart package is `trust_tasks`, and its library is the runtime. The task is
the library `trust-tasks generate --lang dart --out lib/trust_tasks` wrote into
your own package (§1), imported with a prefix.

Every Dart block below is **extracted from a runnable example application**,
[`examples/getting-started-dart`](./examples/getting-started-dart): the same
lock and bundle, and [`bin/main.dart`](./examples/getting-started-dart/bin/main.dart).
The same check holds this page to it, and CI generates, analyzes and runs it.

```sh
cd examples/getting-started-dart
dart pub get
dart run trust_tasks:generate --lock trust-tasks.lock --out lib/trust_tasks
dart run bin/main.dart
```

### Imports

Unlike TypeScript and Go, Dart has an HTTPS package,
[`trust_tasks_https`](./trust-tasks-dart-https/): a client, and a server that
runs `consumeInbound` per request. The proof backend is
[`trust_tasks_proof`](./trust-tasks-dart-proof/), a Data Integrity verifier for
`eddsa-jcs-2022` and `ecdsa-jcs-2019` with `signTrustTask` for the producing
side, interoperating with the Rust `trust-tasks-proof` crate in both
directions; its signer is an `ssi` `DidSigner`. The validator is a stand-in, as
in the other two — [`lib/structural_validator.dart`](./examples/getting-started-dart/lib/structural_validator.dart).

<!-- snippet: examples/getting-started-dart/bin/main.dart#imports -->
```dart
import 'dart:typed_data';

import 'package:ssi/ssi.dart' as ssi;
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_https/io.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

// The task trust-tasks.lock pins, generated into lib/trust_tasks/.
import 'package:trust_tasks_example_getting_started/trust_tasks/acl/grant/v0_3/payload.dart'
    as grant;
// Stands in for a JSON Schema engine — see lib/structural_validator.dart.
import 'package:trust_tasks_example_getting_started/structural_validator.dart';
```

### Identity

<!-- snippet: examples/getting-started-dart/bin/main.dart#identity -->
```dart
/// A `did:key` derived from a fixed seed, so both ends agree on the sender's
/// identity with no key exchange. Real deployments load a real key here.
ssi.DidSigner demoIdentity() {
  final keyPair =
      ssi.Ed25519KeyPair.fromSeed(Uint8List.fromList(List.filled(32, 7)));
  final didDoc = ssi.DidKey.generateDocument(keyPair.publicKey);
  return ssi.DidSigner(
    did: didDoc.id,
    didKeyId: didDoc.verificationMethod[0].id,
    keyPair: keyPair,
    signatureScheme: ssi.SignatureScheme.ed25519,
  );
}
```

### The receiver

<!-- snippet: examples/getting-started-dart/bin/main.dart#receiver -->
```dart
/// The consuming end: authenticate the peer, verify the proof, handle the task.
HttpsServer receiver(String aliceDid) {
  return HttpsServer(
    localVid: serverVid,
    // Bearer token → Verifiable Identifier. This is what the SPEC §4.8.1
    // cross-check compares the document's in-band `issuer` against, so it
    // must be the DID the sender actually signs with.
    auth: StaticBearerAuth({'alice': aliceDid}),
    // Without a verifier the server refuses every proof rather than trust it.
    // `forDidKey()` resolves `did:key` offline — no network, no DID document.
    proofVerifier: DataIntegrityProofVerifier.forDidKey(),
    payloadValidator: const StructuralValidator(),
    // The server keeps the duplicate-execution record (§7.2 item 11) itself:
    // an InMemoryReplayGuard unless you pass a shared one as `replayGuard:`.
  )..on<grant.Payload, grant.Response>(
      spec: grant.spec,
      decode: grant.Payload.fromJson,
      encode: (p) => p.toJson(),
      encodeResponse: (r) => r.toJson(),
      handler: (doc, ctx) async {
        final entry = doc.payload.entry;
        print('  [server] acl/grant from ${ctx.authenticatedSender} '
            '→ subject=${entry.subject} role=${entry.role}');
        return grant.Response(entry: entry);
      },
    );
}
```

The server runs the same pipeline the other languages call by hand. To run it
yourself — over DIDComm v2.1 with
[`trust_tasks_didcomm`](./trust-tasks-dart-didcomm/), say, or any transport of
your own — call `consumeInbound` and switch on its outcome:

```dart
switch (outcome) {
  case Handled(:final response):          return send(response);
  case Rejected(:final error):            return send(error);
  case Suppressed(:final reason):         return log(reason);
  case Accepted():                        return;  // fire-and-forget
  case DuplicateOutcome(:final priorResponse):
    // §7.2 item 11: this document already executed. The recorded response,
    // as JSON — a success or (isError) the refusal.
    if (priorResponse != null) sendJson(priorResponse);
}
```

The outcome is a **sealed** hierarchy, so that `switch` is exhaustive and the
compiler tells you when a case is missing — the one place Dart gives more than
the other three.

### The sender

<!-- snippet: examples/getting-started-dart/bin/main.dart#sender -->
```dart
/// The producing end: build the document, sign it, send it, read the reply.
Future<TrustTaskDocument<grant.Payload>> sender(
  ssi.DidSigner signer,
  Uri serverUrl,
) async {
  // `myVid` must be the *signing* identity: the server cross-checks the
  // in-band `issuer` against whatever its bearer token maps to (§4.8.1).
  final client = HttpsClient(
    base: serverUrl,
    serverVid: serverVid,
    myVid: signer.did,
    token: 'alice',
  );

  // acl/grant/0.3 states every authority axis explicitly — there is no
  // default and no implicit form. Carol may act everywhere, may use no keys,
  // holds the full ceiling of the `moderator` role, and may use what she
  // reads for any purpose (`any` never includes `modelTraining`).
  const payload = grant.Payload(
    entry: grant.AclEntry(
      subject: 'did:web:carol.example',
      role: 'moderator',
      act: {'scope': 'all'},
      keys: {'scope': 'none'},
      capabilities: {'scope': 'ceiling'},
      purposes: {'scope': 'any'},
      label: 'Carol — content moderation',
    ),
    reason: 'onboarding moderator',
  );

  // TRAP: party members and `issuedAt` must be set BEFORE signing.
  // `HttpsClient.send` fills in any that are still unset — and a member filled
  // in after the fact is a member the proof does not cover, which the server
  // rejects as `proofInvalid`.
  final unsigned = TrustTaskDocument<grant.Payload>(
    id: newUrnUuid(),
    type: grant.typeUri,
    issuer: signer.did,
    recipient: serverVid,
    issuedAt: systemClock(), // whole seconds, UTC
    payload: payload,
  );
  final request = TrustTaskDocument<grant.Payload>.fromJson(
    await signTrustTask(unsigned.toJson((p) => p.toJson()), signer),
    (p) => grant.Payload.fromJson(p! as Map<String, dynamic>),
  );

  print('  [client] POST /trust-tasks  id=${request.id}');
  try {
    final resp = await client.send<grant.Payload, grant.Response>(
      request,
      encode: (p) => p.toJson(),
      decodeResponse: grant.Response.fromJson,
    );
    print('  [client] ← ${resp.type}\n'
        '           threadId=${resp.threadId} '
        'entry: subject=${resp.payload.entry.subject} '
        'role=${resp.payload.entry.role}');
  } on TrustTaskErrorException catch (e) {
    print('  [client] ← HTTP ${e.httpStatus} trust-task-error: '
        'code=${e.error.payload.code} retryable=${e.error.payload.retryable}');
  } finally {
    client.close();
  }
  return request;
}
```

Three Dart-specific things worth knowing:

- **Closed value sets are extension types, not enums.** A Dart `enum` throws on a
  value it does not recognise, which would crash your parse the first time a peer
  on a newer MINOR sends one (SPEC §5.2 says it must not). Compare against the
  generated constants and treat anything else as unrecognised.
- **`payloadToJson` is an explicit argument** to `consumeInbound` because Dart
  cannot recover a type argument at runtime. Pass `(p) => p.toJson()`; the
  pipeline needs it for the item-11 digest and for the validator. (The HTTPS
  server's `encode:` is the same thing.)
- **Absent and empty stay distinct for free** — `List<String>?` is null when
  absent and `[]` when present-and-empty, which several specs make load-bearing.

Full detail is in [`trust-tasks-dart/README.md`](./trust-tasks-dart/README.md),
[`trust-tasks-dart-https/README.md`](./trust-tasks-dart-https/README.md)
and [`trust-tasks-dart-proof/README.md`](./trust-tasks-dart-proof/README.md).

---

## 4. The four traps

Each of these is documented somewhere in the code. Collected here because each
one costs an afternoon the first time.

### 4.1 Set the party members and `issuedAt` *before* signing

```rust,ignore
request.issuer     = Some(my_did.to_string());
request.recipient  = Some("did:web:maintainer.example".into());
request.issued_at  = Some(trust_tasks::issued_at_now());  // whole seconds, UTC
request.sign(&secret, SignOptions::new()).await?;   // ← signing comes LAST
```

`issued_at_now()` stamps the current instant in whole seconds, UTC — never
write `chrono::Utc::now()` (or `toISOString()` in TypeScript) into a document
field: the sub-second digits are a fingerprint (SPEC §13.1 item 3), and some
RFC 3339 readers refuse them. TypeScript's `issuedAtNow()` writes the same
form.

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
[`docs/packages/rust.md`](./docs/packages/rust.md#generated-payload-types-are-non_exhaustive--builders-as-of-0140)
for the full reasoning.

### 4.4 The facade and the crates it re-exports share one version

`trust-tasks`, `trust-tasks-rs` and the crates that expose `trust-tasks-rs`
types in their own API — the transport bindings, `-proof` and
`-capability-client` — release together at **one shared version**, so a break
in the runtime is a break in every one of them (release-plz's `core` version
group). `trust-tasks-ceremony`, `trust-tasks-build` and `trust-tasks-cli` do
not depend on `trust-tasks-rs` and version on their own.

Consequence worth knowing: if you depend on the individual crates rather than
the facade, pin them as a set at the same version. Two copies of
`trust-tasks-rs` in one dependency graph produce
`expected TrustTask<Value>, found a different TrustTask<Value>`. Depending on
the facade alone avoids that by construction.

---

## 5. Where to go next

| | |
|---|---|
| The normative framework | [`SPEC.md`](./SPEC.md) — §4.8.1 identity precedence, §7.2 the consumer pipeline, §8 errors |
| The task registry | <https://trusttasks.org/> — every published spec and its schema |
| Transport bindings | [`bindings/`](./bindings/) — the wire format for https, didcomm, didcomm-v1, tsp, push |
| Writing a spec | [`CONTRIBUTING-SPECS.md`](./CONTRIBUTING-SPECS.md) |
| Rust API docs | <https://docs.rs/trust-tasks> (facade) · <https://docs.rs/trust-tasks-rs> (core) |
| More Rust examples | [`trust-tasks-https/examples/`](./trust-tasks-https/examples/) — the two-process version of the round trip above |
