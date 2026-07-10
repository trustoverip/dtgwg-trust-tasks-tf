---
slug: vta/credentials/revoke
version: "0.1"
title: VTA Credentials — Revoke
summary: A context authority revokes a credential it previously issued, ending a cross-context share before its natural expiry.
status: draft
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - credentials
  - revocation
  - verifiable-credential
  - cross-domain
  - share
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Issuing authority
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A revocation is an evidentiary record that an issued credential was withdrawn at a point in time; it is replayed by an auditor and relied on by verifiers, so transport-independent integrity is required.
sideEffects:
  level: destructive
  rationale: "Revokes a previously-issued credential, ending a cross-context share before its natural expiry."
consequences:
  - "Relying parties treat the credential as revoked; the share cannot be resumed under it."
subjectPath: /credentialId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/credentials/revoke:not_found
    meaning: No issued credential with the given id is known to this VTA.
    retryable: false
  - code: vta/credentials/revoke:already_revoked
    meaning: The credential was already revoked.
    retryable: false
related:
  - vta/credentials/issue
  - acl/revoke
---

## Abstract

The **VTA Credentials — Revoke** Trust Task withdraws a credential a VTA previously issued via [`vta/credentials/issue`](../../issue/0.1/spec.md), ending a cross-context share before its `validUntil`. The VTA records the revocation against the credential's id; a verifier that consults the VTA — or that relies on the VTA's short credential lifetimes — will no longer treat the credential as valid.

Revocation is **idempotent in effect** but **MUST** report `vta/credentials/revoke:already_revoked` when the credential was already revoked, so the caller can distinguish "I revoked it now" from "it was already gone".

This task complements the VTA's authoritative control surface — removing the consuming party's access ultimately rests on the VTA's ACL — but provides a credential-granular withdrawal for shares that were issued as standalone Verifiable Credentials.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the issuing authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/credentials/revoke/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.credentialId` with the id returned by the original issuance.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document and verify the `proof`.
2. Respond with `vta/credentials/revoke:not_found` when the id is unknown, and `vta/credentials/revoke:already_revoked` when it was already revoked.
3. Otherwise mark the credential revoked (recording `revokedAt`) and return the `#response` document.

## Definitions

* **Issuing authority.** The party instructing revocation; identified by `issuer`.
* **VTA.** The party that issued the credential and holds its record; identified by `recipient`.
* **Credential id.** The `credentialId` returned by `vta/credentials/issue`.

## Request

```json
{
  "id": "2d51b8a0-7c33-4e12-9af4-1b6e0c2d3f55",
  "type": "https://trusttasks.org/spec/vta/credentials/revoke/0.1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T10:30:00Z",
  "payload": {
    "credentialId": "urn:uuid:c0ffee00-1234-4abc-9def-0123456789ab",
    "reason": "The operator withdrew the share."
  }
}
```

## Response

```json
{
  "id": "7a0e9c41-2f88-4b6d-a3e1-9c0b5d2e4f76",
  "type": "https://trusttasks.org/spec/vta/credentials/revoke/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T10:30:01Z",
  "threadId": "2d51b8a0-7c33-4e12-9af4-1b6e0c2d3f55",
  "payload": {
    "credentialId": "urn:uuid:c0ffee00-1234-4abc-9def-0123456789ab",
    "revokedAt": "2026-06-24T10:30:01Z"
  }
}
```
