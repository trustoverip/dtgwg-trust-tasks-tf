# Changelog

All notable changes to `trust-tasks-didcomm` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate versions independently of `trust-tasks-rs` — it takes its own
leading bump when a `trust-tasks-rs` break reaches it, rather than aligning
to that crate's number (see the `0.6.5` → `0.7.0` release for the shape).

## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v0.17.3...trust-tasks-didcomm-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v0.17.2...trust-tasks-didcomm-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v0.17.1...trust-tasks-didcomm-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v0.17.0...trust-tasks-didcomm-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-didcomm-v0.15.0...trust-tasks-didcomm-v0.17.0) — 2026-08-27


### Changed

- **versioning**: Release the trust-tasks-rs-exposing crates in lockstep ([#315](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/315))


### Specifications

- Bound every free-text payload member with a maxLength (§7.3) ([#296](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/296))

* spec: bound every free-text payload member with a maxLength

  SPEC.md §7.3 (framework 0.5.0) requires that any member holding free
  text declare a `maxLength`. 92 free-text string members across 83 draft
  schemas carried none, leaving the wire contract unbounded and every
  consumer to invent its own ceiling — or none, which is what §10.3
  (schema-validation DoS) exists to prevent.

  Bounds are chosen per member from the vocabulary the registry already
  uses rather than applied uniformly:

    256   `label`, `comment` — a display name or an OpenSSH key comment;
          matches the existing 256 on provision/integration `label` and
          the `name` members alongside it.
    500   requester-authored prose that a surface renders to a human who
          is deciding something; matches task-consent/request/0.1 `note`,
          the registry's considered consent-surface bound.
    1024  `reason`, `description`, `message` — operator or service prose
          recorded for audit or returned as a diagnostic; matches the six
          existing `reason: 1024` and the `description: 1024` in policy/
          and vtc/endorsement-type.
    16384 chat/message `text` — the task's actual content rather than
          metadata about it; matches the corpus's long-form bound on
          vault `secureNotes`.

  All amended specifications are `status: draft`, so the change is made in
  place per SPEC §5.2. Deliberately untouched:

    * 17 members in `retired` specifications, frozen by SPEC §6.4.
    * messaging/_shared/0.1 `AuditEntry.detail` and did-management/
      _shared/0.1 `DomainEntry.label` — shared $defs reachable from a
      retired specification, so bounding them would change a frozen
      specification's effective wire contract.
    * vault/_shared/{0.1,0.2,0.3} `TspMessageEnvelope.message` — opaque
      base64url TSP bytes, not free text.

  The `label` description in vault/_shared/*/vault-entry.schema.json said
  the wire spec enforced no maximum length. It now does, so the sentence
  is corrected rather than left contradicting the schema it annotates.

  `npm run validate` re-checks all 533 fenced example documents against
  the amended schemas; none is rejected by a new bound.



## [0.15.0] - 2026-08-26

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.14, whose generated payload types
  are `#[non_exhaustive]` and carry builders. Code in this crate's reach that
  built a payload with a struct literal now uses `X::builder()`; a `match` on a
  generated enum needs a wildcard arm. See `trust-tasks-rs` 0.14.0 for the
  migration note. No change to this crate's own API.

## [0.14.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.13`.** That release flips
  `IS_PROOF_REQUIRED` on `vta/memory/list/0.1`'s response, so a consumer rejects
  an unproofed response it used to accept. This crate re-exports the generated
  types, so the leading component moves with it. No change to this crate's own
  API.

## [0.13.0] - 2026-08-26

The duplicate-execution defence of SPEC §7.2 item 11, wired onto this
binding's inbound path and **on by default**. 0.12.0 moved the requirement to
`trust-tasks-rs` 0.12 and told callers to wire a `ReplayGuard` themselves; this
release wires it, because a defence every deployment has to remember to switch
on is not a defence.

### Added

- **`DidcommConsumer` — the guarded inbound path.** `receive` /
  `receive_from` unpack an envelope and run `consume_inbound` over the document
  with a `ReplayGuard` and a `FreshnessPolicy` already in place; `consume`
  applies the same pipeline to a document the caller unpacked itself (a
  mediator SDK's own delivery loop). Every verdict of §7.2's *Disposition of a
  duplicate* is applied: a duplicate returns the prior response and does not
  re-dispatch, an in-flight duplicate reports the existing execution rather
  than starting another, a differing document under a reused `id` is
  `idConflict`, and a guard that cannot answer fails closed as `unavailable`
  with `retryable = true` — never by executing.

  **The record is keyed on the document `id`, never on the DIDComm `@id`,
  `thid` or `pthid`.** SPEC §7.2 forbids the substitution, and the reason is
  this binding's own §6: a mediator "can drop, delay, reorder, and re-deliver",
  and a redelivery is a *fresh* DIDComm message carrying the *same* document.
  A record keyed on `@id` would admit it and grant the ACL entry twice —
  `tests/replay.rs` pins this by asserting the two envelopes' `@id`s differ
  before feeding both through the path and asserting the handler ran once.

### Changed

- **BREAKING — the duplicate-execution record defaults ON.** `DidcommConsumer`
  keeps an in-process `InMemoryReplayGuard` and applies
  `FreshnessPolicy::consequential` (`issuedAt` REQUIRED, five-minute acceptance
  window). A consumer that previously accepted an undated document, or the same
  document twice, no longer does — that is a change to what a consumer
  observes, so the leading component moves.

  `DidcommConsumer::with_replay_guard` takes a store shared across replicas
  (the in-process guard is wrong behind a load balancer);
  `DidcommConsumer::without_replay_record` is the explicit opt-out, documented
  with what it re-opens; `with_freshness` widens the acceptance window — and
  with it the record's retention, because §7.2 makes them one bound. There is
  no second TTL.

- `chrono` is now a direct dependency (the pipeline takes a `now`), and
  `async-trait` a dev-dependency (the tests implement a failing guard).

## [0.12.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.12`** (SPEC §7.2 item 11 duplicate
  execution, item 13 freshness). Leading component moves with the re-exported
  types.

  A mediator redelivering a queued message is the most likely source of an
  accidental duplicate, so wiring a `ReplayGuard` here matters more than
  anywhere else. When it is wired, key it on the *document* `id` — §7.2 forbids
  substituting a transport message identifier such as the DIDComm `@id` or
  `thid`.

## [0.11.0] - 2026-08-26

Four fail-open paths in the inbound gate, closed. All of them change what a
consumer observes — envelopes this crate used to accept are now rejected — so
this is **BREAKING** even though no wire format moved.

### Changed

- **BREAKING — a signed-only (bare JWS) envelope is rejected.**
  `unpack_trust_task` accepted `UnpackResult::Signed` and set the local DID to
  `None`. Binding `didcomm/0.2` §2 makes authcrypt a **MUST** and §4 states
  that an envelope with no authenticated sender "MUST NOT enter the framework
  pipeline". A JWS is signed but sealed to nobody: it has no recipient binding,
  so one message could be delivered to every party in a deployment and each
  would accept it — and because the local DID was `None`, SPEC §4.8.1's
  `recipient` cross-check was not failed but *skipped*. Now
  `DidcommError::SignedNotAuthcrypted`.

- **BREAKING — a `sender_kid` with no `#fragment` is an error.** It previously
  reduced to `None`, which downgraded an authenticated identity to an
  unauthenticated one: the pipeline then fell back to the in-band `issuer` with
  the transport cross-check skipped, so a sender with a malformed `kid` could
  name any issuer it liked. Now `DidcommError::UnqualifiedSenderKid`. A
  fragment-less *recipient* kid is still passed through as-is — unlike the
  sender case that fails closed, since a non-DID value simply fails §4.8.1's
  comparison.

- **BREAKING — `expected_sender_did` is enforced, not just used as a lookup
  key.** The authcrypt `skid`/`apu` is chosen by the sender, so the DID it
  carries is a claim; the DID that actually authenticated is the one whose
  public key opened the ECDH-1PU wrap. A peer could authenticate as itself
  while labelling the envelope with another party's DID and have the label
  handed to §4.8.1 as the transport-authenticated sender. The two must now
  agree — `DidcommError::SenderKidMismatch`.

- `BINDING_URI` is `https://trusttasks.org/binding/didcomm/0.2`, matching the
  current binding specification. `ENVELOPE_TYPE` is deliberately unchanged at
  `…/didcomm/0.1/envelope` — binding §1 and §7.1 keep it pinned so `0.1` and
  `0.2` implementations stay mutually intelligible on the wire.

### Added

- `SenderAllowlist`, `unpack_trust_task_from`, and `advertised_sender_did`.

  The crate previously told a multi-peer server to loop over its known senders
  retrying `unpack_trust_task` on `IdentityNotFound` — O(known peers) ECDH-1PU
  decrypts per inbound message. That loop *was* the sender allowlist, but only
  incidentally: it held because a peer the agent had never been given could not
  be unpacked, a property invisible in the type signature and easy to lose.

  `unpack_trust_task_from` reads the envelope's `skid` from the JWE protected
  header, checks the DID it names against a declared `SenderAllowlist` **before
  any decryption**, and unpacks once. An empty allowlist permits nothing;
  `SenderAllowlist::from_agent_peers` reproduces exactly the set the old loop
  could accept, so the migration is behaviour-preserving.

  The `skid` is unauthenticated at the point it is read — it selects a key, it
  proves nothing — which is why the verified sender is re-checked against it
  afterwards (see `SenderKidMismatch` above).

- `base64` is a new dependency, for reading the JWE protected header.

### Migration

```rust
// before
for peer in known_peers {
    match unpack_trust_task::<P>(&wire, &agent, Some(peer)) { .. }
}

// after
let allow = SenderAllowlist::from_agent_peers(&agent); // or ::new([..])
let (doc, handler) = unpack_trust_task_from::<P>(&wire, &agent, &allow)?;
```

A deployment that relied on signed-only envelopes has no compatibility path
here by design: authcrypt them, or carry an in-band `proof` and use a binding
that admits unauthenticated transport.

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
