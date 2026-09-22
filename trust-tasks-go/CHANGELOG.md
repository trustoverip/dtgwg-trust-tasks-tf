# Changelog — `trust-tasks-go`

All notable changes to the Go bindings module.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The module versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

A Go module is published by tagging, so the released version of this module is
the `trust-tasks-go/vX.Y.Z` tag rather than anything in the tree; the `Version`
constant in `trusttasks/version.go` mirrors it. See `RELEASING.md`.

## 0.1.18 — 2026-09-22

## 0.1.17 — 2026-09-22

## 0.1.16 — 2026-09-21

## 0.1.15 — 2026-09-21


### Specifications

- Bring framework 0.6.0 into the registry (SPEC.md + specs/_framework/0.6) (#555)

Companion to trustoverip/dtgwg-trust-tasks-spec#18. Publishes the 0.6
  envelope schema (0.5's shape unchanged, so 0.6.0 becomes targetable; no
  spec is re-targeted) and brings the SPEC.md mirror to 0.6: version and
  date header, plus the Appendix B entry.

## 0.1.14 — 2026-09-21

## 0.1.13 — 2026-09-21

## 0.1.12 — 2026-09-21

## 0.1.11 — 2026-09-21

## 0.1.10 — 2026-09-21

## 0.1.9 — 2026-09-20


### Specifications

- **vtc/endorsement-types/delete**: A criterion requiring the type also blocks its deletion (#523)

The spec described one kind of reference — a live endorsement — and the
  `inUse` refusal as being about orphaned endorsements alone. A second kind
  exists wherever the consumer also holds admission criteria: a criterion
  requiring statements of the type is left asking applicants for evidence the
  community no longer recognises. Where registering such a criterion against
  an unregistered type is itself refused, deleting the type strands the
  criterion in a state it could not have been created in.

  `inUse` now covers both, and gains a `detailsSchema` so a consumer that can
  determine both reports which applies rather than leaving the caller to
  parse prose. `liveEndorsements` and `criteria` are each optional, because
  neither an endorsement store nor a criteria registry is mandatory — and the
  spec says plainly that an absent member means "not applicable here", not
  "none found", so a caller cannot read silence as an all-clear.

  Conformance asks a consumer that can determine both to gather both before
  refusing. Refusing on the first found turns one determination into as many
  round trips as there are kinds of reference, each ending in the same code.

  Security & Privacy gains what the refusal discloses: `details` names
  criteria by identifier, which is a governance fact the authorised
  administrator can already enumerate, and is reachable only after the
  community-admin capability has been verified.

  In place on 0.1 rather than a new version: the spec is `draft`, no payload
  or response shape changes, and no previously-valid document becomes
  invalid.

  Implemented in verifiable-trust-infrastructure#1584, which added the
  criterion check and reported both causes in one refusal.

## 0.1.8 — 2026-09-20

## 0.1.7 — 2026-09-19

## 0.1.6 — 2026-09-17

## 0.1.5 — 2026-09-16


### Added

- **go-capability-client**: Capability wire client for Go (trust-tasks-go/capabilityclient) (#498)

The Go port of the Rust trust-tasks-capability-client crate: pure wire logic,
  no crypto and no transport. Document builders for the governance/capability/*
  (list/enable/disable) and git-trust/* (grant/revoke) families, DIDComm envelope
  parsing, and reply classification — so a capability producer and a management UI
  cannot drift on the contract.

  A separate nested module like tsp/proof/didcomm, but with no third-party
  dependency at all (only the core, for Document and SlugFromTypeURI).

  - Builders mint a fresh id + issuedAt per call; NewAttempt() re-stamps a built
    document under a fresh id and clears proof (a SPEC §8.4 new attempt, distinct
    from a bit-for-bit retry the consumer's item-11 record absorbs).
  - ClassifyGitTrustReply / ParseCapabilityReply correlate on threadId first
    (§4.9) and key idempotent success on the SPEC §8.5 extended error code (both
    spellings), never the non-normative free-text message; the free-text path is
    opt-in via ReplyPolicy.

  Adds a capabilityclient job to go.yml, the Go capability-client cell to the
  capability matrix, and documents the module in CLAUDE.md. 12 tests, go 1.22 +
  stable, gofmt clean.

## 0.1.4 — 2026-09-16


### Added

- **go-didcomm**: DIDComm v2.1 transport binding for Go (trust-tasks-go/didcomm) (#495)

The bindings/didcomm/0.2 binding for Go, rolled on the standard library. A
  separate nested module like tsp and proof so the core stays dependency-free,
  and like proof it pulls in no third-party code: DIDComm v2.1 authcrypt
  (ECDH-1PU key agreement + A256KW key wrapping + A256CBC-HS512 content
  encryption, over X25519) on crypto/ecdh, with the RFC 3394 key-wrap (pinned to
  the RFC's test vector) and RFC 7518 content encryption written in-module.

  aries-framework-go was evaluated and rejected: its packer is a low-level JWE
  primitive needing a full KMS/Crypto/Storage/VDR provider and raw key bytes, it
  does not build/parse the v2 message envelope or verify sender_kid, and the
  framework is archived. Rolling our own keeps the four bindings aligned and
  PQC-extensible.

  The verified skid is authenticated by the ECDH-1PU static secret (a forged
  skid fails the key unwrap) and becomes the §4.8.1 transport-authenticated
  sender; anoncrypt/plaintext are rejected (binding §2/§4). Key-based like tsp
  (a ResolveSender callback, no DID resolution); the consumer keeps the §7.2
  item-11 duplicate-execution record on by default (binding §6) and routes a
  thread-header disagreement as malformedRequest (binding §3.1).

  Adds a didcomm job to go.yml, the Go implementation to the didcomm/0.2 binding
  registry, the Go binding:didcomm cell to the capability matrix, and documents
  the module in CLAUDE.md. Not release-wired yet; a shared cross-library authcrypt
  fixture is a follow-up.

## 0.1.3 — 2026-09-16


### Added

- **go-proof**: Data Integrity proofs for Go (trust-tasks-go/proof) (#493)

A ProofVerifier and signer for the core module's proof seam, rolled on the
  Go standard library — eddsa-jcs-2022 (Ed25519) and ecdsa-jcs-2019
  (P-256/P-384), did:key only, no network. A separate nested module like
  trust-tasks-go/tsp so the core stays dependency-free, but with no third-party
  code at all: the JCS canonicaliser and base58btc codec are written in-module.

  Byte-compatible with the Rust and TypeScript proof libraries — a test
  reproduces the shared eddsa-jcs-2022 fixture's proofValue exactly. Verifier
  binds the proof to the in-band issuer and returns a typed *proof.Error whose
  Kind stays in the logs (every failure is proofInvalid on the wire, SPEC
  §10.4).

  Adds a proof job to go.yml (the core ./... jobs do not reach a nested module),
  marks the Go proof cell shipped in the implementations matrix, and documents
  the module in CLAUDE.md. Not wired into a release yet — unlike tsp there is no
  blocker, so a publish-go-style tag job is a clean follow-up.

## 0.1.2 — 2026-09-16


### Added

- **go-tsp**: TSP transport binding for Go (trust-tasks-go/tsp) (#486)

A separate nested Go module so the core stays dependency-free: PackTrustTask / UnpackTrustTask for HPKE-sealed Direct TSP messages on affinidi-tsp-go, and Consumer.Receive, the guarded inbound path running SPEC §7.2 with the item-11 record on by default. The sealed {type, document} envelope matches the Rust crate and Dart package (pinned by a test).

  Not released: affinidi-tsp-go has no tag, so it is required at a pseudo-version and there is no trust-tasks-go/tsp/v* release wiring yet; go get resolves the pseudo-version from the public repo. go.yml gets a dedicated tsp job; the core ./... jobs exclude the nested module.

## 0.1.1 — 2026-09-16

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `trusttasks/`.
