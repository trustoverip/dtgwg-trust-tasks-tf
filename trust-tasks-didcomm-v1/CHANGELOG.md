# Changelog

All notable changes to `trust-tasks-didcomm-v1` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
