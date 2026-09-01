---
slug: vault/credentials/delete
version: "0.1"
title: "Vault Credentials — Delete"
summary: "Move a stored credential to a recoverable tombstone, or erase it outright."
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - credential-vault
  - holder
  - lifecycle
parties:
  - role: credential-vault consumer
    requirement: REQUIRED
    member: issuer
  - role: credential-vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Delete removes a credential from use, and with `force` erases it irrecoverably. A maintainer that cannot attribute the request cannot answer the only question that matters afterwards: who asked for this to be gone.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed delete re-destroys a credential the holder has since restored. On the `force` path there is no recovery from getting that wrong, so bounding replay is not optional.
sideEffects:
  level: destructive
  rationale: >-
    Without `force`, creates a recoverable tombstone — the credential is retained and blocked from use until its grace window elapses. With `force`, erases immediately and irrecoverably. The destructive class covers the worse of the two.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/credentials/delete:notFound
    meaning: >-
      The maintainer holds no credential under this identifier that this consumer may act on. Deliberately conflates "no such credential" with "not yours" — see Custody scope.
    retryable: false
  - code: vault/credentials/delete:alreadyDeleted
    meaning: >-
      The credential is already a tombstone. Returned only on the default path; `force` is idempotent and succeeds against an already-deleted or absent id.
    retryable: false
related: []
---

## Abstract

The **Vault Credentials — Delete** Trust Task removes a stored credential from use. By default it creates a **recoverable tombstone**: the credential is retained, refused for presentation, hidden from [`query`](../../query/0.1/spec.md) unless `includeDeleted` asks for it, and restorable through [`restore`](../../restore/0.1/spec.md) until `graceUntil` passes — after which the maintainer erases it.

With `force`, it skips that window and erases immediately.

The two-step default exists because a credential cannot be re-obtained by asking nicely. Re-issuance means going back to the issuer, and for an invitation or a one-time membership that may not be possible at all.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents a family that maintainers already implement and drive in production tooling, written down so that the shapes stop being recoverable only by reading an implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

- **`id`** — the local handle from a [`query`](../../query/0.1/spec.md) descriptor. Opaque to the consumer.
- **`reason`** — free text recorded with the transition, for the operator who reads the trail later. OPTIONAL. A consumer **MUST NOT** put credential contents in it.
- **`force`** — erase immediately rather than creating a tombstone. Defaults to `false`. A maintainer **MUST** treat a forced delete as irrecoverable and **MUST NOT** retain the credential for later restore.

A stored credential carries two orthogonal states, and a consumer that collapses them will mis-render its own vault:

- **Validity** (`status`) — `valid`, `expired`, `revoked` or `unknown`. Driven by the credential's own validity window and by status-list checks. The maintainer does not choose it.
- **Archival lifecycle** (`lifecycle`) — `active`, `archived` or `deleted`. Chosen by the consumer through this family. The maintainer records it.

The two do not constrain each other. A credential can be `valid` and `archived`; it can be `revoked` and `active`. "Can I present this?" is answered by both together — only an `active` credential may be presented, and only a `valid` one is worth presenting.

## Request

The credential-vault consumer names one credential. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Deleting a credential, recoverably

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vault/credentials/delete/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "reason": "issued in error"
  }
}
```

## Response

The maintainer confirms the state the credential is now in. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents.

`lifecycle` is the state after the transition, echoed rather than assumed: a consumer that inferred it from the verb it called would be wrong the moment a maintainer's policy differs from its expectation.

`graceUntil` is present on the default path and absent when `force` was set — its absence is how a consumer knows nothing can be restored.

### Tombstoned, with a grace window

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vault/credentials/delete/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "lifecycle": "deleted",
    "graceUntil": "2026-02-01T00:00:00Z"
  }
}
```

## Security & Privacy

### Data carried

The request carries one opaque identifier and optional free text. The response carries the resulting lifecycle state and, where one applies, a deadline.

No credential contents move in either direction. `reason` is the one free-form member: a consumer **MUST NOT** write claim content into it, because it lands in a trail read by people who are entitled to know that a credential changed state without being entitled to know what it said.

### Correlation

The maintainer learns which credential the consumer is managing and when. A delete is the strongest signal in this family about what the holder wants forgotten, and a maintainer that logs the `reason` retains that intent after the credential is gone.

### Retention

On the default path the credential is retained until `graceUntil`, then erased. With `force` it is erased at once.

A maintainer **SHOULD** retain the record *that* a deletion happened even after the credential itself is gone — a holder auditing their own vault needs to be able to tell a credential they deleted from one that was never stored. That record **MUST NOT** carry the credential's contents.

### Consent/purpose

The credential is removed at the holder's request. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required.

### Custody scope

A maintainer holds credentials on behalf of more than one context. A consumer's authority is scoped, and the maintainer **MUST** evaluate that scope against the credential's own context before acting: a consumer scoped to one context **MUST NOT** be able to read, alter or destroy a credential held for another.

Where that check fails, the maintainer **MUST** answer exactly as it would for an identifier it does not hold. Answering `permissionDenied` for a credential that exists and `notFound` for one that does not would let a consumer map another context's vault one identifier at a time, which is the enumeration this family is built to refuse.
