---
slug: vtc/join-requests/submit
version: "0.1"
title: VTC Join-Requests — Submit
summary: An applicant submits a request to join a Verifiable Trust Community, presenting the credentials the community's join policy requires.
status: retired
supersededBy: vtc/join-requests/submit/0.2
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - submit
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The document proof authenticates the applicant — its signer DID is the applicant DID. This replaces the transport-specific signature the pre-migration REST shape carried, and matches what DIDComm authcrypt provides intrinsically.
sideEffects:
  level: mutating
  rationale: "Creates a pending join request."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "The applicant submits on their own behalf — the subject is the proof signer, so the applicant acts as themselves; there is no separate subject field."
errorCodes:
  - code: vtc/join-requests/submit:policyUnsatisfied
    meaning: The presentation did not satisfy the community's active join policy.
    retryable: false
  - code: vtc/join-requests/submit:presentationInvalid
    meaning: The Verifiable Presentation failed verification, or its holder did not match the proof signer.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Submit** Trust Task opens an application to join a community. The applicant presents a W3C Verifiable Presentation (`vp`) whose credentials satisfy the community's join policy, and optionally consents to trust-registry publication. On acceptance the community records a **pending** request and returns its `requestId`, which the applicant polls with [`vtc/join-requests/status`](../../status/0.1/).

The applicant identity is the **document proof's signer** — there is no `applicantDid` or `signature` field. This is the transport-agnostic form: over DIDComm the authcrypt sender is the signer, over REST/TSP the framework proof is, and the payload is identical on every transport.

## Conformance

Producer: supply `vp` (its holder MUST equal the proof signer); optionally `registryConsent` and `extensions`. Carry a proof.

Consumer: verify the proof and the presentation; if the VP fails verification or the holder mismatches the signer, return `presentationInvalid`; if it does not satisfy the active join policy, return `policyUnsatisfied`. Otherwise create a pending request bound to the applicant DID and return `{ requestId, status: pending }`.

## Authorization

*Stated in anticipation of [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15, which binds specifications targeting framework 0.4; this one targets 0.2, where the declaration is not yet required.*

The authorization evidence this task presupposes is the **presentation in `vp`, whose holder MUST equal the envelope proof's signer**. That equality is the whole authorization: it establishes that the party asking to join is the party the presented credentials describe.

`exposure.actsAsSubject` is `true` because the request is made in the subject's own name. A consumer that accepted a presentation whose holder differed from the signer would be admitting one party on another's evidence, which is why the check is stated as an equality rather than as two independent verifications.

The authorization decision is the *consumer*'s alone. This section describes the evidence the task assumes, not an obligation to authorize any particular party, and per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10 verifying the `proof` establishes who asked, never that they may.

## Security & Privacy

**The proof is the sender authentication.** Collapsing the old REST `applicantDid` + hex `signature` into the framework proof removes a hand-rolled auth scheme in favour of the one every conforming consumer already verifies. The VP is the *credential* evidence; the proof is *who submitted it* — distinct concerns, both required.
