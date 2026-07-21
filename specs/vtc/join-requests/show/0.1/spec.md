---
slug: vtc/join-requests/show
version: "0.1"
title: VTC Join-Requests — Show
summary: Fetch one join request by id, including the applicant's presentation and the recorded policy verdict.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - show
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
  requirement: RECOMMENDED
  rationale: Read-only fetch of one join request. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads one join request; persists nothing."
subjectPath: /id
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/show:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/join-requests/show:notFound
    meaning: No join request with the supplied id exists.
    retryable: false
---

## Abstract

The **VTC Join-Requests — Show** Trust Task returns one join request by `id` — the [`JoinRequest`](../../_shared/0.1/join-request.schema.json), including the applicant's `vp` and, once decided, the `policyDecision`. The read-one companion to [`vtc/join-requests/list`](../list/0.1/).

## Conformance

Producer: supply `id`.

Consumer: verify the community-admin capability. Resolve the request; if none, return `notFound`. Return the full `JoinRequest`.

## Security & Privacy

**Admin-class.** Discloses the applicant DID and their presentation — `metadata`, behind the community-admin gate.
