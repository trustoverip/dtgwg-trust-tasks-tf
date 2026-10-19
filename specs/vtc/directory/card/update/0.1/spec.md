---
slug: vtc/directory/card/update
version: '0.1'
title: VTC Directory — Card — Update
summary: 'A member changes their own directory card: values they write themselves, the visibility of each field, their discoverability and who may ask to be introduced to them.'
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
  requirement: REQUIRED
  rationale: The signed document is the community's evidence of what the member chose to show and to whom. A disclosure the member later disputes must be answerable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Two of a member's updates arriving out of order would otherwise leave in force the one they made first — re-widening a field they had narrowed.
sideEffects:
  level: mutating
  rationale: Writes values and settings on the member's own card. Recoverable by a later update.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: true
  rationale: The member writes personal data about themselves (a name, pronouns, an avatar, a biography) and chooses who sees it. The response returns their own card.
retention:
  class: durable
  rationale: The card is kept for as long as the membership, and the signed update now in force is the evidence of the member's choice.
errorCodes:
- code: vtc/directory/card/update:notMember
  meaning: The signer is not an active member of this community.
  retryable: false
- code: vtc/directory/card/update:superseded
  meaning: The community has applied a change to this card issued after this document, or `expectedRevision` does not match. Nothing was changed.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      revision:
        type: integer
        minimum: 0
- code: vtc/directory/card/update:fieldUnknown
  meaning: A value or visibility names a field the schema does not have.
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
- code: vtc/directory/card/update:fieldRetired
  meaning: A value or visibility names a retired or disabled field.
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
- code: vtc/directory/card/update:fieldNotWritable
  meaning: 'A value names a field the member does not write: an `admin`-, `credential`-sourced or derived field.'
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
- code: vtc/directory/card/update:valueInvalid
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
- code: vtc/directory/card/update:valueRefused
  meaning: The community's directory policy refused a value (a nickname already taken where nicknames are unique, a link to a domain it does not allow).
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
      message:
        type: string
        maxLength: 500
- code: vtc/directory/card/update:aboveCeiling
  meaning: A visibility is wider than its field's ceiling.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    - ceiling
    properties:
      field:
        type: string
        maxLength: 128
      ceiling:
        type: string
        maxLength: 32
- code: vtc/directory/card/update:notAvailable
  meaning: '`discoverability.external: true` was sent for a card that is not published. External publication is turned on only by publishing a card the member signed (vtc/directory/card/publish). Nothing was changed.'
  retryable: false
related:
- vtc/directory/card/show
- auth/oob/grant
- vtc/directory/fields/list
- vtc/members/registry-consent
---

## Abstract

The **VTC Directory — Card — Update** Trust Task is how a member controls their entry in the community's directory. In one signed document they may change any of:

- the values they write themselves (`member`-sourced fields);
- the visibility of each field — an audience up to the field's ceiling;
- their discoverability: whether the entry is `listed`, `findable` or `hidden`, what a search may match, and whether they publish an external card;
- who may ask the community to introduce them.

Values written by administrators ([`vtc/directory/card/assert`](../../assert/0.1/spec.md)) and disclosed from credentials ([`vtc/directory/card/present`](../../present/0.1/spec.md)) are written only through their own tasks, but the member chooses their visibility here like any other field's.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is the member, signing with their membership DID or with a key the member delegated for this community under [`auth/oob/grant/0.2`](../../../../../auth/oob/grant/0.2/spec.md) whose `signingScope` covers `vtc/directory`. An update that widens any field's visibility to `public`, or sets `discoverability.external` to `true`, **MUST** be signed by the membership DID itself; a consumer **MUST** refuse such an update signed by a delegated key. A client **MUST NOT** send this document on the member's behalf without the member's decision.

A conforming **consumer** (the community) **MUST**:

1. Resolve the signer to their membership and refuse with `notMember` when they are not an active member.
2. Refuse with `superseded`, changing nothing, when it has applied a change to the card issued later than this document's `issuedAt`, or when `expectedRevision` is present and differs from the card's revision.
3. Refuse a value for a field that is not `member`-sourced with `fieldNotWritable`. `admin`- and `credential`-sourced values are written only through their own tasks, and derived fields are never stored.
4. Refuse an unknown field with `fieldUnknown`, a retired or disabled one with `fieldRetired`, a value that does not fit with `valueInvalid`, and a visibility wider than the field's ceiling with `aboveCeiling`. It **MUST NOT** clamp a too-wide visibility silently: the member must know what they chose.
5. Evaluate its `directoryWrite` policy over the resulting card and refuse with `valueRefused` where it refuses. Policy may refuse a change; it **MUST NOT** widen a visibility or a discoverability setting beyond what the member sent, and **MUST NOT** refuse a change that only narrows.
6. Accept any `listing`, including `hidden`, from any member: a community **MUST NOT** hold a member's entry more discoverable than they chose.
7. Refuse `discoverability.external: true` with `notAvailable` when the card's `external` is false: external publication is turned on only by [`vtc/directory/card/publish`](../../publish/0.1/spec.md), with a card the member signed, and `true` here would publish nothing. Where it is already true, `true` is no change. Accept `external: false` as a narrowing, and withdraw any published card as that specification requires. An external card is a second consent, separate from registry-publication consent, which it requires.
8. Apply the update atomically, increment `revision`, record the document's `id` and `issuedAt` as the change in force, and audit it by reference — the audit row names the card and the fields changed, never their values.
9. Apply a narrowing in every projection from the moment it executes.
10. Accept a `CardBlobRef` as an image value only as [`vtc/directory/images/upload`](../../../images/upload/0.1/spec.md) item 5 allows — an image this member uploaded, or the one the field already holds — and refuse any other with `valueInvalid`.
11. Where the update changes the member's public values while their card is published, withdraw it and set `externalCard` to `awaitingSignature`, as [`card/publish`](../../publish/0.1/spec.md) item 8 requires.

## Authorization

**Authority: being the member.** The `issuer` is the member whose card changes, and the task has no target, so no party can use it for anyone else. No administrator can exercise it: an administrator writes `admin`-sourced values through [`vtc/directory/card/assert`](../../assert/0.1/spec.md) and can never change a member's visibility or discoverability choice. The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); here who signed *is* the authorization question, answered by matching the signer to the membership.

## Definitions

- `DirectoryCard`, `Audience`, `Discoverability`, `Introductions` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).
- **Narrowing** — a change after which no viewer can see or find anything they could not before.

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Show a name and pronouns to members, and become findable only

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000013",
  "type": "https://trusttasks.org/spec/vtc/directory/card/update/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000012",
  "payload": {
    "values": {
      "displayName": "Carol",
      "pronouns": "she/her",
      "bio": null
    },
    "visibility": {
      "pronouns": "members"
    },
    "discoverability": {
      "listing": "findable",
      "search": "exactNickname",
      "external": false
    },
    "introductions": "groups",
    "expectedRevision": 3
  },
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/card/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Show a name and pronouns to members, and become findable only — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000014",
  "type": "https://trusttasks.org/spec/vtc/directory/card/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000012",
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
        "listing": "findable",
        "search": "exactNickname",
        "external": false
      },
      "introductions": "groups",
      "revision": 4,
      "updatedAt": "2026-10-18T09:00:01Z"
    },
    "missingRequired": []
  }
}
```

### Refused: a later change is already in force

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000015",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000016",
  "payload": {
    "code": "vtc/directory/card/update:superseded",
    "message": "A change to this card issued after this document is already applied.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Personal data the member chooses to put on their card, and their choices about who sees it. The community **MUST** keep `member`-sourced values only for the directory; a producer **MUST NOT** place anything in `ext` the member has not chosen to give the community.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The member's own DID is the signer; it is never shown to another member by this task. What another member can link is only what the member chose to show them, and every choice here can narrow it.

### Retention

**Durable** for the card and the signed update now in force, for as long as the membership. On departure the card follows the member's departure disposition, and a rotation of the member's DID carries it, because the card is keyed by `entryId`.

### Consent/purpose

The data is collected for one purpose: the community's member directory, shown to the audiences the member chooses. It **MUST NOT** be reused for anything else — a mailing list, an export to another community, registry publication — without the member's separate decision. Visibility here is not registry-publication consent, and registry-publication consent is not visibility here.
