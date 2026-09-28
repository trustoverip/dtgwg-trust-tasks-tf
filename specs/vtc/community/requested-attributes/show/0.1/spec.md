---
slug: vtc/community/requested-attributes/show
version: "0.1"
title: "VTC Community — Requested Attributes — Show"
summary: Read what the community asks an applicant to tell it about themselves, as persona claim types — never values.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: member or administrator
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
    The community answers only a caller it can identify, and an authenticated transport identifies one. A proof is recommended so the read is attributable on every transport, relayed ones included. What it returns is configuration the community publishes to applicants anyway, so nothing here needs a stronger binding.
sideEffects:
  level: none
  rationale: >-
    Reads one configuration row of the community. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses community configuration — what the community asks an applicant to tell it about themselves, as persona claim types — never values. No personal data about any member or applicant.
retention:
  class: transient
  rationale: >-
    A read. The consumer keeps nothing, and the caller's copy is stale at the next update.
errorCodes: []
related:
  - vtc/community/requested-attributes/update
  - vtc/join-requests/manifest
---

## Abstract

The **VTC Community — Requested Attributes — Show** Trust Task returns what the community asks a prospective applicant to tell it about themselves: a list of persona claim types (`name.display`, `address.country`), each marked required or optional and optionally with the purpose shown to the applicant. It is exactly what [`vtc/join-requests/manifest/0.2`](../../../../join-requests/manifest/0.2/spec.md) publishes as `requestedAttributes`, and what an applicant answers as `attributes` on `vtc/join-requests/submit`.

The list names claim types, never values: a community asks, and an applicant's agent answers from the persona the applicant chooses.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST NOT** answer a caller it cannot identify, and refuses it with `permissionDenied`. Any identified caller may read.
2. **MUST** return the stored list, and **MUST** return `requestedAttributes: []` — not an error — when the community asks for nothing.
3. **MUST** return the list in the order the manifest publishes it.

## Definitions

- **`requestedAttributes`** — at most 32 `RequestedAttribute` entries from [`vtc/_shared/0.1/community-presentation.schema.json`](../../../../_shared/0.1/community-presentation.schema.json), each a claim `type`, whether it is `required` (default true), and an optional `purpose`.

## Request

A member or administrator (`issuer`) sends an empty payload to the community (`recipient`).

### A console loads the requested-attributes page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/community/requested-attributes/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Two attributes, one optional

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/community/requested-attributes/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "requestedAttributes": [
      {
        "type": "name.display",
        "required": true,
        "purpose": "So other members know what to call you."
      },
      {
        "type": "address.country",
        "required": false
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Community configuration only. Claim types and the community's stated purposes — no applicant's answers, which are never part of this configuration.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. A read reveals to the community that an identified caller looked at its configuration, which it learns from any authenticated request.

### Retention

Nothing is kept by the community. A caller's copy is stale at the next update and **SHOULD NOT** be cached as authoritative: vtc/join-requests/manifest is what an applicant relies on.

### Consent/purpose

The purpose is to let an administration surface display the setting it edits, and a member see what the community publishes. It discloses nothing that the join manifest does not already publish to applicants.
