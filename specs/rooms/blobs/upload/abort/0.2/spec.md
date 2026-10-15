---
slug: rooms/blobs/upload/abort
version: "0.2"
title: "Rooms Blobs — Upload — Abort"
summary: "A blob upload is cancelled — by its single-use key (proxied) or its owner (direct); the host discards its staged chunks and releases its reservation."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - file
  - blob
  - upload
parties:
  - role: Upload key or owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Aborting ends an upload and releases space reserved against a member's and a room's limits. Only the upload's key (proxied) or its owner (direct) may do it, and that comparison has to hold on every transport.
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

The **Rooms Blobs — Upload — Abort** Trust Task cancels an upload opened by [`rooms/blobs/upload/begin`](../../begin/0.2/spec.md) and not yet committed. The host discards whatever chunks it staged and releases the reservation the upload held against the member, the room and the storage, so a member who gives up on a file gets the space back at once rather than at the upload's expiry.

An upload that is neither committed nor aborted expires on its own with the same effect. Abort is the prompt form of that expiry, and a producer **SHOULD** send it when it abandons an upload.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`0.1` was authenticated by the party that began the upload. `0.2` follows the upload's transfer mode ([`rooms/blobs/upload/begin/0.2`](../../begin/0.2/spec.md)): a **proxied** upload's is authenticated by its **upload key**, the single-use `did:key` the begin named, so that the member — whose keys their key holder keeps — never signs over the file's bytes; a **direct** upload's is the owner's own, signed like the begin, because no bytes pass through any document. The payload and response are unchanged; who may send this document, and what the host checks before answering, is what changed, which is a behavioural break for a consumer and so a new version. A `0.2` upload accepts only `0.2` documents, and a `0.1` upload only `0.1`.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **SHOULD** abort an upload it abandons, rather than leave it to expire, and **MUST** sign the abort with the upload key the begin named (proxied) or as the owner (direct).

A conforming **host** (`recipient`):

1. **MUST** answer `notFound` unless `uploadId` names an upload and the party the host authenticated for this request is, for a proxied upload, its upload key, or, for a direct upload, its owner, and **MUST** answer `notFound` for a committed upload: a committed blob is removed by dropping it from every record, never by aborting.
2. **MUST** discard the upload's staged chunks — in direct mode, abort the write at storage and delete whatever parts storage holds for it — release its reservation against every scope, and answer `aborted: true`.
3. **MUST** answer a repeated abort of an upload it already aborted with `aborted: false`, for as long as it remembers the upload, and after that **MAY** answer `notFound`.

## Authorization

The authority this task presupposes is **being the upload's key** (proxied) or **its owner** (direct): only the key the begin named, or in direct mode the owner, may abort it. The host consults no access-control list of its own (rooms invariant I5). Verifying the proof establishes who asked; ownership establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **Reservation** — the size and file count an open upload holds against the member, the room and the storage, so that concurrent uploads cannot together exceed a limit.

## Request

The upload key (`issuer`, proxied mode) — or, in direct mode, the owner — cancels the upload with the host (`recipient`).

### Abandon an upload

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000131",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/abort/0.2#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
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
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/abort/0.2#response",
  "issuer": "did:example:host",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
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
