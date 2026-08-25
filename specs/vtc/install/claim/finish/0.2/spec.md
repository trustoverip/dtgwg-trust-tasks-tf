---
slug: vtc/install/claim/finish
version: "0.2"
title: VTC Install Claim — Finish
summary: Complete first-admin enrolment — submit the passkey attestation to derive the admin DID and a setup-session token.
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
  rationale: The install token plus the passkey attestation in the payload are the gate; no admin key exists yet to carry a framework proof.
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
    meaning: The WebAuthn attestation failed verification.
    retryable: false
---

## Abstract

The **VTC Install Claim — Finish** Trust Task completes first-admin enrolment. The installer submits the `installToken`, the `registrationId` from start, and the `webauthnResponse`. The community derives the admin DID from the passkey's Ed25519 key and returns it with a short-lived `setupSessionToken`, which is consumed by [`vtc/admin/bootstrap`](../../../../admin/bootstrap/0.1/) to write the admin ACL.

### Changes from 0.1

`0.1` also required a `didBindingSignature` — a raw Ed25519 signature over the `didBindingChallenge` issued at `start`. **0.2 removes it.** See [`claim/start` 0.2](../../start/0.2/) for the full reasoning; in short, the admin DID is derived from the passkey's key, so that signature was the same key proving control of itself a second time, and WebAuthn does not expose the credential private key to the page, so no browser can produce it at all.

## Conformance

Producer: supply all three fields.

Consumer: verify the install token (`invalidToken`), match the `registrationId` (`registrationMismatch`), and verify the WebAuthn attestation (`bindingInvalid`). On success derive the admin DID from the attested key and return it with a 5-minute setup-session token.

## Authorization

*Stated in anticipation of [SPEC §7.3](../../../../../../SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is three things together, none of which is sufficient alone: a valid **install token**, a matching **`registrationId`**, and a verifying **WebAuthn attestation**.

The combination is the point. The token establishes that this installation was expected, the registration match that this is the claim it was expected for, and the attestation that the authenticator is genuine and holds the key the admin DID will be derived from. Each failure has its own code (`invalidToken`, `registrationMismatch`, `bindingInvalid`) so an operator can tell which assumption broke.

`0.1` named a fourth — a DID-binding signature — on the reasoning that the attestation proved control of the *authenticator* while the signature proved control of the *DID*. That distinction does not hold when the DID is derived from the attested key: both statements are made by one key about itself. A version that let the installer supply a DID they already control would restore the distinction, and would then need a fourth piece of evidence again.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](../../../../../../SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

**Proof of control.** The passkey attestation proves the installer holds the authenticator, and the admin DID is derived from the very key that attestation covers — so the DID cannot be claimed without holding the authenticator. The setup-session token is short-lived and single-audience, limiting the window in which bootstrap can run.

**What this does not prove.** The admin DID is *assigned by* this ceremony, not *asserted by* the installer. Nothing here demonstrates control of any pre-existing identity. A deployment that needs an operator to claim admin under a DID they already hold needs proof of ownership over a verification method in that DID's **active** DID document, verified against a live resolution rather than against material supplied in the request — which is a different task, not a stronger reading of this one.
