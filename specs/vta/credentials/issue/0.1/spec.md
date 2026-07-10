---
slug: vta/credentials/issue
version: "0.1"
title: VTA Credentials — Issue
summary: A context authority issues a scoped, time-boxed Verifiable Credential to a holder after operator step-up approval — the basis for an explicit, revocable cross-context share.
status: draft
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - credentials
  - issuance
  - verifiable-credential
  - scope
  - cross-domain
  - share
  - step-up
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
  rationale: An issuance instruction authorizes the minting of a bearer-usable credential; it is replayed by an auditor and corroborates the resulting credential's provenance, so transport-independent integrity is required. The operator step-up that gates it is itself a separately-verifiable signed artifact.
sideEffects:
  level: mutating
  rationale: "Issues a scoped, time-boxed credential after step-up approval; revocable."
consequences:
  - "Issues a credential attributable to the context authority; valid until expiry or revocation."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "Issues a verifiable credential attributable to the context authority."
errorCodes:
  - code: vta/credentials/issue:holder_invalid
    meaning: The holder identifier is not a resolvable DID.
    retryable: false
  - code: vta/credentials/issue:scope_empty
    meaning: The requested claims object is empty — a share must convey at least one claim.
    retryable: false
  - code: vta/credentials/issue:validity_too_long
    meaning: The requested validity exceeds the issuer's maximum.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        requestedSeconds: { type: integer }
        maxSeconds: { type: integer }
  - code: vta/credentials/issue:step_up_required
    meaning: The operation requires a higher authentication assurance level (operator step-up) that has not been satisfied.
    retryable: true
related:
  - vta/credentials/revoke
  - auth/step-up/approve-response
  - acl/grant
---

## Abstract

The **VTA Credentials — Issue** Trust Task instructs a VTA to mint a **scoped, time-boxed Verifiable Credential** and bind it to a named *holder*. Unlike the fixed authorization credential minted once at integration bootstrap, this task issues an **arbitrary-claims** credential on demand — the mechanism behind an explicit, revocable **cross-context share**: one context (the *issuing authority*) grants a holder in another context exactly the claims named in `payload.claims`, no more, for exactly `payload.validitySeconds`, no longer.

Issuance is **high-trust**: a VTA **MUST** gate this task behind operator **step-up** (an elevated authentication assurance level, see [`auth/step-up/approve-response`](../../../../auth/step-up/approve-response/0.1/spec.md)). The operator-signed step-up response is the unforgeable human authorization; this task is the instruction that step-up authorizes.

The credential's claim vocabulary is opaque to the framework — the issuing authority and the eventual verifier agree on its meaning. The issued credential is itself a standard Verifiable Credential and is verified, presented, and (optionally) revoked by the usual means.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the issuing authority) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/credentials/issue/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.holder` with the recipient DID, `payload.claims` with a non-empty object of the claims to attest, and `payload.validitySeconds` with the desired lifetime.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Refuse the operation unless the caller has satisfied operator step-up; otherwise respond with `vta/credentials/issue:step_up_required`.
3. Refuse with `vta/credentials/issue:scope_empty` when `payload.claims` is empty, and with `vta/credentials/issue:validity_too_long` when `payload.validitySeconds` exceeds its configured maximum.
4. Mint a Verifiable Credential whose `credentialSubject.id` is `payload.holder`, whose claims are `payload.claims`, with `validFrom = now` and `validUntil = now + validitySeconds`, signed by the issuing context's key.
5. Persist a record keyed by the returned `credentialId` so the credential can be revoked ([`vta/credentials/revoke`](../../revoke/0.1/spec.md)) and audited, and return the `#response` document.

## Definitions

* **Issuing authority.** The party instructing issuance; identified by `issuer`. Typically a context admin.
* **VTA.** The agent that holds the signing key and mints the credential; identified by `recipient`.
* **Holder.** The party the credential is issued to; identified by `payload.holder` (becomes `credentialSubject.id`).
* **Claims.** An opaque object of attested statements — the *scope* of the share.
* **Step-up.** An elevated authentication assurance level satisfied by an operator-signed approval (see related task).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/credentials/issue/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Issue a scoped share to a holder in another context

```json
{
  "id": "9c2f1a7e-3b44-4e9a-8c21-0f5b2d6a1e90",
  "type": "https://trusttasks.org/spec/vta/credentials/issue/0.1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T10:00:00Z",
  "payload": {
    "holder": "did:key:z6Mkwork-domain-agent",
    "credentialType": "ScopedShareCredential",
    "claims": {
      "share": {
        "from": "finance",
        "fields": ["invoiceTotal", "dueDate"]
      }
    },
    "validitySeconds": 3600,
    "purpose": "Let the work domain reference this month's invoice total."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.example#key-1",
    "created": "2026-06-24T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

The VTA returns the minted credential and its `credentialId` in the response.

## Response

A *response* document carries `type: https://trusttasks.org/spec/vta/credentials/issue/0.1#response` with a payload validating against the `Response` definition in `payload.schema.json`.

```json
{
  "id": "1b7d4c90-5e21-4a83-b1f6-2c9e7a30d845",
  "type": "https://trusttasks.org/spec/vta/credentials/issue/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-24T10:00:01Z",
  "threadId": "9c2f1a7e-3b44-4e9a-8c21-0f5b2d6a1e90",
  "payload": {
    "credentialId": "urn:uuid:c0ffee00-1234-4abc-9def-0123456789ab",
    "expiresAt": "2026-06-24T11:00:01Z",
    "credential": {
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      "id": "urn:uuid:c0ffee00-1234-4abc-9def-0123456789ab",
      "type": ["VerifiableCredential", "ScopedShareCredential"],
      "issuer": "did:web:vta.example",
      "validFrom": "2026-06-24T10:00:01Z",
      "validUntil": "2026-06-24T11:00:01Z",
      "credentialSubject": {
        "id": "did:key:z6Mkwork-domain-agent",
        "share": { "from": "finance", "fields": ["invoiceTotal", "dueDate"] }
      },
      "proof": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "proofValue": "z..." }
    }
  }
}
```
