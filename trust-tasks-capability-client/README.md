# trust-tasks-capability-client

Client-side wire helpers for the **capability** Trust Task families of the
[Trust Tasks](https://trusttasks.org/) framework:

- `governance/capability/*` — list, enable and disable a community capability;
- `git-trust/*` — grant and revoke commit-signing trust.

This crate owns the *documents*, not a transport: it builds request documents,
parses inbound envelope replies and classifies them. A capability producer (a
community service) and a management UI each keep their own send/receive
plumbing and share this layer, so the two cannot drift on the contract. It has
no crypto dependency. The TypeScript, Go and Dart ports
(`@openvtc/trust-tasks-capability-client`, `trust-tasks-go/capabilityclient`,
`trust_tasks_capability_client`) follow the same rules.

```sh
cargo add trust-tasks-capability-client
```

or, through the facade, `cargo add trust-tasks --features capability-client`
and use `trust_tasks::capability_client`.

## API

| Layer | Items |
|---|---|
| Builders | `build_document` (any type URI), `build_list_document`, `build_toggle_document`, `build_git_trust_grant`, `build_git_trust_revoke`, and `new_attempt` |
| Envelope | `parse_envelope_document` → `(threadId, document)`, `parse_envelope_document_for` (only a reply to the exchange you name), `correlation_thread`, `replies_to` |
| `git-trust` replies | `classify_git_trust_reply` → `WriteOutcome::{Success, IdempotentSuccess, Rejected}`; `classify_git_trust_reply_with_policy` with a `ReplyPolicy` |
| `governance/capability` replies | `parse_envelope_reply`, `parse_capability_reply` → `CapabilityReply` (a `list` of `CapabilitySummary`, an acknowledgement, or an error) |
| Constants | the Type URIs of both families, the DIDComm binding envelope type, and the `git-trust` idempotence codes in both spellings |

Builders return `TrustTask<serde_json::Value>`. Every builder mints a fresh `id`
and stamps `issuedAt` (whole seconds, UTC) on each call.

### Retries versus new attempts

The consumers of these documents keep SPEC §7.2 item 11's duplicate-execution
record, keyed on the document `id` and compared against the whole document, so
sending a request again is one of two different operations (SPEC §8.4):

| Intent | What to send |
|---|---|
| The first send may not have arrived | the same document, unchanged — the consumer absorbs it |
| Something about the request changed | `new_attempt(&previous)`: a new `id`, a new `issuedAt`, and no `proof` |

Anything else under a reused `id` — including the same content re-signed or
re-stamped — is refused `idConflict`.

### Idempotent success is keyed on the error code

A `git-trust` grant that already exists, or a revoke with nothing to revoke,
comes back as an error whose **extended code** (SPEC §8.5) says so —
`git-trust/grant:already_granted`, `git-trust/revoke:not_granted`, or their
lowerCamelCase forms. `classify_git_trust_reply` reports those as
`WriteOutcome::IdempotentSuccess`. It never reads the free-text `message`, which
SPEC §8.2 makes non-normative; the older free-text form is accepted only through
`ReplyPolicy::with_legacy_free_text()`, which is deprecated.

### Signing

Sign a built document yourself, with your own signer, and set its `proof` —
`trust-tasks-proof`'s `sign_trust_task` does it (`eddsa-jcs-2022` by default).
Sign **after** the document is final: a member changed after signing is one the
proof does not cover.

## Versioning

This crate exposes `trust-tasks-rs` types in its public API, so it is released
in one version group with `trust-tasks-rs` and the other crates that do — they
always share a version. Pin them as a set, or depend on the `trust-tasks`
facade.

## License

Apache-2.0 — see
[`LICENSE`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-capability-client/LICENSE),
included in the published crate.
