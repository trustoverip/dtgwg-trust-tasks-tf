---
slug: acl/grant
version: "0.1"
title: ACL — Grant
summary: A granting authority records, in a verifiable form, that a subject has been added to an access-control list with a named role and optional scopes.
status: draft
targetFrameworkVersion: "0.1"
category: access-control
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
    member: issuer
  - role: ACL maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A grant is an evidentiary record that may be replayed by an auditor, used by a downstream service to corroborate authorization decisions, or relied on after the original transport has closed; transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Adds a subject to the ACL with a role; recoverable via acl/revoke."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
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
related:
  - acl/revoke
  - acl/change-role
  - acl/list
---

## Abstract

The **ACL — Grant** Trust Task records the addition of a subject to an access-control list. The *granting authority* declares to the *ACL maintainer* the *AclEntry* the maintainer should hold for the subject after the grant. The maintainer applies its own policy to decide whether to accept the grant; if accepted, the document is the evidentiary record of the change.

The task is **idempotent**: re-emitting an identical grant against an unchanged ACL produces no state change. A grant that changes a subject's *role* **MUST NOT** use this task; use [`acl/change-role`](../../change-role/0.1/spec.md) instead. A grant that *narrows* the subject's scopes is a revocation; use [`acl/revoke`](../../revoke/0.1/spec.md).

The `role` vocabulary and the `scopes` semantics are opaque to the framework — each ACL maintainer defines its own.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the granting authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/acl/grant/0.1`, with itself as `issuer` and the ACL maintainer as `recipient`.
2. Populate `payload.entry` with the *AclEntry* the maintainer should hold for the subject.
3. Include a `proof` member per [SPEC.md §4.7](../../../../SPEC.md#47-proof).

A conforming **consumer** (the ACL maintainer) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the role string is not recognized, respond with `acl/grant:role_not_recognized`. Where the granting authority is not permitted to assign the requested role, respond with the framework's `permission_denied` (see [SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)).
3. Where the subject already exists in the ACL with a different role, respond with `permission_denied` and `details.reason` indicating that role changes use [`acl/change-role`](../../change-role/0.1/spec.md).
4. On acceptance, persist the document as the evidentiary record of the change.

## Definitions

* **Granting authority.** The party invoking the grant; identified by `issuer`. Typically holds an "admin" or equivalent role.
* **ACL maintainer.** The party that holds and enforces the access-control list; identified by `recipient`.
* **Subject.** The party being granted access; identified by `payload.entry.subject`.
* **AclEntry.** The canonical record of one subject's membership in the ACL.
* **Role.** A short opaque string interpreted by the ACL maintainer (e.g. `admin`, `member`, `viewer`).
* **Scopes.** An array of opaque strings restricting where the role applies (e.g. contexts, domains, resource prefixes).

## Request

A *request* document carries `type: https://trusttasks.org/spec/acl/grant/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### A new admin is added

```json
{
  "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "type": "https://trusttasks.org/spec/acl/grant/0.1",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-16T10:00:00Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "label": "Alice — primary admin"
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

The maintainer fills in `createdAt`/`createdBy` and returns the resulting entry in its response.

### Adding a scoped, expiring member

```json
{
  "id": "8a91c7b3-2e62-4a91-a3a4-9d61b75e2f01",
  "type": "https://trusttasks.org/spec/acl/grant/0.1",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-16T10:05:00Z",
  "payload": {
    "entry": {
      "subject": "did:web:contractor.example",
      "role": "member",
      "scopes": ["context:project-alpha"],
      "expiresAt": "2026-08-16T00:00:00Z"
    },
    "reason": "Six-month contractor engagement on project-alpha."
  }
}
```

`proof` is omitted because this example assumes a transport that conveys producer identity end-to-end (per [SPEC.md §4.7.1](../../../../SPEC.md#471-when-to-include-a-proof)). A maintainer retaining the document for audit **SHOULD** require a proof.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/acl/grant/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ entry: AclEntry }`, where `entry` is the canonical AclEntry the maintainer now holds for the subject. The granting authority **SHOULD** treat the maintainer's `entry` as the authoritative post-state, since the maintainer applies its own clock and may normalize fields.

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful grant

Response to the first request example:

```json
{
  "id": "5e3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f3",
  "type": "https://trusttasks.org/spec/acl/grant/0.1#response",
  "threadId": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:org.example",
  "issuedAt": "2026-05-16T10:00:01Z",
  "payload": {
    "entry": {
      "subject": "did:web:alice.example",
      "role": "admin",
      "label": "Alice — primary admin",
      "createdAt": "2026-05-16T10:00:01Z",
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

## Security & Privacy

A grant document is evidence: a captured `acl/grant` Trust Task proves who authorized whom with what role. The **REQUIRED** `proof` ensures the granting authority cannot repudiate the grant and that intermediaries cannot alter its content.

Where the subject is a natural person or the role vocabulary is sensitive (for example, signalling membership in a regulated community), producers **SHOULD** apply transport confidentiality appropriate to the privacy regime.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload; producers **MUST NOT** place data in `ext` that they would not be comfortable signing. The `ext` slot is available at both the payload level and on the `AclEntry` itself — the same namespacing and ignore-unknown rules apply at both levels.
