---
slug: rooms/blobs/upload/begin
version: "0.2"
title: "Rooms Blobs — Upload — Begin"
summary: "A member opens an upload of one file's ciphertext into a data room, committing to its size and digests and reserving space against every limit; the host answers with how the bytes move — straight to storage over per-part URLs, or through the host."
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
    The request opens a writable slot in a room the host does not govern, pre-commits the manifest and part plan every later byte is checked against, and names the key that may send those chunks in proxied mode. Authority comes only from the room-issued chain it carries, presented by the party the proof authenticates; an unattributable begin would let anyone holding a captured presentation spend a member's quota, or name a key of their own to fill it.
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
    The request carries a ciphertext size and digests, which describe bytes that reveal nothing of the file, a presentation that on `open` and `attributed` rooms names the acting member, as every room write does, and an upload key that identifies nobody. The response is a handle, the missing indices, an expiry and, in direct mode, time-limited storage URLs — each a bearer credential for one write of one part, for minutes.
retention:
  class: exchange
  rationale: >-
    The slot, its reservation and any staged chunks or parts live until commit, abort or expiry, and no longer; parts written to storage for an upload never committed are reclaimed.
maxDocumentBytes:
  request: 393216
  response: 327680
  rationale: >-
    The request adds to 0.1's (below) a part plan of at most 1024 sha2-256 digests, about 50 KiB, so 384 KiB. The response's largest member is `targets`: at most 128 storage URLs of up to about 2 KiB each with their signed headers, so 320 KiB.
    0.1's request carries the chunk manifest and an authority chain. A full manifest is 4096 sha2-256 digests, each a 34-byte multihash that encodes to about 47 base58btc characters, so about 205 KiB with its JSON punctuation. An authority chain of up to 8 credentials, plus the membership credential, adds up to about 64 KiB more, and the envelope and proof a few KiB; 320 KiB holds the largest legitimate request with room to spare. That exceeds the 64 KiB a host applies to a Trust Task by default. A 100 MiB file at 256 KiB chunks needs 400 digests, well inside it.
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
  - code: rooms/blobs/upload/begin:noTransferMode
    meaning: "No transfer mode is open to both sides: the producer offered no `uploadKey` (so accepts no proxied transfer) and the host will not transfer this upload directly — the room's storage cannot issue part URLs, the room's tier or the host's policy rules it out, or the part plan does not suit the storage. `details.modes` names the modes the host would use."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [modes]
      properties:
        modes:
          type: array
          uniqueItems: true
          items:
            type: string
            enum: [direct, proxied]
  - code: rooms/blobs/upload/begin:invalidParts
    meaning: "`directParts` is inconsistent with the manifest: `partCount` is not ceil(size / partSize), `partDigests` does not have `partCount` items, or a digest is not sha2-256."
    retryable: false
  - code: rooms/blobs/upload/begin:uploadKeyInUse
    meaning: "`uploadKey` is already bound to another open upload. An upload key is minted for one upload and dies with it; generate a fresh one."
    retryable: false
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
  - rooms/keys/act
  - rooms/blobs/get
  - rooms/records/put
  - backup/put-chunk
---

## Abstract

A file in a data room is **a record plus a blob**. The record is an ordinary room record; its sealed body carries the file's name, type, plaintext digest and the reference to its blob (the `FileManifest` of [`rooms/_shared/0.1/blobs.schema.json`](../../../../_shared/0.1/blobs.schema.json)). The blob is the file's ciphertext, sealed by the member's client under a key derived from the room's storage key, and stored by the host outside the record. A host never sees a file's plaintext, its name or its key.

A blob does not fit in one Trust Task document: a room may accept files of up to 1 GiB, and a host accepts 64 KiB per document by default. So a blob moves in one of two **transfer modes**, and this task is where the host says which.

**Direct.** Where the storage the room is assigned to can issue time-limited URLs for one object and one method — S3-class object storage presigning its requests is the model — the bytes go straight between the member and the storage, and the host carries none of them. The host still decides every object: it authorizes the begin against the room's chain and quota exactly as for a proxied upload, chooses the object, and answers with a PUT target per part, each bound to that object, that method, the part's exact length and its SHA-256, so that storage itself refuses any other bytes. The member writes the parts, then commits; the host checks the object at storage before it records anything.

**Proxied.** Otherwise — storage that cannot presign, a `private` room (see Security & Privacy), or a member who prefers it — the bytes move through the host as a **chunked transfer**, the same shape [`backup/put-chunk`](../../../../../backup/put-chunk/0.1/spec.md) and `vtc/website/upload/*` use:

1. **`rooms/blobs/upload/begin`** (this task) opens a slot. It carries the room-issued authority to write, and the **ciphertext manifest**: the blob's size, the digest of every chunk and the digest of the whole. It names the **upload key** that will sign the rest of the transfer. The host checks the upload against every limit that applies and reserves space for it.
2. [`rooms/blobs/upload/chunk`](../../chunk/0.2/spec.md) sends each chunk by index, signed by the upload key; each is checked against the manifest on arrival.
3. [`rooms/blobs/upload/commit`](../../commit/0.2/spec.md) checks the whole against the manifest, hands the blob to the host's store, and returns its **BlobRef**: the digest of the manifest.
4. [`rooms/blobs/upload/abort`](../../abort/0.2/spec.md) discards a slot early and releases its reservation.

In direct mode there are no chunk documents: commit and abort are the member's own (signed, like this request, through [`rooms/keys/act`](../../../../keys/act/0.1/spec.md)), and no upload key is needed.

The member then writes the record that names the blob, with [`rooms/records/put/0.2`](../../../../records/put/0.2/spec.md). Reading is the mirror image: [`rooms/blobs/get/0.2`](../../../get/0.2/spec.md) returns the manifest and either a time-limited GET for the object (direct) or a download handle for [`rooms/blobs/chunk`](../../../chunk/0.1/spec.md) (proxied).

**The upload key** (proxied mode). The member signs this request — usually through their key holder, with [`rooms/keys/act`](../../../../keys/act/0.1/spec.md), because the member's keys never leave it. The chunks are another matter: a file's bytes must not pass through the key holder, and a chunk document is signed over its bytes. So the begin names `uploadKey`, a `did:key` the client generates for this one upload, and every chunk, commit and abort of the upload is signed by that key instead of by the member. The key is bound to one `uploadId`, identifies nobody, and is worth nothing once the upload ends. The host records the member — the presenter of this request — as the upload's owner and, wherever it records one, its uploader; never the upload key.

**Resume** is this task again. A member whose upload was interrupted sends `begin` with the same room, the same manifest and the same part plan; the host recognises the open slot and answers with the same `uploadId` and the indices it still lacks — and, in direct mode, fresh targets for them. That is also how a direct upload gets targets beyond the first batch, or replaces targets that lapsed.

**A blob the room already holds** is not uploaded twice. If the manifest's BlobRef is already committed in this room, the host answers `alreadyCommitted: true` with nothing missing, and the member commits at once; no byte moves and nothing is charged again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`0.1` bound an upload to the party that began it, and let only that party send its chunks, commit it or abort it. That made the uploader's signing key sign every chunk document, and so see every chunk's bytes. A member whose keys are held by their own agent — which is every member acting through a key holder — could then upload as the member only by passing the whole file through that agent, which the rooms design forbids ([`rooms/keys/file-key`](../../../../keys/file-key/0.1/spec.md)); or upload as their client's own key, which the host then records as the uploader in the member's place.

`0.2` separates the two, and adds a mode in which the bytes do not pass through the host either.

- The begin is the member's act, signed by the member and presenting the member's authority. In **proxied** mode the transfer is the client's, signed by `uploadKey`, a single-use key the begin names.
- In **direct** mode the bytes go to the room's storage over URLs the host binds to one part each, and commit and abort are signed by the member, like the begin. The producer offers direct transfer by sending `directParts`; the response's `mode` says which applies.

The request gains two members and the response two, one of them REQUIRED, so this is a breaking change to a `draft` and a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). [`rooms/blobs/upload/chunk/0.2`](../../chunk/0.2/spec.md), [`commit/0.2`](../../commit/0.2/spec.md) and [`abort/0.2`](../../abort/0.2/spec.md) are the same tasks as their `0.1`, authenticated by the upload key in proxied mode and by the owner in direct mode; a `0.1` chunk, commit or abort never reaches a `0.2` upload.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the member, or an agent acting under an attenuated chain):

1. **MUST** seal the file as `FileManifest` defines, under a fresh `fileId`, before computing the manifest, so that the manifest describes ciphertext only.
2. **MUST** offer at least one mode: `uploadKey` for proxied transfer, `directParts` for direct transfer, or both.
3. **MUST**, when it sends `uploadKey`, generate it afresh for this upload, from no key it holds for any other purpose, and **MUST NOT** use it for anything but this upload's chunk, commit and abort documents. It **SHOULD** discard the key's secret once the upload is committed or aborted.
4. **MUST**, when it sends `directParts`, cut the ciphertext into the parts it describes, and in direct mode **MUST** write each part only to its own target, with exactly the headers the target names, and **MUST NOT** log a target's URL.
5. **MUST** present the entire authority chain, leaf first, ending in a credential issued by the room.
6. **MUST** resume an interrupted upload by sending this task again with the identical manifest and part plan, never by beginning a second upload of the same blob.

A conforming **host** (`recipient`):

1. **MUST** authorize from the presentation alone, verifying every link of the chain against the room as [`rooms/records/put`](../../../../records/put/0.1/spec.md) requires, and **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) unless it confers `write` at the room's scope and its leaf's subject is the party the host authenticated for this request.
2. **MUST** refuse with `filesDisabled` when the room's limits have `filesEnabled: false`.
3. **MUST** refuse with `invalidManifest` a manifest inconsistent with its own `size`, or naming a hash it does not implement.
4. **MUST** check the upload against every scope at once, before reserving anything: the blob's `size` against the smallest `maxFileBytes` among the member, the room and the host ceiling; the member's and the room's file counts and totals, reservations included, with this blob added; and the capacity of the storage the room is assigned to. It **MUST** refuse with `limitExceeded` when any of them would be exceeded. `details.scope` names the scope whose limit refused: for `maxFileBytes`, the scope the smallest applicable limit was set at, and where two scopes set the same value, the narrower (`member`, then `room`, `storage` and `host`); for a count or a total, the narrowest scope that would be exceeded.
5. **MUST** charge the member scope to the **subject at the root of the presented chain**, not to the leaf, so that an agent writing under an attenuated chain spends its member's allowance. On a `private` room, where the host cannot tell members apart, it **MUST NOT** apply member limits; the room, storage and host limits still hold.
6. **MUST** reserve the blob's size and one file against the member, the room and the storage, atomically with the check, so that two concurrent uploads cannot both fit under a limit only one of them fits.
7. **MUST** mint an unguessable `uploadId`, bind it to the room, the manifest, the part plan, the transfer mode, the party it authenticated (the upload's **owner**) and, in proxied mode, `uploadKey`, and answer with the id, the mode, every index it lacks and the slot's expiry.
8. **MUST**, when the same authenticated party begins again with the same room and an identical manifest while a slot for it is open, answer with that slot's `uploadId` and the indices still missing, in direct mode **MUST** issue fresh targets for missing parts, in proxied mode **MUST** rebind the slot to the `uploadKey` this request names — so a client that lost its upload key can resume — and **MUST NOT** reserve a second time. The previously bound key **MUST** then be refused.
9. **MUST**, when the manifest's BlobRef is already committed in this room, answer with an `uploadId` bound as item 7 requires, `missing: []` and `alreadyCommitted: true`, and **MUST NOT** reserve anything. A commit of that upload returns the existing blob and charges nothing (see [`rooms/blobs/upload/commit`](../../commit/0.2/spec.md)). The authorization and `filesDisabled` checks still run first, so the answer discloses nothing to a party that could not upload into the room. A BlobRef committed in another room is never reported: it is uploaded and stored again, in this room.
10. **MAY** limit how many uploads one party holds open, and **MUST** refuse a begin beyond that limit with `tooManyUploads`, retryable, rather than a generic failure.
11. **MUST NOT** expire a slot sooner than **15 minutes** after the last chunk it accepted (or after this begin, before any chunk arrives; in direct mode, after the last begin that resumed it, since the host sees no chunks), and **MUST** extend the slot while chunks keep arriving, up to a total lifetime of at least **24 hours**. A client that lets a slot lapse begins again from nothing. These are floors, not targets: a 1 GiB blob over a slow or intermittent link takes hours, and a slot that expired on a fixed short clock would make such an upload impossible however patiently the client resumed it.
12. **MUST** admit, on any binding, a `rooms/blobs/upload/chunk` document up to that task's `maxDocumentBytes` from the upload key bound to an open upload, whether or not it holds an access-control entry at the host. An upload key never holds one; an open slot exists only because a verified chain opened it and named the key, so admitting the key admits nobody a verified chain did not.
13. **MUST NOT** accept a slot whose total exceeds the transfer ceiling: 4096 chunks of at most 262144 bytes, so 1 GiB. A room's limits may be lower, never higher.
14. **MUST**, in proxied mode, refuse with `uploadKeyInUse` an `uploadKey` already bound to another open upload, and **MUST** accept, for this upload's chunk, commit and abort documents, a proof by `uploadKey` and by nobody else — not the owner, not any other key.
15. **MUST NOT** record, report or attribute anything to `uploadKey`. The owner is the party it authenticated for this request; usage is charged to the subject at the root of its chain (item 5); and wherever the host records who uploaded a blob, it records the owner. In direct mode the host **MUST** ignore an `uploadKey` the producer also sent, and accept commit and abort from the owner alone. Once the upload is committed, aborted or expired, the binding ends and the key **MUST** be refused for anything but an idempotent repeat of the commit or abort that ended it.

16. **MUST** choose the mode after every check above has passed, and before reserving: direct when the producer sent `directParts`, the room's storage can issue the targets item 17 requires for a plan of that shape, and the room's tier and the host's policy allow it; otherwise proxied when the producer sent `uploadKey`; otherwise refuse with `noTransferMode`. It **MUST** refuse with `invalidParts` a part plan inconsistent with the manifest. It **SHOULD NOT** choose direct transfer for a `private` room unless its policy explicitly allows it (see Security & Privacy).
17. **MUST**, in direct mode, choose the storage object itself — never a name the producer supplied — and issue each target for that one object, one part, the method `PUT`, and with the part's exact length and SHA-256 bound into the URL's signature as headers storage enforces (for S3-class storage, `Content-Length` and `x-amz-checksum-sha256` as signed headers), so that storage refuses any other bytes. Each target **MUST** expire within **5 minutes** of issue, and sooner where practical; the storage checks expiry when a request begins, so a part that takes longer to send still completes. A host **MAY** issue targets for only some missing parts at a time.
18. **MUST NOT** log a target's URL, nor anything from which it can be rebuilt.

## Authorization

The authority this task presupposes is the **room's**: a membership credential and an authority chain the room issued, conferring `write` at the room's scope, presented by the party the host authenticated. `uploadKey` carries no authority of its own: it is admitted afterwards only because this request, authorized by the chain, named it. A host **never** consults an access-control list of its own to decide it, which is invariant I5 of the rooms design and what lets a room move between hosts with its credentials intact. Verifying the document's proof establishes who asked; the chain establishes that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

How much the room may store is a separate question and the host's own: its limits are a hosting decision, set by the host, and nothing in a room's credentials can widen them.

## Definitions

- **Blob** — one file's ciphertext, held by the host outside the record that names it.
- **Ciphertext manifest** — `BlobManifest`: the blob's `size`, its chunk manifest and the digest of the whole.
- **BlobRef** — the digest of the manifest's RFC 8785 canonicalization. Content addressing over ciphertext.
- **Upload** — one transfer from begin to commit, abort or expiry, identified by its `uploadId`, in one mode.
- **Direct** / **proxied** — the two transfer modes (`TransferMode`): bytes straight to the room's storage over host-issued targets, or through the host as chunk documents.
- **Part plan** — `directParts` (`PartPlan`): how the producer cuts the blob into parts for direct transfer, with each part's sha2-256 digest. Independent of the chunk manifest, because storage imposes part sizes that chunk documents cannot carry.
- **Target** — a `TransferTarget`: one time-limited, single-object, single-method URL at the room's storage.
- **Owner** — the party the host authenticated for the begin: the member, or an agent acting under an attenuated chain. What the host records and charges.
- **Upload key** — `uploadKey`: a `did:key` the producer generates for one upload, which signs its chunk, commit and abort documents and nothing else. Not a member, not an owner, and never recorded as either.
- **Scopes** — `member` (one member's uploads in one room), `room`, `storage` (the capacity of the store the room is assigned to) and `host` (the host-wide ceiling). One blob's size is bounded by the smallest `maxFileBytes` among the member, the room and the host, so a single object has no limit of its own. Limits are `RoomLimits`; all sizes are ciphertext bytes.

## Request

A member (`issuer`) opens an upload with the room's host (`recipient`), offering the transfer modes it accepts. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A 300 000-byte blob, either way

The member offers both modes: an upload key for a proxied transfer, and a part plan — here one part, the whole blob — for a direct one. The presentation is shortened for the example.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000101",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.2#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "roomId": "did:example:room",
    "uploadKey": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
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
    },
    "directParts": {
      "partSize": 300000,
      "partCount": 1,
      "partDigests": ["zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"]
    }
  }
}
```

## Response

The host answers with the sub-schema reachable via `$anchor: "response"`: the upload id, the transfer mode, the indices it lacks, the slot's expiry, in direct mode the targets to write the missing parts to and, when the room already holds this blob, `alreadyCommitted: true`. A refusal is a `trust-task-error`.

### Direct: write the part to storage

The URL is shortened for the example; a real one carries the storage's signature over the object, the method, the expiry and the two headers.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000105",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.2#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
    "mode": "direct",
    "missing": [0],
    "expiresAt": "2026-10-10T11:00:01Z",
    "targets": [
      {
        "index": 0,
        "method": "PUT",
        "url": "https://storage.example/rooms/b7c1e2/3e0f5a7c?X-Amz-Signature=…",
        "headers": {
          "Content-Length": "300000",
          "x-amz-checksum-sha256": "4f3rT8yq1n0pYxXg2j3QmR0b8gq2sR5uJ6vXl0yH7aE="
        },
        "expiresAt": "2026-10-10T10:05:01Z"
      }
    ]
  }
}
```

### Proxied: send the chunks, signed by the upload key

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000102",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.2#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "uploadId": "3e0f5a7c-9b21-4d8e-a6c4-2f1b0d9e8c7a",
    "mode": "proxied",
    "missing": [0, 1],
    "expiresAt": "2026-10-10T11:00:01Z"
  }
}
```

### The room already holds this blob

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000104",
  "type": "https://trusttasks.org/spec/rooms/blobs/upload/begin/0.2#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "uploadId": "7a1c2e3d-4b5f-4a6e-8d7c-9b0a1f2e3d4c",
    "mode": "direct",
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

**A target is a bearer credential for its lifetime.** Whoever holds the URL may make that one request, so a host binds each to one object, one method, and the part's exact length and checksum, which storage itself enforces; gives it the shortest practical lifetime, never more than 5 minutes; and never logs it (items 17 and 18), and neither does the producer. What a leaked target allows is writing the exact bytes the member committed to, into the object the host chose, for minutes — nothing else.

**Direct transfer shows the member to the storage provider.** The storage sees the member's network address and which object they wrote. On `open` and `attributed` rooms the host already knows who writes, and the provider learns no more than that a member of some room wrote an opaque object. On a `private` room, the tier exists to deny any party the link between a member and their activity in the room, and the provider's view is exactly that link — which is why a host **SHOULD NOT** offer direct transfer for a `private` room unless its policy explicitly allows it, and why the response always names the mode, so a member who wants the host between them and the provider can see whether it is.

**The host's control holds without the bytes.** It authorizes every object at begin, reserves against every limit before issuing a target, and checks the object at storage before commit records it ([`rooms/blobs/upload/commit/0.2`](../../commit/0.2/spec.md)); an object written and never committed is reclaimed. And rate limits fall where they matter: on the few control documents per file — begin, commit, abort — rather than on a document per 256 KiB of bytes, so a host can bound requests per member tightly without bounding the size of the files they share.

**The upload key identifies nobody.** It is generated for one upload and dies with it, so nothing the host sees on the transfer links one upload to another or to a device. The member is named once, on the begin, as every room write names its writer.

**Quota is charged to the person, not the key.** Charging the root subject of the chain means a member cannot multiply their allowance by minting agents, and an agent's uploads count against the member who authorized it.

**Refusals name the limit.** A refusal says which scope refused and by how much, so the member knows whether to delete their own files or ask the room's owner or the host. The numbers disclose nothing the member could not compute from their own uploads, except the room's total, which every member may read through `rooms/info`.

### Data carried

A room identifier, a presentation, a single-use key's identifier, a ciphertext size and digests in; a handle, a mode, a list of indices, an expiry and, in direct mode, storage URLs out. No byte of the file moves in this document.

### Correlation

On `attributed` and `open` rooms the presentation names the acting member, so the host can relate uploads to members, as it already relates writes. On `private` rooms it cannot — and in direct mode the storage provider can, which is why direct transfer is not offered there by default (above). The BlobRef depends on ciphertext under a per-file key, so two rooms holding the same file hold different blobs and cannot be correlated by them.

### Retention

The slot and its reservation live until commit, abort or expiry; in direct mode, so do the parts written to storage, which an expired or aborted upload's owner never commits and the host reclaims. Expiry is idle-based: at least 15 minutes after the last accepted chunk or begin, extended as chunks arrive up to at least 24 hours in all (Conformance, item 11). A host **SHOULD NOT** keep an idle slot much longer than its floor, since a slot holds a reservation against the room's limits.

### Consent/purpose

A member stores a file in a room they joined by accepting an invitation; the membership credential is that consent. The host's purpose is to store and serve the ciphertext to the room's members, and nothing in this task authorizes it to do more.
