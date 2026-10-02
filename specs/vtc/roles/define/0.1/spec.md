---
slug: vtc/roles/define
version: "0.1"
title: VTC Roles — Define
summary: A community administrator creates a custom administrative role at a Verifiable Trust Community, or replaces one — a named capability ceiling and approve ceiling that AclEntry 0.2 entries can then hold.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - vtc
  - role
  - capability
  - ceiling
  - custom-role
  - administration
parties:
  - role: Community administrator
    requirement: REQUIRED
    member: issuer
  - role: VTC
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A role definition bounds the authority of every entry that holds it, now and later. It is authority-defining, is typically executed only after other administrators approve it, and must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The ceiling is replaced wholesale, so a replayed or reordered definition silently widens or narrows every holder by reverting the role to an earlier state. Placing the document in time bounds the duplicate-execution window and lets approvers judge its currency.
sideEffects:
  level: mutating
  rationale: "Creates or replaces a custom role record; recoverable by defining it again or by vtc/roles/delete when no entry holds it."
consequences:
  - Replacing a role changes the effective capabilities of every entry that holds it at their next authorization decision — widening the ceiling widens every holder whose capability scope is `ceiling`.
  - Narrowing a ceiling is a privilege reduction for every holder.
  - A new role becomes grantable through acl/grant/0.2 to anyone whose granter holds its capabilities.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The request carries a role name, its capability ceilings and a reason; the response echoes the stored role. Neither carries personal data about anyone but the requester's provenance.
retention:
  class: durable
  rationale: The accepted document is the record of how the community's authority vocabulary was changed and by whom; every grant of the role is interpreted against it.
errorCodes:
  - code: vtc/roles/define:builtInRole
    meaning: The name is a built-in role, which cannot be created, changed or replaced.
    retryable: false
  - code: vtc/roles/define:exists
    meaning: A custom role of this name already exists and `replaces` is not true.
    retryable: false
  - code: vtc/roles/define:notFound
    meaning: "`replaces` is true and no custom role of this name exists."
    retryable: false
  - code: vtc/roles/define:unknownCapability
    meaning: A capability in `ceiling` or `approveScope` is not in the VTC's capability registry.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: vtc/roles/define:additiveCapability
    meaning: The ceiling names a capability the registry classes as additive. An additive capability is one no role implies; naming it in a ceiling would let a granter without unrestricted authority confer it.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: vtc/roles/define:exceedsDefinerAuthority
    meaning: The ceiling or approve ceiling names a capability (or a qualifier wider than one) that the administrators whose authority defines the role do not themselves hold, or may not themselves approve.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
related:
  - vtc/roles/list
  - vtc/roles/show
  - vtc/roles/delete
  - acl/grant
  - acl/update
---

## Abstract

The **VTC Roles — Define** Trust Task creates a custom administrative role at a Verifiable Trust Community (VTC), or replaces one. A role is a named **ceiling**: the capabilities an [AclEntry 0.2](../../../../acl/_shared/0.2/acl-entry.schema.json) holding it may hold (`ceiling`), and the capabilities it may approve (`approveScope`). It grants nothing on its own; [`acl/grant/0.2`](../../../../acl/grant/0.2/spec.md) grants, bounded by the role.

A community defines a custom role when the built-in set ([`RoleDefinition`](../../_shared/0.1/role.schema.json)) does not fit — for example `events-team` = `vtc.surface.admin` + `vtc.invitations.manage`, or a combination of two built-in roles, since an entry holds one administrative role. Built-in roles cannot be defined, changed or replaced.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** states `ceiling` and `approveScope` in full — empty arrays where the role admits or approves nothing — and sets `replaces: true` only to change an existing custom role.

A conforming **consumer** (the VTC) **MUST**:

1. Refuse a built-in role name with `builtInRole`.
2. Refuse a create (`replaces` absent or false) of an existing name with `exists`, and a replace of a missing name with `notFound`.
3. Refuse a capability not in its registry with `unknownCapability`, and an additive capability in `ceiling` with `additiveCapability`. Neither is resolved by storing the role without it.
4. Refuse a `ceiling` entry the defining administrators do not hold — at a qualifier at least as wide — and an `approveScope` entry they may not themselves approve, with `exceedsDefinerAuthority`. The defining administrators are the requester and, where the VTC executes the definition after an approval, the approvers whose authority it relies on; each is evaluated against its **stored** entry.
5. Store the role with `builtIn: false`, `createdAt` / `createdBy` (or `updatedAt` / `updatedBy` on a replace), and return it.
6. On a replace, apply the new ceilings to every entry holding the role at that entry's next authorization decision. A narrowing is a privilege reduction for each holder, audited as one; a widening widens every holder whose `capabilities` (or `approveCapabilities`) is `{"scope": "ceiling"}`, and the VTC **SHOULD** report to the requester how many entries the replacement affects.
7. Treat a role definition only as a record in its access-control model, evaluated by host code. It **MUST NOT** accept a role definition from, or let one be widened by, a policy document; policy may only refuse.

## Authorization

**Authority: the defining administrators' own capabilities, as a ceiling over the role's.** A custom role is a delegation template — every grant of it later hands out part of what its ceiling names — so what entitles the producer to define it is that the administrators whose authority it rests on hold, themselves, every capability the ceiling names (at a qualifier at least as wide) and may themselves approve every capability the approve ceiling names. A role therefore cannot be used to manufacture a capability nobody behind it held. Item 4 enforces this.

At a VTC implementing the administrative-roles model, defining or replacing a custom role is an act of administering the community's authority vocabulary: it is performed under the capability that assigns roles together with the capability that administers approvals, and the VTC executes it only once an N-of-M approval among its community administrators has been satisfied. That describes the model this task is designed for; the threshold, the approver set and whether a step-up accompanies it are the VTC's policy, which this specification does not declare.

The `proof` identifies the requester so that its stored entry can be read and the definition attributed. It is not the authorization, and nor is any one approver's proof.

## Definitions

* **Custom role.** A community-defined role, `builtIn: false`.
* **Ceiling** (`ceiling`). The capabilities an entry holding the role may hold — never more, possibly less.
* **Approve ceiling** (`approveScope`). The capabilities an entry holding the role may approve an action needing, independent of `ceiling`.
* **Additive capability.** A capability the VTC's registry classes as one no role implies; never in a ceiling.
* **Defining administrators.** The requester and the approvers whose authority a definition is executed under.

## Request

The community administrator sends the VTC a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Define an events team

```json
{
  "id": "urn:uuid:0a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
  "type": "https://trusttasks.org/spec/vtc/roles/define/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T14:00:00Z",
  "payload": {
    "name": "events-team",
    "description": "Runs the community's public pages and event invitations.",
    "ceiling": [
      { "capability": "vtc.surface.admin" },
      { "capability": "vtc.invitations.manage" }
    ],
    "approveScope": [
      { "capability": "vtc.invitations.manage" }
    ],
    "reason": "Delegating event operations to the events volunteers."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-02T14:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The VTC answers with a document of type `https://trusttasks.org/spec/vtc/roles/define/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `role` — the stored [`RoleDefinition`](../../_shared/0.1/role.schema.json), with `builtIn: false` and its provenance.

Where the VTC holds the request pending an approval, it answers when the definition executes, or with `trust-task-error` if the approval is refused or lapses. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### Role defined

```json
{
  "id": "urn:uuid:1b2c3d4e-5f6a-4b7c-9d8e-0f1a2b3c4d5e",
  "type": "https://trusttasks.org/spec/vtc/roles/define/0.1#response",
  "threadId": "urn:uuid:0a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T15:30:00Z",
  "payload": {
    "role": {
      "name": "events-team",
      "builtIn": false,
      "description": "Runs the community's public pages and event invitations.",
      "ceiling": [
        { "capability": "vtc.surface.admin" },
        { "capability": "vtc.invitations.manage" }
      ],
      "approveScope": [
        { "capability": "vtc.invitations.manage" }
      ],
      "createdAt": "2026-10-02T15:30:00Z",
      "createdBy": "did:web:carol.example"
    }
  }
}
```

### Refused: redefining a built-in role

```json
{
  "id": "urn:uuid:2c3d4e5f-6a7b-4c8d-8e9f-1a2b3c4d5e6f",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:3d4e5f6a-7b8c-4d9e-9f0a-2b3c4d5e6f7a",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T14:10:01Z",
  "payload": {
    "code": "vtc/roles/define:builtInRole",
    "message": "moderator is a built-in role and cannot be changed.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a role name, a description, two capability lists with optional resource qualifiers, and a reason. Qualifiers can name the community's repositories, policies or join criteria; a description or reason can reveal internal organisation. A producer **MUST NOT** place personal data in `description`, `reason` or `ext`, and **SHOULD** keep `reason` to what approvers need in order to decide. The smallest payload is a name and two explicit lists.

The security-relevant property is not confidentiality but amplification. A role is applied to every entry that holds it, so replacing one changes many entries at once, and widening a ceiling widens every holder whose capability scope is `ceiling` — without any `acl/*` task naming those holders. That is why the task is bounded by the definers' own authority, why it is the kind of act a VTC places behind an approval, and why consumers report the number of affected entries.

### Correlation

The VTC learns who defined which role, and approvers see the same. Role names and ceilings are community-wide vocabulary and are not, in themselves, about any person; they become correlating only through the entries that hold them, which `acl/list/0.2` (filtered by role) reveals to those permitted to see it. `threadId` joins request to response, which for an approval-gated definition may be separated by a long interval.

### Retention

The accepted document is **durable**: it records how the community's authority vocabulary changed and on whose authority, and every later grant of the role is interpreted against the definition in force. A VTC **SHOULD** retain it for at least as long as any entry holds the role and any audit obligation over it.

### Consent/purpose

The data is collected to define and enforce the community's administrative authority. It **SHOULD NOT** be reused for any other purpose. Whether a definition must be approved by other administrators, and by how many, is the VTC's policy and is not declared here.
