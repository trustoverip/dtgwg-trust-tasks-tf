---
slug: vtc/members/rotate
version: "0.1"
title: VTC Members — Rotate
summary: Complete a member DID rotation by proving control of the old and new keys; re-issues credentials to the new DID.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - did-rotation
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
  requirement: REQUIRED
  rationale: Rotating a member's DID reassigns their identity in the community. It MUST be attributable, in addition to the in-payload dual-key signatures that authorize the rotation itself.
sideEffects:
  level: mutating
  rationale: "Repoints the member to a new DID and re-issues their credentials; recoverable only by a further rotation."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "The subject is the authenticated caller acting on their own membership — the task exercises the member's own authority over their own record."
errorCodes:
  - code: vtc/members/rotate:rotationExpired
    meaning: The rotationId is unknown or its challenge has expired. Start a new rotate-challenge.
    retryable: false
  - code: vtc/members/rotate:signatureInvalid
    meaning: The old or new signature did not verify over the challenge, so control of both keys was not proven.
    retryable: false
---

## Abstract

The **VTC Members — Rotate** Trust Task completes a DID rotation opened by [`vtc/members/rotate-challenge`](../rotate-challenge/0.1/). The member presents the `rotationId`, the old and new DIDs, and a signature over the challenge from **each** key. On success the community repoints the member to `newDid` and re-issues their membership (`vmc`) and role (`roleVec`) credentials to it.

## Conformance

Producer: supply `rotationId`, `oldDid`, `newDid`, and both signatures. Carry a proof.

Consumer: look up the `rotationId`; if unknown or expired, return `rotationExpired`. Verify both signatures over the challenge; if either fails, return `signatureInvalid` and change nothing. Otherwise repoint the member to `newDid`, re-issue the credentials, and return `{ newDid, method, vmc, roleVec }`. Audit the rotation.

## Security & Privacy

**Dual-key proof plus attribution.** The old signature proves the member still controls the retiring key; the new signature proves control of the replacement — together they authorize the rotation. The framework proof is additionally REQUIRED so the ceremony is attributable to the authenticated session, closing the gap between "someone holds these keys" and "this session performed the rotation".
