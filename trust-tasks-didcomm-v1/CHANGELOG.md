# Changelog

All notable changes to `trust-tasks-didcomm-v1` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
