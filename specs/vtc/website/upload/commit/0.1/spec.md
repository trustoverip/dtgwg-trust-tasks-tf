---
slug: vtc/website/upload/commit
version: "0.1"
title: "VTC Website — Upload — Commit"
summary: Complete a website upload — reassemble it, check it against the SHA-256 committed at begin, and write the file into the site, or stage the bundle for deploy.
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
    For a file target this is the act that changes the public website, under the community's name, on the signer's administrator standing; it must be attributable on every transport and afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A committed upload cannot be committed again, but §7.2 item 11 still needs a bounded window to recognise the duplicate.
sideEffects:
  level: mutating
  rationale: >-
    For a `file` target, atomically writes the file into the site, replacing any file at that path — recoverable by writing it again. For a `bundle` target, only stages the verified bytes; nothing on the website changes until vtc/website/deploy.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names an upload; the response reports its hash, size and, for a file, the written path and etag.
retention:
  class: durable
  rationale: >-
    A written file is published content until replaced or deleted, and the write is audited. A staged bundle lives until deployed or expired.
errorCodes:
  - code: vtc/website/upload/commit:notFound
    meaning: "No open upload with this id exists that the caller may commit."
    retryable: false
  - code: vtc/website/upload/commit:incomplete
    meaning: "Chunks are missing. `details.remainingCount` says how many."
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
  - code: vtc/website/upload/commit:digestMismatch
    meaning: "Every chunk verified, but the reassembled content does not hash to the SHA-256 committed at begin — the chunks are right and the order or the commitment is wrong. Nothing is written; the upload is discarded."
    retryable: false
  - code: vtc/website/upload/commit:preconditionFailed
    meaning: "The target file's current content hash is not the `ifMatch` committed at begin — somebody changed it since it was read. Nothing is written; read it again and start a new upload."
    retryable: false
  - code: vtc/website/upload/commit:pathRefused
    meaning: "The target path is refused by the site's path rules, re-checked at commit because the configuration may have changed since begin."
    retryable: false
  - code: vtc/website/upload/commit:singleFileWritesDisabled
    meaning: "The site switched to managed deploy mode since the upload began; a file target can no longer be written."
    retryable: false
related:
  - vtc/website/upload/begin
  - vtc/website/upload/chunk
  - vtc/website/deploy
  - vtc/website/files/show
---

## Abstract

The **VTC Website — Upload — Commit** Trust Task completes an upload once every chunk has been stored ([`vtc/website/upload/chunk`](../../chunk/0.1/spec.md)). The community reassembles the chunks in index order and checks the result against the whole-content SHA-256 committed at [`vtc/website/upload/begin`](../../begin/0.1/spec.md) — the check that catches a correct set of chunks assembled wrongly.

What happens next depends on the target the upload committed to:

- **`file`** — the content is written into the site at the target path, atomically (a temporary file renamed over the target), honouring the `ifMatch` committed at begin. This is the single-file write, and the response reports the file's new etag.
- **`bundle`** — the content is **staged**, verified and inert, and [`vtc/website/deploy`](../../../deploy/0.1/spec.md) publishes it. Deploying replaces the whole site, and is a separate act with its own authorization and audit, so a bundle can be uploaded and checked before anyone decides to put it live.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `notFound` unless `uploadId` names an open upload begun by the caller's identity.
2. **MUST** refuse with `incomplete` while any chunk is missing, and **MUST** keep the upload open so the missing chunks can be sent.
3. **MUST** refuse with `digestMismatch` — and discard the upload — when the reassembled content's SHA-256 is not the committed one.
4. For a `file` target, **MUST** re-check the path rules and the deploy mode (`pathRefused`, `singleFileWritesDisabled`), **MUST** refuse with `preconditionFailed` when `ifMatch` was committed and the file's current hash differs (or the file is absent), and otherwise **MUST** write the file atomically and audit the write against the caller, naming the path, size and hash.
5. For a `bundle` target, **MUST** stage the verified content against the upload id for a bounded time and report `stagedUntil`; it **MUST NOT** change the site.
6. **MUST** make the upload unusable for further chunks once committed.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key (`auth/signing-key/enroll`), the entry of the identity it acts for — read at execution time, on every document of the transfer rather than only the first. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. An upload belongs to the identity that began it: another administrator's reference to it is answered `notFound`, so an upload cannot be completed, aborted or deployed by someone who did not start it.

## Definitions

- **Reassembly** — the chunks concatenated in index order.
- **`file`** — for a file target, the path written, its new `etag` and its size.
- **`stagedUntil`** — for a bundle target, until when vtc/website/deploy can publish it.

## Request

The administrator who began the upload (`issuer`) commits it with the community (`recipient`).

### Commit the page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/vtc/website/upload/commit/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The file was written

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000007",
  "type": "https://trusttasks.org/spec/vtc/website/upload/commit/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:06:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968",
    "target": {
      "kind": "file",
      "path": "/events/index.html",
      "ifMatch": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    },
    "sha256": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
    "sizeBytes": 300000,
    "file": {
      "path": "/events/index.html",
      "etag": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
      "sizeBytes": 300000
    }
  }
}
```

### A bundle was staged

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000008",
  "type": "https://trusttasks.org/spec/vtc/website/upload/commit/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:06:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968",
    "target": {
      "kind": "bundle"
    },
    "sha256": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
    "sizeBytes": 300000,
    "stagedUntil": "2026-09-28T11:06:00Z"
  }
}
```

## Security & Privacy

### Data carried

An upload id in; the target, hash and size out, and for a file the written path and etag.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The upload id links the documents of one transfer, which is its purpose; it is recipient-generated and says nothing about the content.

### Retention

A written file is public content until replaced; the audit row records the administrator, path, size and hash, not the content. A staged bundle is discarded at deploy or expiry.

### Consent/purpose

The purpose is to publish content on the community's website. The content is the community's own public material; nothing in a transfer is personal data unless an administrator puts it on the website, where it would be public anyway.
