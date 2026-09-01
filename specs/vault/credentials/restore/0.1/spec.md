---
slug: vault/credentials/restore
version: "0.1"
title: "Vault Credentials — Restore"
summary: "Return a soft-deleted credential to the active state, while its grace window lasts."
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
    Restore brings back a credential the holder asked to remove. Attribution is what separates the holder changing their mind from someone else undoing their decision.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed restore resurrects a credential the holder deleted again. `issuedAt` bounds how old a transition a maintainer will honour.
sideEffects:
  level: mutating
  rationale: >-
    Changes stored lifecycle state back to active. Nothing is created or destroyed — the credential was retained throughout.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vault/credentials/restore:notFound
    meaning: >-
      The maintainer holds no credential under this identifier that this consumer may act on. Deliberately conflates "no such credential" with "not yours" — see Custody scope.
    retryable: false
  - code: vault/credentials/restore:notDeleted
    meaning: >-
      The credential is not a tombstone. An archived credential returns this too — that one comes back through `unarchive`.
    retryable: false
  - code: vault/credentials/restore:graceExpired
    meaning: >-
      The grace window has passed and the credential has been erased. Distinguished from `notFound` on purpose: "this existed and is now unrecoverable" is a different thing to tell a holder than "this was never here", and only one of them warrants going back to the issuer.
    retryable: false
related: []
---

## Abstract

The **Vault Credentials — Restore** Trust Task returns a soft-deleted credential to the `active` state. It is the recovery path that makes the default [`delete`](../../delete/0.1/spec.md) safe to offer.

It works only while the grace window lasts. After `graceUntil`, the maintainer has erased the credential and there is nothing to restore — which is why a consumer showing a trash view **SHOULD** show the deadline rather than only the fact of deletion.

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

### Recovering a credential deleted by mistake

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vault/credentials/restore/0.1#request",
  "issuer": "did:example:wallet",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "reason": "deleted the wrong row"
  }
}
```

## Response

The maintainer confirms the state the credential is now in. The sub-schema is reachable via `$anchor: "response"`. Failures are `trust-task-error` documents.

`lifecycle` is the state after the transition, echoed rather than assumed: a consumer that inferred it from the verb it called would be wrong the moment a maintainer's policy differs from its expectation.



### Active again

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vault/credentials/restore/0.1#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:wallet",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "cred-7f3a91c2",
    "lifecycle": "active"
  }
}
```

## Security & Privacy

### Data carried

The request carries one opaque identifier and optional free text. The response carries the resulting lifecycle state and, where one applies, a deadline.

No credential contents move in either direction. `reason` is the one free-form member: a consumer **MUST NOT** write claim content into it, because it lands in a trail read by people who are entitled to know that a credential changed state without being entitled to know what it said.

### Correlation

The maintainer learns which credential the consumer is managing and when. A restore tells the maintainer the earlier deletion was a mistake, which is a small signal about the holder's confidence in their own tooling rather than about their affiliations.

### Retention

Nothing new is retained; the credential was held throughout the grace window.

### Consent/purpose

The credential is returned to use at the holder's request. Descriptive only — per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT** declare that consent, approval or a step-up is required.

### Custody scope

A maintainer holds credentials on behalf of more than one context. A consumer's authority is scoped, and the maintainer **MUST** evaluate that scope against the credential's own context before acting: a consumer scoped to one context **MUST NOT** be able to read, alter or destroy a credential held for another.

Where that check fails, the maintainer **MUST** answer exactly as it would for an identifier it does not hold. Answering `permissionDenied` for a credential that exists and `notFound` for one that does not would let a consumer map another context's vault one identifier at a time, which is the enumeration this family is built to refuse.
