---
slug: vtc/website/deploy
version: "0.1"
title: "VTC Website — Deploy"
summary: An administrator publishes a staged, verified site bundle — replacing the live site, or creating a new generation in managed mode.
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
    Deploying replaces what the community publishes on its website, under the community's name, on the signer's administrator standing — the act that most needs to be attributable afterwards.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A staged bundle deploys once, but §7.2 item 11 still needs a bounded window to recognise a duplicate.
sideEffects:
  level: destructive
  rationale: >-
    In live mode the new bundle replaces the site and the previous content is removed, irrecoverably from the community. In managed mode it becomes a new generation and generations beyond the retention count are pruned, irrecoverably; vtc/website/rollback can return only to a generation still retained.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    An upload id in; the deploy mode, the bundle's hash and size, and generation numbers out.
retention:
  class: durable
  rationale: >-
    The deployed site is published content until the next deploy or rollback, and the deploy is audited.
errorCodes:
  - code: vtc/website/deploy:notFound
    meaning: "No committed, staged bundle upload with this id exists that the caller may deploy — including a file upload, which commit already wrote."
    retryable: false
  - code: vtc/website/deploy:bundleRefused
    meaning: "The bundle cannot be published: it is not a gzip-compressed tar, an entry's path escapes the site root or names a hidden or blocklisted file, or it expands past the community's decompression limit. Checked on every entry before anything is extracted, so a refused bundle changes nothing. `details.reason` classifies it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      required: [reason]
      properties:
        reason:
          type: string
          enum: [notTarGz, unsafePath, tooLargeExpanded]
related:
  - vtc/website/upload/begin
  - vtc/website/upload/commit
  - vtc/website/generations/list
  - vtc/website/rollback
---

## Abstract

The **VTC Website — Deploy** Trust Task publishes a whole-site bundle — a gzip-compressed tar — that was uploaded with [`vtc/website/upload/begin`](../../upload/begin/0.1/spec.md) and staged, verified, by [`vtc/website/upload/commit`](../../upload/commit/0.1/spec.md). The upload moved and checked the bytes; this is the decision to put them live.

Before extracting anything the community checks every entry's path against the same rules the public read handler applies, and caps how far the bundle may expand, so a gzip bomb or a hostile path refuses the whole bundle rather than half-writing it. Then, by the site's deploy mode:

- **live** — the bundle is extracted to a staging directory that atomically replaces the site root, and the previous content is removed.
- **managed** — the bundle becomes a new numbered generation, the `current` pointer moves to it, and generations beyond the retention count are pruned. [`vtc/website/generations/list`](../../generations/list/0.1/spec.md) and [`vtc/website/rollback`](../../rollback/0.1/spec.md) work over the generations that remain.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, and answer `notFound` unless `uploadId` names a committed, staged bundle upload begun by the caller's identity.
2. **MUST** check every entry of the bundle, and the expanded size, before writing anything, and refuse with `bundleRefused` on the first failure, leaving the site unchanged.
3. **MUST** make the switch to the new content atomic: a reader of the site sees the previous content or the new, never a mixture.
4. In managed mode **MUST** prune only generations beyond its retention count, never the one it just created.
5. **MUST** consume the staged upload, audit the deploy against the caller — the bundle's hash and size, the mode, and the generation — and answer with the same.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key (`auth/signing-key/enroll`), the entry of the identity it acts for — read at execution time, on every document of the transfer rather than only the first. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. An upload belongs to the identity that began it: another administrator's reference to it is answered `notFound`, so an upload cannot be completed, aborted or deployed by someone who did not start it.

## Definitions

- **Deploy mode** — `live` (the site root is replaced) or `managed` (numbered generations with a `current` pointer); a property of the community's configuration, not of the request.
- **`targetGeneration`** — the generation created in managed mode; `0` in live mode, which has none.
- **`prunedGenerations`** — how many old generations were removed; `0` in live mode.

## Request

The administrator who uploaded the bundle (`issuer`) asks the community (`recipient`) to publish it.

### Publish the staged bundle

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000b",
  "type": "https://trusttasks.org/spec/vtc/website/deploy/0.1#request",
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

### Managed mode: generation 7, one old generation pruned

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000c",
  "type": "https://trusttasks.org/spec/vtc/website/deploy/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:08:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "deployMode": "managed",
    "bundleSha256": "3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8a55",
    "bundleSizeBytes": 300000,
    "targetGeneration": 7,
    "prunedGenerations": 1
  }
}
```

## Security & Privacy

### Data carried

An upload id in; the mode, hash, size and generation numbers out.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The upload id links the documents of one transfer, which is its purpose; it is recipient-generated and says nothing about the content.

### Retention

The deployed content is public until the next deploy or rollback. The audit row records the administrator, hash, size, mode and generation.

### Consent/purpose

The purpose is to publish content on the community's website. The content is the community's own public material; nothing in a transfer is personal data unless an administrator puts it on the website, where it would be public anyway.
