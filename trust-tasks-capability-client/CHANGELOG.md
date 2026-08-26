# Changelog

All notable changes to `trust-tasks-capability-client` are documented in this
file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
