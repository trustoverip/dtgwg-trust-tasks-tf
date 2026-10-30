# Changelog

All notable changes to `trust-tasks-tsp` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate versions independently of `trust-tasks-rs` — it takes its own
leading bump when a `trust-tasks-rs` break reaches it, rather than aligning
to that crate's number (see the `0.6.5` → `0.7.0` release for the shape).

## [0.36.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.36.3...trust-tasks-tsp-v0.36.4) — 2026-10-30


## [0.36.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.36.2...trust-tasks-tsp-v0.36.3) — 2026-10-30


## [0.36.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.36.1...trust-tasks-tsp-v0.36.2) — 2026-10-29


### Added

- **rs**: A guarded consumer for Trust Tasks of any type ([#969](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/969)) ([#980](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/980))

A consumer that opens a document of any type, routes on `type` and answers
  an unknown one `unsupportedType` could not be written: the bindings'
  pack/unpack required `P: Payload`, which `serde_json::Value` and
  `ErrorPayload` are not, and `TspConsumer` (like the DIDComm consumers) had
  no path that kept the duplicate-execution record for such a consumer.

  - The bindings' `pack_trust_task` / `unpack_trust_task` (TSP, DIDComm v2.1,
    DIDComm v1 `build_message`) take any `Serialize` / `DeserializeOwned`
    payload. None of them read a `Payload` constant; the bound only refused
    `TrustTask<Value>` and `ErrorResponse`. Loosening a bound is not a break.
  - `ErrorPayload` implements `Payload`. `TYPE_URI` is the version the library
    emits, now the one constant `TRUST_TASK_ERROR_TYPE_URI` (0.6) that
    `trust_task_error_type_uri()` parses; check-bindings reads it there. The
    policy constants are the 0.6 front matter's (recipient REQUIRED, proof
    RECOMMENDED).
  - `consume_dispatched` runs `consume_inbound`'s pipeline for a
    `TrustTask<Value>` routed by an `AsyncDispatcher`. The spec policy and
    payload schema come from the registration the document routes to (now
    recorded by `on_async`). `unsupportedType`, `unsupportedVersion` and a
    payload the handler's `P` cannot hold are refused before the item-11 record
    is written, so they never burn an `id`. The rest is the same code:
    `consume_inbound` and `consume_dispatched` both call one private
    `consume_core`, so freshness, the §4.8.1 cross-check, proof verification
    over the document as received, the per-spec checks and the record cannot
    diverge. Forward routes along a declared extension are pruned exactly as
    `AsyncDispatcher::dispatch` prunes them, which now shares the routing step.
  - `TspConsumer`, `DidcommConsumer` and `DidcommV1Consumer` gain
    `receive_dispatched` / `consume_dispatched` (and `receive_from_dispatched`
    for DIDComm v2.1), keeping each consumer's record and freshness policy.

  The HTTPS server already routes on `type` with its own record, so it needs
  nothing. TypeScript, Go and Dart have no dispatcher: their consume functions
  take the spec as a value and their unpack is untyped, so the caller routes
  then consumes; CLAUDE.md records the difference.



## [0.36.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.36.0...trust-tasks-tsp-v0.36.1) — 2026-10-28


## [0.36.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.35.4...trust-tasks-tsp-v0.36.0) — 2026-10-27


## [0.35.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.35.3...trust-tasks-tsp-v0.35.4) — 2026-10-27


## [0.35.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.35.2...trust-tasks-tsp-v0.35.3) — 2026-10-26


### Fixed

- **rs**: The transport crates depend on trust-tasks-rs's runtime only ([#939](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/939))

trust-tasks-didcomm, -didcomm-v1, -tsp, -https, -proof and
  -capability-client depended on trust-tasks-rs with its default features,
  which are every specification family. Cargo unifies features across the
  graph, so an application that generates only the tasks it pins
  (trust-tasks-build) still compiled every prebuilt module as soon as it
  depended on any transport, and the facade route in GETTING-STARTED
  (`trust-tasks` with default-features = false and runtime_path) got every
  family anyway.

  None of these crates uses a specification module outside its tests,
  examples and doc examples, so each now depends on the runtime alone
  (default-features = false) and enables `acl` (and `task-consent` for
  trust-tasks-https) as a dev-dependency feature for those.

  Verified per crate in isolation, so no other workspace member can switch
  the families back on: `cargo tree -e features` shows trust-tasks-rs with
  no features for each, and each crate's tests, examples and doctests pass
  with --all-features.

  An application that reached a family's types only because a transport
  switched every family on must now enable that family (or the `all-specs`
  default) on trust-tasks-rs itself.



## [0.35.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.35.1...trust-tasks-tsp-v0.35.2) — 2026-10-26


## [0.35.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.35.0...trust-tasks-tsp-v0.35.1) — 2026-10-26


## [0.35.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.34.2...trust-tasks-tsp-v0.35.0) — 2026-10-26


## [0.34.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.34.1...trust-tasks-tsp-v0.34.2) — 2026-10-24


## [0.34.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.34.0...trust-tasks-tsp-v0.34.1) — 2026-10-23


## [0.34.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.33.2...trust-tasks-tsp-v0.34.0) — 2026-10-23


### Added

- **tsp**: The Trust Tasks TSP binding speaks TSP 1.0 ([#881](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/881))

affinidi-tsp 0.2 -> 0.3. The ToIP TSP specification released 1.0
  (trustoverip/tswg-tsp-specification#86, YTSP-BAA): Rev 3's wire with the
  ESSR sender VID mandatory in every confidential payload. 1.0 and Rev 3 do not
  interoperate in either direction, so a consumer of this binding moves with
  its peers - hence the !.

  No source or API change here: affinidi-tsp 0.3 (affinidi/affinidi-tdk-rs#967)
  keeps every signature this binding uses. MetaEnvelope::parse, which a relay
  calls before unpack, now reads any TSP version.

  Unblocks verifiable-trust-infrastructure, which cannot move to affinidi-tsp
  0.3 while this binding resolves 0.2 beside it.



### Fixed

- **trust-tasks-tsp**: A sender mismatch maps to proofRequired, as the DIDComm bindings' sender failures do ([#882](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/882))
- Binding rejection reasons carry a category, not the error's text ([#877](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/877))

The DIDComm v2.1, DIDComm v1 and TSP bindings built the malformedRequest
  reason in `into_reject_reason` from the error's `Display`. That text holds the
  deserializer's member names and line/column offsets, sender DIDs and VIDs,
  verkeys, and upstream envelope and resolver failures. A caller that answered
  with the reason put all of it on the wire, which SPEC §10.4 forbids. The HTTPS
  binding already avoided this.

  - A body that does not deserialise now maps through
    `RejectReason::malformed_from_serde`, as the HTTPS binding's does.
  - A thread disagreement names the header and member, not their values.
  - Every other malformedRequest carries one constant reason per binding.

  The error's `Display` is unchanged, so logs keep the detail. The codes are
  unchanged; only the reason text differs.

- Proofs are verified over the document as received, so a genuine proof survives a typed re-encoding ([#875](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/875))

Every Rust consumer verified a proof over its parsed TrustTask re-serialised:
  consume_inbound through ProofVerifier::verify, the HTTPS server through
  DynProofVerifier::verify_json on a TrustTask<Value>. Re-serialisation
  reproduces the signed bytes only when every member round-trips, and several
  do not — a generated `#[serde(default)]` member the producer omitted comes
  back as `false`, a member the payload type does not model is dropped, and a
  framework timestamp written `+00:00` is re-rendered as `Z`. A genuine proof
  over any such document was refused proofInvalid.

  - ProofVerifier and DynProofVerifier gain a provided `verify_received(&Value)`;
    the affinidi Verifier overrides it with its exact `verify_raw`, and
    ErasedVerifier forwards it. Existing implementations keep the old,
    re-serialising behaviour by default.
  - ProofPolicy gains `VerifyReceived(&V, &Value)` and `with_received`, which
    turns a caller's `Verify` into it. consume_inbound re-parses the received
    value and refuses proofInvalid when it is not the document being consumed,
    since a proof over one document vouches for nothing about another.
  - The TSP, DIDComm v2 and DIDComm v1 `receive` paths keep the document as
    received and apply `with_received`, so their callers change nothing.
  - The HTTPS server verifies over the request body parsed as a Value, keeping
    the typed parse that refuses a duplicated top-level member.
  - DynProofVerifier's doc no longer claims the erasure is lossless.

  All additive: cargo semver-checks reports no update required for
  trust-tasks-rs, trust-tasks-proof or trust-tasks-https.



## [0.33.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.33.1...trust-tasks-tsp-v0.33.2) — 2026-10-22


## [0.33.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.33.0...trust-tasks-tsp-v0.33.1) — 2026-10-21


## [0.33.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.10...trust-tasks-tsp-v0.33.0) — 2026-10-21


## [0.32.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.9...trust-tasks-tsp-v0.32.10) — 2026-10-21


## [0.32.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.8...trust-tasks-tsp-v0.32.9) — 2026-10-21


## [0.32.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.7...trust-tasks-tsp-v0.32.8) — 2026-10-21


## [0.32.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.6...trust-tasks-tsp-v0.32.7) — 2026-10-20


## [0.32.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.5...trust-tasks-tsp-v0.32.6) — 2026-10-20


## [0.32.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.4...trust-tasks-tsp-v0.32.5) — 2026-10-20


## [0.32.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.3...trust-tasks-tsp-v0.32.4) — 2026-10-19


## [0.32.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.2...trust-tasks-tsp-v0.32.3) — 2026-10-19


## [0.32.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.1...trust-tasks-tsp-v0.32.2) — 2026-10-19


## [0.32.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.32.0...trust-tasks-tsp-v0.32.1) — 2026-10-19


## [0.32.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.16...trust-tasks-tsp-v0.32.0) — 2026-10-18


## [0.31.16](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.15...trust-tasks-tsp-v0.31.16) — 2026-10-18


## [0.31.15](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.14...trust-tasks-tsp-v0.31.15) — 2026-10-17


## [0.31.14](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.13...trust-tasks-tsp-v0.31.14) — 2026-10-17


## [0.31.13](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.12...trust-tasks-tsp-v0.31.13) — 2026-10-16


## [0.31.12](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.11...trust-tasks-tsp-v0.31.12) — 2026-10-16


## [0.31.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.10...trust-tasks-tsp-v0.31.11) — 2026-10-16


## [0.31.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.9...trust-tasks-tsp-v0.31.10) — 2026-10-16


## [0.31.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.8...trust-tasks-tsp-v0.31.9) — 2026-10-16


## [0.31.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.7...trust-tasks-tsp-v0.31.8) — 2026-10-15


## [0.31.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.6...trust-tasks-tsp-v0.31.7) — 2026-10-15


## [0.31.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.5...trust-tasks-tsp-v0.31.6) — 2026-10-15


## [0.31.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.4...trust-tasks-tsp-v0.31.5) — 2026-10-15


## [0.31.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.3...trust-tasks-tsp-v0.31.4) — 2026-10-15


## [0.31.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.2...trust-tasks-tsp-v0.31.3) — 2026-10-15


## [0.31.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.1...trust-tasks-tsp-v0.31.2) — 2026-10-14


## [0.31.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.31.0...trust-tasks-tsp-v0.31.1) — 2026-10-14


## [0.31.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.30.2...trust-tasks-tsp-v0.31.0) — 2026-10-13


## [0.30.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.30.1...trust-tasks-tsp-v0.30.2) — 2026-10-13


## [0.30.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.30.0...trust-tasks-tsp-v0.30.1) — 2026-10-12


## [0.30.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.29.0...trust-tasks-tsp-v0.30.0) — 2026-10-11


## [0.29.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.28.1...trust-tasks-tsp-v0.29.0) — 2026-10-11


## [0.28.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.28.0...trust-tasks-tsp-v0.28.1) — 2026-10-11


## [0.28.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.8...trust-tasks-tsp-v0.28.0) — 2026-10-11


## [0.27.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.7...trust-tasks-tsp-v0.27.8) — 2026-10-10


## [0.27.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.6...trust-tasks-tsp-v0.27.7) — 2026-10-05


## [0.27.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.5...trust-tasks-tsp-v0.27.6) — 2026-10-05


## [0.27.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.4...trust-tasks-tsp-v0.27.5) — 2026-10-05


## [0.27.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.3...trust-tasks-tsp-v0.27.4) — 2026-10-03


## [0.27.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.2...trust-tasks-tsp-v0.27.3) — 2026-10-03


## [0.27.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.1...trust-tasks-tsp-v0.27.2) — 2026-10-03


## [0.27.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.27.0...trust-tasks-tsp-v0.27.1) — 2026-10-03


## [0.27.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.26.3...trust-tasks-tsp-v0.27.0) — 2026-10-03


## [0.26.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.26.2...trust-tasks-tsp-v0.26.3) — 2026-10-02


## [0.26.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.26.1...trust-tasks-tsp-v0.26.2) — 2026-10-02


## [0.26.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.26.0...trust-tasks-tsp-v0.26.1) — 2026-10-02


## [0.26.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.25.4...trust-tasks-tsp-v0.26.0) — 2026-10-01


## [0.25.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.25.3...trust-tasks-tsp-v0.25.4) — 2026-10-01


## [0.25.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.25.2...trust-tasks-tsp-v0.25.3) — 2026-10-01


## [0.25.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.25.1...trust-tasks-tsp-v0.25.2) — 2026-10-01


## [0.25.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.25.0...trust-tasks-tsp-v0.25.1) — 2026-09-30


## [0.25.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.9...trust-tasks-tsp-v0.25.0) — 2026-09-30


## [0.24.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.8...trust-tasks-tsp-v0.24.9) — 2026-09-30


## [0.24.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.7...trust-tasks-tsp-v0.24.8) — 2026-09-29


## [0.24.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.6...trust-tasks-tsp-v0.24.7) — 2026-09-29


## [0.24.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.5...trust-tasks-tsp-v0.24.6) — 2026-09-28


## [0.24.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.4...trust-tasks-tsp-v0.24.5) — 2026-09-28


## [0.24.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.3...trust-tasks-tsp-v0.24.4) — 2026-09-28


## [0.24.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.2...trust-tasks-tsp-v0.24.3) — 2026-09-28


## [0.24.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.1...trust-tasks-tsp-v0.24.2) — 2026-09-27


## [0.24.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.24.0...trust-tasks-tsp-v0.24.1) — 2026-09-27


## [0.24.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.6...trust-tasks-tsp-v0.24.0) — 2026-09-27


## [0.23.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.5...trust-tasks-tsp-v0.23.6) — 2026-09-27


## [0.23.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.4...trust-tasks-tsp-v0.23.5) — 2026-09-27


## [0.23.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.3...trust-tasks-tsp-v0.23.4) — 2026-09-27


## [0.23.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.2...trust-tasks-tsp-v0.23.3) — 2026-09-26


## [0.23.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.1...trust-tasks-tsp-v0.23.2) — 2026-09-26


## [0.23.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.23.0...trust-tasks-tsp-v0.23.1) — 2026-09-26


## [0.23.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.7...trust-tasks-tsp-v0.23.0) — 2026-09-26


## [0.22.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.6...trust-tasks-tsp-v0.22.7) — 2026-09-24


## [0.22.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.5...trust-tasks-tsp-v0.22.6) — 2026-09-24


## [0.22.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.4...trust-tasks-tsp-v0.22.5) — 2026-09-24


## [0.22.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.3...trust-tasks-tsp-v0.22.4) — 2026-09-24


## [0.22.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.2...trust-tasks-tsp-v0.22.3) — 2026-09-23


## [0.22.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.1...trust-tasks-tsp-v0.22.2) — 2026-09-23


## [0.22.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.22.0...trust-tasks-tsp-v0.22.1) — 2026-09-23


## [0.22.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.21...trust-tasks-tsp-v0.22.0) — 2026-09-23


## [0.21.21](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.20...trust-tasks-tsp-v0.21.21) — 2026-09-22


## [0.21.20](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.19...trust-tasks-tsp-v0.21.20) — 2026-09-22


## [0.21.19](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.18...trust-tasks-tsp-v0.21.19) — 2026-09-22


## [0.21.18](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.17...trust-tasks-tsp-v0.21.18) — 2026-09-22


## [0.21.17](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.16...trust-tasks-tsp-v0.21.17) — 2026-09-22


## [0.21.16](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.15...trust-tasks-tsp-v0.21.16) — 2026-09-22


## [0.21.15](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.14...trust-tasks-tsp-v0.21.15) — 2026-09-22


## [0.21.14](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.12...trust-tasks-tsp-v0.21.14) — 2026-09-22


## [0.21.13](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.12...trust-tasks-tsp-v0.21.13) — 2026-09-22


## [0.21.12](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.11...trust-tasks-tsp-v0.21.12) — 2026-09-21


## [0.21.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.10...trust-tasks-tsp-v0.21.11) — 2026-09-21


## [0.21.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.9...trust-tasks-tsp-v0.21.10) — 2026-09-21


## [0.21.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.8...trust-tasks-tsp-v0.21.9) — 2026-09-21


## [0.21.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.7...trust-tasks-tsp-v0.21.8) — 2026-09-21


## [0.21.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.6...trust-tasks-tsp-v0.21.7) — 2026-09-21


## [0.21.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.5...trust-tasks-tsp-v0.21.6) — 2026-09-21


## [0.21.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.4...trust-tasks-tsp-v0.21.5) — 2026-09-20


## [0.21.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.3...trust-tasks-tsp-v0.21.4) — 2026-09-19


## [0.21.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.2...trust-tasks-tsp-v0.21.3) — 2026-09-17


## [0.21.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.1...trust-tasks-tsp-v0.21.2) — 2026-09-16


## [0.21.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.21.0...trust-tasks-tsp-v0.21.1) — 2026-09-16


## [0.21.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.7...trust-tasks-tsp-v0.21.0) — 2026-09-16


### Added

- **tsp**: The Trust Tasks TSP binding speaks spec Rev 3 ([#466](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/466))

`affinidi-tsp` 0.1 -> 0.2. Rev 3 and Rev 2 do not interoperate in either
  direction, so a consumer of this binding moves with its peers or goes silent —
  hence the `!`.

  ## The API change is one idea, applied four times

  HPKE-**Base** replaces HPKE-Auth, and Base does not use the sender's KEM key.
  So `pack`, `pack_nested`, `pack_routed` and `unpack` each lost their
  sender-encryption-key argument: packing now needs only the sender's *signing*
  secret. Nothing else in `pack.rs` moved.

  `next_hop` also now takes the whole `UnpackedMessage` rather than a payload
  slice, because Rev 3 carries the route in the payload frame.

  ## Two tests asserted a leak that Rev 3 closes

  `nested_roundtrip_through_intermediary` and `routed_roundtrips_through_a_relay`
  checked that a keys-free `MetaEnvelope::parse` reports `Nested` / `Routed`.
  That was true in Rev 2, which carried the message type in the *cleartext*
  envelope — so any relay could tell a nested message from a direct one without
  holding a key.

  Rev 3 encrypts it. `MetaEnvelope::message_type` is documented as a keys-free
  placeholder that always reads `Direct`, and a relay needing the real kind must
  open the message. Both tests now assert exactly that, and say why: the old
  assertion was asserting the leak, and the nested test's own comment — "bob's
  identity stays hidden from anyone but the intermediary" — is better served by
  the new behaviour than by the one it was checking.

  ## The dependency refresh this needed, and why it is in the same commit

  Bumping `affinidi-tsp` alone does not build. It pulls `affinidi-did-common`
  0.4, and this workspace's lockfile held a stale patch of nearly every
  `affinidi-*` crate, several of which were pinned down by dev-dependencies that
  had not moved in a long time:

    trust-tasks-didcomm  affinidi-tdk 0.7 -> 0.15, messaging-sdk 0.18 -> 0.25,
                         messaging-test-mediator 0.2 -> 0.8
    trust-tasks-proof    affinidi-did-common 0.3 -> 0.4

  Those three dev-pins were holding `affinidi-did-resolver-cache-sdk` at 0.8.13
  when 0.8.37 was current, which in turn kept the `affinidi-did-common` 0.3 line
  alive beside the 0.4 one. With them moved, the whole family floats to its
  current patches and **no `affinidi-*` crate is duplicated in the graph any
  more** — which is the property that matters here, since two `affinidi-tsp`
  nodes would mean two TSP revisions in one binary.

  Worth recording because the diagnosis was wrong twice before it was right. The
  compile error is `JWK::new` / `OctectParams::new` not found *in
  affinidi-did-common*, which reads as "did-common is broken" and is not: it was
  an `affinidi-crypto` patch old enough to predate those constructors. A `cargo
  check` that fails inside a dependency is not evidence about that dependency.

  Unblocks `verifiable-trust-infrastructure`, which cannot cut over to Rev 3
  while this binding still resolves `affinidi-tsp` 0.1 beside the 0.2 its own
  dependencies now pull.

  44 test suites green under `--no-fail-fast`; clippy clean under `-D warnings`;
  `cargo fmt --all --check` clean.



## [0.20.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.6...trust-tasks-tsp-v0.20.7) — 2026-09-15


## [0.20.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.5...trust-tasks-tsp-v0.20.6) — 2026-09-15


## [0.20.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.4...trust-tasks-tsp-v0.20.5) — 2026-09-14


## [0.20.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.3...trust-tasks-tsp-v0.20.4) — 2026-09-11


## [0.20.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.2...trust-tasks-tsp-v0.20.3) — 2026-09-10


## [0.20.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.1...trust-tasks-tsp-v0.20.2) — 2026-09-10


## [0.20.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.20.0...trust-tasks-tsp-v0.20.1) — 2026-09-10


## [0.20.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.19.4...trust-tasks-tsp-v0.20.0) — 2026-09-10


## [0.19.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.19.3...trust-tasks-tsp-v0.19.4) — 2026-09-10


## [0.19.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.19.2...trust-tasks-tsp-v0.19.3) — 2026-09-09


## [0.19.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.19.1...trust-tasks-tsp-v0.19.2) — 2026-09-09


## [0.19.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.19.0...trust-tasks-tsp-v0.19.1) — 2026-09-09


## [0.19.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.11...trust-tasks-tsp-v0.19.0) — 2026-09-09


## [0.18.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.10...trust-tasks-tsp-v0.18.11) — 2026-09-09


## [0.18.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.9...trust-tasks-tsp-v0.18.10) — 2026-09-09


## [0.18.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.8...trust-tasks-tsp-v0.18.9) — 2026-09-09


## [0.18.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.7...trust-tasks-tsp-v0.18.8) — 2026-09-08


## [0.18.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.6...trust-tasks-tsp-v0.18.7) — 2026-09-08


## [0.18.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.5...trust-tasks-tsp-v0.18.6) — 2026-09-08


## [0.18.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.4...trust-tasks-tsp-v0.18.5) — 2026-09-08


## [0.18.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.3...trust-tasks-tsp-v0.18.4) — 2026-09-07


## [0.18.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.2...trust-tasks-tsp-v0.18.3) — 2026-09-07


## [0.18.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.1...trust-tasks-tsp-v0.18.2) — 2026-09-07


## [0.18.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.18.0...trust-tasks-tsp-v0.18.1) — 2026-09-06


## [0.18.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.10...trust-tasks-tsp-v0.18.0) — 2026-09-06


## [0.17.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.9...trust-tasks-tsp-v0.17.10) — 2026-09-05


## [0.17.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.8...trust-tasks-tsp-v0.17.9) — 2026-09-05


## [0.17.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.7...trust-tasks-tsp-v0.17.8) — 2026-09-04


## [0.17.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.5...trust-tasks-tsp-v0.17.7) — 2026-09-02


## [0.17.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.4...trust-tasks-tsp-v0.17.5) — 2026-09-02


## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.3...trust-tasks-tsp-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.2...trust-tasks-tsp-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.1...trust-tasks-tsp-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.17.0...trust-tasks-tsp-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-tsp-v0.14.0...trust-tasks-tsp-v0.17.0) — 2026-08-27


### Changed

- **versioning**: Release the trust-tasks-rs-exposing crates in lockstep ([#315](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/315))


## [0.14.0] - 2026-08-26

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.14, whose generated payload types
  are `#[non_exhaustive]` and carry builders. Code in this crate's reach that
  built a payload with a struct literal now uses `X::builder()`; a `match` on a
  generated enum needs a wildcard arm. See `trust-tasks-rs` 0.14.0 for the
  migration note. No change to this crate's own API.

## [0.13.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.13`.** That release flips
  `IS_PROOF_REQUIRED` on `vta/memory/list/0.1`'s response, so a consumer rejects
  an unproofed response it used to accept. This crate re-exports the generated
  types, so the leading component moves with it. No change to this crate's own
  API.

## [0.12.0] - 2026-08-26

The duplicate-execution defence of SPEC §7.2 item 11, wired onto this
binding's inbound path and **on by default**. 0.11.0 said a `ReplayGuard`
"remains to be wired"; this release wires it, because a defence every
deployment has to remember to switch on is not a defence.

### Added

- **`TspConsumer` — the guarded inbound path.** `receive` opens a TSP-sealed
  message under every rule `unpack_trust_task` applies (HPKE authenticated
  decryption, signature verification, the `Direct`-carriage check, the
  cleartext-sender cross-check, the envelope `type` check) and then runs
  `consume_inbound` over the document with a `ReplayGuard` and a
  `FreshnessPolicy` already in place; `consume` applies the same pipeline to a
  document another TSP stack unsealed. Every verdict of §7.2's *Disposition of
  a duplicate* is applied: a duplicate returns the prior response and does not
  re-dispatch, an in-flight duplicate reports the running execution rather than
  starting another, a differing document under a reused `id` is `idConflict`,
  and a guard that cannot answer fails closed as `unavailable` with
  `retryable = true` — never by executing.

  **The record is keyed on the document `id`, never on the TSP envelope.** SPEC
  §7.2 forbids substituting a transport identifier, and on this transport the
  point is stark: sealing the same document again yields an envelope sharing no
  bytes with the first, because TSP derives fresh ephemeral material per
  message. There is nothing about the envelope a record *could* key on and
  still absorb a re-send — which binding §5.2's routed and nested carriage
  makes an ordinary event, since each intermediary may hold and re-forward the
  sealed inner message.

### Changed

- **BREAKING — the duplicate-execution record defaults ON.** `TspConsumer`
  keeps an in-process `InMemoryReplayGuard` and applies
  `FreshnessPolicy::consequential` (`issuedAt` REQUIRED, five-minute acceptance
  window). A consumer that previously accepted an undated document, or the same
  document twice, no longer does — a change to what a consumer observes, so the
  leading component moves.

  `with_replay_guard` takes a store shared across replicas;
  `without_replay_record` is the explicit opt-out, documented with what it
  re-opens; `with_freshness` widens the acceptance window — and with it the
  record's retention, because §7.2 makes them one bound. There is no second
  TTL.

- `chrono` is now a direct dependency (the pipeline takes a `now`), and
  `tokio` / `async-trait` are dev-dependencies (the tests drive an async
  pipeline and implement a failing guard).

## [0.11.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.12`** (SPEC §7.2 item 11 duplicate
  execution, item 13 freshness). Leading component moves with the re-exported
  types. A `ReplayGuard` keyed on the *document* `id` remains to be wired.

## [0.8.0] - 2026-08-16

### Changed

- Requires `trust-tasks-rs` 0.8, which adds the `cancelled` standard error code
  (framework 0.4, SPEC §8.3) and the `trust-task-control/0.1` payload types.
  Additive on the Rust side — `StandardCode` has been `#[non_exhaustive]` since
  0.7.0 — so this crate needed no source change.

## [0.7.0] - 2026-08-15

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.7, whose `StandardCode` is now
  `#[non_exhaustive]` and carries the new `idConflict` code (framework 0.4,
  SPEC §8.3). Any `match` over `StandardCode` that this crate's types reach
  needs a wildcard arm.

## [0.6.0] - 2026-08-10

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.6, which narrows `DigestMultibase`
  to the multibase headers CID 1.0 requires. The core types cross this crate's
  public API, so a graph mixing 0.5 with this crate will not type-check. No API
  of this crate changed on its own account.

## [0.5.0] - 2026-08-09

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.5. That release adds a field to
  `TrustTask<P>` for the framework 0.4 `ceremony` member, and the core types
  cross this crate's public API, so a dependency graph mixing 0.4 with this
  crate will not type-check. No API of this crate changed on its own account.

## [0.4.0] - 2026-08-09

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.4. That release changes digest
  payload members from `String` to the validating `DigestMultibase` newtype, and
  the core types cross this crate's public API, so a dependency graph mixing
  `trust-tasks-rs` 0.3 with this crate will not type-check. No API of this crate
  changed on its own account.

## [0.2.2] — 2026-06-25

### Added

- `pack_trust_task_routed(doc, sender, recipient, first_hop, onward_route)` — producer-side
  **Routed** carriage (SPEC binding §5): seals the Trust Task `Direct` to the final
  `recipient`, then wraps it in a `Routed` message relayed through one or more hops (the
  full path is `[first_hop, ..onward_route]`). Completes the binding's carriage set —
  Direct / Nested / Routed. As with nested carriage, the consumer side
  (`unpack_trust_task`) is unchanged; it still opens the innermost `Direct`.

## [0.2.1] — 2026-06-24

### Added

- `pack_trust_task_nested(doc, sender, recipient, intermediary)` — producer-side
  **Nested** carriage (SPEC binding §5): seals the Trust Task `Direct` to the final
  `recipient`, then wraps it in an outer `Nested` envelope sealed to `intermediary`
  (a metadata-privacy wrapper). The messaging mediator unwraps its outer layer and
  forwards the inner `Direct`; the consumer side (`unpack_trust_task`) is unchanged —
  it still opens the innermost `Direct` regardless of carriage.

## [0.2.0] — 2026-06-23

Initial release: the ToIP Trust Spanning Protocol (TSP) transport binding for
the Trust Tasks framework — binding `https://trusttasks.org/binding/tsp/0.1`,
built on `affinidi-tsp` 0.1.

### Added

- `pack_trust_task` / `unpack_trust_task` — seal a `TrustTask<P>` into a TSP
  `Direct` message (HPKE authenticated encryption + Ed25519 signature) and open
  it again, framing the document in the binding envelope object
  (`{ "type": …, "document": … }`).
- `TspHandler` — a `TransportHandler` that surfaces the authenticated `VID_sndr`
  as the framework's transport-authenticated `issuer` and the `VID_rcvr` as the
  `recipient`, feeding SPEC §4.8.1 precedence. A TSP VID is a framework VID
  verbatim — no normalisation, exact string equality.
- `TspError` with `into_reject_reason()` for folding transport failures into the
  framework's `RejectReason`.
- `BINDING_URI` and `ENVELOPE_TYPE` constants.

This release covers **Direct** carriage. Routed/Nested carriage (binding §5) is
relayed by the messaging mediator on the wire; the consumer opens the innermost
`Direct` message, which this binding unpacks.
