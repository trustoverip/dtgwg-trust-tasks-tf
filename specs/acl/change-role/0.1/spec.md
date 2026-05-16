---
slug: acl/change-role
version: "0.1"
title: ACL — Change Role
summary: An authorized party records the transition of a subject's role within an access-control list, with an optimistic concurrency check against the prior role.
status: draft
targetFrameworkVersion: "0.1"
category: permission
keywords:
  - acl
  - access-control
  - authorization
  - role
  - promote
  - demote
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Changing authority
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Role changes are the highest-impact ACL operation — promotions can extend privilege; demotions can withdraw it. A non-repudiable, transport-independent record of the change is necessary for audit, dispute resolution, and downstream parties that retained the prior grant.
errorCodes:
  - code: acl/change-role:role_not_recognized
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
  - code: acl/change-role:state_mismatch
    meaning: The subject's current role does not match payload.fromRole; the change was based on stale state.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        currentRole: { type: string }
related:
  - acl/grant
  - acl/revoke
---

## Abstract

The **ACL — Change Role** Trust Task records the transition of a subject's role in an access-control list. It is the dedicated operation for role transitions; grants and revocations **MUST** use [`acl/grant`](../../grant/0.1/spec.md) and [`acl/revoke`](../../revoke/0.1/spec.md) respectively.

The task is **state-checked**: the producer declares both the role the subject is moving *from* and the role they are moving *to*. The maintainer **MUST** reject the change with `acl/change-role:state_mismatch` if the subject's actual current role does not match `payload.fromRole` — so a race against another administrator surfaces as an error rather than a silent overwrite.

This task changes only the `role`. Scope or label changes are out of scope; combine `acl/change-role` with `acl/grant`/`acl/revoke` under a shared `threadId` if you need both.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the changing authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/change-role/0.1`, with itself as `issuer` and the ACL maintainer as `recipient`.
2. Populate `payload.subject`, `payload.fromRole`, and `payload.toRole`.
3. Include a `proof` member per [SPEC.md §4.7](../../../../SPEC.md#47-proof).

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Confirm the subject's current role in its own ACL equals `payload.fromRole`. If not, respond with `acl/change-role:state_mismatch`.
3. Apply its own policy to decide whether the changing authority may make the requested transition. Where the policy forbids the transition, respond with the framework's `permission_denied` (see [SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)).
4. Where either role string is not recognized, respond with `acl/change-role:role_not_recognized`.
5. On acceptance, persist the document as the evidentiary record of the change.

Maintainers **MAY** require stronger transport-binding-level authentication for transitions into elevated roles (e.g. a passkey step-up). Such requirements are documented by the maintainer and enforced by its transport handler; this task carries intent, not the step-up dance.

## Definitions

* **Changing authority.** The party invoking the role change; identified by `issuer`.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party whose role is changing; identified by `payload.subject`. Self-promotion (`issuer == payload.subject` with `toRole` strictly greater than `fromRole`) **SHOULD** be forbidden by maintainer policy.

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/change-role/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Promotion from member to moderator

```json
{
  "id": "1b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/change-role/0.1",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-10T14:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "member",
    "toRole": "moderator",
    "reason": "Promoted after six months of community contributions."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:org.example#key-1",
    "created": "2026-06-10T14:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5xy..."
  }
}
```

### Stale-state mismatch

A changing authority emits:

```json
{
  "id": "3c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/change-role/0.1",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-11T09:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "member",
    "toRole": "admin"
  }
}
```

But Bob's current role in the maintainer's ACL is `moderator` (changed moments earlier by another admin). The maintainer responds with a `trust-task-error`:

```json
{
  "id": "9c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "3c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-06-11T09:00:01Z",
  "payload": {
    "code": "acl/change-role:state_mismatch",
    "message": "Subject's current role is 'moderator', not 'member'.",
    "retryable": true,
    "details": { "currentRole": "moderator" }
  }
}
```

The changing authority re-reads state and retries from the new prior role.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/change-role/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ entry: AclEntry }`, where `entry.role` equals `payload.toRole` of the request. The changing authority can verify in one step that the transition landed.

Failures (including `acl/change-role:state_mismatch`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful promotion

Response to the first request example:

```json
{
  "id": "2c3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/change-role/0.1#response",
  "threadId": "1b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-06-10T14:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:bob.example",
      "role": "moderator",
      "scopes": ["context:public"],
      "createdAt": "2026-01-01T00:00:00Z",
      "createdBy": "did:web:org.example",
      "updatedAt": "2026-06-10T14:00:01Z",
      "updatedBy": "did:web:org.example"
    }
  }
}
```

## Security & Privacy

Role changes are the highest-impact ACL operation. The **REQUIRED** `proof` is essential: a forged or replayed role change can directly extend privilege.

Maintainers **SHOULD**:

1. Apply the strictest available transport-binding-level authentication for transitions into elevated roles.
2. Preserve the full chain of `acl/change-role` documents for any given subject, so the audit trail describes how privilege was acquired and withdrawn over time.
3. Refuse documents whose `issuedAt` lies outside a narrow freshness window relative to the maintainer's clock — replayed role changes are a known attack vector and the freshness check is cheap.

Where role names themselves carry sensitive meaning, producers **SHOULD** apply transport confidentiality appropriate to the privacy regime.
