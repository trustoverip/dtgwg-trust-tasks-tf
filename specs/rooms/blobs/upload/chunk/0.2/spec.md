---
slug: rooms/blobs/upload/chunk
version: "0.2"
title: "Rooms Blobs — Upload — Chunk"
summary: "An upload's single-use key sends one chunk of an open blob upload by index, checked on arrival against the digest committed for that index."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - file
  - blob
  - upload
  - chunk
parties:
  - role: Upload key
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
    The only authority for a chunk is being the upload key its upload was bound to at begin, which is a comparison against the proof's signer that has to hold on every write. The response is small and is the producer's evidence that a chunk was stored.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replay of identical bytes changes nothing and a replay of different bytes is refused against the manifest — but a replay still extends the slot's expiry, and bounding it is what stops a captured chunk from keeping an upload, and its reservation, open.
sideEffects:
  level: mutating
  rationale: >-
    Stages one chunk. Nothing in the room changes; staged chunks are inert until commit, and discarded at abort or expiry.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a range of ciphertext sealed under a per-file key the host never holds, so it is opaque to the host; the response is an acknowledgement and a count.
retention:
  class: exchange
  rationale: >-
    A staged chunk lives only as long as its upload.
maxDocumentBytes:
  request: 360448
  rationale: >-
    A chunk is up to 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the other payload members and a proof. The 262144-byte ceiling is the one vta/_shared/0.1/backup-transfer derives for a chunk document to survive a 1 MiB mediator message after DIDComm encoding and two forward wrappers.
errorCodes:
  - code: rooms/blobs/upload/chunk:notFound
    meaning: "No open upload with this id exists that the caller may write to. Conflates an unknown id, an expired, aborted or committed upload, and another party's upload."
    retryable: false
  - code: rooms/blobs/upload/chunk:chunkOutOfRange
    meaning: "`index` is not below the manifest's `chunkCount`."
    retryable: false
  - code: rooms/blobs/upload/chunk:chunkMismatch
    meaning: "`digestMultibase` differs from the manifest entry for `index`, the decoded `data` does not match it, or its length is wrong for its position (every chunk but the last is exactly `chunkSize`). The chunk is not stored; re-send the right bytes."
    retryable: false
related:
  - rooms/blobs/upload/begin
  - rooms/blobs/upload/commit
  - backup/put-chunk
---

## Abstract

The **Rooms Blobs — Upload — Chunk** Trust Task writes one chunk of an upload opened by [`rooms/blobs/upload/begin`](../../begin/0.2/spec.md). The chunk restates the digest committed for its index, and the host checks the restated digest against the manifest and the bytes against both, so a chunk sent to the wrong index, or corrupted on the way, is refused on arrival rather than at reassembly.

Chunks may arrive in any order and any may be re-sent: an identical re-send succeeds with `stored: false`. This is the room counterpart of [`backup/put-chunk`](../../../../../backup/put-chunk/0.1/spec.md), with the same chunk vocabulary.

This task exists only in **proxied** mode. In direct mode the bytes go straight to the room's storage and no chunk document is sent.

A chunk carries no room presentation. The open upload is its authority: it exists only because a verified room-issued chain opened it, and the begin that opened it named the one key that may send its chunks. That key is generated for this upload alone and identifies nobody; the upload's owner, recorded at begin, is the member.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`0.1` was authenticated by the party that began the upload. `0.2` is authenticated by the upload's **upload key**: the single-use `did:key` that [`rooms/blobs/upload/begin/0.2`](../../begin/0.2/spec.md) named, so that the party that began the upload — the member, whose keys their key holder keeps — never signs over the file's bytes. The payload and response are unchanged; who may send this document is what changed, which is a behavioural break for a consumer and so a new version. A `0.2` upload accepts only `0.2` documents, and a `0.1` upload only `0.1`.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer**:

1. **MUST** sign each chunk with the upload key the begin named, send chunks only for that upload, and restate for each the digest it committed at that index.

A conforming **host** (`recipient`):

1. **MUST** admit this document, on any binding, up to its `maxDocumentBytes` when its authenticated sender is the upload key of the open upload it names, whether or not that key holds an access-control entry at the host — it never does. It **SHOULD** refuse a larger document, or one naming no upload the sender owns, before parsing more than it must.
2. **MUST** answer `notFound` for an upload in direct mode, which has no chunk documents, and unless `uploadId` names an open proxied upload whose upload key is the party the host authenticated for this request. It **MUST NOT** distinguish an unknown id from another key's upload, and **MUST** answer `notFound` to the upload's owner signing as itself: the owner named a key for the transfer and does not send it.
3. **MUST** refuse with `chunkOutOfRange` an index not below `chunkCount`, and with `chunkMismatch` a chunk whose restated digest, decoded bytes or length disagree with the manifest.
4. **MUST** answer an identical re-send with `stored: false`. Different bytes at an already-stored index cannot arise: every chunk's digest was committed at begin, so they fail item 3 as `chunkMismatch` before anything is compared with what is staged.
5. **MUST** extend the upload's expiry on each accepted chunk to no sooner than 15 minutes later, up to a total slot lifetime of at least 24 hours, as [`rooms/blobs/upload/begin`](../../begin/0.2/spec.md) requires, and **MUST** report the current expiry and how many indices remain.

## Authorization

The authority this task presupposes is **being the upload's key**: the room-issued chain was verified when the upload began, and the begin, signed by the party that chain authorized, named the key that may send its chunks. That key, and only that key, may. The host consults no access-control list of its own (rooms invariant I5). Verifying the proof establishes who is sending; ownership of the upload establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`index`**, **`data`**, **`digestMultibase`** — as `backup/put-chunk` defines them: a zero-based position, the chunk's bytes in unpadded base64url, and the digest of the raw bytes.
- **`remainingCount`** — indices still missing; zero means the upload can be committed.

## Request

The upload key (`issuer`) sends one chunk to the host (`recipient`).

### The second and last chunk

Shortened for the example: a real chunk carries up to 349526 characters of `data`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000111",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/chunk/0.2#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T10:00:05Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000111ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
    "index": 1,
    "digestMultibase": "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
    "data": "q83vASNFZ4k"
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Stored; nothing remains

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000112",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/chunk/0.2#response",
  "issuer": "did:example:host",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-10T10:00:06Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000111ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
    "index": 1,
    "stored": true,
    "remainingCount": 0,
    "expiresAt": "2026-10-10T11:00:06Z"
  }
}
```

## Security & Privacy

**The size gate is ownership, not standing.** A host that applied its usual rule — documents above the default cap only from parties with an access-control entry — would refuse every chunk from a room member, who has none. Admitting the key of an open upload instead admits nobody a verified room chain did not name, and charges the same budget for large documents.

**The key that signs the bytes is not the member's.** A chunk document is signed over its bytes, so whoever signs it sees them. Signing with a key generated for the upload keeps the file off the member's key holder, which by design never handles a file's bytes, while the begin keeps the upload the member's.

### Data carried

A range of ciphertext in, an acknowledgement out. The ciphertext is sealed under a per-file key the host never holds.

### Correlation

The upload id and the upload key link the documents of one transfer, which is their purpose. Both are minted for this upload alone, so neither links it to any other upload, device or member.

### Retention

Staged chunks are discarded at commit (once reassembled and stored), abort, or expiry.

### Consent/purpose

The member stores a file in a room they belong to; the consent is the membership that let them begin the upload. The host's purpose is to store the ciphertext and serve it back to the room's members.
