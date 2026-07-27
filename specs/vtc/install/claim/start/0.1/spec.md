---
slug: vtc/install/claim/start
version: "0.1"
title: VTC Install Claim — Start
summary: Begin first-admin enrolment against a fresh community — exchange the install token for a WebAuthn challenge and a DID-binding challenge.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, install, bootstrap, passkey]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: installer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: The single-use install token (an EdDSA-signed JWT in the payload) is the gate; the community has no admin yet to sign a framework proof.
sideEffects:
  level: none
  rationale: "Issues single-use registration and DID-binding challenges; the enrolment is completed by claim/finish."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/install/claim/start:invalidToken
    meaning: The install token is missing, malformed, expired, or already consumed.
    retryable: false
---

## Abstract

The **VTC Install Claim — Start** Trust Task begins first-admin enrolment on a freshly-installed community that has no admin yet. The installer presents the single-use `installToken` printed by `vtc setup`; the community returns a `registrationId`, WebAuthn `options`, and a `didBindingChallenge`. The installer creates a passkey and signs the DID-binding challenge, then completes at [`vtc/install/claim/finish`](../../finish/0.1/).

## Conformance

Producer: supply `installToken`.

Consumer: verify the install token (`invalidToken` otherwise). Return the registration options and challenges bound to the token's jti.

## Security & Privacy

**Token-gated cold start.** Before an admin exists there is no key to sign a framework proof, so the single-use, audience-scoped install token is the authentication. The candidate must prove control of both the passkey and the derived did:key at claim/finish.
