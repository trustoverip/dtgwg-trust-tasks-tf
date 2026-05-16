---
slug: acl/change-role
version: "1.0"
title: ACL — Change Role
summary: An authorized party records the transition of a subject's role within an access-control list, capturing both prior and resulting state.
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
  - code: acl/change-role:permission_denied
    meaning: The changing authority is not permitted to assign the requested role under the ACL maintainer's policy.
    retryable: false
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
    meaning: The payload.before.role does not match the subject's current role in the maintainer's ACL; the change was based on stale state.
    retryable: true
  - code: acl/change-role:no_change
    meaning: The payload.before.role equals payload.after.role; the change is a no-op.
    retryable: false
related:
  - acl/grant
  - acl/revoke
---

## Abstract

The **ACL — Change Role** Trust Task records the transition of a subject's role in an access-control list. It is the dedicated operation for role transitions; grants and revocations **MUST** use [`acl/grant`](../../grant/1.0/spec.md) and [`acl/revoke`](../../revoke/1.0/spec.md) respectively, not this task.

The task is **state-checked**: the *changing authority* declares the role the subject is moving *from* and the role they are moving *to*. The *ACL maintainer* **MUST** reject the change if the subject's actual current role does not match `payload.before.role` (returning `acl/change-role:state_mismatch`), so racing role changes against another administrator are detected and surfaced rather than silently overwriting each other.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels).

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the changing authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/change-role/1.0`.
2. Identify itself as `issuer`; identify the ACL maintainer as `recipient`.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the changing authority's key material.
5. Populate `payload.before` with the *AclEntry* as it existed prior to the change and `payload.after` with the resulting *AclEntry*. The two **MUST** differ only in `role`, `updatedAt`, and `updatedBy`. A change that also alters `scopes`, `label`, or `expiresAt` **MUST** be split into separate `acl/change-role` and `acl/grant`/`acl/revoke` documents linked by `threadId`.

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the changing authority's declared verification material.
3. Confirm the subject's current role in its own ACL equals `payload.before.role`. If not, respond with `acl/change-role:state_mismatch`.
4. Apply its own policy to decide whether the changing authority may make the requested transition. Where the policy forbids the transition, respond with `acl/change-role:permission_denied`.
5. On acceptance, persist the document as the evidentiary record of the change.

Maintainers **MAY** require a stronger transport-binding-level authentication for transitions into elevated roles (for example, a passkey step-up flow). Such requirements are documented by the maintainer and enforced by its transport handler; the Trust Task carries the intent, not the step-up dance.

## Definitions

* **Changing authority.** The party invoking the role change; identified by `issuer`. Typically holds an "admin" or equivalent role under the maintainer's policy.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party whose role is changing; identified by `payload.subject`. Self-promotion (`issuer == payload.subject` with `toRole` strictly greater than `fromRole`) **SHOULD** be forbidden by maintainer policy.
* **Transition.** A change from `payload.before.role` to `payload.after.role`. Each maintainer defines which transitions are permitted under its policy (for example, "any admin may promote a member to moderator"; "promotion to admin requires a peer admin's countersignature").

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/change-role/1.0` (or `…/1.0#request`), with a payload that validates against the top-level schema in `payload.schema.json`. The producer is the changing authority; the recipient is the ACL maintainer.

### Promotion from member to moderator

```json
{
  "id": "1b3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/change-role/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-10T14:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "member",
    "toRole": "moderator",
    "reason": "Promoted after six months of community contributions.",
    "before": {
      "subject": "did:web:bob.example",
      "role": "member",
      "scopes": ["context:public"],
      "createdAt": "2026-01-01T00:00:00Z",
      "createdBy": "did:web:org.example"
    },
    "after": {
      "subject": "did:web:bob.example",
      "role": "moderator",
      "scopes": ["context:public"],
      "createdAt": "2026-01-01T00:00:00Z",
      "createdBy": "did:web:org.example",
      "updatedAt": "2026-06-10T14:00:00Z",
      "updatedBy": "did:web:org.example"
    }
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

Note that `payload.after` differs from `payload.before` only in `role`, `updatedAt`, and `updatedBy` — `scopes`, `createdAt`, and `createdBy` are preserved verbatim.

### Stale-state mismatch (rejected by consumer)

A changing authority emits:

```json
{
  "id": "3c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/change-role/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-11T09:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "fromRole": "member",
    "toRole": "admin",
    "before": { "subject": "did:web:bob.example", "role": "member" },
    "after":  { "subject": "did:web:bob.example", "role": "admin" }
  }
}
```

But Bob's current role in the maintainer's ACL is `moderator` (changed by another admin moments earlier). The maintainer responds with:

```json
{
  "id": "9c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/trust-task-error/1.0",
  "threadId": "3c5e2a1b-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-06-11T09:00:01Z",
  "payload": {
    "code": "acl/change-role:state_mismatch",
    "message": "Subject's current role is 'moderator', not 'member'. Re-read state and retry.",
    "retryable": true
  }
}
```

The changing authority refreshes its view of the ACL and retries from the new prior state, rather than silently overwriting another admin's recent change.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/change-role/1.0#response`, with a payload that validates against the sub-schema reachable via `$anchor: "response"` in `payload.schema.json`. The producer is the ACL maintainer; the recipient is the changing authority.

The response payload carries a single member, `entry`, which is the *AclEntry* the maintainer now holds for the subject. The `entry` value **MUST** equal the request's `payload.after`, including the new `role`. The changing authority can verify in one step that the transition landed as requested.

A failure (including `acl/change-role:state_mismatch`) is **not** a `#response` document; failures use `trust-task-error` — see [SPEC.md §8](../../../../SPEC.md#8-error-responses).

### Successful promotion

Response to the first request example above:

```json
{
  "id": "2c3c5e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/change-role/1.0#response",
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

`entry.role` is now `moderator`, confirming the transition.

## Security & Privacy

Role changes are the highest-impact ACL operation. The `proof` requirement (**REQUIRED**) is essential: a forged or replayed role change can directly extend privilege within the ecosystem. Maintainers **SHOULD**:

1. Apply the strictest available transport-binding-level authentication for transitions into elevated roles.
2. Preserve the full chain of `acl/change-role` documents for any given subject, so the audit trail describes how privilege was acquired and withdrawn over time.
3. Refuse `acl/change-role` documents whose `issuedAt` lies outside a narrow freshness window relative to the maintainer's clock — replayed role changes are a known attack vector and the freshness check is cheap.

Where role names themselves carry sensitive meaning (for example, indicating membership in a regulated function), producers **SHOULD** apply transport confidentiality appropriate to the underlying privacy regime.
