# @openvtc/trust-tasks

TypeScript bindings **and the consumer pipeline** for the [Trust Tasks
framework](https://trusttasks.org) registry.

Two halves: generated types for every specification in the registry, and a
hand-written implementation of the SPEC.md §7.2 checks that decide whether an
inbound document may be acted on. See [Consuming a document](#consuming-a-document)
— types alone will not make a conforming consumer.

Every spec under [`dtgwg-trust-tasks-tf/specs/`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/specs)
that defines a `payload.schema.json` is compiled — via
`scripts/build-ts-bindings.mjs` (json-schema-to-typescript) — to a
TypeScript module under
[`src/<slug>/<version>/payload.ts`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/trust-tasks-ts/src),
giving you a typed `Payload` and (where the spec defines one)
`Response` interface for each Trust Task. The package's root
`index.ts` re-exports every module under a stable name keyed by the
spec slug.

The Rust, Go and Dart counterparts are
[`trust-tasks-rs`](https://crates.io/crates/trust-tasks-rs),
[`trust-tasks-go`](https://pkg.go.dev/github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go)
and [`trust_tasks`](https://pub.dev/packages/trust_tasks); all four are
regenerated together so their wire shapes stay identical.

> **The prebuilt specification modules are deprecated** (all but the
> framework's own `trust-task-*` ones) and will be removed in a future release.
> Import the runtime from `@openvtc/trust-tasks/runtime`, pin the tasks you use
> with the `trust-tasks` CLI, and generate them with
> [`@openvtc/trust-tasks-gen`](https://www.npmjs.com/package/@openvtc/trust-tasks-gen)
> (`trust-tasks generate --lang ts --out src/trust-tasks`). Editors flag the
> root's namespace exports; a deep import of a module file is not flagged, but
> is deprecated all the same.

## Install

```sh
npm install @openvtc/trust-tasks
```

Requires **Node.js 20.19 or later** (`engines`), or a browser or React Native
runtime with the global Web Crypto API (`crypto.randomUUID`, which browsers
expose only in a secure context). The package is ESM; its `exports` carry a
`default` condition, so `require()` also works on a Node.js that can load ESM
synchronously (20.19+, 22.12+).

## Installing the `trust-tasks` command

Pinning tasks and generating their types uses the `trust-tasks` CLI, one
binary for every language. **No Rust toolchain is needed** — any one of:

- download the archive for your platform (Linux x86_64/aarch64, Windows
  x86_64; on macOS use `cargo binstall` or `cargo install`) from a
  [`trust-tasks-cli-v*` release](https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases?q=trust-tasks-cli&expanded=true), check it against its `.sha256`, and
  put `trust-tasks` on your `PATH`;
- `cargo binstall trust-tasks-cli` (the same prebuilt archive);
- `cargo install trust-tasks-cli` (builds from source, Rust 1.95+).

See [GETTING-STARTED](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/GETTING-STARTED.md#installing-the-trust-tasks-command) for the
commands.

## What you bring yourself

| | |
|---|---|
| Payload validator (SPEC §7.2 item 2) | yours — a function over [`ajv`](https://www.npmjs.com/package/ajv) or any Draft 2020-12 engine ([below](#validating-payloads-at-runtime)) |
| Proof verifier / signer (item 6) | [`@openvtc/trust-tasks-proof`](https://www.npmjs.com/package/@openvtc/trust-tasks-proof) — `eddsa-jcs-2022`, `ecdsa-jcs-2019`, `did:key` |
| Transport | TSP: [`@openvtc/trust-tasks-tsp`](https://www.npmjs.com/package/@openvtc/trust-tasks-tsp). HTTPS and DIDComm have no package yet — carry the document yourself (e.g. `fetch`) per the [binding](https://github.com/trustoverip/dtgwg-trust-tasks-tf/tree/main/bindings) |

## Usage

Import the runtime from `@openvtc/trust-tasks/runtime`, and the tasks you use
from the modules `trust-tasks-gen-ts` generated into your own source tree:

```sh
npm install @openvtc/trust-tasks && npm install -D @openvtc/trust-tasks-gen
trust-tasks add acl/grant/0.3                           # pins it in trust-tasks.lock
trust-tasks generate --lang ts --out src/trust-tasks    # writes the modules
```

```ts
import { issuedAtNow, type TrustTaskDocument } from "@openvtc/trust-tasks/runtime";
import { AclGrant_v0_3 } from "./trust-tasks/index.js"; // generated

// Compose a request payload, type-checked against the spec's
// payload.schema.json. `Payload` is a stable alias every module
// exports, so you never have to look up the title-derived interface
// name.
const grant: AclGrant_v0_3.Payload = {
  entry: {
    subject: "did:key:z6MkAlice",
    role: "admin",
    act: { scope: "contexts", contexts: ["ctx/payments"] },
    keys: { scope: "all" },
    capabilities: { scope: "ceiling" },
    purposes: { scope: "any" },
  },
  reason: "onboarding",
};

const request: TrustTaskDocument<AclGrant_v0_3.Payload> = {
  id: crypto.randomUUID(),
  type: AclGrant_v0_3.TYPE_URI,
  issuer: "did:key:z6MkAlice",
  recipient: "did:web:maintainer.example",
  issuedAt: issuedAtNow(), // whole seconds — never toISOString() (SPEC §13.1 item 3)
  payload: grant,
};

// Each module exports the Response sub-schema too when the spec
// defines one — the framework's #response fragment convention.
// Fire-and-forget specs export neither `Response` nor
// `RESPONSE_TYPE_URI`, because SPEC.md §4.4.1 forbids emitting a
// `#response` document for them.
const reply: AclGrant_v0_3.Response = {
  entry: { /* … */ },
};
```

A generated module has the same shape as the prebuilt module of the same
version in this package (`AclGrant_v0_3` from the package root), so moving off
the prebuilt tree is a change to the import line. The prebuilt modules are
listed in [`src/index.ts`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-ts/src/index.ts);
importing the package root loads every one of them.

## Module layout

Each prebuilt spec module lands at a stable path (`trust-tasks-gen-ts` names the
modules it generates the same way):

```
src/
├── acl/grant/0.3/payload.ts           // AclGrant_v0_3
├── auth/authenticate/0.3/payload.ts   // AuthAuthenticate_v0_3
├── did-management/did/register/0.1/
│   └── payload.ts                     // DidManagementDidRegister_v0_1
├── vault/proxy-login/0.2/payload.ts   // VaultProxyLogin_v0_2
└── …
```

Slug `/` separators in the URI map to directory boundaries; the
exported name is generated by Pascal-casing the slug segments and
appending `_v<MAJOR>_<MINOR>`. Multiple versions of the same spec
land in sibling directories (`0.1/`, `1.0/`, …) and export
distinct names — there is no "latest" alias by design.

### Shared definitions

A definition several specs `$ref` — `Ext` (the SPEC §4.5.1 extension object),
`AclEntry`, `VaultEntry`, `DigestMultibase` — is declared **once**, in
`src/_shared/components.ts`, and re-exported from every spec module that uses
it under the name that module has always used. So `AclGrant_v0_3.AclEntry` and
`AclShow_v0_3.AclEntry` are the same type, not two copies of one shape:

```ts
import { SharedComponents, type AclGrant_v0_3 } from "@openvtc/trust-tasks"; // prebuilt

// Write a helper once, over the shared definition, and it takes any spec's.
const namespaces = (ext: SharedComponents.Ext) => Object.keys(ext);
```

Where one definition *name* covers more than one shape — `VaultEntry` differs
between `vault/_shared/0.1`, `0.2` and `0.3` — `components.ts` qualifies each
(`VaultEntry_VaultV0_1`, …) rather than pretending they are interchangeable.
The spec modules still export the bare `VaultEntry`, bound to the version they
were generated against.

Before 0.15.0 these definitions were copied into every module that referenced
them, and repeated copies inside one module were numbered `Ext1`, `Ext2`, `Vid1`
and so on. Those numbered names are gone; see
[CHANGELOG.md](./CHANGELOG.md) for the one-line migration.

## Consuming a document

A conforming *consumer* applies the checks in
[SPEC.md §7.2](https://trusttasks.org/SPEC#72-consumer-requirements) before
acting on an inbound document. Three of them — recipient-REQUIRED, proof-REQUIRED
and audience binding — are declared *per specification* and cannot be derived
from the document itself, so every generated module exports the declarations as
`SPEC` (and `RESPONSE_SPEC`, where the spec defines a response).

`consumeInbound` runs items 2–8, item 11's duplicate-execution record and item
13's freshness bound, then calls your handler. Item 2 (payload schema) runs
through the `payloadPolicy` you pass: `{ kind: "validate", validate }` checks the
payload against `spec.payloadSchema`, which every generated module carries. Item 3 is a check that the
document's `type` is `spec.typeUri` (`#request` and no fragment alike), or a
later version whose `extends` chain reaches it: anything else is refused
`unsupportedVersion` (the same specification) or `unsupportedType` (another
one), before the item-11 record. The chain is read from the generated index
plus any `declareExtends` tables — `consumeInbound` takes no index of its own. A
document re-read with `readForSpec` for the same `spec` has been routed already
(with whatever index you gave `readForSpec`) and is not checked again; one
re-read for a different spec is checked like any other. Item 1 (framework
schema) belongs to your parse.

```ts
import {
  consequentialChecks,
  consumeInbound,
  InMemoryReplayGuard,
  respondWith,
  StaticTransport,
} from "@openvtc/trust-tasks/runtime";
import { AclGrant_v0_3 } from "./trust-tasks/index.js"; // generated

// The guard *is* the duplicate-execution record — one per consumer, held for
// the process's lifetime. Back it with a shared store if you run replicas.
const guard = new InMemoryReplayGuard();

const outcome = await consumeInbound<AclGrant_v0_3.Payload, AclGrant_v0_3.Response>({
  transport: new StaticTransport({ issuer: peerVid }), // what the transport authenticated
  spec: AclGrant_v0_3.SPEC,
  proofPolicy: { kind: "verify", verify: myVerifier },
  payloadPolicy: { kind: "validate", validate: myValidator }, // or acceptUnvalidated
  // acl/grant is consequential: a replayed envelope must not grant twice.
  checks: consequentialChecks(guard), // or notConsequentialChecks()
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
    // §7.2 item 11: this document already executed. Not an error — return the
    // prior result where there is one, otherwise emit nothing.
    return outcome.priorResponse === undefined ? undefined : send(outcome.priorResponse);
  case "accepted":
    return undefined; // fire-and-forget: nothing to emit
}
```

**The envelope is checked before anything reads it.** `JSON.parse(body) as
TrustTaskDocument<P>` checks nothing, so `consumeInbound` first refuses
`malformedRequest` a document whose framework members are not the types SPEC
§4.2 gives them (a string `id` and `type`, an object `payload`, and so on), or
which nests objects and arrays more than `MAX_NESTING_DEPTH` (128) levels deep.
The refusal is addressed only from the members that are strings.
`validateEnvelope` is the same check, exported for a caller that routes on a
document before consuming it.

**Choose the checks deliberately.** `consequentialChecks(guard)` is correct for
any task whose execution grants access, moves value, discloses a secret, or is
otherwise irreversible — SPEC §7.2 item 11 makes duplicate-execution protection
normative for those, and every transport binding delegates it to the consumer.
`notConsequentialChecks()` keeps no record and is conformant only where the task
is not consequential, or where the specification declares repeated execution
safe and intended.

**Choose a proof policy deliberately.** `{ kind: "verify" }` honours in-band
proofs. `{ kind: "rejectIfPresent" }` is for consumers with integrity from
another layer — it refuses a proof-bearing document rather than silently
dropping the proof, which would mislead the producer about the guarantees of
the exchange. `{ kind: "acceptUnverified" }` is the explicit opt-out and only
safe where the transport already provides equivalent end-to-end integrity. A
specification that declares `proof` REQUIRED rejects a proofless document under
all three.

The pipeline mirrors `consume_inbound` in
[`trust-tasks-rs`](https://crates.io/crates/trust-tasks-rs) check for check, so
a TypeScript consumer and a Rust one reach the same verdict on the same
document.

### Accepting a later version that `extends` yours

A specification version may declare `extends: "<M.K>"` (SPEC §5.2): it keeps
everything `M.K` defines and only adds. Every generated module carries its
direct declaration as `EXTENDS`, and `EXTENDS_BY_TYPE_URI` indexes them, so a
library upgrade alone teaches a consumer of `x/0.2` to accept `x/0.3`. Pass the
parsed document through `readForSpec` before `consumeInbound`:

```ts
const read = readForSpec<XGet_v0_2.Payload>(parsed, XGet_v0_2.SPEC);
if (read.kind === "refused") return emit(rejectWith(parsed, newId(), toErrorPayload(read.reason)));
const outcome = await consumeInbound({ ...opts, spec: XGet_v0_2.SPEC, doc: read.doc });
```

The implemented version passes through unchanged. A later version whose
`extends` chain reaches it comes back with its payload pruned to the members
your version's schema knows and its `type` kept as received, so `respondWith`
answers `<received type>#response`. Anything else is refused
`unsupportedVersion`. `consumeInbound` verifies the proof and records the item-11
digest over the document **as received**, not the pruned copy — pass `read.doc`
itself, not a copy of it.

## Validating payloads at runtime

Item 2 needs no fetch: every module carries its payload schema, with every
`$ref` inlined, as `SPEC.payloadSchema` (and `RESPONSE_SPEC.payloadSchema`), and
`consumeInbound` applies it through `payloadPolicy`. What this package does not
bundle is the JSON Schema validator itself — pass `{ kind: "validate", validate }`
with a function over [`ajv`](https://www.npmjs.com/package/ajv) or any other
Draft 2020-12 validator, or `{ kind: "acceptUnvalidated" }` to say explicitly
that you rely on another layer. Item 1 (the framework schema) belongs to your
parse, beyond the envelope member types and nesting bound `consumeInbound`
checks itself.

## Versioning

The package version follows [Semantic Versioning](https://semver.org/) over
**this package's own API** — the exported types, the runtime, and the generated
module surface. A breaking change to any of those bumps the leading non-zero
component (`0.2.x` → `0.3.0` while below 1.0); anything additive is a patch.
Behavioural changes count: when a specification starts declaring `proof`
REQUIRED, `consumeInbound` rejects documents it previously accepted, and the
version has to reflect that even though the wire format is unchanged.

The version is **not** tied to the [`SPEC.md`](https://trusttasks.org/SPEC)
framework revision, which it once claimed to track. The two move for different
reasons — a framework revision can change the spec-authoring contract without
altering a single generated type. Read a document's framework version from the
specification's `targetFrameworkVersion` declaration instead.

A bump in an individual spec's own `MAJOR.MINOR` ships as a new exported module
name (`AclGrant_v1_0` alongside `AclGrant_v0_1`) and so is additive here.

`trust-tasks-rs` and `@openvtc/trust-tasks` are usually released together and
carry matching version numbers, so a given pair is known to be generated from
the same registry revision. They diverge when a change is breaking on one side
only — 0.15.0 is such a release, hoisting the shared definitions in a way that
is safe here (TypeScript is structurally typed) and a coherence break in Rust.

One divergence is standing and worth knowing before you write a `switch`:
`StandardCode` is a **closed** union here and `#[non_exhaustive]` in
`trust-tasks-rs`, so a new SPEC §8.3 error code is a **minor** bump on this
package and a patch on the crate. Narrow with `isStandardCode` and keep a
`default` arm to be immune to it; the reasoning is on the type itself and in
[CHANGELOG.md](./CHANGELOG.md).

## License

Apache-2.0 — see [`LICENSE`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-ts/LICENSE),
included in the published package.

## Contributing

The package has two halves, maintained differently:

- **The specification modules** (`src/<slug>/<version>/`, `src/_shared/`,
  `src/index.ts`) are generated, never edited by hand. To change them, add or
  modify a spec under `specs/` in the
  [dtgwg-trust-tasks-tf](https://github.com/trustoverip/dtgwg-trust-tasks-tf)
  repo and run `npm run build-ts-bindings` from the repo root.
- **The runtime** (`src/_runtime/`, exported as `@openvtc/trust-tasks/runtime`)
  is hand-written: the §7.2 pipeline, the document helpers and the transport
  types. Change it like any other source, with tests under `test/`. See
[`CONTRIBUTING-SPECS.md`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/CONTRIBUTING-SPECS.md)
for the spec authoring guide.
