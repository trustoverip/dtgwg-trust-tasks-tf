---
slug: rooms/blobs/upload/abort
version: "0.1"
title: "Rooms Blobs — Upload — Abort"
summary: "A member cancels a blob upload they began; the host discards its staged chunks and releases its reservation."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - file
  - blob
  - upload
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Aborting ends an upload and releases space reserved against a member's and a room's limits. Only the party that began the upload may do it, and that comparison has to be attributable on every transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. An abort is idempotent, but SPEC §7.2 item 11 still needs a bounded window to recognise a duplicate.
sideEffects:
  level: mutating
  rationale: >-
    Discards an upload's staged chunks and releases its reservation. Nothing in the room changes: an uncommitted upload was never part of it.
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: >-
    The request names an upload; the response says whether this request ended it.
retention:
  class: transient
  rationale: >-
    Abort removes state; it keeps none beyond what lets a retried abort be answered.
errorCodes:
  - code: rooms/blobs/upload/abort:notFound
    meaning: "No upload with this id exists that the caller may abort. Conflates an unknown id, an expired or committed upload, and another party's upload."
    retryable: false
related:
  - rooms/blobs/upload/begin
  - rooms/blobs/upload/commit
---

## Abstract

The **Rooms Blobs — Upload — Abort** Trust Task cancels an upload opened by [`rooms/blobs/upload/begin`](../../begin/0.1/spec.md) and not yet committed. The host discards whatever chunks it staged and releases the reservation the upload held against the member, the room and the storage, so a member who gives up on a file gets the space back at once rather than at the upload's expiry.

An upload that is neither committed nor aborted expires on its own with the same effect. Abort is the prompt form of that expiry, and a producer **SHOULD** send it when it abandons an upload.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **SHOULD** abort an upload it abandons, rather than leave it to expire.

A conforming **host** (`recipient`):

1. **MUST** answer `notFound` unless `uploadId` names an upload begun by the party the host authenticated for this request, and **MUST** answer `notFound` for a committed upload: a committed blob is removed by dropping it from every record, never by aborting.
2. **MUST** discard the upload's staged chunks and release its reservation against every scope, and answer `aborted: true`.
3. **MUST** answer a repeated abort of an upload it already aborted with `aborted: false`, for as long as it remembers the upload, and after that **MAY** answer `notFound`.

## Authorization

The authority this task presupposes is **ownership of the upload**: only the party the upload was bound to when it began may abort it. The host consults no access-control list of its own (rooms invariant I5). Verifying the proof establishes who asked; ownership establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **Reservation** — the size and file count an open upload holds against the member, the room and the storage, so that concurrent uploads cannot together exceed a limit.

## Request

The member who began the upload (`issuer`) cancels it with the host (`recipient`).

### Abandon an upload

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000131",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/abort/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T10:02:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000131ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a"
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Aborted

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000132",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/abort/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:02:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000131ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
    "aborted": true
  }
}
```

## Security & Privacy

### Data carried

An upload handle in; the handle and a boolean out.

### Correlation

None beyond the upload handle, which links the documents of one transfer.

### Retention

The host discards the staged chunks and the reservation. It **MAY** remember the aborted handle until the upload's original expiry, so that a retried abort is answered `aborted: false`.

### Consent/purpose

The member withdraws a file they had begun to store. Nothing else is affected.
