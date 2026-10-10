---
slug: rooms/blobs/upload/commit
version: "0.1"
title: "Rooms Blobs — Upload — Commit"
summary: "A member completes a blob upload: the host reassembles it, checks it against the manifest committed at begin, stores it, and returns the BlobRef a record can then name."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - file
  - blob
  - upload
  - quota
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
    Commit is the act that makes a blob durable in a room and turns a reservation into usage charged to a member. It must be attributable to the party that owns the upload on every transport, relayed ones included.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A second commit of one upload changes nothing, but SPEC §7.2 item 11 still needs a bounded window to recognise the duplicate.
sideEffects:
  level: mutating
  rationale: >-
    Stores the verified blob in the host's blob store and converts the upload's reservation into usage. Recoverable: a blob no record names is orphaned and collected, and its usage released.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names an upload; the response reports the blob's reference and ciphertext size, both of which the member already holds.
retention:
  class: durable
  rationale: >-
    A committed blob persists until no record names it, then for the host's grace window, then until its store deletes it. A store that publishes ciphertext may keep it longer still; see Security & Privacy.
errorCodes:
  - code: rooms/blobs/upload/commit:notFound
    meaning: "No upload with this id exists that the caller may commit. Conflates an unknown id, an expired or aborted upload, and another party's upload."
    retryable: false
  - code: rooms/blobs/upload/commit:incomplete
    meaning: "Chunks are missing. `details.remainingCount` says how many; `rooms/blobs/upload/begin` with the same manifest says which."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      required: [remainingCount]
      properties:
        remainingCount:
          type: integer
          minimum: 1
  - code: rooms/blobs/upload/commit:digestMismatch
    meaning: "Every chunk verified, but the reassembled ciphertext does not hash to the manifest's `digest` — the chunks are right and the order or the commitment is wrong. Nothing is stored; the upload is discarded and its reservation released."
    retryable: false
related:
  - rooms/blobs/upload/begin
  - rooms/blobs/upload/chunk
  - rooms/blobs/upload/abort
  - rooms/records/put
---

## Abstract

The **Rooms Blobs — Upload — Commit** Trust Task completes an upload opened by [`rooms/blobs/upload/begin`](../../begin/0.1/spec.md). The host checks that every chunk has arrived, checks the reassembled ciphertext against the digest committed in the manifest, hands the blob to its store, and answers with the blob's **BlobRef**: the digest of the manifest.

A committed blob is not yet a file in the room. It becomes one when a record names it, with [`rooms/records/put/0.2`](../../../../records/put/0.2/spec.md). Until then it counts against the member's and the room's usage, and if no record ever names it, it is orphaned and collected like any blob whose last record let go of it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer**:

1. **MUST** name the committed blob in a record promptly, or abandon it knowing it will be collected. A producer **MUST NOT** treat a committed blob as part of the room before a record names it.

A conforming **host** (`recipient`):

1. **MUST** answer `notFound` unless `uploadId` names an upload begun by the party the host authenticated for this request.
2. **MUST** refuse with `incomplete` while any chunk is missing, and **MUST** keep the upload open so it can be finished.
3. **MUST** check the reassembled ciphertext against the manifest's `digest`, and on a mismatch **MUST** discard the upload, release its reservation and refuse with `digestMismatch`.
4. **MUST** store the blob durably in the store the room is assigned to before answering, and **MUST** record it as committed **in this room**: the room, the BlobRef, the size, the store, and — on `open` and `attributed` rooms — the member it is charged to. A blob committed in one room is never referenceable from another.
5. **MUST** convert the upload's reservation into usage, charged to the subject at the root of the chain that began it, atomically with recording the blob.
6. **MUST** answer a repeated commit of an already-committed upload by the same party with the same `blobRef` and `size`, without storing or charging anything a second time, for as long as it remembers the upload.
7. **MUST NOT** store or charge a second copy when a blob with the same BlobRef is already committed in the room; it **MUST** release the new upload's reservation and answer with the existing blob. This includes an upload that begin answered with `alreadyCommitted: true`: it has no chunks and no reservation, and its commit answers at once with the existing blob.

## Authorization

The authority this task presupposes is **ownership of the upload**: the room-issued chain was verified when the upload began, and only the party it was bound to may commit it. The host consults no access-control list of its own (rooms invariant I5). Verifying the proof establishes who asked; ownership of the upload establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

The chain's validity is not re-checked at commit: the reservation was made under it, and a member removed between begin and commit has committed a blob no record of theirs can name, since `rooms/records/put` checks a fresh chain.

## Definitions

- **Committed** — stored durably and recorded as belonging to this room. A committed blob is named by its BlobRef.
- **Orphaned** — committed, but named by no live record. An orphan stops counting against the member and the room at once, and is collected after the host's grace window.

## Request

The member who began the upload (`issuer`) commits it with the host (`recipient`).

### Commit after the last chunk

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000121",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/commit/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T10:00:07Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000121ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a"
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Committed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000122",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/commit/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:08Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000121ff",
  "payload": {
    "blobRef": "zQmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk",
    "size": 300000
  }
}
```

### Refused: a chunk is still missing

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000123",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:08Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000121ff",
  "payload": {
    "code": "rooms/blobs/upload/commit:incomplete",
    "retryable": true,
    "details": {
      "remainingCount": 1
    }
  }
}
```

## Security & Privacy

**A blob belongs to one room.** Recording the room at commit is what lets [`rooms/records/put/0.2`](../../../../records/put/0.2/spec.md) refuse a record naming a blob that was never uploaded, or was uploaded to another room. A record can never point at nothing, or at another room's file.

**Stores differ in what deletion means.** On most stores, collecting an orphan removes the ciphertext. A store that publishes ciphertext, so that anyone holding its address can fetch it, may keep it indefinitely, and deletion there is not erasure. On such a store the only deletion that holds is cryptographic: once nobody can derive the file's key, the ciphertext is noise. A host using such a store **SHOULD** say so to the room's members.

### Data carried

An upload handle in; a BlobRef and a ciphertext size out.

### Correlation

The BlobRef is a digest over ciphertext under a per-file key, so it links nothing beyond this blob: two uploads of the same file produce different BlobRefs.

### Retention

The blob persists while any record names it, then for the host's grace window as an orphan, then until its store deletes it. The upload's staged chunks are discarded once the blob is stored.

### Consent/purpose

The member stores a file in a room they belong to. The host's purpose is to keep the ciphertext and serve it to the room's members; it cannot read it.
