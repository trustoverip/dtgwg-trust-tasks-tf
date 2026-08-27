# Changelog

All notable changes to `trust-tasks-tsp` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate versions independently of `trust-tasks-rs` — it takes its own
leading bump when a `trust-tasks-rs` break reaches it, rather than aligning
to that crate's number (see the `0.6.5` → `0.7.0` release for the shape).

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
