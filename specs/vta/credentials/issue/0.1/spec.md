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
  - governance
  - policy
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
  - code: vta/credentials/issue:profileViolation
    meaning: payload.credentialType names a claims profile defined by this specification and payload.claims does not satisfy it (shape mismatch, or a policyHash that does not match the canonicalized policy).
    retryable: false
related:
  - vta/credentials/revoke
  - auth/step-up/approve-response
  - acl/grant
  - credential-exchange/query
  - vtc/endorsements/issue
  - policy/activate
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
---

## Abstract

The **VTA Credentials — Issue** Trust Task instructs a VTA to mint a **scoped, time-boxed Verifiable Credential** and bind it to a named *holder*. Unlike the fixed authorization credential minted once at integration bootstrap, this task issues an **arbitrary-claims** credential on demand — the mechanism behind an explicit, revocable **cross-context share**: one context (the *issuing authority*) grants a holder in another context exactly the claims named in `payload.claims`, no more, for exactly `payload.validitySeconds`, no longer.

Issuance is **high-trust**: a VTA **MUST** gate this task behind operator **step-up** (an elevated authentication assurance level, see [`auth/step-up/approve-response`](../../../../auth/step-up/approve-response/0.1/spec.md)). The operator-signed step-up response is the unforgeable human authorization; this task is the instruction that step-up authorizes.

The credential's claim vocabulary is opaque to the framework — the issuing authority and the eventual verifier agree on its meaning. The issued credential is itself a standard Verifiable Credential and is verified, presented, and (optionally) revoked by the usual means.

Certain `credentialType` values name a **claims profile** defined by this specification: for those types, `payload.claims` is no longer opaque but MUST satisfy the profile's shape and rules (see [Claims profiles](#claims-profiles)). One profile is defined so far: `GovernancePolicyCredential`, which turns this task into the issuance leg of *governance policy as a credential* — a domain's governing parameters attested by the VTA so that enforcement components load policy out of the credential and attestations cite a `policyHash` whose issuance chain a verifier can check.

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
6. When `payload.credentialType` names a claims profile defined below, enforce that profile's additional rules; refuse a violation with `vta/credentials/issue:profileViolation`.

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

## Claims profiles

A claims profile binds a `credentialType` value to a required `claims` shape and to additional issuance rules. Profiles keep this task *one* task: the minting mechanism, the step-up gate, the persistence and revocation contract are all unchanged — a profile only constrains what a conforming consumer accepts as `claims` for that type, exactly as a schema-carrying endorsement type does in [`vtc/endorsements/issue`](../../../../vtc/endorsements/issue/0.1/spec.md). A `credentialType` that names no profile remains fully opaque, as before.

### GovernancePolicyCredential

**Intent.** A domain's governance policy — the parameters an enforcement component (e.g. an LLM gateway) applies to traffic it handles: model allowlist, budget caps, upstream pin, privacy tier — attested by the VTA instead of read from a host-editable file. An attestation then cites `claims.policyHash`, and a verifier checks the chain: *this hash is a policy the VTA issued for this domain and has not revoked.* The trust root for policy moves off the enforcing host.

When `payload.credentialType` is `GovernancePolicyCredential`:

1. `payload.claims` MUST validate against the `GovernancePolicyClaims` definition in `payload.schema.json`: `domain` (REQUIRED), `policy` (REQUIRED — the complete governing parameter document, opaque to the framework), `policyHash` (REQUIRED — a multibase-encoded multihash over the JCS [RFC 8785] canonicalization of `policy`), plus optional `contextId` and `policyMediaType`.
2. The consumer MUST recompute `policyHash` from `claims.policy` and refuse a mismatch with `vta/credentials/issue:profileViolation`. A credential whose hash does not commit to its own embedded policy would let an attestation cite one policy while the enforcing component loads another.
3. **Single-active rule.** At most one unexpired, unrevoked `GovernancePolicyCredential` exists per `(contextId, domain)`. Issuing a successor MUST atomically revoke the predecessor (per [`vta/credentials/revoke`](../../revoke/0.1/spec.md) semantics, recorded as a supersession) and return its id as `supersedes` in the response — the same displaced-id auditability that [`policy/activate`](../../../../policy/activate/0.1/spec.md) provides via `previousPolicyId`. Rotation is therefore one task invocation, not an issue-then-revoke pair the operator can half-complete.
4. **Published revocation.** The minted credential MUST carry a `credentialStatus` entry (e.g. W3C Bitstring Status List) resolvable *without authenticating to the issuing VTA*, and revocation MUST be effected by publishing the corresponding status flip. Third parties verify attestations that cite this credential; a revocation only the issuer can observe is invisible exactly where it matters. This adopts the revocation mechanics of [`vtc/endorsements/issue`](../../../../vtc/endorsements/issue/0.1/spec.md) (published slot, never reclaimed) without adopting that family's community-governance plane — the profile promise attaches to this `credentialType`, not to the task URI, so untyped shares are unaffected.
5. **Distribution is presentation, not a new task.** The VTA holds the credentials it mints under this profile and answers [`credential-exchange/query`](../../../../credential-exchange/query/0.1/spec.md) for them (DCQL `type_values: ["GovernancePolicyCredential"]`, optionally constrained by `domain`), presenting via [`credential-exchange/present`](../../../../credential-exchange/present/0.1/spec.md). An enforcing component fetches the current policy for a domain it serves as an ordinary pre-trusted verifier; no dedicated fetch task exists, deliberately.
6. **Consumption.** The enforcing component MUST load its runtime policy from `claims.policy` of a credential it has verified (signature, validity window, status, `domain` binding) and MUST fail closed — refuse traffic for the domain — when no such credential is obtainable. Falling back to local configuration reintroduces the host-resident policy this profile exists to retire.

`validitySeconds` SHOULD be bounded (hours-to-days, not months): expiry is the backstop that limits how long a stale cached status list can keep a superseded policy citable.

### Issue a domain's governance policy

```json
{
  "id": "5f0a2c61-9d7e-4b3a-8f12-6c4d0e9b7a33",
  "type": "https://trusttasks.org/spec/vta/credentials/issue/0.1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:00:00Z",
  "payload": {
    "holder": "did:web:gateway.example",
    "credentialType": "GovernancePolicyCredential",
    "claims": {
      "domain": "did:web:acme.example",
      "contextId": "ctx-acme",
      "policy": {
        "modelAllowlist": ["claude-fable-5", "claude-sonnet-4-5"],
        "budget": { "usdPerDay": 250 },
        "upstream": "https://llm.upstream.example",
        "privacyTier": "restricted"
      },
      "policyHash": "zQmYtUcp8bLrRk4nD2eW7vX5oS1qA9fJ3hT6gN0mC8xKvPe"
    },
    "validitySeconds": 86400,
    "purpose": "Rotate acme's LLM gateway governance policy (budget raise approved in change CR-1182)."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:vta.example#key-1",
    "created": "2026-07-29T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z9dw..."
  }
}
```

The response carries the minted credential (whose `credentialSubject` embeds the claims above and whose `credentialStatus` names the published status-list slot) and, because a policy was already active for `(ctx-acme, did:web:acme.example)`, the displaced credential's id in `supersedes`.
