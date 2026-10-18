---
slug: vtc/directory/fields/define
version: '0.1'
title: VTC Directory — Fields — Define
summary: 'An administrator adds a field to the community''s directory schema: its type, who writes it, its ceiling and its default visibility.'
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
  rationale: A field definition fixes, for the field's life, what members may be asked to disclose and to whom. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed define of a since-retired key would be refused, but a delayed one could land after the community changed its mind.
sideEffects:
  level: mutating
  rationale: Adds a field definition. Recoverable by retiring it; the key is never reused.
consequences:
- Members may fill the field in, and its default visibility applies to every card that has no choice for it.
- The field enters the community's published directory disclosure statement.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries a field definition and a reason, and echoes the stored definition. No member data.
retention:
  class: durable
  rationale: The definition governs what every card may hold and to whom it may be shown, and is part of the community's published disclosure.
errorCodes:
- code: vtc/directory/fields/define:fieldExists
  meaning: A field with this key exists, or existed and was retired. A key is never reused with a different meaning.
  retryable: false
- code: vtc/directory/fields/define:reservedKey
  meaning: 'The key has no `.`: keys without one are reserved to the base fields.'
  retryable: false
- code: vtc/directory/fields/define:aboveCeiling
  meaning: '`defaultVisibility` is wider than `ceiling`.'
  retryable: false
- code: vtc/directory/fields/define:valueInvalid
  meaning: 'The constraints do not fit the type (options on a text field, `facet` on a type that cannot be one, `claim` without `source: credential`).'
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
- code: vtc/directory/fields/define:tooManyFields
  meaning: The community already has 64 active community fields.
  retryable: false
related:
- vtc/directory/fields/list
- vtc/directory/fields/update
- vtc/directory/fields/retire
---

## Abstract

The **VTC Directory — Fields — Define** Trust Task adds a community field to the directory schema — an employer, a chapter, a verified professional registration. Every community already has the base fields (`displayName`, `nickname`, `avatar`, `pronouns`, `bio`, `links`, and the derived `role`, `joinedAt`, `did`, `personhood`); this task adds to them. Base fields are changed with [`vtc/directory/fields/update`](../../update/0.1/spec.md), never defined.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** supplies a reverse-DNS `key` and states `ceiling` and `defaultVisibility`.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.admin` with `permissionDenied`.
2. Refuse a key without a `.` with `reservedKey`, and a key that exists or has ever existed with `fieldExists`.
3. Refuse a `defaultVisibility` wider than `ceiling` with `aboveCeiling`, and constraints that do not fit the type with `valueInvalid`. A `claim` field **MUST** have `source: credential`; a `facet` **MUST** be of type `enum`, `multiEnum` or `country`.
4. Refuse with `tooManyFields` when 64 community fields are already `active`.
5. Store the field `active`, return it, and regenerate its published directory disclosure statement from the new schema.

A new field changes no existing card: no card holds a value for it until its member, an administrator or a presentation writes one.

## Authorization

**Authority: `vtc.directory.admin`** at the community, read from the signer's access-control entry at execution time. Defining a field decides what members may be asked to disclose, and at most to whom — the community's directory schema. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

The community's `directoryWrite` policy purpose may refuse a definition. A community may also choose to treat a new field whose ceiling is `public` as privacy-affecting and require other administrators' agreement before it executes; that is the community's policy, which this specification does not declare.

## Definitions

- `FieldInput`, `FieldDefinition` — see [`field-definition.schema.json`](../../../_shared/0.1/field-definition.schema.json).

## Request

The directory administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Define a verified registration field

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000005",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/define/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000004",
  "payload": {
    "field": {
      "key": "org.example.registration",
      "label": "Professional registration",
      "type": "claim",
      "source": "credential",
      "constraints": {
        "claim": {
          "credentialType": "ProfessionalRegistration",
          "claimPath": "/registrationNumber"
        }
      },
      "ceiling": "members",
      "defaultVisibility": "admins",
      "searchable": false,
      "facet": false,
      "order": 50
    },
    "reason": "Members asked to show that they are registered."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/fields/define/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Define a verified registration field — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000006",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/define/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000004",
  "payload": {
    "field": {
      "key": "org.example.registration",
      "label": "Professional registration",
      "type": "claim",
      "source": "credential",
      "base": false,
      "constraints": {
        "claim": {
          "credentialType": "ProfessionalRegistration",
          "claimPath": "/registrationNumber"
        }
      },
      "required": false,
      "ceiling": "members",
      "defaultVisibility": "admins",
      "searchable": false,
      "facet": false,
      "order": 50,
      "state": "active",
      "updatedAt": "2026-10-18T09:00:01Z"
    }
  }
}
```

### Refused: a base key

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000007",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000008",
  "payload": {
    "code": "vtc/directory/fields/define:reservedKey",
    "message": "Keys without a dot are reserved to the base fields.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A field definition and an optional reason. A producer **MUST NOT** place personal data in `label`, `description`, `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. A field definition is community-wide configuration and is not about any person.

### Retention

**Durable.** The definition governs every card and is part of the community's published disclosure statement; the community keeps it, and the document that created it, for as long as the field or any value of it exists.

### Consent/purpose

To let a community ask its members for more than the base fields, within a ceiling it publishes. The ceiling is a promise to members about the widest audience the field will ever reach without their own choice.
