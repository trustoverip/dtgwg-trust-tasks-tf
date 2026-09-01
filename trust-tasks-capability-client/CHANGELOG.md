# Changelog

All notable changes to `trust-tasks-capability-client` are documented in this
file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-capability-client-v0.17.3...trust-tasks-capability-client-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-capability-client-v0.17.2...trust-tasks-capability-client-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-capability-client-v0.17.1...trust-tasks-capability-client-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-capability-client-v0.17.0...trust-tasks-capability-client-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-capability-client-v0.14.0...trust-tasks-capability-client-v0.17.0) — 2026-08-27


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

The producer half of SPEC §7.2 item 11. 0.11.0 flagged that this client
"should no longer reuse a document `id` across attempts"; this release makes
the two ways of sending a request again into two named operations, because as
of this same release the consumer enforces the difference.

### Added

- **`new_attempt(&previous)` — a fresh attempt at a request already built.**
  Same addressing, type and payload; **fresh `id`**, fresh `issuedAt`, and no
  carried-over `proof` (the old one committed to the old `id`, so re-sending it
  would ship a signature over a document that no longer exists).

  This is the counterpart of a §8.4 retry, and the two are now genuinely
  different operations:

  * a **retry** is `previous` itself, resent bit-for-bit — the consumer's
    record absorbs it and returns whatever the first execution determined;
  * a **new attempt** is any send whose bytes differ, including a re-stamped
    `issuedAt` or a `proof` re-signed over identical content. It **MUST** carry
    a fresh `id`, and `new_attempt` is how to mint one.

  Anything else — a reused `id` with altered content — is rejected `idConflict`
  by a `trust-tasks-rs` 0.12 consumer, and the DIDComm and TSP bindings in this
  release keep that record **on by default**, so it will be rejected by every
  consumer this client talks to. The module docs carry the table; a test drives
  all three cases through the real `InMemoryReplayGuard` rather than restating
  the rule.

  Note the correlation consequence: where the previous request opened its own
  exchange (no `threadId`), SPEC §4.9 names that exchange by the document `id`,
  so a new attempt opens a *new* exchange — hold `correlation_thread` of the
  returned document, not of the previous one. An explicit `threadId` is
  preserved.

### Changed

- **Audited: the builders were already correct.** `build_document`, and
  therefore `build_list_document`, `build_toggle_document`,
  `build_git_trust_grant` and `build_git_trust_revoke`, mint a fresh
  `urn:uuid:` `id` and stamp `issuedAt` on *every* call, so a caller that
  rebuilds a request has always been making a new attempt rather than reusing
  an `id`. The gap was that the crate offered no way to express the *other*
  intent — re-sending a document already built and signed — and left a caller
  who reached for "re-stamp and re-sign" with a document the consumer now
  refuses. That is what `new_attempt` closes; no builder behaviour changed.

- **The leading component moves** to keep this crate in step with the
  behavioural break its siblings ship in the same release. The API change here
  is additive, but a producer built against 0.11 will start receiving
  `idConflict` from consumers it previously reached, and `issuedAt` is now
  required by the bindings' default freshness policy — both are changes to what
  this client observes in the field.

- `tokio` is a dev-dependency (one test drives the async `ReplayGuard`).

## [0.11.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.12`** (SPEC §7.2 item 11 duplicate
  execution, item 13 freshness). Leading component moves with the re-exported
  types.

  As a *producer*, this client should no longer reuse a document `id` across
  attempts: §8.4 defines a retry as a bit-for-bit identical resend, and as of
  `trust-tasks-rs` 0.12.0 there is a consumer that enforces it — a fresh attempt
  carrying a reused `id` and altered content is rejected with `idConflict`.

## [0.10.0] - 2026-08-26

### Changed

- **BREAKING — idempotent success is decided by the error `code`, not by the
  error `message`.** `classify_git_trust_reply` previously returned
  `WriteOutcome::IdempotentSuccess` for a `taskFailed` whose free-text
  `message` contained `already_granted:` or `not_granted:`. SPEC §8.2 defines
  `message` as non-normative text "intended for logs and operator UI", so that
  made the client's behaviour depend on wording no emitter promised to keep
  stable — and, in the other direction, silently reported a *genuine*
  `taskFailed` whose operator message happened to quote the phrase as a
  successful write.

  The control surface is now the namespaced extended code of SPEC §8.5, which
  the registry entries already declare: `git-trust/grant:already_granted`
  (`GIT_TRUST_ALREADY_GRANTED_CODE`) and `git-trust/revoke:not_granted`
  (`GIT_TRUST_NOT_GRANTED_CODE`). Their §4.10 lowerCamelCase spellings are
  accepted too, so a later registry re-casing is not a flag day.

- **BREAKING — replies must be correlated to their request before they are
  classified.** `classify_git_trust_reply`, `parse_capability_reply` and
  `parse_envelope_reply` now take an `expected_thread_id` and return `None` for
  a document threaded to any other exchange (SPEC §4.9). Previously
  `parse_envelope_document` extracted the `threadId` and nothing ever compared
  it, so any reply arriving on a shared inbound path could resolve a write the
  caller was still waiting on — including the `IdempotentSuccess` shortcut,
  which is the outcome an attacker would most want to forge.

  `correlation_thread(&request)` gives the value to hold; `replies_to` is the
  bare predicate; `parse_envelope_document_for` is the correlating counterpart
  of `parse_envelope_document` (which is unchanged, and now documents that its
  `threadId` is a dispatch key rather than a check).

### Added

- `ReplyPolicy`, `classify_git_trust_reply_with_policy`, `correlation_thread`,
  `replies_to`, `parse_envelope_document_for`, and the four extended-code
  constants.

### Deprecated

- `ReplyPolicy::accept_legacy_free_text_idempotence` (and
  `ReplyPolicy::with_legacy_free_text`) restore the pre-0.10 free-text
  matching, for a peer that still emits only that form. It is **opt-in**, never
  the default.

  **Removal: the next MAJOR (0.11.0).** Delete `ReplyPolicy`,
  `classify_git_trust_reply_with_policy`, and the `// DEPRECATED:` block in
  `classify_git_trust_reply_with_policy` together — they exist only for each
  other. The fix belongs on the emitting side: send the extended code.

### Migration

```rust
// before
let outcome = classify_git_trust_reply(&reply);

// after — hold the thread from the moment you send
let request = build_git_trust_grant(..);
let thread = correlation_thread(&request).to_string();
let outcome = classify_git_trust_reply(&reply, &thread);
```

A peer that still answers with a free-text `taskFailed`, until it is fixed:

```rust
let outcome = classify_git_trust_reply_with_policy(
    &reply, &thread, ReplyPolicy::with_legacy_free_text(),
);
```
