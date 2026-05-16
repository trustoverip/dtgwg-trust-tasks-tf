---
slug: acl/revoke
version: "1.0"
title: ACL — Revoke
summary: A revoking party records, in a verifiable form, that a subject has been removed from an access-control list, or that some of the subject's scopes have been withdrawn.
status: draft
targetFrameworkVersion: "0.1"
category: permission
keywords:
  - acl
  - access-control
  - authorization
  - revoke
  - remove
  - leave
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Revoking party
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: A revocation is the evidentiary counterpart to a grant; the maintainer, the former subject, and any downstream party that retained the grant document need to be able to verify, after the fact, that the revocation was authorized.
errorCodes:
  - code: acl/revoke:subject_not_present
    meaning: The subject named in the payload is not currently in the ACL.
    retryable: false
  - code: acl/revoke:last_authority_protected
    meaning: The revocation would leave the ACL with no party able to perform a privileged operation; the maintainer's policy forbids it.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        protectedRole: { type: string }
        remainingHolders:
          type: array
          items: { type: string }
related:
  - acl/grant
  - acl/change-role
---

## Abstract

The **ACL — Revoke** Trust Task records the removal of a subject from an access-control list, or the withdrawal of some of the subject's scopes.

Three patterns share this task:

1. **Full removal.** `payload.scopes` is omitted; the entry is removed entirely.
2. **Scope reduction.** `payload.scopes` lists the scopes to remove; the entry remains with the remaining scopes.
3. **Self-removal.** `issuer == payload.subject`. The maintainer's policy decides whether self-revoke is permitted for the subject's current role (for example, the last admin may be protected).

The maintainer constructs the canonical resulting state and returns it in its response — the producer never needs to compute or transmit a `before`/`after` pair.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the revoking party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/revoke/1.0`, with itself as `issuer` and the ACL maintainer as `recipient`.
2. Populate `payload.subject` with the VID of the subject being revoked, and optionally `payload.scopes` to scope-reduce rather than fully remove.
3. Include a `proof` member per [SPEC.md §4.7](../../../../SPEC.md#47-proof).

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Confirm the revoking party is either the subject themselves (self-revoke) or a party authorized to remove the subject. If neither, respond with the framework's `permission_denied` (see [SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)).
3. If the subject is not in the ACL, respond with `acl/revoke:subject_not_present`.
4. Reject any revocation that would leave the ACL with no holder of a privileged role required by the maintainer's policy, returning `acl/revoke:last_authority_protected`.
5. On acceptance, persist the document as the evidentiary record of the change.

## Definitions

* **Revoking party.** The party invoking the revocation; identified by `issuer`. May be an authorized administrator or the subject themselves.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party being removed (or partially de-scoped); identified by `payload.subject`.
* **Self-revocation.** A revocation where `issuer == payload.subject`. Consumers **MUST** recognize this case explicitly and apply the maintainer's self-revoke policy.

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/revoke/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Full removal by an administrator

```json
{
  "id": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/revoke/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-20T11:00:00Z",
  "payload": {
    "subject": "did:web:contractor.example",
    "reason": "Engagement completed."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:org.example#key-1",
    "created": "2026-05-20T11:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4ab..."
  }
}
```

`scopes` is absent, so the maintainer removes the entry entirely.

### Scope reduction

```json
{
  "id": "7a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/revoke/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-21T09:30:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "scopes": ["context:project-beta"],
    "reason": "Project-beta access withdrawn; project-alpha access retained."
  }
}
```

The maintainer removes `context:project-beta` from Alice's scopes; her entry remains in the ACL.

### Self-revocation

```json
{
  "id": "f0b2c5a1-8d3e-4c4a-92b1-1e8d4cbe7104",
  "type": "https://trusttasks.org/spec/acl/revoke/1.0",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-06-01T08:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "reason": "Resigning from the organization."
  }
}
```

`issuer == payload.subject`; the maintainer applies its self-revoke policy.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/revoke/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ entry: AclEntry | null }`:

* `null` when the revocation was a full removal — the subject is gone from the ACL.
* The resulting *AclEntry* when the revocation was a scope reduction — the subject remains with fewer scopes.

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful full removal

Response to the first request example:

```json
{
  "id": "ae2a1c44-7b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/revoke/1.0#response",
  "threadId": "9e2a1c44-7b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-05-20T11:00:01Z",
  "payload": {
    "entry": null
  }
}
```

### Successful scope reduction

Response to the scope-reduction example:

```json
{
  "id": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f02",
  "type": "https://trusttasks.org/spec/acl/revoke/1.0#response",
  "threadId": "7a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-05-21T09:30:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "createdAt": "2026-04-01T00:00:00Z",
      "createdBy": "did:web:org.example",
      "updatedAt": "2026-05-21T09:30:01Z",
      "updatedBy": "did:web:org.example"
    }
  }
}
```

## Security & Privacy

A captured `acl/revoke` document proves that a subject's access ended at a particular moment. The **REQUIRED** `proof` ensures the revocation is non-repudiable and tamper-evident.

Maintainers **SHOULD** preserve revocation records alongside the original grants they cancel. Where retention is bounded by privacy regulation, maintainers **SHOULD** retain at least `id`, `threadId`, `issuer`, `issuedAt`, and `payload.subject` so the audit trail remains intact even if other fields are trimmed.

Where the subject is a natural person, the response payload's `entry` (on scope reduction) carries sensitive identifying information; producers **SHOULD** apply transport confidentiality appropriate to the privacy regime.
