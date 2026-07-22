---
slug: vtc/endorsements/issue
version: "0.1"
title: VTC Endorsements — Issue
summary: A community issues a Verifiable Endorsement Credential of a registered type to a subject, allocating a published status-list slot so foreign verifiers can check revocation.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - endorsements
  - credentials
  - issuance
  - verifiable-credential
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
  rationale: An issuance instruction mints a credential a third party will rely on; it is replayed by an auditor and corroborates the resulting credential's provenance, so transport-independent integrity is required.
sideEffects:
  level: mutating
  rationale: "Mints a signed endorsement credential and consumes a status-list slot; revocable, but the slot is not reclaimed."
consequences:
  - "Issues a credential attributable to the community; valid until expiry or revocation."
  - "Permanently consumes one slot on the community's shared Revocation status list."
subjectPath: /subjectDid
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsements/issue:permissionDenied
    meaning: The consumer holds neither the community-admin nor the issuer capability.
    retryable: false
  - code: vtc/endorsements/issue:typeNotRegistered
    meaning: "`typeUri` is not registered in this community's endorsement-type registry."
    retryable: false
  - code: vtc/endorsements/issue:claimSchemaViolation
    meaning: "`claim` failed validation against the endorsement type's declared claimSchema."
    retryable: false
  - code: vtc/endorsements/issue:claimTooLarge
    meaning: "`claim` exceeds the 8 KiB serialised cap."
    retryable: false
  - code: vtc/endorsements/issue:statusListExhausted
    meaning: The community's status list has no free slot; an operator must provision a new list.
    retryable: true
---

## Abstract

The **VTC Endorsements — Issue** Trust Task mints a **Verifiable Endorsement
Credential** (VEC) — the community attesting a claim of a *registered* type
about a subject DID. It returns an
[`Endorsement`](../../../_shared/0.1/endorsement.schema.json), which embeds the
registry-wide
[`IssuedCredential`](../../../../credentials/_shared/0.1/credentials.schema.json)
receipt and adds the two VTC-specific parts: the `typeUri` and the allocated
`statusListIndex`.

### Why this is not `vta/credentials/issue`

The minting *mechanism* is shared — and this task reuses it literally, via the
`credentials/_shared` component. The *trust operation* differs on three axes,
which is why it is a separate Trust Task rather than a variant of one:

1. **Third-party revocation-verifiability.** A VEC is checked by a *foreign*
   community (see `vtc/auth/recognise`), so it MUST carry a published
   status-list slot a stranger can read. A `vta/credentials/*` share credential
   is verified once by its recipient and revoked by removing an ACL entry —
   there is no published bit. One URI cannot promise both.
2. **Approval plane.** Endorsement issuance may run on the community's
   *self-management* plane, gated by policy with no human in the loop.
   `vta/credentials/issue` is on the *management* plane, gated by operator
   step-up.
3. **Governance gating.** `typeUri` MUST already be registered via
   `vtc/endorsement-types/register`; the VTA task treats `credentialType` as a
   free string.

## Conformance

Producer: supply `subjectDid`, a registered `typeUri`, and a non-empty `claim`.

Consumer:

1. Verify the community-admin **or** issuer capability against the live ACL —
   do not infer it from a cached token role.
2. Reject an unregistered `typeUri` with `typeNotRegistered`.
3. Reject a `claim` over 8 KiB with `claimTooLarge`; when the endorsement type
   declares a `claimSchema`, validate `claim` against it and reject a failure
   with `claimSchemaViolation`.
4. Allocate the next free slot on the shared Revocation status list
   (`statusListExhausted` if none remains), sign the VEC, persist the row, and
   return the full `Endorsement`.

The status-list slot MUST be allocated durably before the credential is
returned — a credential handed out with no reachable revocation slot cannot be
revoked.

## Security & Privacy

**Issuer-class mutation.** The community's signature is applied to a claim a
third party will rely on, so the framework proof is REQUIRED and the live-ACL
check is mandatory (a JWT role alone is insufficient — a token minted before an
issuer grant was withdrawn must not still issue).

Slot allocation is **not reclaimed** on revocation, by design: reusing a slot
would silently un-revoke a credential for any verifier holding a cached list.
