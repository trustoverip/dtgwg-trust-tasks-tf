---
slug: vtc/join-requests/reject
version: "0.1"
title: VTC Join-Requests — Reject
summary: An administrator rejects a pending join request, optionally recording a reason.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - reject
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Refusing an applicant is a community decision that MUST be attributable to the operator who made it.
sideEffects:
  level: mutating
  rationale: "Rejects the request; recoverable only by the applicant re-applying."
consequences:
  - "Refuses the applicant; they are not admitted."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/reject:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/join-requests/reject:notFound
    meaning: No join request with the supplied id exists.
    retryable: false
  - code: vtc/join-requests/reject:notPending
    meaning: The request is not in the pending state, so it cannot be rejected.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Reject** Trust Task rejects a pending join request `id`, with an optional operator `reason`. The decision counterpart to [`vtc/join-requests/approve`](../../approve/0.1/).

## Conformance

Producer: supply `id`; optionally `reason`. Carry a proof.

Consumer: verify the community-admin capability. Resolve the request (`notFound` if absent); if not `pending`, return `notPending`. Otherwise set status to `rejected`, record the `reason`, and return `{ requestId, status: rejected }`. Audit the decision.

## Security & Privacy

**Community decision, so attributable.** Refusal is proof-REQUIRED and audited with the operator reason.
