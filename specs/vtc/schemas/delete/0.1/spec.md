---
slug: vtc/schemas/delete
version: "0.1"
title: "VTC Schemas — Delete"
summary: An administrator removes a credential type from the community's schema registry, so it is no longer issued, or no longer accepted as evidence.
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
    The write changes which credentials the community will mint or accept as evidence, under the community's name, and is authorized only by the signer's administrator standing — so the signer must be attributable on every transport and the change attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed registration would put back an entry an administrator has since replaced or deleted, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: destructive
  rationale: >-
    Removes the entry and its `credentialSchema`. For an `issues` type, issuance of that type is refused from then on; for an `accepts` type, no new Accepts criterion may reference it. The entry is not recoverable from the community — registering the type again creates a new entry from whatever the administrator still holds — which is what makes this destructive rather than mutating.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a type URI; the response echoes it.
retention:
  class: durable
  rationale: >-
    The deletion is audited against the administrator; the entry itself is gone.
errorCodes:
  - code: vtc/schemas/delete:notFound
    meaning: "No entry is registered for this type URI."
    retryable: false
  - code: vtc/schemas/delete:inUse
    meaning: "A registered Accepts criterion references this type in its DCQL query. Deleting it would leave that criterion unevaluable; delete or re-register the criterion first. `details.criterionIds` MAY list up to 16 of them."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      properties:
        criterionIds:
          type: array
          maxItems: 16
          items:
            type: string
            maxLength: 128
related:
  - vtc/schemas/register
  - vtc/schemas/show
  - vtc/schemas/accepts/delete
---

## Abstract

The **VTC Schemas — Delete** Trust Task removes one entry from the community's schema registry. An `issues` type can no longer be minted; an `accepts` type can no longer be referenced by a new Accepts criterion. Credentials already issued are unaffected — deletion changes what the community does next, not what it did.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, before looking the type up.
2. **MUST** answer `notFound` when no entry is registered for `typeUri`.
3. **SHOULD** refuse with `inUse` while a registered Accepts criterion's query references the type. (The reference implementation does not yet make this check; a criterion left referencing a deleted type fails when a ceremony next evaluates it, which is later and less legible than refusing here.)
4. **MUST** audit the deletion, naming the type and the administrator, and answer with the deleted `typeUri`.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`typeUri`** — the registry key to remove.

## Request

A community administrator (`issuer`) names the type to the community (`recipient`).

### Retire an endorsement type

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/schemas/delete/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "typeUri": "https://riverside.example/credentials/WorkshopSafety"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Deleted

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/schemas/delete/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "typeUri": "https://riverside.example/credentials/WorkshopSafety"
  }
}
```

## Security & Privacy

### Data carried

Registry configuration an administrator wrote. The description member is free text, bounded at 1024 characters, written by the registering administrator and read by other administrators (and, for an Accepts criterion, published to applicants and used as the `purpose` a join query shows the holder); it is untrusted, attributed to its author wherever rendered, and **MUST NOT** carry personal data.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The audit trail ties each change to the administrator who made it.

### Retention

The entry lives until it is replaced or deleted. The audit record names the entry and the administrator, not the entry's content.

### Consent/purpose

The purpose is to set which credentials the community mints, and which it accepts as evidence in its ceremonies. What is registered here changes what applicants are asked to present, so an administrator **SHOULD** register the least evidence the community's governance requires.
