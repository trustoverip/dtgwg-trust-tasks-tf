# Changelog

All notable changes to `trust-tasks-tsp` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

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
