# Changelog

All notable changes to `trust-tasks-tsp` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate versions independently of `trust-tasks-rs` — it takes its own
leading bump when a `trust-tasks-rs` break reaches it, rather than aligning
to that crate's number (see the `0.6.5` → `0.7.0` release for the shape).

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
