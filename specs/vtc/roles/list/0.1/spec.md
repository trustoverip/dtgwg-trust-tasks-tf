---
slug: vtc/roles/list
version: "0.1"
title: VTC Roles — List
summary: A querying party asks a Verifiable Trust Community for its administrative role vocabulary — every built-in and custom role, with each role's capability ceiling and approve ceiling.
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
  rationale: The role vocabulary is read over an authenticated transport to build a grant or explain one; a proof matters when the answer is retained or relied on by a third party, for instance as the ceiling a grant was judged against.
sideEffects:
  level: none
  rationale: "Read-only enumeration of role records."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses the community's administrative vocabulary — role names, descriptions, capability ceilings and their resource qualifiers — and the provenance of custom roles.
retention:
  class: transient
  rationale: The answer is a point-in-time read of a small vocabulary; the VTC's role records are the record.
errorCodes: []
related:
  - vtc/roles/show
  - vtc/roles/define
  - vtc/roles/delete
  - acl/grant
---

## Abstract

The **VTC Roles — List** Trust Task returns a Verifiable Trust Community's administrative roles: the built-in set and any custom roles, each as a [`RoleDefinition`](../../_shared/0.1/role.schema.json) carrying its capability ceiling and approve ceiling. A client reads it to know which roles it can grant through [`acl/grant/0.2`](../../../../acl/grant/0.2/spec.md) and what each admits.

It is the enumeration half of the split pair with [`vtc/roles/show`](../../show/0.1/spec.md), which fetches one role by name and answers `notFound` for an unknown one. The list is not paged: a community's role vocabulary is small and bounded (seven built-in roles plus what the community defines), so a complete answer is always practical.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTC) **MUST**:

1. Apply its own policy to decide whether the querying party may read the vocabulary, and refuse with `permissionDenied` where it may not. A VTC **SHOULD** let any administrator read it, since every grant and approval is judged against it.
2. Return every role matching `includeBuiltIn` (all roles when it is absent or `true`; custom roles only when `false`), each with its complete `ceiling` and `approveScope` stated explicitly — the records it actually enforces, not a summary.

## Definitions

* **Built-in role.** A role fixed by the VTC implementation (`builtIn: true`).
* **Custom role.** A community-defined role (`builtIn: false`).
* Ceiling and approve ceiling are as in [`RoleDefinition`](../../_shared/0.1/role.schema.json).

## Request

The querying party sends the VTC a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Custom roles only

```json
{
  "id": "urn:uuid:7b8c9d0e-1f2a-4b3c-9d4e-6f7a8b9c0d1e",
  "type": "https://trusttasks.org/spec/vtc/roles/list/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T17:00:00Z",
  "payload": {
    "includeBuiltIn": false
  }
}
```

## Response

The VTC answers with a document of type `https://trusttasks.org/spec/vtc/roles/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `roles` — every matching role, complete. **MAY** be empty.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### The community's custom roles

```json
{
  "id": "urn:uuid:8c9d0e1f-2a3b-4c4d-8e5f-7a8b9c0d1e2f",
  "type": "https://trusttasks.org/spec/vtc/roles/list/0.1#response",
  "threadId": "urn:uuid:7b8c9d0e-1f2a-4b3c-9d4e-6f7a8b9c0d1e",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T17:00:01Z",
  "payload": {
    "roles": [
      {
        "name": "events-team",
        "builtIn": false,
        "description": "Runs the community's public pages and event invitations.",
        "ceiling": [
          { "capability": "vtc.surface.admin" },
          { "capability": "vtc.invitations.manage" }
        ],
        "approveScope": [{ "capability": "vtc.invitations.manage" }],
        "createdAt": "2026-10-02T15:30:00Z",
        "createdBy": "did:web:carol.example"
      },
      {
        "name": "acme-steward",
        "builtIn": false,
        "description": "Repository and vetting stewardship for the acme namespace.",
        "ceiling": [
          { "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" },
          { "capability": "vtc.vetting.manage" }
        ],
        "approveScope": [
          { "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" }
        ],
        "createdAt": "2026-09-14T08:00:00Z",
        "createdBy": "did:web:carol.example"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries one optional flag. The response carries the community's administrative vocabulary: role names, descriptions, capability ceilings with resource qualifiers (which can name repositories, policies and join criteria), and the identifiers of the administrators who defined custom roles. It carries no information about which subjects hold which role; that is `acl/list/0.2`, under its own access policy.

### Correlation

The vocabulary is community-wide and not about any person, except for the `createdBy` / `updatedBy` provenance of custom roles, which links administrators to the roles they defined. A VTC **MAY** omit provenance for querying parties that have no need of it.

### Retention

A point-in-time read; the querying party **SHOULD NOT** retain it beyond the decision it informs. Where it keeps an answer as evidence of the ceiling a grant was judged against, it **SHOULD** require a `proof`.

### Consent/purpose

The data is disclosed so that administrators can construct and review grants and approvals. It **SHOULD NOT** be reused for other purposes. Who may read the vocabulary is the VTC's policy.
