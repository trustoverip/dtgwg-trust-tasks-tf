---
slug: vtc/join-requests/approve
version: "0.1"
title: VTC Join-Requests — Approve
summary: An administrator approves a pending join request, admitting the applicant to the community.
status: retired
supersededBy: vtc/join-requests/decide
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - approve
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
  rationale: Admitting a member changes community membership; the decision MUST be attributable to the operator who made it.
sideEffects:
  level: mutating
  rationale: "Approves the request and admits the applicant as a member."
consequences:
  - "Admits the applicant to the community as a member."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/approve:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/join-requests/approve:notFound
    meaning: No join request with the supplied id exists.
    retryable: false
  - code: vtc/join-requests/approve:notPending
    meaning: The request is not in the pending state, so it cannot be approved.
    retryable: false
---

## Abstract

> **Retired.** Superseded by [`vtc/join-requests/decide`](../../decide/0.1/), which carries this task's payload as `{ id, decision: "approved" }`. The payloads, admin gate, lifecycle check and proof posture of approve and reject were identical; the decision is one enum field, not two tasks.

The **VTC Join-Requests — Approve** Trust Task approves a pending join request `id`, admitting the applicant as a member. The decision counterpart to [`vtc/join-requests/reject`](../../reject/0.1/).

## Conformance

Producer: supply `id`. Carry a proof.

Consumer: verify the community-admin capability. Resolve the request (`notFound` if absent); if it is not `pending`, return `notPending`. Otherwise admit the applicant, set status to `approved`, and return `{ requestId, status: approved }`. Audit the decision.

## Security & Privacy

**Membership change, so attributable.** Admitting a member is proof-REQUIRED and audited. Deployments that require a second approver gate this behind the community's confirm flow at the enforcement point, orthogonal to this payload.
