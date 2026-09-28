---
slug: vtc/schemas/list
version: "0.1"
title: "VTC Schemas — List"
summary: An administrator pages through the community's schema registry — the credential types it issues and accepts — without their schema bodies.
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
  requirement: RECOMMENDED
  rationale: >-
    Authority is the signer's administrator standing, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport, relayed ones included. What it returns is the community's own configuration, not personal data.
sideEffects:
  level: none
  rationale: >-
    Reads the registry. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses registry configuration — type URIs, catalog bindings, descriptions, and which administrator registered each — to an administrator.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes: []
related:
  - vtc/schemas/show
  - vtc/schemas/register
  - vtc/schemas/delete
---

## Abstract

The **VTC Schemas — List** Trust Task pages through the community's schema registry, in type-URI order, optionally restricted to the `issues` or the `accepts` half. Each item is a summary: every member of the stored entry except its `credentialSchema` body, with `hasCredentialSchema` saying whether there is one. [`vtc/schemas/show`](../../show/0.1/spec.md) returns the body.

The split is what keeps a page bounded. A registry entry carries a whole JSON Schema, and a listing that returned every body would make every page as large as the sum of the schemas on it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** return entries in ascending `typeUri` order, restricted to `kind` when it is present, at most `limit` per page, with `nextCursor` present exactly when more remain.
3. **MUST NOT** include `credentialSchema` in an item, and **MUST** set `hasCredentialSchema` to whether the entry carries one.
4. **MUST** return `items: []`, not an error, for an empty registry.

## Definitions

- **Item** — a `SchemaSummary` from [`vtc/_shared/0.1/schema-registry.schema.json`](../../../_shared/0.1/schema-registry.schema.json).
- **`kind`** — restricts the page to one half of the registry.

## Request

A community administrator (`issuer`) asks the community (`recipient`) for a page.

### The issues half, first page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/schemas/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "kind": "issues",
    "limit": 50
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One page, nothing further

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/schemas/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "items": [
      {
        "typeUri": "https://riverside.example/credentials/WorkshopSafety",
        "kind": "issues",
        "description": "Completed the workshop safety induction.",
        "hasCredentialSchema": true,
        "createdAt": "2026-09-28T10:00:01Z",
        "createdByDid": "did:example:administrator"
      },
      {
        "typeUri": "https://openvtc.org/credentials/MembershipCredential",
        "dtgType": "MembershipCredential",
        "kind": "issues",
        "hasCredentialSchema": false,
        "createdAt": "2026-01-01T00:00:00Z",
        "createdByDid": "did:example:administrator"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The community's own registry configuration: type URIs, DTG catalog bindings, JSON Schemas, DCQL queries and administrators' descriptions. The description members are free text written by administrators, bounded at 1024 characters, read by other administrators (and, for a criterion, by applicants in the join manifest); untrusted, and attributed to `createdByDid` wherever rendered. `createdByDid` names an administrator.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The response names the administrator who last registered each entry — information about the community's own administrators, disclosed only to another administrator.

### Retention

A read. The community keeps nothing; the caller's copy is stale at the next registration or deletion.

### Consent/purpose

The purpose is to let administrators see and maintain what the community issues and accepts. The registry is community configuration and not a privacy-sensitive enumeration; it is disclosed to administrators because they maintain it.
