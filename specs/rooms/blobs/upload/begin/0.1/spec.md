---
slug: rooms/blobs/upload/begin
version: "0.1"
title: "Rooms Blobs — Upload — Begin"
summary: "A member opens an upload of one file's ciphertext into a data room, committing to its size and every chunk's digest before any byte moves, and reserving space for it against the member's, the room's and the host's limits."
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
    The request opens a writable slot in a room the host does not govern and pre-commits the manifest every later chunk is checked against. Authority comes only from the room-issued chain it carries, and the slot is bound to the party the proof authenticates; an unattributable begin would let anyone holding a captured presentation spend a member's quota.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. The presentation it carries is valid for a bounded time, and a replayed begin must be placeable in the window in which that presentation was.
sideEffects:
  level: mutating
  rationale: >-
    Opens an upload slot and reserves its size and one file against every limit that applies. Nothing in the room changes: staged bytes are inert until commit, and a slot never committed expires having released its reservation.
subjectPath: /roomId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a ciphertext size and digests, which describe bytes that reveal nothing of the file, and a presentation that on `open` and `attributed` rooms names the acting member, as every room write does. The response is a handle, the missing indices and an expiry.
retention:
  class: exchange
  rationale: >-
    The slot, its reservation and any staged chunks live until commit, abort or expiry, and no longer.
maxDocumentBytes:
  request: 327680
  rationale: >-
    The request carries the chunk manifest and an authority chain. A full manifest is 4096 sha2-256 digests, each a 34-byte multihash that encodes to about 47 base58btc characters, so about 205 KiB with its JSON punctuation. An authority chain of up to 8 credentials, plus the membership credential, adds up to about 64 KiB more, and the envelope and proof a few KiB; 320 KiB holds the largest legitimate request with room to spare. That exceeds the 64 KiB a host applies to a Trust Task by default. A 100 MiB file at 256 KiB chunks needs 400 digests, well inside it.
errorCodes:
  - code: rooms/blobs/upload/begin:filesDisabled
    meaning: "The host stores no files for this room. Records without files are unaffected."
    retryable: false
  - code: rooms/blobs/upload/begin:invalidManifest
    meaning: "`chunks.chunkCount` is not ceil(size / chunks.chunkSize), or `chunks.chunkDigests` does not have exactly `chunkCount` items, or a digest names a hash the host does not implement."
    retryable: false
  - code: rooms/blobs/upload/begin:limitExceeded
    meaning: "The upload would not fit one of the limits that apply. `details` names the scope whose limit refuses it (for `maxFileBytes`, the scope the smallest applicable limit comes from), the measure, the limit, current use with reservations, and what this upload would add."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [scope, measure, limit, used, requested]
      properties:
        scope:
          type: string
          enum: [member, room, storage, host]
        measure:
          type: string
          enum: [maxFiles, maxFileBytes, maxBytes]
        limit:
          type: integer
          minimum: 0
        used:
          type: integer
          minimum: 0
        requested:
          type: integer
          minimum: 0
  - code: rooms/blobs/upload/begin:tooManyUploads
    meaning: "The authenticated party already holds as many open uploads as the host allows one party. Commit, abort or let one lapse, then begin again. `details.maxOpen`, when present, states the host's limit."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        maxOpen:
          type: integer
          minimum: 1
related:
  - rooms/blobs/upload/chunk
  - rooms/blobs/upload/commit
  - rooms/blobs/upload/abort
  - rooms/blobs/get
  - rooms/records/put
  - backup/put-chunk
---

## Abstract

A file in a data room is **a record plus a blob**. The record is an ordinary room record; its sealed body carries the file's name, type, plaintext digest and the reference to its blob (the `FileManifest` of [`rooms/_shared/0.1/blobs.schema.json`](../../../../_shared/0.1/blobs.schema.json)). The blob is the file's ciphertext, sealed by the member's client under a key derived from the room's storage key, and stored by the host outside the record. A host never sees a file's plaintext, its name or its key.

A blob does not fit in one Trust Task document: a room may accept files of up to 1 GiB, and a host accepts 64 KiB per document by default. So a blob moves as a **chunked transfer**, the same shape [`backup/put-chunk`](../../../../../backup/put-chunk/0.1/spec.md) and `vtc/website/upload/*` use:

1. **`rooms/blobs/upload/begin`** (this task) opens a slot. It carries the room-issued authority to write, and the **ciphertext manifest**: the blob's size, the digest of every chunk and the digest of the whole. The host checks the upload against every limit that applies and reserves space for it.
2. [`rooms/blobs/upload/chunk`](../../chunk/0.1/spec.md) sends each chunk by index; each is checked against the manifest on arrival.
3. [`rooms/blobs/upload/commit`](../../commit/0.1/spec.md) checks the whole against the manifest, hands the blob to the host's store, and returns its **BlobRef**: the digest of the manifest.
4. [`rooms/blobs/upload/abort`](../../abort/0.1/spec.md) discards a slot early and releases its reservation.

The member then writes the record that names the blob, with [`rooms/records/put/0.2`](../../../../records/put/0.2/spec.md). Reading is the mirror image: [`rooms/blobs/get`](../../../get/0.1/spec.md) returns the manifest and a download handle, and [`rooms/blobs/chunk`](../../../chunk/0.1/spec.md) returns each chunk.

**Resume** is this task again. A member whose upload was interrupted sends `begin` with the same room and the same manifest; the host recognises the open slot and answers with the same `uploadId` and the indices it still lacks.

**A blob the room already holds** is not uploaded twice. If the manifest's BlobRef is already committed in this room, the host answers `alreadyCommitted: true` with nothing missing, and the member commits at once; no byte moves and nothing is charged again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the member, or an agent acting under an attenuated chain):

1. **MUST** seal the file as `FileManifest` defines, under a fresh `fileId`, before computing the manifest, so that the manifest describes ciphertext only.
2. **MUST** present the entire authority chain, leaf first, ending in a credential issued by the room.
3. **MUST** resume an interrupted upload by sending this task again with the identical manifest, never by beginning a second upload of the same blob.

A conforming **host** (`recipient`):

1. **MUST** authorize from the presentation alone, verifying every link of the chain against the room as [`rooms/records/put`](../../../../records/put/0.1/spec.md) requires, and **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) unless it confers `write` at the room's scope and its leaf's subject is the party the host authenticated for this request.
2. **MUST** refuse with `filesDisabled` when the room's limits have `filesEnabled: false`.
3. **MUST** refuse with `invalidManifest` a manifest inconsistent with its own `size`, or naming a hash it does not implement.
4. **MUST** check the upload against every scope at once, before reserving anything: the blob's `size` against the smallest `maxFileBytes` among the member, the room and the host ceiling; the member's and the room's file counts and totals, reservations included, with this blob added; and the capacity of the storage the room is assigned to. It **MUST** refuse with `limitExceeded` when any of them would be exceeded. `details.scope` names the scope whose limit refused: for `maxFileBytes`, the scope the smallest applicable limit was set at, and where two scopes set the same value, the narrower (`member`, then `room`, `storage` and `host`); for a count or a total, the narrowest scope that would be exceeded.
5. **MUST** charge the member scope to the **subject at the root of the presented chain**, not to the leaf, so that an agent writing under an attenuated chain spends its member's allowance. On a `private` room, where the host cannot tell members apart, it **MUST NOT** apply member limits; the room, storage and host limits still hold.
6. **MUST** reserve the blob's size and one file against the member, the room and the storage, atomically with the check, so that two concurrent uploads cannot both fit under a limit only one of them fits.
7. **MUST** mint an unguessable `uploadId`, bind it to the room, the manifest and the party it authenticated, and answer with the id, every index it lacks and the slot's expiry.
8. **MUST**, when the same authenticated party begins again with the same room and an identical manifest while a slot for it is open, answer with that slot's `uploadId` and the indices still missing, and **MUST NOT** reserve a second time.
9. **MUST**, when the manifest's BlobRef is already committed in this room, answer with an `uploadId` bound as item 7 requires, `missing: []` and `alreadyCommitted: true`, and **MUST NOT** reserve anything. A commit of that upload returns the existing blob and charges nothing (see [`rooms/blobs/upload/commit`](../../commit/0.1/spec.md)). The authorization and `filesDisabled` checks still run first, so the answer discloses nothing to a party that could not upload into the room. A BlobRef committed in another room is never reported: it is uploaded and stored again, in this room.
10. **MAY** limit how many uploads one party holds open, and **MUST** refuse a begin beyond that limit with `tooManyUploads`, retryable, rather than a generic failure.
11. **MUST NOT** expire a slot sooner than **15 minutes** after the last chunk it accepted (or after this begin, before any chunk arrives), and **MUST** extend the slot while chunks keep arriving, up to a total lifetime of at least **24 hours**. A client that lets a slot lapse begins again from nothing. These are floors, not targets: a 1 GiB blob over a slow or intermittent link takes hours, and a slot that expired on a fixed short clock would make such an upload impossible however patiently the client resumed it.
12. **MUST** admit, on any binding, a `rooms/blobs/upload/chunk` document up to that task's `maxDocumentBytes` from the party that owns an open upload, whether or not that party holds an access-control entry at the host. Room members ordinarily hold none; an open slot exists only because a verified chain opened it, so admitting its owner admits nobody a verified chain did not.
13. **MUST NOT** accept a slot whose total exceeds the transfer ceiling: 4096 chunks of at most 262144 bytes, so 1 GiB. A room's limits may be lower, never higher.

## Authorization

The authority this task presupposes is the **room's**: a membership credential and an authority chain the room issued, conferring `write` at the room's scope, presented by the party the host authenticated. A host **never** consults an access-control list of its own to decide it, which is invariant I5 of the rooms design and what lets a room move between hosts with its credentials intact. Verifying the document's proof establishes who asked; the chain establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

How much the room may store is a separate question and the host's own: its limits are a hosting decision, set by the host, and nothing in a room's credentials can widen them.

## Definitions

- **Blob** — one file's ciphertext, held by the host outside the record that names it.
- **Ciphertext manifest** — `BlobManifest`: the blob's `size`, its chunk manifest and the digest of the whole.
- **BlobRef** — the digest of the manifest's RFC 8785 canonicalization. Content addressing over ciphertext.
- **Upload** — one chunked transfer from begin to commit, abort or expiry, identified by its `uploadId`.
- **Scopes** — `member` (one member's uploads in one room), `room`, `storage` (the capacity of the store the room is assigned to) and `host` (the host-wide ceiling). One blob's size is bounded by the smallest `maxFileBytes` among the member, the room and the host, so a single object has no limit of its own. Limits are `RoomLimits`; all sizes are ciphertext bytes.

## Request

A member (`issuer`) opens an upload with the room's host (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A 300 000-byte blob in two chunks

The presentation is shortened for the example.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000101",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "roomId": "did:example:room",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.signature",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-leaf.signature",
        "eyJhbGciOiJFZERTQSJ9.authority-room.signature"
      ]
    },
    "manifest": {
      "size": 300000,
      "chunks": {
        "chunkSize": 262144,
        "chunkCount": 2,
        "chunkDigests": [
          "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc",
          "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX"
        ]
      },
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    }
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`: the upload id, the indices it lacks, the slot's expiry and, when the room already holds this blob, `alreadyCommitted: true`. A refusal is a `trust-task-error`.

### Slot opened

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000102",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
    "missing": [0, 1],
    "expiresAt": "2026-10-10T11:00:01Z"
  }
}
```

### The room already holds this blob

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000104",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "uploadId": "7a1c2e3d-4b5f-4a6e-8d7c-9b0a1f2e3d4c",
    "missing": [],
    "expiresAt": "2026-10-10T10:15:01Z",
    "alreadyCommitted": true
  }
}
```

### Refused: the member's allowance is full

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000103",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "code": "rooms/blobs/upload/begin:limitExceeded",
    "message": "This file would take you past your allowance in this room.",
    "retryable": false,
    "details": {
      "scope": "member",
      "measure": "maxBytes",
      "limit": 209715200,
      "used": 209600000,
      "requested": 300000
    }
  }
}
```

## Security & Privacy

**The host stores what it cannot read.** Everything that describes the file — its name, type, plaintext size and plaintext digest — is sealed in the record. The manifest here describes ciphertext only, so what the host learns is that a member stores a blob of a given size at a given time, which it would learn from storing it anyway.

**Quota is charged to the person, not the key.** Charging the root subject of the chain means a member cannot multiply their allowance by minting agents, and an agent's uploads count against the member who authorized it.

**Refusals name the limit.** A refusal says which scope refused and by how much, so the member knows whether to delete their own files or ask the room's owner or the host. The numbers disclose nothing the member could not compute from their own uploads, except the room's total, which every member may read through `rooms/info`.

### Data carried

A room identifier, a presentation, a ciphertext size and digests in; a handle, a list of indices and an expiry out. No byte of the file moves in this document.

### Correlation

On `attributed` and `open` rooms the presentation names the acting member, so the host can relate uploads to members, as it already relates writes. On `private` rooms it cannot. The BlobRef depends on ciphertext under a per-file key, so two rooms holding the same file hold different blobs and cannot be correlated by them.

### Retention

The slot and its reservation live until commit, abort or expiry. Expiry is idle-based: at least 15 minutes after the last accepted chunk, extended as chunks arrive up to at least 24 hours in all (Conformance, item 11). A host **SHOULD NOT** keep an idle slot much longer than its floor, since a slot holds a reservation against the room's limits.

### Consent/purpose

A member stores a file in a room they joined by accepting an invitation; the membership credential is that consent. The host's purpose is to store and serve the ciphertext to the room's members, and nothing in this task authorizes it to do more.
