---
slug: vtc/directory/fields/update
version: '0.1'
title: VTC Directory — Fields — Update
summary: An administrator changes a directory field's label, help text, constraints, order, ceiling or default visibility, or disables or re-enables a base field.
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
  rationale: Changing a ceiling changes the widest audience members' data may reach. The change must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed or reordered update could restore a ceiling the community has since narrowed.
sideEffects:
  level: mutating
  rationale: Replaces the named members of a field definition. Recoverable by a later update.
consequences:
- Narrowing a ceiling or disabling a field takes effect in every projection at once.
- 'Widening a ceiling or default changes no member''s recorded visibility: it widens only what members may now choose, and the default for cards with no choice.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Carries the changed members of a field definition and a reason; echoes the stored definition.
retention:
  class: durable
  rationale: Each ceiling in force is a promise to members about who may see their values; the community must be able to show which was in force when.
errorCodes:
- code: vtc/directory/fields/update:fieldUnknown
  meaning: No field has this key.
  retryable: false
- code: vtc/directory/fields/update:fieldRetired
  meaning: The field is retired and cannot be changed.
  retryable: false
- code: vtc/directory/fields/update:aboveCeiling
  meaning: The resulting `defaultVisibility` would be wider than the resulting `ceiling`.
  retryable: false
- code: vtc/directory/fields/update:valueInvalid
  meaning: The constraints do not fit the field's type, or `enabled` was sent for a community field.
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
related:
- vtc/directory/fields/list
- vtc/directory/fields/define
- vtc/directory/fields/retire
---

## Abstract

The **VTC Directory — Fields — Update** Trust Task changes a field of the directory schema. Only the members sent change. `key`, `type` and `source` are fixed for the field's life: a field that needs a different meaning is a new field.

**Narrowing and widening are not symmetric.** Lowering a ceiling, disabling a base field or making a field unsearchable protects members and applies at once. Raising a ceiling is privacy-affecting: it lets members' data reach a wider audience than the community promised when they filled it in. Even then it never widens a choice a member made — a member who chose `members` stays at `members` — but it does widen the default for cards with no choice, and what members may now pick.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.admin` with `permissionDenied`.
2. Refuse an unknown key with `fieldUnknown` and a retired field with `fieldRetired`.
3. Refuse a result in which `defaultVisibility` is wider than `ceiling` with `aboveCeiling`, and `enabled` on a community field with `valueInvalid`.
4. **MUST NOT** change any member's recorded visibility as a consequence of an update. Where the new ceiling is narrower than a member's choice, the projection intersects the two (the choice is kept, the ceiling wins); where it is wider, the choice is unchanged.
5. Apply a narrowing in every projection from the moment the update executes.
6. Regenerate its published directory disclosure statement.

**Widening is held, not refused.** A community **SHOULD** treat an update that widens a ceiling, widens a default visibility, makes a field searchable or makes it a facet as a privacy-affecting change of policy: it holds it for the agreement of its other administrators before executing, and answers with `trust-task-next-step` naming the held action. An update that both narrows and widens is held whole. Whether, and by whose agreement, a widening executes is the community's policy, which this specification does not declare.

## Authorization

**Authority: `vtc.directory.admin`** at the community, read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The `directoryWrite` policy purpose may refuse an update; it may not execute a widening the community holds for agreement.

## Definitions

- `FieldDefinition` — see [`field-definition.schema.json`](../../../_shared/0.1/field-definition.schema.json).
- **Widening** — any change after which some viewer may see a value they could not see before, or find an entry they could not find before.

## Request

The directory administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Narrow the bio to members

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000a",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/update/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000009",
  "payload": {
    "key": "bio",
    "ceiling": "members",
    "reason": "Bios are not for the public profile in this community."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/fields/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Narrow the bio to members — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000b",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000009",
  "payload": {
    "field": {
      "key": "bio",
      "label": "About",
      "type": "longText",
      "source": "member",
      "base": true,
      "jsContactPath": "notes",
      "required": false,
      "ceiling": "members",
      "defaultVisibility": "members",
      "searchable": false,
      "facet": false,
      "order": 40,
      "state": "active",
      "constraints": {
        "maxLength": 500
      },
      "updatedAt": "2026-10-18T09:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The changed members of one field definition and a reason. No member data. A producer **MUST NOT** place personal data in `label`, `description`, `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. A field definition is community-wide configuration.

### Retention

**Durable.** Each ceiling is a promise to members; the community keeps the history of the ceilings in force so it can answer for what a member's data could reach at any time.

### Consent/purpose

To maintain the directory schema. A widening extends a disclosure members did not choose when they filled the field in, which is why it never changes a member's own choice.
