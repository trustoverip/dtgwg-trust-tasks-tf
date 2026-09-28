---
slug: vtc/community/requested-attributes/update
version: "0.1"
title: "VTC Community — Requested Attributes — Update"
summary: An administrator replaces what the community asks an applicant to tell it about themselves, as persona claim types — never values.
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
    The write changes what the community tells every prospective applicant, under the community's name, and is authorized only by the signer's administrator standing — so the signer has to be attributable on every transport, and the change has to be attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would silently put back a configuration an administrator has since replaced, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Replaces one configuration row. Recoverable by another update; nothing is issued, revoked or deleted beyond the row itself.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries community configuration an administrator wrote; the response echoes what was stored. No personal data.
retention:
  class: durable
  rationale: >-
    The stored value is the community's configuration until the next update, and the change is recorded in the audit trail against the administrator who made it.
errorCodes:
  - code: vtc/community/requested-attributes/update:duplicateType
    meaning: "The same claim `type` appears more than once. Two answers to one question is no answer; ask for each attribute once."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      required: [type]
      properties:
        type:
          type: string
          maxLength: 128
related:
  - vtc/community/requested-attributes/show
  - vtc/join-requests/manifest
---

## Abstract

The **VTC Community — Requested Attributes — Update** Trust Task replaces what the community asks prospective applicants to tell it about themselves — the list [`vtc/join-requests/manifest/0.2`](../../../../join-requests/manifest/0.2/spec.md) publishes as `requestedAttributes`, and that `vtc/join-requests/submit` checks an applicant's `attributes` against.

The update replaces the whole list. An empty list means the community asks for nothing, and the manifest then publishes no `requestedAttributes`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller that does not hold administrator standing with `permissionDenied`, resolving a delegated signing key to the identity it acts for.
2. **MUST** refuse a list with a claim `type` more than once with `duplicateType`, naming the first repeated type in `details.type`. The 32-entry bound is enforced by the schema.
3. **MUST** apply the new list to join requests submitted after it is stored. A request already submitted was answered against the list the applicant was shown, and **MUST NOT** be refused for missing an attribute added afterwards.
4. **MUST** record the change in its audit trail — the types added and removed, and the administrator — and **MUST** refuse with `unavailable` rather than make an unaudited change. A change that adds and removes nothing is not audited.
5. **MUST** answer with the list it stored.

## Authorization

The authority this task presupposes is **administrator standing at the community**, read at execution time. The decision is the community's ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`requestedAttributes`** — the whole new list, at most 32 entries, no claim type twice.

## Request

A community administrator (`issuer`) sends the whole new list to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Ask for a display name, and optionally a country

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/community/requested-attributes/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
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

### Stop asking for anything

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vtc/community/requested-attributes/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "requestedAttributes": []
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: what it stored. A refusal is a `trust-task-error`.

### Stored

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/vtc/community/requested-attributes/update/0.1#response",
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

Community configuration an administrator wrote. Each `purpose` is free text, bounded at 256 characters, written by an administrator and shown to applicants *before they disclose* — it is the community's own statement of why it asks, and a client **MUST** attribute it to the community. Asking for an attribute is asking an applicant for personal data: an administrator **SHOULD** ask for the least the community needs.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The audit trail ties each change to the administrator who made it; that attribution is the point of requiring a proof.

### Retention

The stored value lives until the next update. The audit record of the change is durable and names the administrator and what changed, not the whole value.

### Consent/purpose

The purpose is to set how the community presents itself to, or what it asks of, prospective applicants. Using the value for anything else — for instance deciding an application on an attribute the community did not publish as requested — is outside it.
