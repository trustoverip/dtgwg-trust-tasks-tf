---
slug: vtc/directory/fields/list
version: '0.1'
title: VTC Directory — Fields — List
summary: Read a community's directory schema — base and community-defined fields, their types, ceilings and default visibility — as the caller may see it.
status: draft
targetFrameworkVersion: 0.6.0
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
  rationale: A read of community configuration whose answer depends on who asks (only administrators see retired fields); an attributable caller gets a correctly scoped answer.
sideEffects:
  level: none
  rationale: Reads the field schema; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'Returns field definitions and the community''s listing default: configuration, never any member''s value.'
retention:
  class: transient
  rationale: A read; nothing is kept beyond the audit record of it.
errorCodes: []
related:
- vtc/directory/fields/define
- vtc/directory/fields/update
- vtc/directory/fields/retire
- vtc/directory/card/update
---

## Abstract

The **VTC Directory — Fields — List** Trust Task returns the community's directory schema: every field a card can hold, what type it is, who writes it, the widest audience it may ever reach (its **ceiling**) and the audience a new card gives it. A member needs it to edit their card — the visibility picker offers only audiences up to each field's ceiling — and an administrator needs it to change the schema.

**Why there is no `show`.** The registry's default is a `list` and `show` pair. This collection does not need the pair: it is bounded (at most 64 active fields plus the base set), it is always read whole, and existence of a field is never in question for a caller, because every active field is returned to every member. A single field is read from this list.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** sends `includeRetired` only as an administrator.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Return every base field and every `active` and `disabled` community field to every caller it serves, in `order`, with its ceiling and default visibility. A field's definition is configuration, not member data, and members need it to choose their own visibility.
3. Return `retired` fields only to holders of `vtc.directory.admin` who asked for them.
4. Return `listingDefaults`.

## Authorization

**Authority: standing in the community.** Any active member may read the schema, because every member's card is written against it. Administrators holding `vtc.directory.admin` additionally see retired fields. The `proof`, where present, identifies the caller so their standing can be read from their access-control entry at execution time; it is not itself the authorization ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `FieldDefinition`, `ListingDefaults` — see [`field-definition.schema.json`](../../../_shared/0.1/field-definition.schema.json).
- **Ceiling** — the widest audience a field may ever reach in this community.

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member reads the schema

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000002",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/list/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000001",
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/fields/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A member reads the schema — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000003",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000001",
  "payload": {
    "fields": [
      {
        "key": "displayName",
        "label": "Name",
        "type": "text",
        "source": "member",
        "base": true,
        "jsContactPath": "name.full",
        "required": true,
        "ceiling": "public",
        "defaultVisibility": "members",
        "searchable": true,
        "facet": false,
        "order": 0,
        "state": "active",
        "constraints": {
          "maxLength": 64
        }
      },
      {
        "key": "did",
        "label": "DID",
        "type": "did",
        "source": "derived",
        "base": true,
        "required": false,
        "ceiling": "members",
        "defaultVisibility": "admins",
        "searchable": false,
        "facet": false,
        "order": 90,
        "state": "active"
      },
      {
        "key": "org.example.chapter",
        "label": "Chapter",
        "type": "enum",
        "source": "admin",
        "base": false,
        "required": false,
        "ceiling": "members",
        "defaultVisibility": "members",
        "searchable": false,
        "facet": true,
        "order": 40,
        "state": "active",
        "constraints": {
          "options": [
            {
              "value": "sydney",
              "label": "Sydney"
            },
            {
              "value": "berlin",
              "label": "Berlin"
            }
          ]
        }
      }
    ],
    "listingDefaults": {
      "defaultListing": "listed"
    }
  }
}
```

## Security & Privacy

### Data carried

Field definitions and the listing default: the community's configuration. No member's value travels in either direction.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The schema is the same for every member, so reading it tells the community nothing about the caller beyond that they read it.

### Retention

Nothing beyond the audit record of the read.

### Consent/purpose

The schema is disclosed so that members can fill in and control their own cards, and administrators can maintain it. It is also the input from which the community generates its published directory disclosure statement.
