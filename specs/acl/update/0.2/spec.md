---
slug: acl/update
version: "0.2"
title: ACL — Update
summary: Amend an existing AclEntry 0.2 — widen its act scope, or replace its approve, capability or key scope, label, expiry or step-up — leaving role changes to acl/change-role and act narrowing to acl/revoke.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - acl
  - access-control
  - authorization
  - capability
  - amend
parties:
  - role: Granting authority
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: An amendment changes what a subject may do or approve, and is bounded by — and attributed to — the authority of the party that made it. It has the same evidentiary standing as the grant it modifies and may be relied on after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Every member replaces the stored value wholesale, so an out-of-order or replayed copy widens or narrows access by reverting to an earlier state rather than by expressing an intent. Placing the document in time is how the maintainer tells the two apart.
sideEffects:
  level: mutating
  rationale: "Replaces attributes of an existing entry; recoverable by amending again, or by acl/revoke/0.2."
consequences:
  - The subject's act scope, approve scope, capabilities, approvable capabilities, allowed keys, expiry or step-up requirement change at its next authorization decision.
  - "Setting `capabilities` to `ceiling` returns the entry to its role's full ceiling; clearing `expiresAt` makes a time-boxed grant permanent."
  - Widening `approve` or `approveCapabilities` lets the subject ratify operations it could not previously approve.
  - Narrowing `capabilities`, `approveCapabilities`, `approve` or `keys` is a privilege reduction and binds live sessions.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The payload names one subject and the attributes being set; the response echoes the realized entry to its amender and asserts nothing about anyone else.
retention:
  class: durable
  rationale: An accepted amendment is part of the entry's evidentiary history — in particular a reduction, which an auditor needs to find distinctly from a cosmetic change.
subjectPath: /subject
errorCodes:
  - code: acl/update:notFound
    meaning: No entry exists for `subject`. This task amends; use acl/grant to create.
    retryable: false
  - code: acl/update:roleChangeNotPermitted
    meaning: The payload attempted a role change. Use acl/change-role, which requires the current role.
    retryable: false
  - code: acl/update:narrowingNotPermitted
    meaning: The replacement `act` removes act scope the entry currently holds. Use acl/revoke/0.2.
    retryable: false
  - code: acl/update:invalidActScope
    meaning: The replacement act or approve scope cannot be held at this maintainer — it names a context the maintainer does not hold, or uses the `contexts` shape at a maintainer without contexts.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member]
      properties:
        member:
          type: string
          enum: [act, approve]
        offendingContexts:
          type: array
          items: { type: string }
  - code: acl/update:unknownCapability
    meaning: A capability in the replacement `capabilities` or `approveCapabilities` is not in the maintainer's capability registry.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: acl/update:capabilityOutsideCeiling
    meaning: A non-additive capability in the replacement set lies outside the ceiling of the entry's role.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [role, capabilities]
      properties:
        role: { type: string }
        capabilities:
          type: array
          items: { type: string }
  - code: acl/update:additiveRequiresUnrestricted
    meaning: The replacement set adds an `additive` capability and the amending party does not hold unrestricted act authority.
    retryable: false
  - code: acl/update:additiveWithinCeiling
    meaning: A capability is marked `additive` although the role's ceiling already includes it.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: acl/update:approveWiderThanGranter
    meaning: The replacement approve scope or approvable capabilities exceed what the amending party may itself approve.
    retryable: false
  - code: acl/update:delegationExceedsGranter
    meaning: The amended entry would hold authority the amending party does not hold — on act scope, a capability or its qualifier, the key filter, or expiry.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [axes]
      properties:
        axes:
          type: array
          minItems: 1
          items:
            type: string
            enum: [act, capabilities, keys, expiresAt]
related:
  - acl/grant
  - acl/change-role
  - acl/revoke
  - acl/show
---

## Abstract

The **ACL — Update** Trust Task amends an entry that already exists. Version 0.2 re-points it at [AclEntry 0.2](../../_shared/0.2/acl-entry.schema.json): the replaceable members are the explicit act scope (`act`, widening only), the approve scope (`approve`), the capability scope (`capabilities`), the approvable-capability scope (`approveCapabilities`), the key scope (`keys`), `label`, `expiresAt` and `stepUp`. 0.1's `scopes` and `allowedKeys` are gone, replaced by `act` and `keys`, and 0.1's "explicit `null` clears the filter" is gone with them: every authority-bearing replacement is an explicit scope.

The task fills the gap the rest of the family leaves: [`acl/grant`](../../grant/0.2/spec.md) creates and is not a role-change path, [`acl/change-role/0.2`](../../change-role/0.2/spec.md) moves the role and nothing else, and [`acl/revoke/0.2`](../../revoke/0.2/spec.md) removes an entry or narrows its act scope. None of them can say "this entry keeps its role but holds one fewer capability" without a revoke-then-grant window in which the subject holds nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)). [`acl/update/0.1`](../0.1/spec.md) remains current for maintainers holding AclEntry 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** send `subject` plus only the members being changed, omit what should stay, send explicit `null` to clear `label`, `expiresAt` or a `stepUp` member, and state every authority-bearing replacement in its explicit form (`{"scope": "ceiling"}`, `{"scope": "all"}`, `{"scope": "none"}` or a listed set) — there is no null form for those.

A conforming **consumer** **MUST**:

1. Refuse an unknown subject with `notFound`; this task does not create.
2. Refuse any role member with `roleChangeNotPermitted` (the schema rejects it; the code is for a consumer that reached the payload another way).
3. Compare a replacement `act` with the stored one and refuse a narrowing with `narrowingNotPermitted`.
4. Check every replacement capability as [`acl/grant/0.2`](../../grant/0.2/spec.md) does — `unknownCapability`, `capabilityOutsideCeiling`, `additiveWithinCeiling`, `additiveRequiresUnrestricted` — and never accept a replacement by dropping a capability.
5. Evaluate the **amended** entry against the amending party's stored entry ([CONVENTIONS §9](../../_shared/0.2/CONVENTIONS.md)) and refuse one that exceeds it with `delegationExceedsGranter` or `approveWiderThanGranter`. Replacing a listed set with `ceiling` (capabilities, approvable capabilities) or `all` (keys), lifting an expiry, and widening an approve scope are grants, bounded like one.
6. Refuse an amendment whose subject is the amending party itself with `permissionDenied`.
7. Apply each replacement wholesale. An omitted member leaves the stored value unchanged.
8. Treat a replacement that narrows `capabilities`, `approveCapabilities`, `approve` or `keys` as a **privilege reduction**: record it in the audit trail as a reduction, distinguishable from a cosmetic amendment, and apply it at the subject's next authorization decision rather than letting the wider authority survive until a session, token or cached decision expires.
9. Apply the additive-only rule to `stepUp`: a per-entry setting may raise the required assurance above the system floor, never lower it.
10. Record the amendment with the amending party's identity (`updatedBy`) and any `reason`. An amendment does not change `delegatedBy` unless the maintainer's policy re-delegates the entry to the amending party, in which case the entry is thereafter bounded by that party.

### What it deliberately cannot do

**It cannot change a role.** That transition belongs to `acl/change-role`, which requires the current role as a compare-and-swap. Two callers amending the same entry concurrently would otherwise silently overwrite one another on the attribute where a lost update is a privilege change rather than a cosmetic one.

**It cannot narrow `act`.** Narrowing where a subject may act is the reduction an auditor most needs to find, and it should appear in exactly one place — [`acl/revoke/0.2`](../../revoke/0.2/spec.md). If it were expressible here, a revocation could be performed by a task whose name says "amend".

**The four narrowings it does carry.** `acl/revoke/0.2` narrows only the act scope; it cannot express a reduction of capabilities, approvable capabilities, approve scope or keys, so refusing them here as well would leave them tightenable by no task at all. The exception relocates the obligations rather than relaxing them: item 8 above is the revocation doctrine, applied here.

## Authorization

**Authority: the amending party's own stored entry, as a ceiling over the amended entry.** An amendment is a re-grant of the entry as it will stand afterwards, so what entitles the producer is the same as for [`acl/grant/0.2`](../../grant/0.2/spec.md): every element of the amended entry must be contained in the amending party's live entry, and an additive capability requires unrestricted act authority. The bound is evaluated over the whole resulting entry, not only the members sent — a party that may amend one member of an entry still may not leave it holding authority the party lacks. Where the maintainer's contexts are hierarchical, the amending party must also be able to modify the entry as it stands: its act scope must cover every context in the entry's act and approve scopes, not merely overlap them.

Narrowing is held to the same bound and no other: a party entitled to amend an entry may narrow it. The `proof` identifies the amending party so its stored entry can be read and the change attributed; it is not the authorization. Whether a particular amendment — clearing an expiry, widening approve authority — is additionally subject to an approval or a step-up is the maintainer's policy and is not declared here.

## Definitions

* **Replacement.** Every member replaces the stored value; omitted leaves it. Authority-bearing members are replaced only by an explicit scope; `label`, `expiresAt` and `stepUp` members accept explicit `null` to clear.
* **Narrowing.** A replacement under which the entry may do, reach or approve strictly less on that member's axis.
* **Privilege reduction.** A narrowing of any axis, including those this task carries. Audited as a reduction and effective at the next authorization decision.
* Other terms are as in [`acl/grant/0.2`](../../grant/0.2/spec.md) and [CONVENTIONS](../../_shared/0.2/CONVENTIONS.md).

## Request

The amending party sends the ACL maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Narrow a repository manager to one repository

Bob held `git.repo.manage` across the `acme` namespace; the community narrows him to one repository, by id. This is a privilege reduction: the maintainer audits it as one and it binds Bob's next request.

```json
{
  "id": "urn:uuid:1b7c2d9e-6a54-4f1b-9c3d-2e8f7a6b5c41",
  "type": "https://trusttasks.org/spec/acl/update/0.2#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T11:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "capabilities": {
      "scope": "listed",
      "grants": [{ "capability": "git.repo.manage", "resource": "git-repo:github.com/acme/r#4211" }]
    },
    "reason": "Scope reduced to the payments repository."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-02T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z2pQExampleProofValue"
  }
}
```

### Widen a context administrator's act scope

Alice's act scope grows from one context to two. Widening is permitted here; narrowing it back would go through `acl/revoke/0.2`.

```json
{
  "id": "urn:uuid:2c8d3e0f-7b65-4a2c-8d4e-3f9a8b7c6d52",
  "type": "https://trusttasks.org/spec/acl/update/0.2#request",
  "issuer": "did:web:root-admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-10-02T11:05:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "act": { "scope": "contexts", "contexts": ["ctx/payments", "ctx/refunds"] }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:root-admin.example#key-1",
    "created": "2026-10-02T11:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3qRExampleProofValue"
  }
}
```

## Response

The ACL maintainer answers with a document of type `https://trusttasks.org/spec/acl/update/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `entry` — the realized AclEntry the maintainer now holds, after the amendment.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### Narrowing accepted

Response to the first request example:

```json
{
  "id": "urn:uuid:3d9e4f1a-8c76-4b3d-9e5f-4a0b9c8d7e63",
  "type": "https://trusttasks.org/spec/acl/update/0.2#response",
  "threadId": "urn:uuid:1b7c2d9e-6a54-4f1b-9c3d-2e8f7a6b5c41",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T11:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:bob.example",
      "role": "repo-manager",
      "act": { "scope": "all" },
      "keys": { "scope": "none" },
      "capabilities": {
        "scope": "listed",
        "grants": [{ "capability": "git.repo.manage", "resource": "git-repo:github.com/acme/r#4211" }]
      },
      "approve": { "scope": "all" },
      "approveCapabilities": {
        "scope": "listed",
        "grants": [{ "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" }]
      },
      "createdAt": "2026-10-02T10:05:01Z",
      "createdBy": "did:web:carol.example",
      "delegatedBy": "did:web:carol.example",
      "updatedAt": "2026-10-02T11:00:01Z",
      "updatedBy": "did:web:carol.example",
      "expiresAt": "2027-04-02T00:00:00Z"
    }
  }
}
```

The approve axis is independent and unchanged: Bob may still approve `git.repo.manage` across `acme`, though he may now exercise it only on one repository. A maintainer that wants the two to move together sends both members.

### Refused: an act-scope narrowing

Had the second request instead dropped `ctx/payments`, the maintainer would refuse it and direct the caller to `acl/revoke/0.2`:

```json
{
  "id": "urn:uuid:4e0f5a2b-9d87-4c4e-8f6a-5b1c0d9e8f74",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:2c8d3e0f-7b65-4a2c-8d4e-3f9a8b7c6d52",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:root-admin.example",
  "issuedAt": "2026-10-02T11:05:01Z",
  "payload": {
    "code": "acl/update:narrowingNotPermitted",
    "message": "The replacement act scope removes ctx/payments. Use acl/revoke/0.2.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries one subject identifier and the replacement values of the members being changed; the response carries the realized entry. Context paths and resource qualifiers can reveal organizational structure, and a capability change can reveal a change in a person's standing. A producer **MUST NOT** place in `label`, `reason` or `ext` anything it would not be comfortable signing and having retained, and **SHOULD NOT** put personal data there. The smallest payload is `subject` plus the one member being changed.

Four replacements are privilege **increases** that warrant at least the gate of the original grant: clearing `expiresAt` (a time-boxed grant becomes permanent), setting `capabilities` to `ceiling` (the entry returns to its role's full ceiling), setting `keys` to `all` (every key its act scope covers), and widening `approve` or `approveCapabilities` (a subject able to approve can ratify operations it could not authorize). Each is spelt as an explicit value rather than as the removal of a filter, so an increase is always visible in the payload and no serializer behaviour — dropping a null, omitting an empty list — can produce one.

### Correlation

The maintainer learns who amended which entry and how; an intermediary without transport confidentiality learns the same and can join the subject across tasks on its VID. `threadId` joins request to response. A sequence of amendments to one subject is, by design, correlatable at the maintainer — it is the entry's history.

### Retention

An accepted amendment is **durable** evidence, retained for at least the life of the entry and any audit obligation over it. A reduction especially: it is the record that authority was withdrawn, and an auditor reconstructing an incident needs it distinct from a cosmetic change. The duplicate-execution record is bounded by the acceptance window `issuedAt` supplies.

### Consent/purpose

The data is collected to change and enforce an access-control decision and attribute the change. It **SHOULD NOT** be reused beyond that purpose. Whether a given amendment needs an approval or a step-up is the maintainer's policy and is not declared here.
