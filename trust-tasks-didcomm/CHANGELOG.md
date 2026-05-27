# Changelog

All notable changes to `trust-tasks-didcomm` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

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

[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
