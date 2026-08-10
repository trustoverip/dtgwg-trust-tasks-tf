# Changelog

All notable changes to `trust-tasks-didcomm` are documented in this file.
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

## [0.1.3] — 2026-06-01

### Changed

- **Upgraded `affinidi-messaging-didcomm` 0.13 → 0.14.** This is a public
  dependency bump — the binding re-exposes didcomm's `Message`,
  `DIDCommAgent`, `UnpackResult`, and `DIDCommError` in its API, so
  consumers that pin their own `affinidi-messaging-didcomm` should move to
  0.14 alongside this release.
- `unpack_trust_task` now handles didcomm 0.14's enlarged
  `UnpackResult::Encrypted`. The new `legacy_kek_used` (pre-0.14 ECDH-1PU
  KEK migration signal), `non_repudiation`, and inner-JWS `signer_kid`
  fields are accepted but not yet acted on: the SPEC §4.8.1
  transport-authenticated sender remains the authcrypt `sender_kid`. The
  enum is now `#[non_exhaustive]`, and any unrecognised variant fails
  closed with `DidcommError::UnauthenticatedSender`.
- Tracks `trust-tasks-rs` 0.1.4.

## [0.1.2] — 2026-05-27

### Changed

- Track `trust-tasks-rs` 0.1.2. No public API changes in this crate;
  the bump exists so DIDComm consumers can `cargo update -p
  trust-tasks-didcomm` and pick up the new spec families
  (`did-management/*`, `webvh/*`, `vault/*`, `device/*`, `policy/*`,
  `provision/integration`, etc.) over DIDComm without further
  dependency surgery.

## [0.1.0] — initial pre-release, tracks `SPEC.md` 0.1

### Added

- DIDComm v2.1 transport binding for the Trust Tasks framework
  (SPEC §9). Binding URI:
  `https://trusttasks.org/binding/didcomm/0.1`. Envelope `type`:
  `https://trusttasks.org/binding/didcomm/0.1/envelope`.
- `DidcommHandler` — `TransportHandler` impl that surfaces the
  authcrypt-verified sender DID as the framework's transport peer
  for SPEC §4.8.1 cross-check.
- `pack_trust_task(doc, agent, sender_did, recipient_did)` — wraps
  a typed `TrustTask<P>` in a DIDComm v2.1 `Message` carrying
  `ENVELOPE_TYPE`, then authcrypts via
  `affinidi-messaging-didcomm`'s `DIDCommAgent::pack_authcrypt`.
- `unpack_trust_task::<P>(wire, agent, expected_sender)` — unpacks
  and verifies the envelope, rejects anoncrypt / plaintext via
  `DidcommError::UnauthenticatedSender`, derives the `DidcommHandler`
  from the verified `sender_kid`.
- `DidcommError::into_reject_reason()` maps envelope-level failures
  into the framework's `RejectReason` taxonomy.
- `ENVELOPE_TYPE` public const so other DIDComm-aware tooling can
  route on the same identifier.
- `examples/local_roundtrip.rs` — full alice ↔ bob loop in-process.
- `tests/end_to_end.rs` — four scenarios over the bare
  `DIDCommAgent` (happy path, forged in-band issuer surfaces as
  `IssuerMismatch`, wire envelope shape, wrong envelope type).
- `tests/mediator_e2e.rs` (`#[ignore]`) — integration test against
  `affinidi-messaging-test-mediator`: spawns the full mediator
  stack, registers two `did:peer` users as LOCAL on the mediator's
  identity store, packs/unpacks via `ATM` (`affinidi-messaging-sdk`
  0.18) carrying the framework `ENVELOPE_TYPE`, asserts the verified
  sender from `UnpackMetadata` slots into `DidcommHandler::peer()`.

[0.1.3]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/trust-tasks-didcomm-v0.1.3
[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
