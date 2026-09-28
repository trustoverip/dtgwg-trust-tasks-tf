---
slug: vtc/website/upload/chunk
version: "0.1"
title: "VTC Website — Upload — Chunk"
summary: Send one chunk of an open website upload by index, checked on arrival against the digest committed for that index.
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
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request places bytes inside the community, and the only authority for that is having opened the upload — a comparison against its owner that has to be attributable on every write. The response is small and is the producer's evidence that a chunk was stored.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replay of identical bytes changes nothing and a replay of different bytes is refused against the manifest — but a replay still extends the slot's expiry, and bounding it is what stops a captured chunk from keeping an upload open.
sideEffects:
  level: mutating
  rationale: >-
    Stages one chunk. Nothing on the website changes; staged chunks are inert until commit, and discarded at abort or expiry.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a range of website content — material the community will publish, not personal data — and the response is an acknowledgement and a count.
retention:
  class: exchange
  rationale: >-
    A staged chunk lives only as long as its upload.
maxDocumentBytes:
  request: 360448
  rationale: >-
    A chunk is up to 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the other payload members and a proof. The 262144-byte ceiling is the one vta/_shared/0.1/backup-transfer derives for a chunk document to survive a 1 MiB mediator message after DIDComm encoding and two forward wrappers.
errorCodes:
  - code: vtc/website/upload/chunk:notFound
    meaning: "No open upload with this id exists that the caller may write to. Conflates an unknown id, an expired or committed upload, and another administrator's upload."
    retryable: false
  - code: vtc/website/upload/chunk:chunkOutOfRange
    meaning: "`index` is not below the manifest's `chunkCount`."
    retryable: false
  - code: vtc/website/upload/chunk:chunkMismatch
    meaning: "`digestMultibase` differs from the manifest entry for `index`, the decoded `data` does not match it, or its length is wrong for its position (every chunk but the last is exactly `chunkSize`). The chunk is not stored; re-send the right bytes."
    retryable: false
  - code: vtc/website/upload/chunk:alreadyStored
    meaning: "Different bytes are already staged at this index. Never returned for an identical re-send, which succeeds with `stored: false`."
    retryable: false
related:
  - vtc/website/upload/begin
  - vtc/website/upload/commit
  - backup/put-chunk
---

## Abstract

The **VTC Website — Upload — Chunk** Trust Task writes one chunk of an upload opened by [`vtc/website/upload/begin`](../../begin/0.1/spec.md). The chunk restates the digest committed for its index, and the community checks the restated digest against the manifest and the bytes against both — so a chunk sent to the wrong index, or corrupted on the way, is refused on arrival rather than at reassembly.

Chunks may arrive in any order and any may be re-sent: an identical re-send succeeds with `stored: false`. This is the website counterpart of [`backup/put-chunk`](../../../../../backup/put-chunk/0.1/spec.md), with the same chunk vocabulary.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `notFound` unless `uploadId` names an open upload begun by the caller's identity.
2. **MUST** refuse with `chunkOutOfRange` an index not below `chunkCount`, and with `chunkMismatch` a chunk whose restated digest, decoded bytes or length disagree with the manifest.
3. **MUST** answer an identical re-send with `stored: false`, and **MUST** refuse different bytes at an already-stored index with `alreadyStored`.
4. **MAY** extend the upload's expiry on each stored chunk, never past its own ceiling, and **MUST** report the current expiry and how many indices remain.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key (`auth/signing-key/enroll`), the entry of the identity it acts for — read at execution time, on every document of the transfer rather than only the first. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. An upload belongs to the identity that began it: another administrator's reference to it is answered `notFound`, so an upload cannot be completed, aborted or deployed by someone who did not start it.

## Definitions

- **`index`**, **`data`**, **`digestMultibase`** — as backup/put-chunk defines them: a zero-based position, the chunk's bytes in unpadded base64url, and the digest of the raw bytes.
- **`remainingCount`** — indices still missing; zero means the upload can be committed.

## Request

The administrator who began the upload (`issuer`) sends one chunk to the community (`recipient`).

### The second and last chunk

Shortened for the example: a real chunk carries up to 349526 characters of `data`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vtc/website/upload/chunk/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968",
    "index": 1,
    "digestMultibase": "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
    "data": "PGh0bWw-PC9odG1sPgo"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Stored; nothing remains

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/vtc/website/upload/chunk/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968",
    "index": 1,
    "stored": true,
    "remainingCount": 0,
    "expiresAt": "2026-09-28T11:05:00Z"
  }
}
```

## Security & Privacy

### Data carried

A range of website content in, an acknowledgement out. The content is what the community will publish.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The upload id links the documents of one transfer, which is its purpose; it is recipient-generated and says nothing about the content.

### Retention

Staged chunks are discarded at commit (once reassembled), abort, or expiry.

### Consent/purpose

The purpose is to publish content on the community's website. The content is the community's own public material; nothing in a transfer is personal data unless an administrator puts it on the website, where it would be public anyway.
