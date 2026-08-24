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

The **VTC Join-Requests — Status** Trust Task lets an applicant poll their own join request. It returns the current `status`; when `deferred`, `needs` names what the applicant must supply and `presentationDefinition` describes the additional evidence to present. When `rejected`, `code`, `reason` and `decidedAt` say why and when. The applicant is the proof signer, checked against the request's owner.

`requestId` is **optional**. An applicant whose first reply was lost never received an id, and a poll resolved from their own authenticated DID is the only form available to them — a refusal they cannot ask about is a refusal they cannot act on. A consumer that is given the id MUST prefer it over inferring the request from the caller.

The three refusal members are the applicant's half of a rejection. `code` is stable and safe to branch on; `reason` carries the decider's words when there were any; `decidedAt` is when the decision was taken, not when this poll was produced — on an admin refusal the two diverge by however long the applicant takes to ask.

## Conformance

Producer: supply `requestId` when you hold one; omit it when you do not. Carry a proof; the signer MUST be the request's applicant.

Consumer: resolve the request — from `requestId` when supplied, otherwise from the proof signer's own DID — and confirm the proof signer owns it; if not (or it is absent), return `notFound` — the same code for both, so a poller cannot probe for requests it does not own. Return the current `status`, plus `needs`/`presentationDefinition` when deferred, and `code`/`reason`/`decidedAt` when rejected.

A consumer MUST NOT return the refusal members for any status other than `rejected`: they say a decision was taken, and emitting them beside a `pending` status would tell an applicant their request had been refused when it had not.

## Security & Privacy

**Ownership-gated read.** Unknown request and not-your-request collapse to a single `notFound`, so status is not an oracle over other applicants' requests. The proof binds the poll to the applicant, replacing the transport-specific signature.
