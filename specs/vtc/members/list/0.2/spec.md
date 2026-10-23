---
slug: vtc/members/list
version: "0.2"
title: VTC Members — List
summary: List a Verifiable Trust Community's members, newest paged, each joined with its ACL role, optionally filtered by role.
status: draft
targetFrameworkVersion: "0.5.0"
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
errorCodes: []
---

## Abstract

The **VTC Members — List** Trust Task returns a Verifiable Trust Community's members. Each entry is a [`MemberResponse`](../../../_shared/0.1/member.schema.json) — the membership record joined with its ACL entry, so the caller gets `role` and `label` inline without a second lookup. Optional `role` filters the result; `cursor`/`limit` page it.

It is the enumeration companion to [`vtc/members/show`](../../show/0.1/) (one member) and [`vtc/members/update`](../../update/0.1/).

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`**. It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. `totalEstimate` is unchanged. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

Producer: optional `role` filter; `cursor` to continue, `limit` to bound the page.

Consumer: verify the community-admin capability. Return matching members as `MemberResponse`s, joining each membership row with its ACL role. Clamp `limit` to 1..=200 (default 50). Return `nextCursor` when more pages remain; `totalEstimate` only when it is cheap to compute (a maintainer that cannot MAY return null).

## Security & Privacy

### Data carried

The request carries an optional `role` filter and paging members. The response
carries [`MemberResponse`](../../../_shared/0.1/member.schema.json)s — among
other members each one's DID, role and label, join time, publication consent
and departure preference, personhood assertion, and credential identifiers —
plus an optional `totalEstimate`.

**Member records are admin-class.** The list discloses each member's DID, role, and credential ids — community-internal metadata, not secret material, so `exposure.discloses` is `metadata`. The community-admin gate is what limits it; a future per-member-privacy (PMF) layer may narrow what a non-admin caller sees, but Phase 1 gates on the admin role alone.

### Correlation

A full traversal is the community's roster: every member DID joined to its role,
join time and credential identifiers, which a reader can match against the
credentials those members present elsewhere. `threadId` correlates request and
response, and `nextCursor`/`cursor` link the pages of one traversal; a
responder MUST NOT accept a cursor it did not issue. The specification imposes
no further rule on correlation.

### Retention

The task reads the membership registry and persists nothing. The specification
imposes no retention rule on the administrator's copy of the list.

### Consent/purpose

The purpose is for a community administrator to enumerate the community's
members and their roles. Each record carries the member's own `publishConsent`;
the specification states no further purpose limitation on the listing.
