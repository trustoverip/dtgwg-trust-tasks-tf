---
slug: vtc/endorsements/show
version: "0.1"
title: VTC Endorsements — Show
summary: Fetch one issued endorsement credential by id, including its status-list slot and revocation state.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - endorsements
  - credentials
  - show
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community issuer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only fetch of one endorsement. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads one endorsement row; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsements/show:permissionDenied
    meaning: The consumer holds neither the community-admin nor the issuer capability.
    retryable: false
  - code: vtc/endorsements/show:notFound
    meaning: No endorsement with the supplied `endorsementId` exists in this community.
    retryable: false
---

## Abstract

The **VTC Endorsements — Show** Trust Task returns one issued
[`Endorsement`](../../../_shared/0.1/endorsement.schema.json) by
`endorsementId` — the same shape `vtc/endorsements/list` returns per item,
including the embedded issuance receipt, the `statusListIndex`, and `revokedAt`
(null while live).

## Conformance

Producer: supply `endorsementId`.

Consumer: verify the community-admin **or** issuer capability against the live
ACL. Resolve the row; if none exists, return `notFound`. Otherwise return the
full `Endorsement`. Revoked rows are returned (with `revokedAt` set), not
hidden — a caller needs to read the revocation state.

## Security & Privacy

**Admin/issuer-class metadata** (`discloses: metadata`). The row exposes the
subject DID and the attested claim, so it stays behind the capability gate; the
signed credential itself is separately verifiable and is not secret to a party
already holding it.
