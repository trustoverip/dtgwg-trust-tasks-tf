---
slug: vtc/endorsements/revoke
version: "0.1"
title: VTC Endorsements — Revoke
summary: Revoke an issued endorsement credential by flipping its published status-list bit; reports alreadyRevoked on re-revocation rather than succeeding silently.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - endorsements
  - credentials
  - revocation
  - status-list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community issuer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Revocation withdraws a credential third parties rely on. The instruction MUST be attributable and non-repudiable.
sideEffects:
  level: mutating
  rationale: "Flips the credential's published revocation bit; not reversible through this task."
consequences:
  - "Relying parties — including foreign communities — treat the endorsement as revoked from the next status-list fetch."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsements/revoke:permissionDenied
    meaning: The consumer holds neither the community-admin nor the issuer capability.
    retryable: false
  - code: vtc/endorsements/revoke:notFound
    meaning: No endorsement with the supplied `endorsementId` exists in this community.
    retryable: false
  - code: vtc/endorsements/revoke:alreadyRevoked
    meaning: The endorsement was already revoked.
    retryable: false
---

## Abstract

The **VTC Endorsements — Revoke** Trust Task withdraws an issued endorsement by
flipping its bit on the community's shared Revocation status list, at the
`statusListIndex` allocated by `vtc/endorsements/issue`. Because that list is
published, revocation is visible to foreign verifiers without contacting this
community.

The response carries the registry-wide
[`RevocationReceipt`](../../../../credentials/_shared/0.1/credentials.schema.json)
plus the `statusListIndex` whose bit was flipped, so a caller can confirm the
published effect.

## Conformance

Producer: supply `endorsementId`, optionally with a `reason` for the audit
record.

Consumer:

1. Verify the community-admin **or** issuer capability against the live ACL.
2. Respond `notFound` when the id is unknown.
3. Respond **`alreadyRevoked`** when the endorsement was already revoked — do
   not re-flip the bit or re-emit audit envelopes.
4. Otherwise flip the status-list bit, record `revokedAt` and the `reason`, and
   return the `#response` document.

### Idempotency: aligned with the registry contract

Revocation is **idempotent in effect** but **MUST** report `alreadyRevoked` on
re-revocation, matching the registry-wide contract stated on
[`RevocationReceipt`](../../../../credentials/_shared/0.1/credentials.schema.json)
and mirroring `vta/credentials/revoke:already_revoked`. The caller has to be
able to distinguish "I revoked it now" from "it was already gone".

> **Change from the pre-migration VTC behaviour.** The superseded
> `credentials/endorsements/revoke/1.0` returned `200 OK` silently on
> re-revocation. That diverged from the rest of the registry and is corrected
> here; implementations MUST adopt the error.

## Security & Privacy

**Issuer-class mutation.** Behind the live-ACL admin-or-issuer gate with a
REQUIRED framework proof. The status-list slot is **not** reclaimed — reusing
it would silently un-revoke this credential for any verifier holding a cached
copy of the list.
