---
slug: vtc/roles/show
version: "0.1"
title: VTC Roles — Show
summary: A querying party fetches one administrative role of a Verifiable Trust Community by name — its capability ceiling, approve ceiling and, optionally, how many entries hold it.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - vtc
  - role
  - capability
  - ceiling
parties:
  - role: Querying party
    requirement: REQUIRED
    member: issuer
  - role: VTC
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A single-role read is typically consumed over an authenticated transport while building or reviewing a grant; a proof matters when the answer is retained or relied on by a third party.
sideEffects:
  level: none
  rationale: "Read-only read of one role record."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses one role's name, description, ceilings and provenance, and optionally a count of the entries holding it — never their subjects.
retention:
  class: transient
  rationale: The answer is a point-in-time read; the VTC's role record is the record.
errorCodes:
  - code: vtc/roles/show:notFound
    meaning: No built-in or custom role of this name exists.
    retryable: false
related:
  - vtc/roles/list
  - vtc/roles/define
  - vtc/roles/delete
  - acl/list
---

## Abstract

The **VTC Roles — Show** Trust Task fetches one administrative role of a Verifiable Trust Community (VTC) by name, as a [`RoleDefinition`](../../_shared/0.1/role.schema.json), with an optional count of the ACL entries holding it. It is the single-record half of the split pair with [`vtc/roles/list`](../../list/0.1/spec.md): an unknown name is a definite `notFound`, which a list cannot express. The holder count is what a requester needs before [`vtc/roles/delete`](../../delete/0.1/spec.md) (which refuses a held role) or before replacing a role through [`vtc/roles/define`](../../define/0.1/spec.md) (which changes every holder at once).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTC) **MUST**:

1. Apply its own policy to decide whether the querying party may read the role, refusing with `permissionDenied` where it may not.
2. Refuse an unknown name with `notFound`.
3. Return the role record it enforces, with `ceiling` and `approveScope` stated in full.
4. Where it includes `holders`, count every ACL entry whose `role` is this role — the same set `vtc/roles/delete` counts for `inUse` — so that a zero here means a deletion will not be refused for holders. It **MAY** omit `holders` for a querying party that may not learn it.

## Definitions

* **Holder.** An ACL entry whose `role` is the named role.
* Other terms are as in [`RoleDefinition`](../../_shared/0.1/role.schema.json).

## Request

The querying party sends the VTC a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Look up a built-in role

```json
{
  "id": "urn:uuid:9d0e1f2a-3b4c-4d5e-9f6a-8b9c0d1e2f3a",
  "type": "https://trusttasks.org/spec/vtc/roles/show/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T17:10:00Z",
  "payload": {
    "name": "moderator"
  }
}
```

## Response

The VTC answers with a document of type `https://trusttasks.org/spec/vtc/roles/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `role` — the role record.
* `holders` — optional; the number of entries holding the role.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### The moderator role

```json
{
  "id": "urn:uuid:0e1f2a3b-4c5d-4e6f-8a7b-9c0d1e2f3a4b",
  "type": "https://trusttasks.org/spec/vtc/roles/show/0.1#response",
  "threadId": "urn:uuid:9d0e1f2a-3b4c-4d5e-9f6a-8b9c0d1e2f3a",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T17:10:01Z",
  "payload": {
    "role": {
      "name": "moderator",
      "builtIn": true,
      "description": "Manages members, decides joins and issues invitations.",
      "ceiling": [
        { "capability": "vtc.members.manage" },
        { "capability": "vtc.join.decide" },
        { "capability": "vtc.invitations.manage" }
      ],
      "approveScope": [
        { "capability": "vtc.members.manage" },
        { "capability": "vtc.join.decide" },
        { "capability": "vtc.invitations.manage" }
      ]
    },
    "holders": 4
  }
}
```

## Security & Privacy

### Data carried

The request carries a role name. The response carries that role's description, ceilings with any resource qualifiers, the provenance of a custom role, and optionally a holder count. It never names the holders; a party entitled to see them uses `acl/list/0.2` filtered by role.

### Correlation

The role record is community vocabulary, not about a person, except for custom-role provenance. The holder count reveals the size of a role's membership, which for a small role (one `credential-officer`) can come close to identifying who holds it; a VTC **MAY** omit `holders` for querying parties that do not administer roles.

### Retention

A point-in-time read; the querying party **SHOULD NOT** retain it beyond the decision it informs.

### Consent/purpose

The data is disclosed so that administrators can build, review, replace and delete roles. It **SHOULD NOT** be reused for other purposes. Who may read a role is the VTC's policy.
