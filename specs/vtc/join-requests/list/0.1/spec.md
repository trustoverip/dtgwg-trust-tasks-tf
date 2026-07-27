---
slug: vtc/join-requests/list
version: "0.1"
title: VTC Join-Requests — List
summary: List a community's join requests, optionally filtered by status, newest paged.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - community
  - list
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
  rationale: Read-only inventory of join requests. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the join-request registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/list:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Join-Requests — List** Trust Task returns a community's join requests as [`JoinRequest`](../../../_shared/0.1/join-request.schema.json) entries, optionally filtered by `status`, paged by `cursor`/`limit`. The enumeration companion to [`vtc/join-requests/show`](../../show/0.1/).

## Conformance

Producer: optional `status` filter; `cursor`/`limit` to page.

Consumer: verify the community-admin capability. Return matching requests, clamping `limit` to 1..=200 and setting `nextCursor` when more remain.

## Security & Privacy

**Admin-class.** Each entry carries the applicant DID and their submitted presentation, so the list is behind the community-admin gate; `exposure.discloses` is `metadata`.
