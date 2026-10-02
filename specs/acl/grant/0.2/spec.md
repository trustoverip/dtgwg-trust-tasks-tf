---
slug: acl/grant
version: "0.2"
title: ACL — Grant
summary: A granting authority adds a subject to an access-control list with a role, an explicit act scope, and optionally an approve scope and a narrowed, qualified or additive capability set.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - acl
  - access-control
  - authorization
  - role
  - capability
  - delegation
  - grant
parties:
  - role: Granting authority
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A grant is the evidentiary record every later authorization decision is read from. It may be replayed by an auditor, corroborated by a downstream service, or relied on after the original transport has closed, and the bound it is evaluated against is the granter's authority — which the maintainer can only attribute if the granter cannot repudiate the document.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A replayed grant re-admits a subject that has since been revoked, or re-asserts capabilities its granter has since lost. Only the duplicate-execution record of SPEC §7.2 item 11 stops the second execution, and that record is bounded by the acceptance window this member supplies.
sideEffects:
  level: mutating
  rationale: "Adds a subject to the ACL with a role, act scope, and optional approve scope and capabilities; recoverable via acl/revoke."
consequences:
  - The subject can act wherever `act` reaches, with the effective capability set, from the next authorization decision.
  - A non-none `approve` lets the subject ratify other parties' actions — including actions it cannot perform itself.
  - An additive capability is held outside the role's ceiling and survives a later role change only if re-granted.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The request carries a subject identifier and the authority to be held; the response echoes the realized entry back to its granter and discloses nothing about any other subject.
retention:
  class: durable
  rationale: The accepted document is the evidentiary record of the grant and of the granter it was delegated from; deleting it removes the only attributable account of why the subject holds what it holds.
subjectPath: /entry/subject
errorCodes:
  - code: acl/grant:roleNotRecognized
    meaning: The role string is not part of the ACL maintainer's role vocabulary.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        offendingRole: { type: string }
        knownRoles:
          type: array
          items: { type: string }
  - code: acl/grant:invalidActScope
    meaning: The act or approve scope cannot be held at this maintainer — it names a context the maintainer does not hold, or uses the `contexts` shape at a maintainer that has no contexts. (An empty `contexts` list never reaches this check; the schema refuses it.)
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
  - code: acl/grant:unknownCapability
    meaning: A capability in `capabilities` or `approveCapabilities` is not in the maintainer's capability registry. It is refused, never treated as granted and never silently dropped.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: acl/grant:capabilityOutsideCeiling
    meaning: A non-additive capability lies outside the ceiling of the entry's role. The grant is refused rather than accepted with the capability omitted; the author revisits the role.
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
  - code: acl/grant:additiveRequiresUnrestricted
    meaning: The entry carries an `additive` capability and the granting authority does not hold unrestricted act authority (`act` of `all`).
    retryable: false
  - code: acl/grant:additiveWithinCeiling
    meaning: A capability is marked `additive` although the role's ceiling already includes it. The flag would let an ordinary capability survive a later role change, so it is refused.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: acl/grant:approveWiderThanGranter
    meaning: The entry's approve scope or approvable capabilities exceed what the granting authority may itself approve.
    retryable: false
  - code: acl/grant:delegationExceedsGranter
    meaning: The entry would hold authority the granting authority does not hold — on act scope, a capability or its qualifier, the key filter, or expiry — evaluated against the granter's stored entry.
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
  - acl/update
  - acl/revoke
  - acl/change-role
  - acl/show
  - acl/list
  - vtc/roles/define
---

## Abstract

The **ACL — Grant** Trust Task adds a subject to an access-control list. The *granting authority* declares to the *ACL maintainer* the *AclEntry* the maintainer should hold for the subject; the maintainer applies its own policy, bounds the entry by the granter's own authority, and — if it accepts — holds the realized entry and keeps the document as the evidentiary record.

Version 0.2 re-points the task at [AclEntry 0.2](../../_shared/0.2/acl-entry.schema.json). The change that motivates it is that **every element of authority is stated rather than inferred**. 0.1's `scopes` list meant "everywhere" for one role and "nowhere" for another when empty, depending on a maintainer convention the document did not carry; 0.2 replaces it with an explicit `act` scope (`all`, `none`, or a non-empty list of contexts), and replaces 0.1's `approve {all, scopes}` with the same explicit shape and 0.1's `allowedKeys` (absent meant every key) with an explicit, REQUIRED `keys` scope. It adds capability-level authority — a narrowed, optionally resource-qualified capability set bounded by the role's ceiling, and a separate set of capabilities the subject may approve — and records the granter the entry was delegated from.

The task is **idempotent**: re-emitting an identical grant against an unchanged ACL produces no state change. It does not change roles ([`acl/change-role`](../../change-role/0.2/spec.md)) and does not narrow ([`acl/revoke`](../../revoke/0.2/spec.md)). Role and capability vocabularies are opaque to this family; each maintainer publishes its own (the [`vtc/roles`](../../../vtc/roles/list/0.1/spec.md) family is one).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)). It supersedes nothing yet: [`acl/grant/0.1`](../0.1/spec.md) remains current for maintainers that have not adopted AclEntry 0.2, and [CONVENTIONS §8](../../_shared/0.2/CONVENTIONS.md) documents how a maintainer serving both maps one onto the other.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the granting authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/grant/0.2`, with itself as `issuer` and the ACL maintainer as `recipient`, carrying a `proof` ([SPEC §4.7](/SPEC.md#47-proof)) and an `issuedAt`.
2. State `payload.entry.act` explicitly. It **MUST NOT** rely on any member's absence to confer authority, and **SHOULD** state `approve: {"scope": "none"}` rather than omit `approve` where it means none.
3. State `keys` and `capabilities` explicitly, and state `approveCapabilities` wherever the subject is meant to approve anything — its absence means none.

A conforming **consumer** (the ACL maintainer) **MUST**, in addition to validating the document and verifying the `proof`:

1. Refuse an unrecognised role with `roleNotRecognized`. It **MUST NOT** fall back to a default role.
2. Refuse an `act` or `approve` scope it cannot hold with `invalidActScope`. An entry with no `act`, `keys` or `capabilities`, or with an empty `contexts`, `keys` or `grants` list, is refused at validation (`malformedRequest`) and is never read as unrestricted or as none.
3. Refuse an unrecognised capability with `unknownCapability`; refuse a non-additive capability outside the role's ceiling with `capabilityOutsideCeiling`; refuse an `additive` capability the ceiling already includes with `additiveWithinCeiling`. None of these is resolved by accepting the grant and dropping the capability.
4. Evaluate the entry against the **granter's stored entry** — never against a token or credential summarising it — and refuse the grant of a granter with no live entry with `permissionDenied`. The resulting entry **MUST NOT** exceed the granter on any axis ([CONVENTIONS §9](../../_shared/0.2/CONVENTIONS.md)): an additive capability without unrestricted granter act authority is `additiveRequiresUnrestricted`; an approve scope or approvable set wider than the granter's own is `approveWiderThanGranter`; act scope, capabilities and qualifiers, keys or expiry beyond the granter's is `delegationExceedsGranter`; a role above the granter's is `permissionDenied`.
5. Refuse a grant whose subject is the granter itself with `permissionDenied`.
6. Where the subject already exists with a different role, refuse with `permissionDenied` and `details.reason` indicating that role changes use [`acl/change-role`](../../change-role/0.2/spec.md). Where it exists with the same role, the grant is an idempotent re-assertion or, if it widens, is handled as [`acl/update/0.2`](../../update/0.2/spec.md) would handle it; a grant **MUST NOT** narrow an existing entry.
7. Populate `createdAt`, `createdBy` and `delegatedBy` itself, ignoring any producer-supplied values, and return the realized entry.
8. Re-evaluate the entry whenever the `delegatedBy` granter's own entry narrows or is removed, so that the entry does not retain authority its granter has lost (the maintainer's policy decides whether that withdraws, narrows or queues for review — it **MUST NOT** silently leave the entry in force).
9. On acceptance, persist the document as the evidentiary record of the grant.

## Authorization

**Authority: the granting authority's own stored entry, as a ceiling.** A grant is a delegation — the granter hands the subject part of what the granter itself holds — so what entitles the producer to the outcome is that every element of the requested entry is contained in the granter's live entry: its role is not above the granter's, its act scope is within the granter's, each capability is held by the granter at a qualifier at least as wide, its approve scope and approvable capabilities are within the granter's own, and its keys and expiry are no wider. An additive capability additionally requires the granter to hold unrestricted act authority. Conformance items 4 and 5 enforce this.

The maintainer's own role model then bounds the result a second time: the requested entry must fit inside its role's ceiling (item 3). The two bounds are independent — a granter may hold a capability the subject's role does not admit, and a role may admit a capability the granter does not hold — and the grant is accepted only where both admit it.

The `proof` establishes *who* asked and that the entry is *unaltered* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is what lets the maintainer look up the granter's stored entry and attribute the delegation in `delegatedBy`; it is not itself the authorization. Maintainers commonly require more than a single authorized granter for some grants — granting an authority-conferring capability, or approve authority — and may route such a grant through an approval before it executes. That is the maintainer's policy; this specification describes what the grant assumes and does not require any such gate.

## Definitions

* **Granting authority.** The party invoking the grant; identified by `issuer`.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party being granted access; `payload.entry.subject`.
* **Role.** An opaque string naming a **ceiling** — the most an entry with that role may hold. It never grants anything itself.
* **Act scope** (`act`). Where the subject may make a change: `all`, `none`, or a non-empty list of contexts (and their descendants).
* **Approve scope** (`approve`). Where the subject may ratify a change made by someone else; same three shapes, independent of `act`; absent means none.
* **Capability.** One administrative power in the maintainer's registry, optionally **qualified** by a resource. `capabilities` states `ceiling` (the role's full ceiling), `none`, or a listed set that narrows the ceiling. An **additive** capability is one no role implies, held beside the role.
* **Approvable capabilities** (`approveCapabilities`). The capabilities the subject may approve an action needing, within its approve scope — `ceiling`, `none` or listed; absent means none.
* **Keys** (`keys`). The keys the subject may invoke a signing oracle on, within its act scope — `all`, `none` or listed.
* **Delegated by** (`delegatedBy`). The granter whose authority bounds the entry, recorded by the maintainer.
* **Least-privilege approver.** An entry with `act: {scope: none}` and an approve scope other than none.

## Request

The granting authority sends the ACL maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A context administrator at a key-holding maintainer

A super-administrator of a VTA-style maintainer makes Alice an administrator of one context. `capabilities` is `ceiling`, so Alice holds the full ceiling of the `admin` role — within `ctx/payments` only — and may use every key that context reaches. She approves nothing.

```json
{
  "id": "urn:uuid:4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/grant/0.2#request",
  "issuer": "did:web:root-admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-10-02T10:00:00Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "act": { "scope": "contexts", "contexts": ["ctx/payments"] },
      "keys": { "scope": "all" },
      "capabilities": { "scope": "ceiling" },
      "approve": { "scope": "none" },
      "label": "Alice — payments context admin"
    },
    "reason": "Owns the payments integration."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:root-admin.example#key-1",
    "created": "2026-10-02T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kgExampleProofValue"
  }
}
```

### A qualified, narrowed role at a community maintainer

A community administrator of a VTC-style maintainer (which has no contexts, so states `act` as `all` or `none`) makes Bob a repository manager for one git namespace. `capabilities` is `listed`, so it narrows the `repo-manager` ceiling to exactly these qualified grants; `keys` is `none` because the community operates no signing oracle for him. Bob may approve the same capability at the same qualifier.

```json
{
  "id": "urn:uuid:8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/grant/0.2#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T10:05:00Z",
  "payload": {
    "entry": {
      "subject": "did:web:bob.example",
      "role": "repo-manager",
      "act": { "scope": "all" },
      "keys": { "scope": "none" },
      "capabilities": {
        "scope": "listed",
        "grants": [{ "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" }]
      },
      "approve": { "scope": "all" },
      "approveCapabilities": {
        "scope": "listed",
        "grants": [{ "capability": "git.repo.manage", "resource": "git-ns:github.com/acme" }]
      },
      "expiresAt": "2027-04-02T00:00:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-02T10:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4rTExampleProofValue"
  }
}
```

### A least-privilege approver

Dana may approve vetter grants and may initiate nothing: `act` is `none`, `capabilities` and `keys` are `none`, and the approve axis names what she may ratify.

```json
{
  "id": "urn:uuid:c1d2e3f4-5a6b-4c7d-8e9f-0a1b2c3d4e5f",
  "type": "https://trusttasks.org/spec/acl/grant/0.2#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T10:10:00Z",
  "payload": {
    "entry": {
      "subject": "did:web:dana.example",
      "role": "approver",
      "act": { "scope": "none" },
      "keys": { "scope": "none" },
      "capabilities": { "scope": "none" },
      "approve": { "scope": "all" },
      "approveCapabilities": {
        "scope": "listed",
        "grants": [{ "capability": "vtc.vetting.manage" }]
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-02T10:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5aBExampleProofValue"
  }
}
```

## Response

The ACL maintainer answers with a document of type `https://trusttasks.org/spec/acl/grant/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `entry` — the canonical AclEntry the maintainer now holds, with `createdAt`, `createdBy` and `delegatedBy` populated. The granting authority **SHOULD** treat it as the authoritative post-state, since the maintainer applies its own clock and may normalize fields.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### Successful grant

Response to the first request example:

```json
{
  "id": "urn:uuid:5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/grant/0.2#response",
  "threadId": "urn:uuid:4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:root-admin.example",
  "issuedAt": "2026-10-02T10:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "act": { "scope": "contexts", "contexts": ["ctx/payments"] },
      "keys": { "scope": "all" },
      "capabilities": { "scope": "ceiling" },
      "approve": { "scope": "none" },
      "label": "Alice — payments context admin",
      "createdAt": "2026-10-02T10:00:01Z",
      "createdBy": "did:web:root-admin.example",
      "delegatedBy": "did:web:root-admin.example"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.example#key-1",
    "created": "2026-10-02T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z6abExampleProofValue"
  }
}
```

### Refused: a capability outside the role's ceiling

Had the second request named `vtc.policy.admin` for a `repo-manager`, the maintainer would refuse the whole grant rather than accept it without the capability:

```json
{
  "id": "urn:uuid:9b02d8c4-3f73-4b02-b4b5-ae72c86f3012",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T10:05:01Z",
  "payload": {
    "code": "acl/grant:capabilityOutsideCeiling",
    "message": "vtc.policy.admin is not within the repo-manager role's ceiling.",
    "retryable": false,
    "details": {
      "role": "repo-manager",
      "capabilities": ["vtc.policy.admin"]
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries one subject identifier and the authority that subject is to hold: a role, act and approve scopes, capabilities with their resource qualifiers, a key filter, an expiry and an optional label and reason. Resource qualifiers and context paths can reveal organizational structure (which repositories, which payment contexts exist), and a role or capability can reveal a person's position in a regulated or sensitive community. A producer **MUST NOT** place in `label`, `reason` or `ext` anything it would not be comfortable signing and having retained as evidence, and **SHOULD NOT** put personal data there at all. The smallest payload that answers the task is a subject, a role and an explicit `act`; everything else narrows or adds authority and is the producer's choice.

The explicit `act` is a security property of the wire, not a convenience. An encoding in which an empty list means "everywhere" for one role and "nowhere" for another cannot be read correctly without both members, so every call site is one omission away from inverting a grant. 0.2 makes the empty list a validation failure. The same reasoning makes `keys` and `capabilities` REQUIRED and explicit, and makes an absent `approve` or `approveCapabilities` mean none: in 0.1, a serializer that omitted an empty `allowedKeys` turned "no keys" into "every key"; in 0.2 no omission can widen a grant.

### Correlation

The ACL maintainer learns the subject, the granter and the authority conferred, and with `delegatedBy` it can reconstruct the chain of delegation behind every entry — which is the purpose of the member. An intermediary on a transport without confidentiality can observe the same, and can join the subject across tasks on the subject VID. A producer can reduce correlation by granting to a pairwise subject identifier where the maintainer permits; the granter's identifier is unavoidable, since the grant is bounded by the granter's authority. `threadId` joins a request to its response and nothing else.

### Retention

The accepted document is **durable** evidence: it is the record of who delegated what to whom, and the maintainer re-reads the granter relationship whenever the granter's own authority changes. A maintainer **SHOULD** retain it for at least the life of the entry and of any audit obligation over it. The refused document is retained as an audit record of the attempt, at the maintainer's discretion. The duplicate-execution record of [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) is bounded by the acceptance window `issuedAt` supplies; a window of minutes is typical for an interactive grant.

### Consent/purpose

The data is collected to record and enforce an access-control decision and to attribute it to the authority that made it. It **SHOULD NOT** be reused for profiling the subject beyond that purpose. Whether a particular grant must be approved by additional parties, or must be preceded by a step-up, is the maintainer's policy and is not declared here.
