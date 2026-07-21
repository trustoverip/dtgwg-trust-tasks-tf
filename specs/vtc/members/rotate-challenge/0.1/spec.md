---
slug: vtc/members/rotate-challenge
version: "0.1"
title: VTC Members — Rotate Challenge
summary: Open a DID-rotation ceremony for the calling member and return the challenge bytes to sign with the old and new keys.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - did-rotation
  - challenge
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
  rationale: Opening the ceremony only issues a challenge; the binding proof is the paired old/new signatures presented to vtc/members/rotate.
sideEffects:
  level: none
  rationale: "Issues a single-use rotation challenge; the ceremony is completed by a separate task."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "The subject is the authenticated caller acting on their own membership — the task exercises the member's own authority over their own record."
errorCodes:
  - code: vtc/members/rotate-challenge:notMember
    meaning: The authenticated caller is not a member of this community.
    retryable: false
---

## Abstract

The **VTC Members — Rotate Challenge** Trust Task begins a member DID rotation. It returns a `rotationId`, an expiry, the `signingPayloadHex` the member signs with **both** the old and new keys, and the `canonicalTemplate` those signatures cover. The optional `reason` is a self-asserted motive, bound to the session and recorded on the audit envelope — intent, not evidence, and not covered by either signature. The ceremony is completed by [`vtc/members/rotate`](../rotate/0.1/).

## Conformance

Producer: optionally supply `reason`.

Consumer: verify the caller is a member. Mint a single-use, TTL'd `rotationId` bound to the session, and return the challenge material. Record the `reason` against the pending rotation.

## Security & Privacy

**Challenge only.** This task proves nothing on its own — control of the old and new keys is demonstrated at `vtc/members/rotate`. The challenge is single-use and expires, anchoring replay resistance for the rotation.
