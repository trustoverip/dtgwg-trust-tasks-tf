# Changelog

All notable changes to `trust-tasks-tsp` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

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
