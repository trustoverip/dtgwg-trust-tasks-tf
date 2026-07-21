---
slug: vtc/members/list
version: "0.1"
title: VTC Members — List
summary: List a Verifiable Trust Community's members, newest paged, each joined with its ACL role, optionally filtered by role.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
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
  rationale: Read-only inventory of community members. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the membership registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/list:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Members — List** Trust Task returns a Verifiable Trust Community's members. Each entry is a [`MemberResponse`](../../_shared/0.1/member.schema.json) — the membership record joined with its ACL entry, so the caller gets `role` and `label` inline without a second lookup. Optional `role` filters the result; `cursor`/`limit` page it.

It is the enumeration companion to [`vtc/members/show`](../show/0.1/) (one member) and [`vtc/members/update`](../update/0.1/).

## Conformance

Producer: optional `role` filter; `cursor` to continue, `limit` to bound the page.

Consumer: verify the community-admin capability. Return matching members as `MemberResponse`s, joining each membership row with its ACL role. Clamp `limit` to 1..=200 (default 50). Return `nextCursor` when more pages remain; `totalEstimate` only when it is cheap to compute (a maintainer that cannot MAY return null).

## Security & Privacy

**Member records are admin-class.** The list discloses each member's DID, role, and credential ids — community-internal metadata, not secret material, so `exposure.discloses` is `metadata`. The community-admin gate is what limits it; a future per-member-privacy (PMF) layer may narrow what a non-admin caller sees, but Phase 1 gates on the admin role alone.
