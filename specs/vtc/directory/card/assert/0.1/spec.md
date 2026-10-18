---
slug: vtc/directory/card/assert
version: '0.1'
title: VTC Directory — Card — Assert
summary: An administrator writes the values of admin-sourced directory fields on a member's card — the facts the community asserts about a member, which the member reads and cannot change.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: directory administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: An assertion is the community speaking about a member; it must stay attributable to the administrator who made it after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed assertion would restore a value since corrected.
sideEffects:
  level: mutating
  rationale: Writes admin-sourced values on a member's card. Recoverable by a later assertion.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: Carries facts the community asserts about a named member (a chapter, a membership grade). Returns only which fields were written and the card's revision.
retention:
  class: durable
  rationale: The assertion is the community's statement about the member, and the member may ask who made it and when.
errorCodes:
- code: vtc/directory/card/assert:notFound
  meaning: No active member has this entry.
  retryable: false
- code: vtc/directory/card/assert:fieldUnknown
  meaning: A value names a field the schema does not have.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    properties:
      field:
        type: string
        maxLength: 128
- code: vtc/directory/card/assert:fieldRetired
  meaning: A value names a retired or disabled field.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    properties:
      field:
        type: string
        maxLength: 128
- code: vtc/directory/card/assert:fieldNotWritable
  meaning: A value names a field that is not `admin`-sourced.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    properties:
      field:
        type: string
        maxLength: 128
- code: vtc/directory/card/assert:valueInvalid
  meaning: A value does not fit its field's type or constraints.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
- code: vtc/directory/card/assert:valueRefused
  meaning: The community's directory policy refused the assertion.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      message:
        type: string
        maxLength: 500
related:
- vtc/directory/card/update
- vtc/directory/fields/define
---

## Abstract

The **VTC Directory — Card — Assert** Trust Task writes the values of `admin`-sourced fields — what the community itself states about a member, such as their chapter or membership grade. The member sees these values on their card and chooses their visibility like any other field's, but cannot change them.

An assertion is self-asserted by the community. It is never presented as verified: only a `credential`-sourced value, which names its issuer, carries a verification badge.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.admin` with `permissionDenied`.
2. Refuse an `entryId` that names no active member with `notFound`.
3. Refuse a value for a field that is not `admin`-sourced with `fieldNotWritable`, and an unknown, retired or ill-fitting one with `fieldUnknown`, `fieldRetired` or `valueInvalid`.
4. Evaluate its `directoryWrite` policy and refuse with `valueRefused` where it refuses.
5. Write the values, leave the member's visibility choices untouched, increment the card's `revision`, and audit the assertion by reference.
6. **MUST NOT** return the member's card or any of their other values: the response says only which fields were written.

## Authorization

**Authority: `vtc.directory.admin`** at the community, read from the signer's access-control entry at execution time, under the community's `directoryWrite` policy purpose. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `EntryId`, `FieldValue` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).

## Request

The directory administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Record a member's chapter

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000018",
  "type": "https://trusttasks.org/spec/vtc/directory/card/assert/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000017",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "values": {
      "org.example.chapter": "sydney"
    },
    "reason": "Moved to the Sydney chapter."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/card/assert/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Record a member's chapter — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000019",
  "type": "https://trusttasks.org/spec/vtc/directory/card/assert/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000017",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "written": [
      "org.example.chapter"
    ],
    "revision": 5
  }
}
```

## Security & Privacy

### Data carried

Facts about one member, named by `entryId`. A producer **MUST NOT** put in `reason` or `ext` anything about the member that the assertion itself does not state.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The member is named by their community-scoped `entryId`, never by DID.

### Retention

**Durable** for as long as the value is held; on the member's departure it follows their departure disposition with the rest of the card.

### Consent/purpose

The community stating, in its own directory, facts it is responsible for. The values are shown only within the member's own visibility choices, and are not to be reused beyond the directory.
