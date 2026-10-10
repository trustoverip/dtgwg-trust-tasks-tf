---
slug: rooms/blobs/chunk
version: "0.1"
title: "Rooms Blobs — Chunk"
summary: "A member fetches one chunk of a blob download they opened, and checks it against the manifest on arrival."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - file
  - blob
  - download
  - chunk
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The only authority for a chunk is owning the download it belongs to, which is a comparison against the handle's owner that has to be attributable on every transport. The response needs no proof for integrity — the reader checks each chunk against the manifest it verified — but one is recommended so a served chunk is attributable to the host.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed request re-serves bytes the owner already fetched, and may extend the handle's expiry. Bounding replay stops a captured request from keeping a download open after its owner stopped.
sideEffects:
  level: none
  rationale: >-
    Reads one chunk and counts its bytes as egress in the room's usage. Nothing in the room changes.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses a range of ciphertext sealed under a per-file key, to the party that opened the download under the room's read authority. It is opaque without that key.
retention:
  class: transient
  rationale: >-
    The host keeps nothing of the request beyond the egress count.
maxDocumentBytes:
  response: 360448
  rationale: >-
    A chunk is up to 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the other payload members and a proof. The 262144-byte ceiling is the one vta/_shared/0.1/backup-transfer derives for a chunk document to survive a 1 MiB mediator message after DIDComm encoding and two forward wrappers.
errorCodes:
  - code: rooms/blobs/chunk:notFound
    meaning: "No open download with this id exists that the caller may read from. Conflates an unknown id, an expired download, and another party's download."
    retryable: false
  - code: rooms/blobs/chunk:chunkOutOfRange
    meaning: "`index` is not below the manifest's `chunkCount`."
    retryable: false
related:
  - rooms/blobs/get
  - backup/get-chunk
---

## Abstract

The **Rooms Blobs — Chunk** Trust Task returns one chunk of a blob whose download was opened by [`rooms/blobs/get`](../../get/0.1/spec.md). Chunks may be fetched in any order, in parallel, and again; the reader checks each against the manifest it verified, so the host is trusted for availability only.

This is the room counterpart of [`backup/get-chunk`](../../../../backup/get-chunk/0.1/spec.md), with the same chunk vocabulary. A chunk request carries no room presentation: the open download is its authority, and it belongs to the party that opened it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the reader):

1. **MUST** check each chunk's decoded bytes against the digest at its index in the manifest it verified against the BlobRef, and **MUST** refuse a chunk that does not match, whatever `digestMultibase` the response restates.
2. **MUST** accept a response up to this task's `maxDocumentBytes.response` from the host serving its download.

A conforming **host** (`recipient`):

1. **MUST** answer `notFound` unless `downloadId` names an open download opened by the party the host authenticated for this request. It **MUST NOT** distinguish an unknown id from another party's download.
2. **MUST** refuse with `chunkOutOfRange` an index not below `chunkCount`.
3. **MUST** return the chunk's bytes exactly as committed, with the digest the manifest states for that index.
4. **MAY** extend the download's expiry as chunks are fetched, never past its own ceiling.

## Authorization

The authority this task presupposes is **ownership of an open download**: the room-issued chain conferring `read` was verified when the download was opened, and the download is bound to the party the host authenticated then. The host consults no access-control list of its own (rooms invariant I5). Verifying the proof establishes who is asking; ownership of the download establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`index`**, **`data`**, **`digestMultibase`** — as `backup/get-chunk` defines them: a zero-based position, the chunk's bytes in unpadded base64url, and the digest of the raw bytes.

## Request

The member who opened the download (`issuer`) asks the host (`recipient`) for one chunk.

### The first chunk

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000151",
  "type": "https://trusttasks.org/spec/rooms/blobs/chunk/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T12:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000151ff",
  "payload": {
    "downloadId": "7a1c2e4b-5d6f-4a8b-9c0d-1e2f3a4b5c6d",
    "index": 0
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One chunk

Shortened for the example: a real chunk carries up to 349526 characters of `data`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000152",
  "type": "https://trusttasks.org/spec/rooms/blobs/chunk/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T12:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000151ff",
  "payload": {
    "index": 0,
    "digestMultibase": "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc",
    "data": "q83vASNFZ4k"
  }
}
```

## Security & Privacy

**The size gate runs both ways.** A response carrying a chunk exceeds the 64 KiB a party accepts by default. A reader expecting chunks accepts them because it opened the download; a host sends them only to the party that did.

### Data carried

A download handle and an index in; a range of ciphertext and its digest out.

### Correlation

The download handle links the requests of one download, which is its purpose. Egress is counted in the room's usage as totals.

### Retention

The host keeps nothing of the request beyond its egress count. The reader **SHOULD** discard ciphertext once decrypted, and **SHOULD NOT** write decrypted plaintext anywhere the member did not ask it to.

### Consent/purpose

A member reads a file in a room they joined; their membership is the consent. The host's purpose is to serve the room's ciphertext to its members.
