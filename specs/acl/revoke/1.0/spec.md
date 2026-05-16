---
slug: acl/revoke
version: "1.0"
title: ACL — Revoke
summary: A revoking party records, in a verifiable form, that a subject has been removed from an access-control list, or that some of the subject's scopes have been withdrawn.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - acl
  - access-control
  - authorization
  - revoke
  - remove
  - leave
authors:
  - DTGWG Governance TF
parties:
  - role: Revoking party
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: A revocation is the evidentiary counterpart to a grant; the maintainer, the former subject, and any downstream party that retained the grant document need to be able to verify, after the fact, that the revocation was authorized.
errorCodes:
  - code: acl/revoke:permission_denied
    meaning: The revoking party is neither the subject (self-revocation) nor a party permitted to remove the subject under the maintainer's policy.
    retryable: false
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

The **ACL — Revoke** Trust Task records the removal of a subject from an access-control list, or the withdrawal of some of the subject's scopes. The *revoking party* asserts to the *ACL maintainer* that the named *subject* is no longer entitled to the access that the prior state described.

Three patterns share this task:

1. **Full removal.** `payload.scopes` is omitted; the entry is removed entirely; `payload.after` is `null`.
2. **Scope reduction.** `payload.scopes` lists the scopes to remove; the entry remains with the remaining scopes; `payload.after` is the resulting *AclEntry*.
3. **Self-removal.** `issuer == payload.subject`. The transport binding maps this to a self-revoke endpoint where applicable. Maintainer policy decides whether self-revoke is permitted for the subject's current role (for example, the last admin may be protected).

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels).

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the revoking party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/revoke/1.0`.
2. Identify itself as `issuer`; identify the ACL maintainer as `recipient`.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the revoking party's key material.
5. Populate `payload.before` with the *AclEntry* as it existed prior to this revocation, and `payload.after` with the resulting *AclEntry* (or `null` for full removal).

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the revoking party's declared verification material.
3. Apply its own policy: confirm that the revoking party is either the subject themselves (self-revoke) or a party authorized to remove the subject. If neither, respond with `acl/revoke:permission_denied`.
4. Reject any revocation that would leave the ACL with no holder of a privileged role required by the maintainer's policy, returning `acl/revoke:last_authority_protected`.
5. On acceptance, persist the document as the evidentiary record of the change.

## Definitions

* **Revoking party.** The party invoking the revocation; identified by `issuer`. May be a maintainer-authorized administrator or the subject themselves (self-revocation).
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party being removed (or partially de-scoped); identified by `payload.subject`.
* **Self-revocation.** A revocation where `issuer == payload.subject`. Consumers **MUST** recognize this case explicitly and apply the maintainer's self-revoke policy (which may protect last-authority roles).

## Examples

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
    "reason": "Engagement completed.",
    "before": {
      "subject": "did:web:contractor.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "createdAt": "2026-05-16T10:05:00Z",
      "createdBy": "did:web:org.example",
      "expiresAt": "2026-08-16T00:00:00Z"
    },
    "after": null
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
    "reason": "Project-beta access withdrawn; project-alpha access retained.",
    "before": {
      "subject": "did:web:alice.example",
      "role": "member",
      "scopes": ["context:project-alpha", "context:project-beta"],
      "createdAt": "2026-04-01T00:00:00Z",
      "createdBy": "did:web:org.example"
    },
    "after": {
      "subject": "did:web:alice.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "createdAt": "2026-04-01T00:00:00Z",
      "createdBy": "did:web:org.example",
      "updatedAt": "2026-05-21T09:30:00Z",
      "updatedBy": "did:web:org.example"
    }
  }
}
```

The entry remains in the ACL; only `context:project-beta` was removed from the subject's scope set.

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
    "reason": "Resigning from the organization.",
    "before": {
      "subject": "did:web:alice.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "createdAt": "2026-04-01T00:00:00Z",
      "createdBy": "did:web:org.example"
    },
    "after": null
  }
}
```

`issuer == payload.subject`. The maintainer recognizes this as a self-revocation; because Alice's role is `member` (not the last `admin`), the maintainer applies its self-revoke policy and removes the entry.

## Security & Privacy

Revocation is as security-critical as authorization: a captured `acl/revoke` document is the proof, after the fact, that a subject's access ended at a particular moment. The `proof` requirement (**REQUIRED**) ensures the revocation is non-repudiable and tamper-evident.

Maintainers **SHOULD** preserve revocation records alongside the original grants they cancel; together they describe the full lifecycle of an ACL entry. Where retention is bounded by privacy regulation, maintainers **SHOULD** retain at least the `id`, `threadId`, `issuer`, `issuedAt`, and `payload.subject` fields, so the audit trail of grants and revocations remains intact even if `before`/`after` payloads are trimmed.

Where the subject is a natural person, `payload.before` will typically carry their `role`, `scopes`, and possibly a `label` — all of which **MAY** be sensitive. Producers **SHOULD** apply transport confidentiality appropriate to the privacy regime in force.
