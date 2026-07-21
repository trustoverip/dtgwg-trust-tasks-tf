---
slug: vtc/members/renew
version: "0.1"
title: VTC Members — Renew
summary: A member renews their community membership, re-issuing their membership and role credentials.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - community
  - renew
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A member renewing their own membership is gated by the authenticated session; a proof adds attribution but is not required for a self-service refresh.
sideEffects:
  level: mutating
  rationale: "Re-issues the member's membership and role credentials; recoverable by renewing again."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "The subject is the authenticated caller acting on their own membership — the task exercises the member's own authority over their own record."
errorCodes:
  - code: vtc/members/renew:notMember
    meaning: The authenticated caller is not a member of this community.
    retryable: false
---

## Abstract

The **VTC Members — Renew** Trust Task refreshes the calling member's membership, re-issuing their Verifiable Membership Credential (`vmc`) and role credential (`roleVec`) and reporting the current `personhood` state (and whether it changed). The subject is the authenticated caller; there are no request parameters.

## Conformance

Producer: send with no parameters.

Consumer: verify the caller is a member; if not, return `notMember`. Re-issue the membership and role credentials, and return `{ did, vmc, roleVec, personhood, personhoodChanged }`. The credentials are opaque W3C Verifiable Credentials at this layer.

## Security & Privacy

**Self-service, session-gated.** Renewal acts only on the caller's own membership, so the authenticated session is the primary gate and proof is RECOMMENDED for attribution. The re-issued credentials carry their own cryptographic integrity.
