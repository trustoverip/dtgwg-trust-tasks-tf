# Changelog

All notable changes to `trust-tasks-didcomm-v1` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.36.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.36.3...trust-tasks-didcomm-v1-v0.36.4) — 2026-10-30


## [0.36.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.36.2...trust-tasks-didcomm-v1-v0.36.3) — 2026-10-30


## [0.36.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.36.1...trust-tasks-didcomm-v1-v0.36.2) — 2026-10-29


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



## [0.36.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.36.0...trust-tasks-didcomm-v1-v0.36.1) — 2026-10-28


## [0.36.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.35.4...trust-tasks-didcomm-v1-v0.36.0) — 2026-10-27


## [0.35.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.35.3...trust-tasks-didcomm-v1-v0.35.4) — 2026-10-27


### Added

- The prebuilt specification modules are deprecated in all four libraries; generate the tasks you use ([#954](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/954))

Milestone 5 of docs/design/consumer-side-generation.md. Every prebuilt module
  outside the framework's own specifications (trust-task-*, trust-ceremony-*,
  _framework/) is deprecated, so that an application is told to pin the tasks it
  uses with the trust-tasks CLI and generate them instead. Nothing is removed:
  the withdrawal batch that removes them is chosen later, at least 90 days after
  this ships.

  Each library marks the prebuilt entry point where its toolchain reports a
  deprecation, never the module body, which is also what the consumer-side
  generators emit:

  - Rust: #[deprecated] on each gated family module in trust-tasks-rs's
    specs/mod.rs (trust-tasks-codegen). It reaches every path and item under the
    module; framework modules and trust-tasks-build's output are unmarked.
  - TypeScript: the root index re-exports each module as
    `import * as _X …; /** @deprecated … */ export import X = _X;`, because
    TypeScript ignores @deprecated on `export * as`. API compatibility against
    0.27.8: nothing removed or changed. A deep import of a module file is not
    marked; TypeScript has no module-level deprecation.
  - Go: a `Deprecated:` paragraph in each prebuilt package's doc, which gopls and
    staticcheck (SA1019) report on import.
  - Dart: @Deprecated on each prebuilt library's `library` directive, which the
    analyzer reports on import from another package. This package's own tests
    use them, so analysis_options ignores the same-package diagnostic SDKs
    before 3.4 report.

  The TS, Go and Dart equivalence tests mask exactly the mark; every consumer
  generator is still held to the prebuilt tree as before. Hand-written Rust that
  uses prebuilt modules on purpose — the acl-specimen test modules, the
  integration tests and the examples — carries #[allow(deprecated)]; none of the
  runtime does.

  GETTING-STARTED, the four library READMEs, the design doc and CLAUDE.md say
  so, and point at each language's generator.



## [0.35.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.35.2...trust-tasks-didcomm-v1-v0.35.3) — 2026-10-26


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



## [0.35.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.35.1...trust-tasks-didcomm-v1-v0.35.2) — 2026-10-26


## [0.35.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.35.0...trust-tasks-didcomm-v1-v0.35.1) — 2026-10-26


## [0.35.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.34.2...trust-tasks-didcomm-v1-v0.35.0) — 2026-10-26


### Added

- **codegen**: Superseded and retired spec versions are deprecated in every library, and withdrawn ones leave them in a quarterly batch ([#921](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/921))

* feat(go): mark superseded and retired spec versions deprecated, and omit withdrawn ones

  Every generated request package now exports `const Deprecation` (empty for a
  current version, otherwise "superseded by <slug>/<M.N>", "retired; use
  <slug>/<M.N>" or "retired"), and a deprecated version's Payload and Response
  types carry a `// Deprecated:` paragraph. A version with `status: withdrawn`
  gets no package at all. The rule is derived independently from front matter,
  per SPEC "Superseded and Withdrawn Versions".



## [0.34.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.34.1...trust-tasks-didcomm-v1-v0.34.2) — 2026-10-24


## [0.34.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.34.0...trust-tasks-didcomm-v1-v0.34.1) — 2026-10-23


## [0.34.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.33.2...trust-tasks-didcomm-v1-v0.34.0) — 2026-10-23


### Fixed

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



## [0.33.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.33.1...trust-tasks-didcomm-v1-v0.33.2) — 2026-10-22


## [0.33.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.33.0...trust-tasks-didcomm-v1-v0.33.1) — 2026-10-21


## [0.33.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.10...trust-tasks-didcomm-v1-v0.33.0) — 2026-10-21


## [0.32.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.9...trust-tasks-didcomm-v1-v0.32.10) — 2026-10-21


## [0.32.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.8...trust-tasks-didcomm-v1-v0.32.9) — 2026-10-21


## [0.32.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.7...trust-tasks-didcomm-v1-v0.32.8) — 2026-10-21


## [0.32.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.6...trust-tasks-didcomm-v1-v0.32.7) — 2026-10-20


## [0.32.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.5...trust-tasks-didcomm-v1-v0.32.6) — 2026-10-20


## [0.32.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.4...trust-tasks-didcomm-v1-v0.32.5) — 2026-10-20


## [0.32.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.3...trust-tasks-didcomm-v1-v0.32.4) — 2026-10-19


## [0.32.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.2...trust-tasks-didcomm-v1-v0.32.3) — 2026-10-19


## [0.32.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.1...trust-tasks-didcomm-v1-v0.32.2) — 2026-10-19


## [0.32.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.32.0...trust-tasks-didcomm-v1-v0.32.1) — 2026-10-19


## [0.32.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.16...trust-tasks-didcomm-v1-v0.32.0) — 2026-10-18


## [0.31.16](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.15...trust-tasks-didcomm-v1-v0.31.16) — 2026-10-18


## [0.31.15](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.14...trust-tasks-didcomm-v1-v0.31.15) — 2026-10-17


## [0.31.14](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.13...trust-tasks-didcomm-v1-v0.31.14) — 2026-10-17


## [0.31.13](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.12...trust-tasks-didcomm-v1-v0.31.13) — 2026-10-16


## [0.31.12](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.11...trust-tasks-didcomm-v1-v0.31.12) — 2026-10-16


## [0.31.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.10...trust-tasks-didcomm-v1-v0.31.11) — 2026-10-16


## [0.31.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.9...trust-tasks-didcomm-v1-v0.31.10) — 2026-10-16


## [0.31.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.8...trust-tasks-didcomm-v1-v0.31.9) — 2026-10-16


## [0.31.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.7...trust-tasks-didcomm-v1-v0.31.8) — 2026-10-15


## [0.31.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.6...trust-tasks-didcomm-v1-v0.31.7) — 2026-10-15


## [0.31.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.5...trust-tasks-didcomm-v1-v0.31.6) — 2026-10-15


## [0.31.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.4...trust-tasks-didcomm-v1-v0.31.5) — 2026-10-15


## [0.31.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.3...trust-tasks-didcomm-v1-v0.31.4) — 2026-10-15


## [0.31.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.2...trust-tasks-didcomm-v1-v0.31.3) — 2026-10-15


## [0.31.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.1...trust-tasks-didcomm-v1-v0.31.2) — 2026-10-14


## [0.31.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.31.0...trust-tasks-didcomm-v1-v0.31.1) — 2026-10-14


## [0.31.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.30.2...trust-tasks-didcomm-v1-v0.31.0) — 2026-10-13


## [0.30.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.30.1...trust-tasks-didcomm-v1-v0.30.2) — 2026-10-13


## [0.30.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.30.0...trust-tasks-didcomm-v1-v0.30.1) — 2026-10-12


## [0.30.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.29.0...trust-tasks-didcomm-v1-v0.30.0) — 2026-10-11


## [0.29.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.28.1...trust-tasks-didcomm-v1-v0.29.0) — 2026-10-11


## [0.28.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.28.0...trust-tasks-didcomm-v1-v0.28.1) — 2026-10-11


## [0.28.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.8...trust-tasks-didcomm-v1-v0.28.0) — 2026-10-11


## [0.27.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.7...trust-tasks-didcomm-v1-v0.27.8) — 2026-10-10


## [0.27.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.6...trust-tasks-didcomm-v1-v0.27.7) — 2026-10-05


## [0.27.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.5...trust-tasks-didcomm-v1-v0.27.6) — 2026-10-05


## [0.27.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.4...trust-tasks-didcomm-v1-v0.27.5) — 2026-10-05


## [0.27.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.3...trust-tasks-didcomm-v1-v0.27.4) — 2026-10-03


## [0.27.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.2...trust-tasks-didcomm-v1-v0.27.3) — 2026-10-03


## [0.27.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.1...trust-tasks-didcomm-v1-v0.27.2) — 2026-10-03


## [0.27.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.27.0...trust-tasks-didcomm-v1-v0.27.1) — 2026-10-03


## [0.27.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.26.3...trust-tasks-didcomm-v1-v0.27.0) — 2026-10-03


## [0.26.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.26.2...trust-tasks-didcomm-v1-v0.26.3) — 2026-10-02


## [0.26.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.26.1...trust-tasks-didcomm-v1-v0.26.2) — 2026-10-02


## [0.26.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.26.0...trust-tasks-didcomm-v1-v0.26.1) — 2026-10-02


## [0.26.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.25.4...trust-tasks-didcomm-v1-v0.26.0) — 2026-10-01


## [0.25.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.25.3...trust-tasks-didcomm-v1-v0.25.4) — 2026-10-01


## [0.25.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.25.2...trust-tasks-didcomm-v1-v0.25.3) — 2026-10-01


## [0.25.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.25.1...trust-tasks-didcomm-v1-v0.25.2) — 2026-10-01


## [0.25.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.25.0...trust-tasks-didcomm-v1-v0.25.1) — 2026-09-30


## [0.25.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.9...trust-tasks-didcomm-v1-v0.25.0) — 2026-09-30


## [0.24.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.8...trust-tasks-didcomm-v1-v0.24.9) — 2026-09-30


## [0.24.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.7...trust-tasks-didcomm-v1-v0.24.8) — 2026-09-29


## [0.24.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.6...trust-tasks-didcomm-v1-v0.24.7) — 2026-09-29


## [0.24.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.5...trust-tasks-didcomm-v1-v0.24.6) — 2026-09-28


## [0.24.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.4...trust-tasks-didcomm-v1-v0.24.5) — 2026-09-28


## [0.24.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.3...trust-tasks-didcomm-v1-v0.24.4) — 2026-09-28


## [0.24.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.2...trust-tasks-didcomm-v1-v0.24.3) — 2026-09-28


## [0.24.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.1...trust-tasks-didcomm-v1-v0.24.2) — 2026-09-27


## [0.24.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.24.0...trust-tasks-didcomm-v1-v0.24.1) — 2026-09-27


## [0.24.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.6...trust-tasks-didcomm-v1-v0.24.0) — 2026-09-27


## [0.23.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.5...trust-tasks-didcomm-v1-v0.23.6) — 2026-09-27


## [0.23.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.4...trust-tasks-didcomm-v1-v0.23.5) — 2026-09-27


## [0.23.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.3...trust-tasks-didcomm-v1-v0.23.4) — 2026-09-27


## [0.23.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.2...trust-tasks-didcomm-v1-v0.23.3) — 2026-09-26


## [0.23.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.1...trust-tasks-didcomm-v1-v0.23.2) — 2026-09-26


## [0.23.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.23.0...trust-tasks-didcomm-v1-v0.23.1) — 2026-09-26


## [0.23.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.7...trust-tasks-didcomm-v1-v0.23.0) — 2026-09-26


## [0.22.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.6...trust-tasks-didcomm-v1-v0.22.7) — 2026-09-24


## [0.22.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.5...trust-tasks-didcomm-v1-v0.22.6) — 2026-09-24


## [0.22.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.4...trust-tasks-didcomm-v1-v0.22.5) — 2026-09-24


## [0.22.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.3...trust-tasks-didcomm-v1-v0.22.4) — 2026-09-24


## [0.22.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.2...trust-tasks-didcomm-v1-v0.22.3) — 2026-09-23


## [0.22.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.1...trust-tasks-didcomm-v1-v0.22.2) — 2026-09-23


## [0.22.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.22.0...trust-tasks-didcomm-v1-v0.22.1) — 2026-09-23


## [0.22.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.21...trust-tasks-didcomm-v1-v0.22.0) — 2026-09-23


## [0.21.21](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.20...trust-tasks-didcomm-v1-v0.21.21) — 2026-09-22


## [0.21.20](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.19...trust-tasks-didcomm-v1-v0.21.20) — 2026-09-22


## [0.21.19](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.18...trust-tasks-didcomm-v1-v0.21.19) — 2026-09-22


## [0.21.18](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.17...trust-tasks-didcomm-v1-v0.21.18) — 2026-09-22


## [0.21.17](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.16...trust-tasks-didcomm-v1-v0.21.17) — 2026-09-22


## [0.21.16](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.15...trust-tasks-didcomm-v1-v0.21.16) — 2026-09-22


## [0.21.15](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.14...trust-tasks-didcomm-v1-v0.21.15) — 2026-09-22


## [0.21.14](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.12...trust-tasks-didcomm-v1-v0.21.14) — 2026-09-22


## [0.21.13](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.12...trust-tasks-didcomm-v1-v0.21.13) — 2026-09-22


## [0.21.12](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.11...trust-tasks-didcomm-v1-v0.21.12) — 2026-09-21


## [0.21.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.10...trust-tasks-didcomm-v1-v0.21.11) — 2026-09-21


### Specifications

- Generate SPEC.md from the canonical framework specification ([#559](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/559))

SPEC.md was a hand-ported copy of trustoverip/dtgwg-trust-tasks-spec,
  and had drifted: it lacked #14's versioning sections (Document Status,
  three-part framework versions, ratification), among others. It is now an
  output.

  - scripts/generate-framework-spec.mjs assembles the canonical Spec-Up-T
    sources (specs.json markdown_paths + term definitions), numbers the
    headings in canonical order, rewrites named anchors to numbered ones,
    and emits legacy anchors as aliases. --check fails on drift.
  - scripts/framework-spec-legacy-anchors.json maps every anchor of the old
    hand-maintained copy to its canonical section, so existing links keep
    resolving (generator rejects an alias that collides with a heading).
  - build-registry: every /SPEC.md#… link in spec and binding prose,
    including the bindings' absolute GitHub links, must name a section or
    alias. Found 15 links broken before this change; fixed.
  - deploy.yml regenerates SPEC.md from canonical main before the build, so
    trusttasks.org serves the latest canonical text, and redeploys daily and
    on workflow_dispatch / repository_dispatch (framework-spec-updated).
  - Canonical order moves Terminology to §3, Conformance to §17, and the old
    §10-§13 (Security & Privacy, Discovery, Task control, References) to
    §12/§13, §10, §11 and §18. Qualified references ("SPEC §N", "SPEC.md §N",
    links to SPEC.md) are rewritten across specs, bindings, docs, scripts and
    library comments; bindings regenerated.
  - CLAUDE.md, CONTRIBUTING-SPECS.md, README and the website point framework
    changes at the canonical repository.



## [0.21.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.9...trust-tasks-didcomm-v1-v0.21.10) — 2026-09-21


## [0.21.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.8...trust-tasks-didcomm-v1-v0.21.9) — 2026-09-21


## [0.21.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.7...trust-tasks-didcomm-v1-v0.21.8) — 2026-09-21


## [0.21.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.6...trust-tasks-didcomm-v1-v0.21.7) — 2026-09-21


## [0.21.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.5...trust-tasks-didcomm-v1-v0.21.6) — 2026-09-21


## [0.21.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.4...trust-tasks-didcomm-v1-v0.21.5) — 2026-09-20


## [0.21.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.3...trust-tasks-didcomm-v1-v0.21.4) — 2026-09-19


## [0.21.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.2...trust-tasks-didcomm-v1-v0.21.3) — 2026-09-17


## [0.21.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.1...trust-tasks-didcomm-v1-v0.21.2) — 2026-09-16


## [0.21.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.21.0...trust-tasks-didcomm-v1-v0.21.1) — 2026-09-16


## [0.21.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.7...trust-tasks-didcomm-v1-v0.21.0) — 2026-09-16


## [0.20.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.6...trust-tasks-didcomm-v1-v0.20.7) — 2026-09-15


## [0.20.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.5...trust-tasks-didcomm-v1-v0.20.6) — 2026-09-15


## [0.20.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.4...trust-tasks-didcomm-v1-v0.20.5) — 2026-09-14


## [0.20.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.3...trust-tasks-didcomm-v1-v0.20.4) — 2026-09-11


## [0.20.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.2...trust-tasks-didcomm-v1-v0.20.3) — 2026-09-10


## [0.20.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.1...trust-tasks-didcomm-v1-v0.20.2) — 2026-09-10


## [0.20.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.20.0...trust-tasks-didcomm-v1-v0.20.1) — 2026-09-10


## [0.20.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.19.4...trust-tasks-didcomm-v1-v0.20.0) — 2026-09-10


## [0.19.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.19.3...trust-tasks-didcomm-v1-v0.19.4) — 2026-09-10


## [0.19.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.19.2...trust-tasks-didcomm-v1-v0.19.3) — 2026-09-09


## [0.19.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.19.1...trust-tasks-didcomm-v1-v0.19.2) — 2026-09-09


## [0.19.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.19.0...trust-tasks-didcomm-v1-v0.19.1) — 2026-09-09


## [0.19.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.11...trust-tasks-didcomm-v1-v0.19.0) — 2026-09-09


## [0.18.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.10...trust-tasks-didcomm-v1-v0.18.11) — 2026-09-09


## [0.18.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.9...trust-tasks-didcomm-v1-v0.18.10) — 2026-09-09


## [0.18.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.8...trust-tasks-didcomm-v1-v0.18.9) — 2026-09-09


## [0.18.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.7...trust-tasks-didcomm-v1-v0.18.8) — 2026-09-08


## [0.18.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.6...trust-tasks-didcomm-v1-v0.18.7) — 2026-09-08


## [0.18.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.5...trust-tasks-didcomm-v1-v0.18.6) — 2026-09-08


## [0.18.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.4...trust-tasks-didcomm-v1-v0.18.5) — 2026-09-08


## [0.18.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.3...trust-tasks-didcomm-v1-v0.18.4) — 2026-09-07


## [0.18.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.2...trust-tasks-didcomm-v1-v0.18.3) — 2026-09-07


## [0.18.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.1...trust-tasks-didcomm-v1-v0.18.2) — 2026-09-07


## [0.18.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.18.0...trust-tasks-didcomm-v1-v0.18.1) — 2026-09-06


## [0.18.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.10...trust-tasks-didcomm-v1-v0.18.0) — 2026-09-06


## [0.17.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.9...trust-tasks-didcomm-v1-v0.17.10) — 2026-09-05


## [0.17.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.8...trust-tasks-didcomm-v1-v0.17.9) — 2026-09-05


## [0.17.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.7...trust-tasks-didcomm-v1-v0.17.8) — 2026-09-04


## [0.17.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.5...trust-tasks-didcomm-v1-v0.17.7) — 2026-09-02


## [0.17.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.4...trust-tasks-didcomm-v1-v0.17.5) — 2026-09-02


## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.3...trust-tasks-didcomm-v1-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.2...trust-tasks-didcomm-v1-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.1...trust-tasks-didcomm-v1-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.17.0...trust-tasks-didcomm-v1-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v1-v0.14.0...trust-tasks-didcomm-v1-v0.17.0) — 2026-08-27


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
deployment has to remember to switch on is not a defence — and on v1 there is
nothing else: binding §6 records the transport's freshness guarantee as
"**None**".

### Added

- **`DidcommV1Consumer` — the guarded inbound path.** `receive` takes an
  `UnpackResult`, applies the authenticated-sender gate, the §2/§2.3 carriage
  gate, the attachment lookup and the `~thread` cross-check exactly as
  `unpack_trust_task` does, then runs `consume_inbound` over the document with
  a `ReplayGuard` and a `FreshnessPolicy` already in place. `consume` applies
  the same pipeline to a document an Aries framework's own delivery loop
  extracted. Every verdict of §7.2's *Disposition of a duplicate* is applied:
  a duplicate returns the prior response and does not re-dispatch, an in-flight
  duplicate reports the running execution rather than starting another, a
  differing document under a reused `id` is `idConflict`, and a guard that
  cannot answer fails closed as `unavailable` with `retryable = true` — never
  by executing.

  **The record is keyed on the document `id`, never on the v1 message `@id` or
  the `~thread` decorator.** SPEC §7.2 forbids the substitution, and
  `MessageV1`'s own documentation makes the same point: the `@id` is the
  *transport* identifier, with a different lifetime. A mediator redelivery is a
  new v1 message — `MessageV1::new` mints a fresh UUID `@id` — carrying the
  same document, so a record keyed on `@id` would admit it and grant twice.
  `tests/replay.rs` asserts the two messages' `@id`s differ before feeding both
  through the path and asserting the handler ran once.

### Changed

- **BREAKING — the duplicate-execution record defaults ON.**
  `DidcommV1Consumer` keeps an in-process `InMemoryReplayGuard` and applies
  `FreshnessPolicy::consequential` (`issuedAt` REQUIRED, five-minute acceptance
  window). A consumer that previously accepted an undated document, or the same
  document twice, no longer does — a change to what a consumer observes, so the
  leading component moves.

  `with_replay_guard` takes a store shared across replicas;
  `without_replay_record` is the explicit opt-out, documented with what it
  re-opens; `with_freshness` widens the acceptance window — and with it the
  record's retention, because §7.2 makes them one bound. There is no second
  TTL.

- `chrono` is now a direct dependency (the pipeline takes a `now`), and the
  crate has dev-dependencies for the first time (`tokio`, `async-trait`) so the
  inbound path can be tested at all.

## [0.11.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.12`** (SPEC §7.2 item 11 duplicate
  execution, item 13 freshness). Leading component moves with the re-exported
  types. As with the v2 binding, a `ReplayGuard` keyed on the *document* `id`
  remains to be wired.

## [0.10.0] - 2026-08-26

### Added

- **The `legacy-basic-message` feature — a sunset for the `0.1` carriage.**
  Binding `didcomm-v1/0.2` §2.3 requires a `0.2` consumer to accept `0.1`'s
  Aries `basic-message` carriage as well as this binding's dedicated message
  type. `basic-message` is the Aries **chat** type, so while that gate is open
  any chat message from any established connection that carries a `trust-task`
  attachment is a framework input — from every peer, with no end date. §2.3
  puts the contraction in a future MAJOR but gave implementations no way to
  reach it early.

  The legacy carriage now sits behind a Cargo feature. It is **on by default**,
  because §2.3 makes accepting it a MUST and nothing in this repository depends
  on it either way; turning it off is a deliberate departure from that MUST,
  available to a deployment that knows all its peers have migrated:

  ```toml
  trust-tasks-didcomm-v1 = { version = "0.10", default-features = false }
  ```

  With the feature off, such a message is `DidcommV1Error::WrongMessageType`.

- **The legacy carriage is surfaced as superseded**, which is §2.3's SHOULD and
  was previously not done at all: every message arriving on it is logged at
  `warn` through the `log` facade (naming the sender, so an operator can see
  *which* peers have not migrated), and reported as
  `Carriage::LegacyBasicMessage` on the handler for callers that would rather
  meter it than grep logs. `DidcommV1Handler::carriage()` is the accessor;
  `with_carriage` sets it.

- `log` is a new dependency (the facade only — a binary that installs no logger
  pays nothing).

### Notes

Default-on means the default build behaves exactly as 0.9.0 did, so this is a
minor bump. The observable additions are the `Carriage` enum and the handler
accessor.

## [0.9.0] and earlier

See the repository history; this crate kept no changelog before 0.10.0.
