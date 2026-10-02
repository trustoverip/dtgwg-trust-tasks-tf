---
slug: acl/change-role
version: "0.2"
title: ACL — Change Role
summary: An authorized party moves a subject's AclEntry 0.2 from one role to another, with a compare-and-swap on the current role and the entry re-checked against the new role's ceilings.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - acl
  - access-control
  - authorization
  - role
  - promote
  - demote
parties:
  - role: Changing authority
    requirement: REQUIRED
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Role changes are the highest-impact ACL operation — a promotion raises an entry's ceiling, a demotion withdraws authority. A non-repudiable, transport-independent record is necessary for audit, dispute resolution, and parties that retained the prior grant.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A role change overwrites the entry rather than incrementing it, so a stale copy applied out of order silently reinstates a role an operator has already moved the subject off. The issue time lets the maintainer order two changes to the same entry and refuse the older one.
sideEffects:
  level: mutating
  rationale: "Reassigns a subject's role; recoverable by changing it back."
consequences:
  - Every axis the entry states as `ceiling` follows the new role's ceiling — a promotion widens it, a demotion narrows it.
  - A demotion is a privilege reduction effective at the subject's next authorization decision.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The request names one subject and two roles; the response returns the subject's resulting entry to the changing authority.
retention:
  class: durable
  rationale: The chain of role changes for a subject is how an auditor reconstructs how privilege was acquired and withdrawn.
subjectPath: /subject
errorCodes:
  - code: acl/change-role:roleNotRecognized
    meaning: The fromRole or toRole string is not part of the ACL maintainer's role vocabulary.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        offendingRole: { type: string }
        knownRoles:
          type: array
          items: { type: string }
  - code: acl/change-role:stateMismatch
    meaning: The subject's current role does not match payload.fromRole; the change was based on stale state.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        currentRole: { type: string }
  - code: acl/change-role:capabilityOutsideCeiling
    meaning: The entry lists a non-additive capability (to hold or to approve) that the new role's ceiling does not include. Narrow the entry with acl/update/0.2 first; the role change is refused rather than dropping the capability.
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
  - code: acl/change-role:additiveWithinCeiling
    meaning: The entry holds an additive capability that the new role's ceiling includes, so it would no longer be additive. Re-grant it as an ordinary capability with acl/update/0.2 first.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [capabilities]
      properties:
        capabilities:
          type: array
          items: { type: string }
  - code: acl/change-role:delegationExceedsGranter
    meaning: Under the new role the entry would hold authority the changing authority does not hold itself — typically because an axis stated as `ceiling` now reaches capabilities the changer lacks.
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
            enum: [capabilities, approveCapabilities]
related:
  - acl/grant
  - acl/update
  - acl/revoke
---

## Abstract

The **ACL — Change Role** Trust Task moves a subject from one role to another. It is the only task that changes a role: [`acl/grant/0.2`](../../grant/0.2/spec.md) refuses a role change on an existing subject, and [`acl/update/0.2`](../../update/0.2/spec.md) cannot express one. The producer states the role it believes is current (`fromRole`) and the target (`toRole`); a mismatch is `stateMismatch`, so a race against another administrator surfaces as an error rather than a silent overwrite.

Version 0.2 returns an [AclEntry 0.2](../../_shared/0.2/acl-entry.schema.json). Because a role in 0.2 is a **ceiling**, a role change changes what the entry's explicit scopes resolve to: an axis stated as `ceiling` follows the new role, and a listed capability must still fit inside it. The task changes only `role`; every other member is carried over and re-checked.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)). [`acl/change-role/0.1`](../0.1/spec.md) remains current for maintainers holding AclEntry 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Respond with `stateMismatch` where the stored role is not `fromRole`, and `roleNotRecognized` where either role is unknown.
2. Re-check the entry against `toRole`: a listed non-additive capability, or listed approvable capability, outside the new ceilings is `capabilityOutsideCeiling`; an additive capability the new ceiling includes is `additiveWithinCeiling`. It **MUST NOT** accept the change and drop the capability.
3. Evaluate the resulting entry against the changing authority's stored entry: `toRole` **MUST NOT** be above the changer's own role (`permissionDenied`), and the resulting effective capabilities and approvable capabilities **MUST** be within the changer's (`delegationExceedsGranter`).
4. Refuse a change whose subject is the changing authority itself with `permissionDenied`.
5. Apply the change at the subject's next authorization decision — a demotion **MUST NOT** wait for a session or token to expire — and re-evaluate the entries whose `delegatedBy` is the subject.
6. On acceptance, set `updatedAt` / `updatedBy`, persist the document as the evidentiary record, and return the resulting entry.

## Authorization

**Authority: standing to modify the subject's entry, bounded by the changer's own role and capabilities.** A role change is a re-grant of the entry under a different ceiling, so what entitles the changing authority is the same as for [`acl/grant/0.2`](../../grant/0.2/spec.md): its own stored entry must cover the entry (every context the entry may act or approve in), its role must be at least `toRole`, and it must itself hold what the entry will hold under `toRole`. Items 3 and 4 enforce this. `stateMismatch` is a concurrency failure, not a permission one.

A promotion into a role whose ceiling includes an **authority-conferring capability**, or unrestricted authority, is consent-gated at some maintainers (the VTI specification's VTI-APV-014 / VTI-APV-018). A demotion that narrows or removes **another subject's** unrestricted act scope or authority-conferring capability is, at some maintainers, held for the consent of a party other than both the requester and the subject — and where none exists, accompanied by the requester's re-authentication, a notice to the subject and a highest-severity audit row (VTI-APV-019). These describe policy this task is commonly processed under; this specification does not require any of them.

The `proof` identifies the changing authority so its stored entry can be read and the change attributed; it is not the authorization.

## Definitions

* **Changing authority.** The party invoking the change; `issuer`.
* **ACL maintainer.** The party that holds and enforces the list; `recipient`.
* **Subject.** The party whose role changes; `payload.subject`.
* **Promotion / demotion.** A change to a role whose ceiling is wider / narrower than the current one's. Roles need not be totally ordered; "above" is the maintainer's ordering, and where two roles are incomparable a maintainer evaluates item 3 on the capabilities alone.

## Request

The changing authority sends the ACL maintainer a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Promote a member to moderator

```json
{
  "id": "urn:uuid:1b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/change-role/0.2#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T19:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "credential-officer",
    "toRole": "moderator",
    "reason": "Taking over membership operations."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-02T19:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5xyExampleProofValue"
  }
}
```

Bob's entry states `capabilities: {"scope": "ceiling"}`, so it needs no edit: it resolves to the moderator ceiling once the change lands. Had it listed `vtc.credentials.issue`, the change would be refused with `capabilityOutsideCeiling` until the list was replaced through `acl/update/0.2`.

## Response

The ACL maintainer answers with a document of type `https://trusttasks.org/spec/acl/change-role/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `entry` — the resulting AclEntry; `entry.role` equals the request's `toRole`.

Failures (including `stateMismatch`) use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### Successful promotion

```json
{
  "id": "urn:uuid:2c3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/change-role/0.2#response",
  "threadId": "urn:uuid:1b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T19:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:bob.example",
      "role": "moderator",
      "act": { "scope": "all" },
      "keys": { "scope": "none" },
      "capabilities": { "scope": "ceiling" },
      "approve": { "scope": "all" },
      "approveCapabilities": { "scope": "ceiling" },
      "createdAt": "2026-06-01T00:00:00Z",
      "createdBy": "did:web:carol.example",
      "delegatedBy": "did:web:carol.example",
      "updatedAt": "2026-10-02T19:00:01Z",
      "updatedBy": "did:web:carol.example"
    }
  }
}
```

### Stale-state mismatch

```json
{
  "id": "urn:uuid:9c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:3c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T19:05:01Z",
  "payload": {
    "code": "acl/change-role:stateMismatch",
    "message": "Subject's current role is 'moderator', not 'credential-officer'.",
    "retryable": true,
    "details": { "currentRole": "moderator" }
  }
}
```

The changing authority re-reads the entry and retries from the new current role.

## Security & Privacy

### Data carried

The request carries one subject and two role identifiers; the response carries the resulting entry. Role names can carry sensitive meaning (membership of a regulated function); producers **SHOULD** apply transport confidentiality appropriate to the privacy regime and **MUST NOT** put personal data in `reason` or `ext`.

The amplification to keep in mind: in 0.2 a role change moves every axis an entry states as `ceiling`. A promotion therefore widens capabilities without any capability being named in the request — which is why item 3 bounds the resulting entry, not the request.

### Correlation

The maintainer learns who moved whom between which roles and when. The chain of changes for one subject is correlatable by design. `threadId` joins request and response.

### Retention

The accepted document is **durable**: maintainers **SHOULD** keep the full chain of role changes for a subject so the audit trail shows how privilege was acquired and withdrawn. The duplicate-execution record is bounded by the acceptance window `issuedAt` supplies, and a narrow window is cheap protection against replayed role changes.

### Consent/purpose

The data is collected to change and record a subject's role. It **SHOULD NOT** be reused for other purposes. Whether a given promotion or demotion requires a third party's consent or a step-up is the maintainer's policy and is not declared here.
