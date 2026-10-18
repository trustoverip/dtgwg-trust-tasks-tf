---
slug: vtc/directory/card/show
version: '0.1'
title: VTC Directory — Card — Show
summary: 'A member reads their own directory card whole: every value, the visibility of each field, their discoverability and introduction settings, and its revision.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The card returned is the signer's own, so the community must know who is asking; transports that authenticate the sender already establish it.
sideEffects:
  level: none
  rationale: Reads the signer's card; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Returns the signer's own card — personal data, but only the signer's own, and to the signer.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/directory/card/show:notMember
  meaning: The signer is not an active member of this community.
  retryable: false
related:
- vtc/directory/card/update
- vtc/directory/fields/list
- vtc/directory/entries/show
---

## Abstract

The **VTC Directory — Card — Show** Trust Task returns the signer's own directory card: everything on it, how visible each field is, how discoverable the entry is and who may ask for an introduction. The subject always sees their whole card, including values they show to nobody but administrators and values administrators or credentials wrote.

The task has no target. A caller reads only their own card; another member's entry is read, projected, with [`vtc/directory/entries/show`](../../../entries/show/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Resolve the signer to their membership and refuse with `notMember` when they are not an active member.
2. Return the signer's card whole, including `admin`- and `credential`-sourced values, and `missingRequired`.
3. Where the member has no card yet, return one with no values, every visibility at the field's default, the community's `defaultListing`, `search: byName`, `external: false`, `introductions: anyone` and `revision: 0`. A card is created for every member; it is never absent.

## Authorization

**Authority: being the member.** The card returned is the signer's own, and the task names no other subject, so no party can use it to read anyone else's card. The signer is matched to their membership at execution time; that match is the whole entitlement.

## Definitions

- `DirectoryCard` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member reads their card

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000010",
  "type": "https://trusttasks.org/spec/vtc/directory/card/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000f",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/card/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A member reads their card — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000011",
  "type": "https://trusttasks.org/spec/vtc/directory/card/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000f",
  "payload": {
    "card": {
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "values": {
        "displayName": "Carol",
        "pronouns": "she/her"
      },
      "visibility": {
        "displayName": "members",
        "pronouns": "members"
      },
      "discoverability": {
        "listing": "listed",
        "search": "byName",
        "external": false
      },
      "introductions": "anyone",
      "revision": 3,
      "updatedAt": "2026-10-17T12:00:00Z"
    },
    "missingRequired": []
  }
}
```

## Security & Privacy

### Data carried

The signer's own card. Nothing about any other member.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The community already knows the member; the read tells it only that they looked.

### Retention

Nothing beyond the audit record of the read.

### Consent/purpose

So a member can see and correct what the community holds about them, and what it shows to whom.
