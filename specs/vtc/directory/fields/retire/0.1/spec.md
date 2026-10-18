---
slug: vtc/directory/fields/retire
version: '0.1'
title: VTC Directory — Fields — Retire
summary: 'An administrator retires a community directory field: it is hidden at once, its values are purged after the community''s retention period, and its key is never reused.'
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
  rationale: Retiring a field schedules the purge of every member's value of it. The decision must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.
sideEffects:
  level: destructive
  rationale: Every value of the field is purged once the retention period passes, and the key can never be used again. Hiding is immediate; the purge is irreversible.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names a field and a reason; returns the retired definition and its purge date.
retention:
  class: durable
  rationale: The retirement is why members' values were purged; the community keeps it as the record of that erasure.
errorCodes:
- code: vtc/directory/fields/retire:fieldUnknown
  meaning: No field has this key.
  retryable: false
- code: vtc/directory/fields/retire:fieldRetired
  meaning: The field is already retired.
  retryable: false
- code: vtc/directory/fields/retire:baseField
  meaning: Base fields are disabled with vtc/directory/fields/update, never retired.
  retryable: false
related:
- vtc/directory/fields/update
- vtc/directory/fields/list
---

## Abstract

The **VTC Directory — Fields — Retire** Trust Task withdraws a community field. The field disappears from every projection, card editor and search at once; its values are purged after the community's retention period; and its key is never reused, so a value written under one meaning can never be read under another.

There is no delete. Deleting a definition while values exist would orphan them, and reusing its key would silently reinterpret them.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.admin` with `permissionDenied`.
2. Refuse an unknown key with `fieldUnknown`, a retired field with `fieldRetired` and a base field with `baseField`.
3. Remove the field from every projection, search index and facet from the moment it executes.
4. Purge every stored value of the field no later than `purgeAfter`, and keep the key reserved for ever.
5. Regenerate its published directory disclosure statement.

## Authorization

**Authority: `vtc.directory.admin`** at the community, read from the signer's access-control entry at execution time. Retiring is a narrowing — it only ever withdraws data from view — so it needs no more than the capability that defines the schema. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **Retention period** — how long the community keeps values of a retired field before purging them, its own setting.

## Request

The directory administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Retire a field

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000d",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/retire/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000c",
  "payload": {
    "key": "org.example.employer",
    "reason": "No longer collected."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/fields/retire/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Retire a field — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000e",
  "type": "https://trusttasks.org/spec/vtc/directory/fields/retire/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000000c",
  "payload": {
    "field": {
      "key": "org.example.employer",
      "label": "Employer",
      "type": "text",
      "source": "member",
      "base": false,
      "required": false,
      "ceiling": "members",
      "defaultVisibility": "members",
      "searchable": true,
      "facet": false,
      "order": 40,
      "state": "retired",
      "retiredAt": "2026-10-18T09:00:01Z",
      "purgeAfter": "2026-11-17T09:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A key and a reason. A producer **MUST NOT** place personal data in `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Community configuration only.

### Retention

**Durable** for the retirement itself. The purge it schedules is the point: values are kept no longer than the retention period, which exists so a mistaken retirement can be reversed by an operator before data is lost.

### Consent/purpose

Data minimisation: a community stops collecting what it no longer needs, and erases what it collected.
