---
slug: acl/grant
version: "1.0"
title: ACL — Grant
summary: A granting authority records, in a verifiable form, that a subject has been added to an access-control list with a named role and optional scopes.
status: draft
targetFrameworkVersion: "0.1"
category: permission
keywords:
  - acl
  - access-control
  - authorization
  - role
  - grant
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Granting authority
    requirement: REQUIRED
  - role: ACL maintainer
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: A grant is an evidentiary record that may be replayed by an auditor, used by a downstream service to corroborate authorization decisions, or relied on after the original transport has closed; transport-independent integrity is required.
errorCodes:
  - code: acl/grant:permission_denied
    meaning: The granting authority is not permitted to grant the requested role under the ACL maintainer's policy.
    retryable: false
  - code: acl/grant:role_not_recognized
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
  - code: acl/grant:expiry_in_past
    meaning: The supplied expiresAt lies in the past relative to the maintainer's clock.
    retryable: false
  - code: acl/grant:self_grant_prohibited
    meaning: The granting authority and the subject are the same VID; the maintainer's policy forbids self-grants.
    retryable: false
related:
  - acl/revoke
  - acl/change-role
  - acl/list
---

## Abstract

The **ACL — Grant** Trust Task records the addition of a subject to an access-control list. The *granting authority* asserts to the *ACL maintainer* that the *subject* identified in the payload is now entitled to the named `role`, optionally constrained to `scopes` and bounded by `expiresAt`. The maintainer applies its own policy to decide whether to accept the grant; if accepted, the document itself is the evidentiary record of the change.

The task is **idempotent** on `(subject, role, scopes)`: re-emitting an identical grant against an unchanged ACL produces no state change, and the `before` and `after` payload members will be equal. A grant that changes the subject's *role* **MUST NOT** use this task; use [`acl/change-role`](../../change-role/1.0/spec.md) instead. A grant that *narrows* the subject's scopes is a revocation; use [`acl/revoke`](../../revoke/1.0/spec.md).

This specification deliberately leaves the `role` vocabulary and the `scopes` semantics opaque. Each ACL maintainer publishes its own role list and scope conventions; the Trust Task carries the strings but does not interpret them.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels). The schema **MAY** change without notice while the cross-ecosystem ACL pattern stabilizes.

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the granting authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/grant/1.0`.
2. Identify itself as `issuer`; identify the ACL maintainer as `recipient`. Per [SPEC.md §4.8.1](../../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the granting authority's key material.
5. Populate `payload.after` with the resulting *AclEntry* and `payload.before` with the prior entry (or `null` if the subject was not previously in the ACL).

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the granting authority's declared verification material.
3. Apply its own policy to decide whether to accept the grant. Where the role string is not recognized, respond with `acl/grant:role_not_recognized`. Where the granting authority is not permitted to assign the requested role, respond with `acl/grant:permission_denied`.
4. On acceptance, persist the document (or a reference to it) as the evidentiary record of the change. On rejection, **SHOULD** return an *error response* per [SPEC.md §8](../../../../SPEC.md#8-error-responses).

The maintainer **MUST NOT** apply this task to change a subject's existing role; receipt of an `acl/grant` whose `payload.before.role` differs from `payload.after.role` **MUST** be rejected with `acl/grant:permission_denied` and a `details.reason` indicating that role changes use [`acl/change-role`](../../change-role/1.0/spec.md).

## Definitions

* **Granting authority.** The party invoking the grant; identified by `issuer`. Typically holds an "admin" or equivalent role under the maintainer's policy.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party being granted access; identified by `payload.subject`. Need not be a party in the framework sense, since the subject does not participate in the protocol exchange.
* **AclEntry.** The canonical record of one subject's membership in the ACL. Carried in `payload.before` (prior state) and `payload.after` (resulting state).
* **Role.** A short string interpreted by the ACL maintainer. The framework does not constrain the vocabulary; common examples include `admin`, `member`, `viewer`. Each maintainer publishes its role list as part of its ecosystem governance.
* **Scopes.** An array of opaque strings restricting where the role applies (for example, contexts, domains, resource prefixes). Their interpretation is defined by the maintainer.

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/grant/1.0` (or the explicit form `…/1.0#request`), with a payload that validates against the top-level schema in `payload.schema.json`. The producer is the granting authority; the recipient is the ACL maintainer.

### A new admin is added

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/grant/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-16T10:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "role": "admin",
    "label": "Alice — primary admin",
    "before": null,
    "after": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "label": "Alice — primary admin",
      "createdAt": "2026-05-16T10:00:00Z",
      "createdBy": "did:web:org.example"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:org.example#key-1",
    "created": "2026-05-16T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

`before` is `null` because Alice was not previously in the ACL.

### Adding a scoped, expiring member

```json
{
  "id": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/grant/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-16T10:05:00Z",
  "payload": {
    "subject": "did:web:contractor.example",
    "role": "member",
    "scopes": ["context:project-alpha"],
    "expiresAt": "2026-08-16T00:00:00Z",
    "before": null,
    "after": {
      "subject": "did:web:contractor.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "createdAt": "2026-05-16T10:05:00Z",
      "createdBy": "did:web:org.example",
      "expiresAt": "2026-08-16T00:00:00Z"
    }
  }
}
```

`proof` is omitted in this example because the maintainer is reached over a mutually-authenticated TLS channel that conveys the issuer's identity end-to-end; per [SPEC.md §4.7.1](../../../../SPEC.md#471-when-to-include-a-proof) the document **MAY** carry the proof in-band but is not required to. A maintainer that retains the document for audit **SHOULD** require a proof.

### Idempotent re-grant

```json
{
  "id": "c4d2f713-9a8e-4d04-b29c-2f1b0b4cbe71",
  "type": "https://trusttasks.org/spec/acl/grant/1.0",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-17T09:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "role": "admin",
    "before": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "createdAt": "2026-05-16T10:00:00Z",
      "createdBy": "did:web:org.example"
    },
    "after": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "createdAt": "2026-05-16T10:00:00Z",
      "createdBy": "did:web:org.example"
    }
  }
}
```

`before` equals `after`; the maintainer recognizes this as a no-op and acknowledges without mutating state.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/grant/1.0#response`, with a payload that validates against the sub-schema reachable via `$anchor: "response"` in `payload.schema.json`. The producer is the ACL maintainer; the recipient is the granting authority.

The response payload carries a single member, `entry`, which is the *AclEntry* the maintainer now holds for the subject. The `entry` value **MUST** equal the request's `payload.after`. This lets the granting authority verify, in one step, that the maintainer accepted the grant verbatim.

A failure is **not** an `#response` document; failures use the framework's `trust-task-error` type — see [SPEC.md §8](../../../../SPEC.md#8-error-responses) and this spec's [Error codes](#error-codes).

### Successful grant of a new admin

Response to the first request example above:

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/grant/1.0#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-05-16T10:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "label": "Alice — primary admin",
      "createdAt": "2026-05-16T10:00:00Z",
      "createdBy": "did:web:org.example"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:maintainer.example#key-1",
    "created": "2026-05-16T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6ab..."
  }
}
```

`threadId` carries the originating request's `id` so the granting authority can pair the response with its request.

### Successful grant of a scoped, expiring member

Response to the second request example:

```json
{
  "id": "9b91c7b3-2e62-4a91-a3a4-9d61b75e2f02",
  "type": "https://trusttasks.org/spec/acl/grant/1.0#response",
  "threadId": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-05-16T10:05:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:contractor.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "createdAt": "2026-05-16T10:05:01Z",
      "createdBy": "did:web:org.example",
      "expiresAt": "2026-08-16T00:00:00Z"
    }
  }
}
```

The maintainer's `createdAt` may differ from the request's `payload.after.createdAt` because the maintainer applies its own clock; the granting authority **SHOULD** accept the maintainer's value.

## Security & Privacy

A grant document is **evidence**: a captured `acl/grant` Trust Task is sufficient to prove, after the fact, who authorized whom with what role. The `proof` requirement (**REQUIRED**) ensures the granting authority cannot repudiate the grant and that no intermediary can alter the subject, role, scopes, or expiry without invalidating the proof.

The `payload.subject` member identifies the entity being granted access. Where the subject is a natural person and the maintainer's role vocabulary is sensitive (for example, roles that signal membership in a regulated community), producers **SHOULD** apply transport confidentiality appropriate to the underlying privacy regime.

The optional `metadata` extension is **not** signed separately from the rest of the payload; producers **MUST NOT** place data in `metadata` that they would not be comfortable signing.
