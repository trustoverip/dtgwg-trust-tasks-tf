---
slug: vault/get
version: "0.3"
wireCompatibleWith: "0.1"
title: Vault — Get
summary: A vault consumer fetches the metadata view of a single vault entry by id; secret material is never returned.
status: draft
targetFrameworkVersion: "0.4"
category: credentials
keywords:
  - vault
  - credentials
  - get
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only by id, usually session-authenticated. Recommended on non-session-bound transports so the maintainer can attribute the request to a specific consumer key.
sideEffects:
  level: none
  rationale: "Read-only metadata read of a vault entry; secret material is never returned."
subjectPath: /id
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/get:notFound
    meaning: No entry with this id exists in any context the consumer can read.
    retryable: false
  - code: vault/get:permissionDenied
    meaning: The entry exists but the consumer lacks VaultRead on its context.
    retryable: false
---

## Abstract

The **Vault — Get** Trust Task returns the metadata view of one entry. Consumers use it to refresh a single row after a `sync/event` notification, or to fetch detail for a UI panel after a list result.

Like `vault/list/0.1`, this task **never returns secret material**. Use `vault/release/0.1` to obtain secret bytes.

## Changes from 0.2

The attachment digest carried in `VaultEntry.attachments[]` moves from
**`sha256`** — a bare lowercase-hex SHA-256 — to **`digestMultibase`**, the
framework's
[`DigestMultibase`](../../../_framework/0.3/framework.schema.json): a
multibase-encoded multihash.

A bare hex string hard-codes one algorithm into the wire contract, so moving off
SHA-256 later would need a schema revision rather than a different multihash
prefix, and it names no base encoding, leaving a verifier to infer base16 from
context. The digest is taken over the encrypted blob **bytes**, not over a
canonicalization — the blob is an opaque artifact rather than a JSON document —
so unlike the credential digests converged in the same sweep, no reproducibility
defect is being fixed here. This is the encoding argument alone.

The change arrives through the shared component: this version re-pins its
`$ref`s from `vault/_shared/0.2` to `vault/_shared/0.3`, which is why it is a new
version at all ([SPEC.md §5.4](../../../../SPEC.md#54-migrating-between-versions)
couples the two). Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under
[§5.2](../../../../SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2`
remains published and pinned to `vault/_shared/0.2`; `vault/proxy-login` and
`vault/release` stay on `0.2` deliberately, since they reference only
`SiteTarget` and `SecretKind` and never expose an attachment digest.

## Conformance

A conforming **producer** **MUST** populate `payload.id` with a non-empty entry identifier. A conforming **consumer** **MUST** authorise the requesting consumer against the context the entry belongs to, return `vault/get:notFound` when the entry is absent OR the consumer lacks visibility into its context (so existence cannot be probed by enumeration), and return the entry in the metadata-only view per the shared `VaultEntry` schema.

Maintainers MAY return `vault/get:permissionDenied` instead of `vault/get:notFound` only when the consumer can already prove existence via another channel (e.g. the entry id appeared in a prior list response with redacted fields). The default is to conflate the two to deny enumeration.

## Payload

`payload.id` (REQUIRED) — the entry id.

`payload.ext` (optional).

## Response

`payload.entry` — the VaultEntry in metadata-only view.

`payload.redactedFields` (optional) — names of fields the maintainer redacted.

## Security & Privacy

**Enumeration resistance.** Conflating `notFound` and `permissionDenied` is the default for a reason: a consumer that can distinguish them can probe id space to map who has what. Maintainers operating in lower-trust environments (e.g. publicly-listed vaults) MUST keep them conflated.

Other guidance: see the Security & Privacy section of `vault/list/0.1`. The same secret-leakage, timing-data, and audit considerations apply here, scoped to a single entry.
