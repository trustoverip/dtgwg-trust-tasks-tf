---
slug: vtc/website/files/show
version: "0.1"
title: "VTC Website — Files — Show"
summary: An administrator reads one website file's bytes, by range, with its content hash and type — so a file of any size can be read one bounded document at a time.
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
  requirement: RECOMMENDED
  rationale: >-
    Authority is the signer's administrator standing, which the community can establish from a verified proof or a transport-authenticated sender. The content is the community's published website, so a proof buys attribution rather than confidentiality.
sideEffects:
  level: none
  rationale: >-
    Reads a file. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses website content — material the community serves publicly — and its hash and type, to an administrator.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
maxDocumentBytes:
  response: 360448
  rationale: >-
    The response carries up to 262144 bytes of the file, which base64url-encodes to 349526 characters, plus the envelope, metadata and a proof — the same arithmetic and the same 1 MiB mediator reasoning as vtc/website/upload/chunk. A file larger than one range is read in several.
errorCodes:
  - code: vtc/website/files/show:notFound
    meaning: "No servable file exists at this path."
    retryable: false
  - code: vtc/website/files/show:pathRefused
    meaning: "The path escapes the site root, names a hidden file or directory, or carries a blocklisted extension; the site would not serve it, so it is not read."
    retryable: false
  - code: vtc/website/files/show:changed
    meaning: "The file's current content hash is not the `ifMatch` the request carried — it changed between two ranged reads. Start again from offset 0."
    retryable: false
  - code: vtc/website/files/show:rangeOutOfBounds
    meaning: "`offset` is past the end of the file."
    retryable: false
related:
  - vtc/website/files/list
  - vtc/website/upload/begin
---

## Abstract

The **VTC Website — Files — Show** Trust Task reads one file of the community's website — the bytes [`vtc/website/files/list`](../../list/0.1/spec.md) deliberately leaves out — with its content hash and content type. A file may be up to the community's maximum file size (10 MiB by default), which does not fit one bounded document, so the read is **ranged**: each request names an `offset` and a `length` up to 256 KiB, and the response says whether it reached the end.

A ranged read needs no session on the community: each range is read from the file as it is. To make several ranges one consistent read, a caller passes the `etag` the first range returned as `ifMatch` on the rest; if the file changes in between, the community answers `changed` rather than hand back a mixture of two versions. This is why the read is not a staged transfer like `backup/get-chunk` — nothing needs staging, because the file on disk and its hash already are the transfer's terms.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** apply the public read handler's path rules and answer `pathRefused` for a path it would not serve, and `notFound` for no file.
3. **MUST** refuse with `changed` when `ifMatch` is present and differs from the file's current content hash.
4. **MUST** refuse with `rangeOutOfBounds` an `offset` greater than the file's size; an `offset` equal to it returns an empty `data` with `complete: true`.
5. **MUST** return at most `length` bytes (262144 when absent) starting at `offset`, with the file's whole-content `etag`, its total `sizeBytes`, its `contentType` as the site would serve it, and `complete: true` exactly when the range reaches the end.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key (`auth/signing-key/enroll`), the entry of the identity it acts for — read at execution time, on every document of the transfer rather than only the first. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`etag`** — the lowercase hex SHA-256 of the whole file, as vtc/website/files/list reports it.
- **Range** — `length` bytes from `offset`.
- **`complete`** — the range reached the end of the file.

## Request

A community administrator (`issuer`) asks the community (`recipient`) for a range of one file.

### The first range of a page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000d",
  "type": "https://trusttasks.org/spec/vtc/website/files/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "path": "/events/index.html"
  }
}
```

### The next range, pinned to the version already read

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000e",
  "type": "https://trusttasks.org/spec/vtc/website/files/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "path": "/events/index.html",
    "offset": 262144,
    "ifMatch": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### A small file, read whole

Shortened for the example.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000f",
  "type": "https://trusttasks.org/spec/vtc/website/files/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:09:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "path": "/events/index.html",
    "etag": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
    "sizeBytes": 14,
    "contentType": "text/html",
    "offset": 0,
    "data": "PGh0bWw-PC9odG1sPgo",
    "complete": true
  }
}
```

## Security & Privacy

### Data carried

A path and a range in; website content, its hash, size and type out.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The upload id links the documents of one transfer, which is its purpose; it is recipient-generated and says nothing about the content.

### Retention

Nothing is kept. A caller's copy is stale at the next write to the path.

### Consent/purpose

The purpose is to publish content on the community's website. The content is the community's own public material; nothing in a transfer is personal data unless an administrator puts it on the website, where it would be public anyway.
