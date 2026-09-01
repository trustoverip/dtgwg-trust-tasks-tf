---
slug: vault/credentials/purge
version: "0.1"
title: "Vault Credentials — Purge"
summary: "Erase a stored credential immediately and irrecoverably."
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
    Purge is the one operation in this family with no recovery path at all. A maintainer that cannot attribute it cannot answer who destroyed the credential, and there is nothing left to examine afterwards.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    There is no undoing a purge, so a captured request that stays valid indefinitely is a standing capability to destroy. `issuedAt` is what bounds it.
sideEffects:
  level: destructive
  rationale: >-
    Erases the credential outright. No tombstone, no grace window, no restore.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/credentials/purge:notFound
    meaning: >-
      The maintainer holds no credential under this identifier that this consumer may act on. Deliberately conflates "no such credential" with "not yours" — see Custody scope.
    retryable: false
related: []
---

## Abstract

The **Vault Credentials — Purge** Trust Task erases a stored credential **immediately and irrecoverably**. Unlike the default [`delete`](../../delete/0.1/spec.md) it creates no tombstone, and unlike an archived credential there is nothing left to bring back.

It exists for the case where the holder's requirement is that the material stop existing — a credential received in error that names someone else, or one whose retention the holder is obliged to end. `delete` is the everyday verb; this one is for when a grace window is itself the problem.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents a family that maintainers already implement and drive in production tooling, written down so that the shapes stop being recoverable only by reading an implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

- **`id`** — the local handle from a [`query`](../../query/0.1/spec.md) descriptor. Opaque to the consumer.
- **`reason`** — free text recorded with the transition, for the operator who reads the trail later. OPTIONAL. A consumer **MUST NOT** put credential contents in it.

A stored credential carries two orthogonal states, and a consumer that collapses them will mis-render its own vault:

- **Validity** (`status`) — `valid`, `expired`, `revoked` or `unknown`. Driven by the credential's own validity window and by status-list checks. The maintainer does not choose it.
- **Archival lifecycle** (`lifecycle`) — `active`, `archived` or `deleted`. Chosen by the consumer through this family. The maintainer records it.

The two do not constrain each other. A credential can be `valid` and `archived`; it can be `revoked` and `active`. "Can I present this?" is answered by both together — only an `active` credential may be presented, and only a `valid` one is worth presenting.

## Request

The credential-vault consumer names one credential. The top-level schema is in [`payload.schema.json`](payload.schema.json).

### Erasing a credential outright

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vault/credentials/purge/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "reason": "received in error; names a third party"
  }
}
```

## Response

The maintainer confirms the state the credential is now in. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents.

`lifecycle` is the state after the transition, echoed rather than assumed: a consumer that inferred it from the verb it called would be wrong the moment a maintainer's policy differs from its expectation.

`lifecycle` reads `deleted` because that is the terminal state; no `graceUntil` accompanies it, and its absence is the signal that nothing can be restored.

### Erased

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vault/credentials/purge/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "lifecycle": "deleted"
  }
}
```

## Security & Privacy

### Data carried

The request carries one opaque identifier and optional free text. The response carries the resulting lifecycle state and, where one applies, a deadline.

No credential contents move in either direction. `reason` is the one free-form member: a consumer **MUST NOT** write claim content into it, because it lands in a trail read by people who are entitled to know that a credential changed state without being entitled to know what it said.

### Correlation

The maintainer learns which credential the consumer is managing and when. As with `delete`, a recorded `reason` outlives the credential. Where the purpose of purging was to end retention of material about a third party, a maintainer **SHOULD** consider whether the reason it keeps re-states what the holder was trying to remove.

### Retention

The credential is erased. A maintainer **SHOULD** retain the record that a purge occurred — a vault that can be emptied without trace cannot be audited — and that record **MUST NOT** carry the credential's contents.

### Consent/purpose

The credential is destroyed at the holder's request. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required.

### Custody scope

A maintainer holds credentials on behalf of more than one context. A consumer's authority is scoped, and the maintainer **MUST** evaluate that scope against the credential's own context before acting: a consumer scoped to one context **MUST NOT** be able to read, alter or destroy a credential held for another.

Where that check fails, the maintainer **MUST** answer exactly as it would for an identifier it does not hold. Answering `permissionDenied` for a credential that exists and `notFound` for one that does not would let a consumer map another context's vault one identifier at a time, which is the enumeration this family is built to refuse.
