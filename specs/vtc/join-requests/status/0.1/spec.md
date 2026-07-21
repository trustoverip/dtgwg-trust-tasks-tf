---
slug: vtc/join-requests/status
version: "0.1"
title: VTC Join-Requests — Status
summary: An applicant polls the state of their pending join request, learning what more is needed if deferred.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - status
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
  rationale: The poller must prove they are the applicant that owns the request; the document proof's signer is checked against the request's applicant. This replaces the pre-migration REST signature.
sideEffects:
  level: none
  rationale: "Reads the request's current state; persists nothing."
subjectPath: /requestId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/status:notFound
    meaning: No join request with the supplied requestId exists, or it does not belong to the proof signer.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Status** Trust Task lets an applicant poll their own join request. It returns the current `status`; when `deferred`, `needs` names what the applicant must supply and `presentationDefinition` describes the additional evidence to present. The applicant is the proof signer, checked against the request's owner.

## Conformance

Producer: supply `requestId`. Carry a proof; the signer MUST be the request's applicant.

Consumer: resolve the request and confirm the proof signer owns it; if not (or it is absent), return `notFound` — the same code for both, so a poller cannot probe for requests it does not own. Return the current `status`, plus `needs`/`presentationDefinition` when deferred.

## Security & Privacy

**Ownership-gated read.** Unknown request and not-your-request collapse to a single `notFound`, so status is not an oracle over other applicants' requests. The proof binds the poll to the applicant, replacing the transport-specific signature.
