---
slug: vtc/install/claim/start
version: "0.2"
title: VTC Install Claim — Start
summary: Begin first-admin enrolment against a fresh community — exchange the install token for a WebAuthn registration challenge.
status: draft
targetFrameworkVersion: "0.5"
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
  rationale: "Issues a single-use registration challenge; the enrolment is completed by claim/finish."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/install/claim/start:invalidToken
    meaning: The install token is missing, malformed, expired, or already consumed.
    retryable: false
---

## Abstract

The **VTC Install Claim — Start** Trust Task begins first-admin enrolment on a freshly-installed community that has no admin yet. The installer presents the single-use `installToken` printed by `vtc setup`; the community returns a `registrationId` and WebAuthn `options`. The installer creates a passkey, then completes at [`vtc/install/claim/finish`](../../finish/0.2/).

### Changes from 0.1

`0.1` also returned a `didBindingChallenge` — 32 random bytes the candidate `did:key` was required to sign, submitted back as `didBindingSignature` at `finish`. **0.2 removes it, and the removal is a correction rather than a relaxation.**

The admin DID is *derived from* the passkey's public key. A signature over a server challenge by that same key therefore proves nothing the WebAuthn attestation has not already proven — it is one key asserting control of itself, twice. Worse, it is not producible: WebAuthn never exposes the credential private key to the page, so no browser can sign raw bytes with it. An implementation can only satisfy `0.1` by reaching into a software authenticator for private-key material, which is exactly what a hardware authenticator exists to prevent.

Proof of control in `0.2` is the attestation, and the DID follows from the key it attests.

A future version **MAY** reintroduce a binding for a different purpose: letting an installer claim admin under a DID they *already control* rather than one derived from the passkey. That is a genuinely separate proof — a different key, held elsewhere — and it would be expressed as proof of ownership over a verification method present in the DID's **active** DID document, signed outside the WebAuthn ceremony. It is deliberately absent here rather than approximated.

## Conformance

Producer: supply `installToken`.

Consumer: verify the install token (`invalidToken` otherwise). Return the registration options bound to the token's jti.

## Security & Privacy

**Token-gated cold start.** Before an admin exists there is no key to sign a framework proof, so the single-use, audience-scoped install token is the authentication. The candidate proves control of the passkey at claim/finish, and the admin DID is derived from the key that attestation covers.
