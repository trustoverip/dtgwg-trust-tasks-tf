---
slug: vtc/install/claim/finish
version: "0.1"
title: VTC Install Claim — Finish
summary: Complete first-admin enrolment — submit the passkey attestation and DID-binding signature to derive the admin DID and a setup-session token.
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
  rationale: The install token plus the passkey attestation and DID-binding signature in the payload are the gate; no admin key exists yet to carry a framework proof.
sideEffects:
  level: mutating
  rationale: "Derives the admin DID from the passkey and mints a short-lived setup-session token; the ACL write itself happens at admin/bootstrap."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/install/claim/finish:invalidToken
    meaning: The install token is missing, malformed, expired, or already consumed.
    retryable: false
  - code: vtc/install/claim/finish:registrationMismatch
    meaning: The registrationId does not match an open enrolment for this token.
    retryable: false
  - code: vtc/install/claim/finish:bindingInvalid
    meaning: The WebAuthn attestation or the DID-binding signature failed verification.
    retryable: false
---

## Abstract

The **VTC Install Claim — Finish** Trust Task completes first-admin enrolment. The installer submits the `installToken`, the `registrationId` from start, the `webauthnResponse`, and the `didBindingSignature` over the challenge. The community derives the admin DID from the passkey's Ed25519 key and returns it with a short-lived `setupSessionToken`, which is consumed by [`vtc/admin/bootstrap`](../../../../admin/bootstrap/0.1/) to write the admin ACL.

## Conformance

Producer: supply all four fields.

Consumer: verify the install token (`invalidToken`), match the `registrationId` (`registrationMismatch`), and verify both the WebAuthn attestation and the DID-binding signature (`bindingInvalid`). On success derive the admin DID and return it with a 5-minute setup-session token.

## Security & Privacy

**Dual proof of control.** The passkey attestation proves control of the authenticator; the DID-binding signature proves control of the did:key derived from it — so the admin DID cannot be claimed without holding both. The setup-session token is short-lived and single-audience, limiting the window in which bootstrap can run.
