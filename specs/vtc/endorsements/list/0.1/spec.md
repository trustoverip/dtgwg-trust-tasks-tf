---
slug: vtc/endorsements/list
version: "0.1"
title: VTC Endorsements — List
summary: Page through the community's issued endorsement credentials, optionally filtered by type or subject; live and revoked rows both surface.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - endorsements
  - credentials
  - list
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
  rationale: Read-only enumeration. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads endorsement rows; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsements/list:permissionDenied
    meaning: The consumer holds neither the community-admin nor the issuer capability.
    retryable: false
  - code: vtc/endorsements/list:invalidCursor
    meaning: The supplied `cursor` is malformed or has expired.
    retryable: false
---

## Abstract

The **VTC Endorsements — List** Trust Task pages through the community's issued
[`Endorsement`](../../../_shared/0.1/endorsement.schema.json) rows. Filters:
`typeUri` (a registered endorsement type), `subjectDid`, and `includeRevoked`.

Cursor pagination follows the same convention as the rest of the `vtc/*`
families: an opaque `cursor`, a `limit` clamped to `1..=200` (default 50), and
a `nextCursor` that is `null` on the last page.

## Conformance

Producer: send an empty payload for the first page; pass the previous page's
`nextCursor` as `cursor` to continue.

Consumer: verify the community-admin **or** issuer capability against the live
ACL. Apply the supplied filters, clamp `limit`, and return the page. By default
**both live and revoked rows surface** — consumers filter on `revokedAt`; set
`includeRevoked: false` to omit revoked rows server-side. Reject a malformed or
expired `cursor` with `invalidCursor` rather than silently restarting from the
first page.

## Security & Privacy

**Admin/issuer-class metadata** (`discloses: metadata`). Enumeration reveals
every subject the community has endorsed and under which types, so the
capability gate is load-bearing — this task must not be exposed to ordinary
members.
