# Go: `trust-tasks-go` and its nested modules

Notes specific to the Go module. The repository-wide guide is
[`docs/DEVELOPMENT.md`](../DEVELOPMENT.md).

## The Go module — four things that bite

`trust-tasks-go` follows Rust's shape, not TypeScript's: one package per
specification version, cross-file `$ref`s inlined per package, **no component
hoisting**. Go is nominally typed, so hoisting `Ext` into one shared package
would merge type identities the way it would in Rust (#283 declined it there for
the same reason). TypeScript hoists because structural typing makes it free.

1. **A directory starting with `_` is invisible to the go command.** `_shared`
   and `_framework` are therefore emitted as `shared` and `framework`. A package
   under an underscored directory would be generated, committed, and silently
   excluded from `go build ./...` — green board, unreachable code.

2. **One Go package is one *directory*, not one file.** TypeScript gives
   `auth/_shared/0.1/session.ts` and `tokens.ts` separate namespaces, and both
   declare `Ext`. Side by side in Go that is a redeclaration error, so every
   schema gets its own directory: `specs/auth/shared/tokens/v0_1`.

3. **Optional members are pointers, *including slices and maps*.** `omitempty` on
   a plain slice drops an empty one from the wire, collapsing "present and empty"
   into "absent" — which `acl`'s `allowedKeys` documents at length as a
   privilege-escalation defect ("PRESENT-BUT-EMPTY means authorized on NO keys —
   the opposite of absent"). `*[]string` is ugly and it is the only spelling that
   can say it. `trust-tasks-go/smoke` pins this, on `vta/contexts/update/1.0`'s
   `signableKeys` ("absent: every key; present-but-empty: no keys").

4. **Go from v2 onward needs the major in the module path.** `trust-tasks-go/v2`
   in `go.mod`, in every import, and in `MODULE_PATH` in the generator.
   `scripts/release-bump-go.sh` refuses to propose such a bump and tells you what to
   do instead — it is a human migration, not a version number.

And the one that has no undo: **a Go release cannot be retracted.**
`proxy.golang.org` caches a tag permanently and by design. `verify-go` builds,
vets and tests *before* `publish-go` tags for that reason. crates.io and npm both have a
retraction window; Go does not.

### The generated packages are their own module (`trust-tasks-go/specs`)

`trust-tasks-go/specs/go.mod` makes the prebuilt packages a **separate module**,
`github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/specs` (#1112), so
the core module is the runtime and the generator and no longer ships ~34 MB of
generated code. Import paths do not change. The runtime has not imported the
tree since #1307 (the extends index is generated into
`trusttasks/extends_index.go`).

- **The split takes three releases, and this is the awkward middle.** Every
  published core up to v0.12.x contains `specs/`, so a `specs` module requiring
  one sees each of its packages in two modules and the go command refuses the
  import as ambiguous. Its `go.mod` requires v0.12.0 with a `TODO(#1112 step 3)`
  and **does not build** until step 3 moves that requirement to the first core
  without `specs/`. Until then it is unreleased, has no `go.yml` job (it is in
  `NOT_BUILT_YET` in `scripts/check-release-lists.mjs`), and is excluded from
  the core's release watch and API check like every nested module.
- **`build-go-bindings` keeps `specs/go.mod` and `go.sum`**: its `clean()`
  wipes everything else under `specs/`. The drift job is unchanged.
- **The core's `./smoke` drives generated packages without importing them.**
  `smoke/smoke_test.go` copies the packages `smoke/testdata/smoke/smoke_test.go`
  imports into a throwaway copy of the `specs` module whose `go.mod` replaces the
  core with the checkout, and runs the smoke test there, so it still exercises
  real generated code through the real pipeline in the core's `go test ./...`.
  It skips when `specs/` is not beside the core (a module-cache copy). Step 3
  can move it into the `specs` module's own job.
- **Importers of prebuilt packages** stay on core v0.12.x until the `specs`
  module is published, then add it (`go get …/trust-tasks-go/specs`). A package
  `cmd/trust-tasks-gen` generated needs only the core.
- **Step 3**, after the first core without `specs/` is on the proxy: require it
  in `specs/go.mod` and `go mod tidy`; drop `specs` from `NOT_BUILT_YET`; add a
  `go.yml` job building, vetting and testing `trust-tasks-go/specs`; give it a
  `version.go`, a `GO_MODULES` entry in `release-pr.sh`, a `release-bump-go.sh`
  arm (watching `specs/`, `trust-tasks-go/specs` and
  `scripts/build-go-bindings.mjs`), a `publish-go` matrix leg (tag
  `trust-tasks-go/specs/vX.Y.Z`) and a "What gets published" row; move the smoke
  test into it if wanted; update the docs; release it.

### `ConsumeReceived` — the inbound body every message binding shares

A binding that opens an envelope and authenticates its sender ends up holding
the document's bytes and who sent them; from there the work is the same for
every envelope. `trusttasks.ConsumeReceived(ctx, received, transport, spec,
decode, handler, opts)` is that work, once (#1110): it answers a body that does
not parse with `malformedRequest` addressed from the routing members that did,
runs the binding's own check (`ReceiveOptions.BindingCheck` — DIDComm's §3.1
thread cross-check), refuses members differing only by case, reads a later
version forward with `ReadForSpecWith`, decodes, and runs `ConsumeInbound` with
the bytes as `Received` (proof, item-11 digest and `maxDocumentBytes` over what
arrived). Every refusal precedes the replay record. It refuses a transport that
authenticated no sender (`ErrNoAuthenticatedSender`), because those answers are
only safe when they go to the authenticated party. `trusttasks.NewURNUUID()`
mints document ids and panics when `crypto/rand` fails. A binding needing a
check of its own adds it through `BindingCheck` rather than copying the body.

### The `trust-tasks-go/tsp` nested module

The TSP transport binding is a **separate Go module** under `trust-tasks-go/tsp`,
not a package of the core. The core module has zero dependencies (its selling
point, and what the website claims), and this one pulls in `affinidi-tsp-go` and
its crypto. Keeping them apart is the whole point.

- **The core CI jobs do not touch it.** `go build/vet/test ./...` in
  `trust-tasks-go` excludes a nested module, so none of them pulls the TSP
  dependency. A dedicated `tsp` job in `go.yml` builds and tests it on its own
  floor (`go 1.27`, inherited from `affinidi-tsp-go`). `gofmt -l .` in the core
  job does recurse into the subdir, so keep the tsp files gofmt-clean.
- **It is not released yet.** `affinidi-tsp-go` has no tag, so `go.mod` requires
  it at a pseudo-version and there is no `trust-tasks-go/tsp/v*` release wiring in
  `publish.yml`. `go get` of the module still works (the proxy resolves the
  pseudo-version from the public repo). When `affinidi-tsp-go` is tagged: bump the
  requirement to the tag and add a `publish-go`-style tag job for
  `trust-tasks-go/tsp/vX.Y.Z`.
- **The Go TSP library signs Ed25519 only** and does no DID resolution — the
  caller supplies the sender `*tsp.Identity` (read `AdvertisedSender`, look it up
  in your own store). The sealed `{type, document}` envelope matches the Rust
  crate and the Dart package byte-for-byte, pinned by a test in each.

### The `trust-tasks-go/proof` nested module

The Data Integrity proof backend is a **separate Go module** under
`trust-tasks-go/proof`, like `tsp`, so the core stays dependency-free. Unlike
`tsp` it pulls in **no third-party code**: it rolls `eddsa-jcs-2022` (Ed25519)
and `ecdsa-jcs-2019` (P-256/P-384) on the standard library — `crypto/ed25519`,
`crypto/ecdsa`, the SHA-2 family — with a JCS canonicaliser and a base58btc codec
written in the module. Its only requirement is the core module whose
`trusttasks.ProofVerifier` interface it satisfies (required at the published
`v0.1.2`, no `replace`, exactly as `tsp` requires the core).

- **Do NOT use the core's `CanonicalJSON` here.** It documents itself as only
  *equality-equivalent* to RFC 8785 (byte-wise member ordering, `encoding/json`
  number formatting) and explicitly warns it must not be used for the
  eddsa-jcs-2022 proof input. `proof/jcs.go` is a real JCS — UTF-16 code-unit
  member ordering and the ECMAScript Number-to-String number form — because for
  a proof the *bytes* are the interoperable artifact, not just the equality they
  induce.
- **A test reproduces the shared `eddsa-jcs-2022` fixture's `proofValue` byte for
  byte** (Ed25519 is deterministic; the fixture is signed with seed `[7; 32]`),
  so a document signed by any of the Rust/TS/Dart/Go backends verifies with the
  others. The fixture is a copy of `trust-tasks-ts-proof`'s in `proof/testdata/`.
  Keep that test when touching either the canonicaliser or the signer. `created`
  is written the way chrono does (UTC, `Z`, no `.000`).
- **The verifier binds the proof to the in-band `issuer`** and returns a typed
  `*proof.Error` whose `Kind` is for logs only — every failure reaches the wire
  as `proofInvalid` (§10.4). `did:key` only, no network.
- **The core CI jobs do not reach it** (`./...` in `trust-tasks-go` excludes a
  nested module); a dedicated `proof` job in `go.yml` builds/vets/tests it on its
  `go 1.22` floor. `gofmt -l .` in the core job **does** recurse into the subdir,
  so keep these files gofmt-clean.
- **Released on its own `trust-tasks-go/proof/vX.Y.Z` tag**, independently of the
  core. It carries a `version.go` with a `const Version` (the manifest surrogate,
  as the core's does), a matrix leg in `publish-go`, and a `GO_MODULES` entry in
  `scripts/release-pr.sh` (`release-bump-go.sh trust-tasks-go/proof`). The core's release WATCH **excludes**
  the nested modules, so a change under `trust-tasks-go/proof` releases proof, not
  the core. Like every Go tag it is irreversible, so `verify-go` builds/vets/tests
  the module before `publish-go` tags it.

### The `trust-tasks-go/didcomm` nested module

The DIDComm v2.1 transport binding (`bindings/didcomm/0.2`) is a **separate Go
module** under `trust-tasks-go/didcomm`, like `tsp` and `proof`. Like `proof`,
and unlike `tsp`, it pulls in **no third-party code**: it rolls DIDComm v2.1
authcrypt on the standard library.

- **`aries-framework-go` was evaluated and rejected.** Its DIDComm packer is a
  low-level JWE primitive (`Pack(contentType, payload, senderID, recipientsPubKeys)`
  on raw key bytes, needing a full KMS/Crypto/Storage/VDR provider) that does not
  build or parse the v2 message envelope or surface a verified `sender_kid`→DID,
  and the framework is archived (last release 2023). Rolling our own — the same
  call the maintainers made for `proof` and TSP — keeps the four bindings structurally
  aligned and PQC-extensible.
- **The one authcrypt profile it implements** is the DIDComm v2 default:
  ECDH-1PU key agreement + `A256KW` key wrapping + `A256CBC-HS512`, over X25519.
  `crypto/ecdh` does the key agreement; `kdf.go` (Concat KDF), `keywrap.go`
  (RFC 3394, **pinned to the RFC's own test vector**) and the RFC 7518
  content-encryption in `jwe.go` are hand-rolled. anoncrypt and any other
  alg/enc are rejected on unpack — no authenticated sender (binding §2/§4).
- **It matches the DIDComm ecosystem's ECDH-1PU exactly, which is subtle.** DIDComm
  authcrypt is *tag-in-KDF*: encrypt the content **first**, then derive the KEK
  with the content-encryption tag fed to the Concat KDF as a length-prefixed
  `SuppPrivInfo` (draft-madden §2.3), and `apv = base64url(SHA-256(sorted recipient
  kids joined by "."))`, `apu = base64url(skid)`. `aries-askar`, `didcomm-python`,
  `didcomm-rust` and `affinidi` all do this; a plain Concat KDF with `apv = rawKid`
  (what this binding did through `v0.1.0`) is spec-shaped but does **not**
  interoperate. Do not "simplify" the KEK derivation back — it is load-bearing.
  The rework to fix it was the `v0.1.0` → `v0.2.0` breaking change.
- **`skid` is authenticated by the ECDH-1PU static secret, not trusted on its
  face.** The recipient resolves the sender's key *from* `skid` and derives the
  KEK with it; a forged `skid` yields a KEK that fails the key unwrap. So there
  is no separate skid-mismatch check like the Rust binding's (that existed only
  because the affinidi API separated the lookup key from `skid`).
- **Key-based like `tsp`** — the consumer holds the recipient's X25519 private
  key and takes a `ResolveSender(did) → *PeerKey` callback; no DID resolution.
  The §3.1 thread cross-check lives in the consumer (it needs both the envelope
  header and the parsed document), and a disagreement is `malformedRequest`
  routed to the sender, never `identityMismatch`.
- **Core CI does not reach it**; a `didcomm` job in `go.yml` builds/vets/tests on
  the `go 1.22` floor. Keep the files gofmt-clean (the core job's `gofmt -l .`
  recurses in). **Released on its own `trust-tasks-go/didcomm/vX.Y.Z` tag** the
  same way `proof` is — a `version.go`, a `publish-go` matrix leg, and a
  `GO_MODULES` entry in `release-pr.sh` — independent of the core. **Cross-library interop is now
  asserted both directions**: `interop_test.go` here unpacks a JWE packed by
  `affinidi-messaging-didcomm` (affinidi → Go), and `trust-tasks-didcomm/tests/
  interop.rs` unpacks a Go-packed JWE with affinidi (Go → affinidi). The fixtures
  and their regeneration harnesses are in `testdata/` (see its README).

### The `trust-tasks-go/capabilityclient` nested module

The capability wire client is a **separate Go module** under
`trust-tasks-go/capabilityclient`, like `tsp`/`proof`/`didcomm`, but it is
**pure wire logic** — no crypto, no transport — the Go port of the Rust
`trust-tasks-capability-client` crate. Document builders for the
`governance/capability/*` and `git-trust/*` families, envelope parsing, and reply
classification. No third-party dependency (only the core, for `Document` and
`SlugFromTypeURI`).

- **It reuses the core's `SlugFromTypeURI`** for a Type URI's slug (the core
  strips the `#request`/`#response` fragment and validates the namespace); the
  response-variant check is a `#response` suffix test.
- **Idempotent success is keyed on the SPEC §8.5 extended error code**
  (`git-trust/grant:already_granted` etc., both snake_case and lowerCamelCase),
  never the non-normative free-text `message` — the free-text path is opt-in via
  `ReplyPolicy`, matching the Rust and TypeScript ports.
- **`NewAttempt` clears `proof` and re-stamps `id`/`issuedAt`** — a SPEC §8.4 new
  attempt, distinct from a bit-for-bit retry the consumer's item-11 record
  absorbs. Sign built documents with `trust-tasks-go/proof`.
- **Core CI does not reach it**; a `capabilityclient` job in `go.yml` builds/vets/
  tests it on the `go 1.22` floor. Keep the files gofmt-clean. **Released on
  its own `trust-tasks-go/capabilityclient/vX.Y.Z` tag** the same way `proof`
  is — a `version.go`, a `publish-go` matrix leg, a `GO_MODULES` entry in
  `release-pr.sh` and a `release-bump-go.sh` case — and the core's release
  watch excludes it, so a change here releases this module, not the core.
