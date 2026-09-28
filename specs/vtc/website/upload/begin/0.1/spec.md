---
slug: vtc/website/upload/begin
version: "0.1"
title: "VTC Website — Upload — Begin"
summary: An administrator opens an upload of one website file or a whole-site bundle, committing to its size, its SHA-256 and the digest of every chunk before any byte moves.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request opens a writable slot in the community and pre-commits what will be written to it — the manifest every chunk is checked against — so an unattributable one is an anonymous party arranging to place bytes on the community's public website. The same signer must be the one that sends the chunks and commits them.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed begin opens a second slot from one authorization; bounding replay keeps the number of open uploads equal to the number of times an administrator asked for one.
sideEffects:
  level: mutating
  rationale: >-
    Reserves an upload slot. Nothing on the website changes — staged bytes are inert until commit, and a slot never committed expires having changed nothing.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a target, a size and digests — descriptive metadata about bytes the community does not yet hold. The response is a handle and an expiry.
retention:
  class: exchange
  rationale: >-
    The slot and any staged chunks live until commit, abort or expiry, and no longer.
maxDocumentBytes:
  request: 262144
  rationale: >-
    The request carries the chunk manifest — one digest per chunk, up to 4096 of them (about 205 KiB for sha2-256 in base58btc) — which can exceed the 64 KiB a community applies to a Trust Task by default. A 50 MiB bundle at 256 KiB chunks needs 200 digests, well inside it.
errorCodes:
  - code: vtc/website/upload/begin:notConfigured
    meaning: "The community serves no website — it has no site root configured — so there is nothing to upload into."
    retryable: false
  - code: vtc/website/upload/begin:tooLarge
    meaning: "`expectedSizeBytes` exceeds the community's limit for the target: its maximum file size for a `file`, its maximum bundle size for a `bundle`. `details.maxSizeBytes` states the limit."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      required: [maxSizeBytes]
      properties:
        maxSizeBytes:
          type: integer
          minimum: 0
  - code: vtc/website/upload/begin:pathRefused
    meaning: "The target `path` escapes the site root, names a hidden file or directory, carries a blocklisted extension, or contains a character the site refuses. Checked here, before any byte moves, and again at commit."
    retryable: false
  - code: vtc/website/upload/begin:singleFileWritesDisabled
    meaning: "The community's website is in managed deploy mode, where every change lands as a new generation; a `file` target cannot be written. Upload a `bundle` and deploy it."
    retryable: false
  - code: vtc/website/upload/begin:invalidManifest
    meaning: "`chunks.chunkCount` is not ceil(expectedSizeBytes / chunkSize), or `chunks.chunkDigests` does not have exactly `chunkCount` items."
    retryable: false
related:
  - vtc/website/upload/chunk
  - vtc/website/upload/commit
  - vtc/website/upload/abort
  - vtc/website/deploy
  - backup/initiate-import
---

## Abstract

A community's website is served from files on the community's own host, and an administrator changes it in two ways: by writing one file, or by deploying a bundle — a gzip-compressed tar of the whole site — that replaces it. Both carry bytes, and bytes do not fit the 64 KiB a community accepts in one Trust Task document by default: a file may be 10 MiB, a bundle 50 MiB. Raising the limit for every task to fit them would open every other task to bodies it has no use for.

So website content moves as a **chunked transfer**, the same shape [`backup/initiate-import`](../../../../../backup/initiate-import/0.1/spec.md) and [`backup/put-chunk`](../../../../../backup/put-chunk/0.1/spec.md) use for a node's encrypted state:

1. **`vtc/website/upload/begin`** (this task) opens a slot and pre-commits the target, the total size, the SHA-256 of the whole content, and the digest of every chunk.
2. [`vtc/website/upload/chunk`](../../chunk/0.1/spec.md) sends each chunk by index; each is checked against the manifest on arrival.
3. [`vtc/website/upload/commit`](../../commit/0.1/spec.md) reassembles, checks the whole against the committed SHA-256, and applies a `file` target — or stages a `bundle` for [`vtc/website/deploy`](../../../deploy/0.1/spec.md).
4. [`vtc/website/upload/abort`](../../abort/0.1/spec.md) discards a slot early.

The content hash is committed here, in a signed document, before any byte moves. That is what bounds the damage from a substituted chunk: the community refuses bytes that hash differently from what the administrator committed to.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, and **MUST** refuse with `notConfigured` when it serves no website.
2. **MUST** refuse with `malformedRequest` a `file` target without `path`, a `bundle` target with `path` or `ifMatch`.
3. **MUST** refuse with `tooLarge` a `file` larger than its maximum file size, or a `bundle` larger than its maximum bundle size (10 MiB and 50 MiB in the reference implementation's defaults), before reserving anything.
4. For a `file` target, **MUST** refuse with `singleFileWritesDisabled` when its site is in managed deploy mode, and **MUST** refuse with `pathRefused` a path the public read handler would refuse to serve — checked now, so an upload that could never commit is not accepted.
5. **MUST** refuse with `invalidManifest` a manifest inconsistent with `expectedSizeBytes`.
6. **MUST** mint an unguessable `uploadId`, record the target, size, SHA-256 and manifest against it and against the caller's identity, and answer with the id and the slot's expiry.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key (`auth/signing-key/enroll`), the entry of the identity it acts for — read at execution time, on every document of the transfer rather than only the first. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. An upload belongs to the identity that began it: another administrator's reference to it is answered `notFound`, so an upload cannot be completed, aborted or deployed by someone who did not start it.

## Definitions

- **Upload** — one chunked transfer of bytes into the community, from begin to commit, abort or expiry.
- **Target** — a `file` at a path, or a whole-site `bundle`.
- **Manifest** — `chunkSize`, `chunkCount` and one digest per chunk, as [`vta/_shared/0.1/backup-transfer.schema.json`](../../../../../vta/_shared/0.1/backup-transfer.schema.json) defines `ChunkManifest`. The chunk ceiling of 256 KiB is the one that schema derives: the largest power of two whose chunk document survives a 1 MiB mediator message after DIDComm encoding and two forward wrappers.
- Website shapes are in [`vtc/_shared/0.1/website-transfer.schema.json`](../../../../_shared/0.1/website-transfer.schema.json).

An **empty file cannot be uploaded**: a transfer carries at least one byte, as `ExpectedSizeBytes` requires. Deleting a file is vtc/website/files/delete.

## Request

A community administrator (`issuer`) opens an upload with the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Replace one page, only if it has not changed since it was read

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/website/upload/begin/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "target": {
      "kind": "file",
      "path": "/events/index.html",
      "ifMatch": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    },
    "expectedSha256": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
    "expectedSizeBytes": 300000,
    "chunks": {
      "chunkSize": 262144,
      "chunkCount": 2,
      "chunkDigests": [
        "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc",
        "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX"
      ]
    }
  }
}
```

### A whole-site bundle

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/website/upload/begin/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "target": {
      "kind": "bundle"
    },
    "expectedSha256": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
    "expectedSizeBytes": 300000,
    "chunks": {
      "chunkSize": 262144,
      "chunkCount": 2,
      "chunkDigests": [
        "zQmRq2ZwqJ3vWJrK1v8R8oPqWXhD4nVb1JmT9pV6uYh3aBc",
        "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX"
      ]
    }
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the upload id and its expiry. A refusal is a `trust-task-error`.

### Slot opened

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/website/upload/begin/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968",
    "expiresAt": "2026-09-28T11:00:00Z"
  }
}
```

## Security & Privacy

### Data carried

A target, a size, a whole-content SHA-256 and per-chunk digests in; a handle and an expiry out. No content moves in this document.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The upload id links the documents of one transfer, which is its purpose; it is recipient-generated and says nothing about the content.

### Retention

The slot lives until commit, abort or expiry. The community **SHOULD** keep the expiry short (an hour in the reference design) and extend it as chunks arrive.

### Consent/purpose

The purpose is to publish content on the community's website. The content is the community's own public material; nothing in a transfer is personal data unless an administrator puts it on the website, where it would be public anyway.
