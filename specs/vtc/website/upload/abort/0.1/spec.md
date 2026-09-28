---
slug: vtc/website/upload/abort
version: "0.1"
title: "VTC Website — Upload — Abort"
summary: Cancel an open or staged website upload and discard its bytes.
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
    The community decides whether the caller may abort by comparing the signer with the upload's owner, which has to be attributable on every transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay changes nothing — an aborted upload stays aborted — but §7.2 item 11 still needs a bounded window.
sideEffects:
  level: destructive
  rationale: >-
    Discards the staged bytes of the upload irreversibly; the upload cannot be resumed and must be begun again. Nothing on the website changes.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    An upload id in, a flag out.
retention:
  class: exchange
  rationale: >-
    Ends the upload's exchange; nothing is kept but the fact that it was aborted.
errorCodes:
  - code: vtc/website/upload/abort:notFound
    meaning: "No upload with this id exists that the caller may abort."
    retryable: false
related:
  - vtc/website/upload/begin
  - backup/abort
---

## Abstract

The **VTC Website — Upload — Abort** Trust Task discards an upload before it expires — one still receiving chunks, or a committed bundle staged for [`vtc/website/deploy`](../../../deploy/0.1/spec.md) that an administrator has decided not to publish. An upload would be collected at expiry anyway; abort releases a large one at once, and makes the decision explicit. It is the website counterpart of [`backup/abort`](../../../../../backup/abort/0.1/spec.md), and is idempotent in the same way.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `notFound` unless `uploadId` names an upload begun by the caller's identity that is open, staged, or already aborted.
2. **MUST** discard every staged byte of the upload and refuse any further document naming it.
3. **MUST** answer `aborted: false`, not an error, for an upload already aborted.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key (`auth/signing-key/enroll`), the entry of the identity it acts for — read at execution time, on every document of the transfer rather than only the first. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's. An upload belongs to the identity that began it: another administrator's reference to it is answered `notFound`, so an upload cannot be completed, aborted or deployed by someone who did not start it.

## Definitions

- **`aborted`** — whether this request is what ended the upload.

## Request

The administrator who began the upload (`issuer`) aborts it.

### Abandon a staged bundle

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000009",
  "type": "https://trusttasks.org/spec/vtc/website/upload/abort/0.1#request",
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

### Aborted

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000a",
  "type": "https://trusttasks.org/spec/vtc/website/upload/abort/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:07:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "uploadId": "8c7b6a59-4837-4261-9f0e-1d2c3b4a5968",
    "aborted": true
  }
}
```

## Security & Privacy

### Data carried

An upload id in, a flag out.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The upload id links the documents of one transfer, which is its purpose; it is recipient-generated and says nothing about the content.

### Retention

Nothing is kept of the upload's bytes.

### Consent/purpose

The purpose is to publish content on the community's website. The content is the community's own public material; nothing in a transfer is personal data unless an administrator puts it on the website, where it would be public anyway.
