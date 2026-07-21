---
slug: vtc/members/show
version: "0.1"
title: VTC Members — Show
summary: Fetch one Verifiable Trust Community member by DID, joined with its ACL role. The read-one companion to vtc/members/list.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
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
  rationale: Read-only fetch of one member. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads one membership record; persists nothing."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/show:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/members/show:notFound
    meaning: No member with the supplied `did` exists in this community.
    retryable: false
---

## Abstract

The **VTC Members — Show** Trust Task returns one community member — the [`MemberResponse`](../../_shared/0.1/member.schema.json) for `did`, the membership record joined with its ACL role. The read-one companion to [`vtc/members/list`](../list/0.1/).

The member `did` is carried in the payload (not only a transport path), so the task dispatches identically over REST and DIDComm and the subject is visible to policy evaluation via `subjectPath: /did`.

## Conformance

Producer: supply `did`.

Consumer: verify the community-admin capability. Resolve the member by `did`; if none exists, return `notFound`. Otherwise return the full `MemberResponse`, the same shape `vtc/members/list` returns per item.

## Security & Privacy

**Admin-class.** Discloses the member's DID, role, and credential ids — community-internal metadata (`discloses: metadata`), behind the community-admin gate.

**Enumeration.** An unknown DID returns `notFound` while a real one returns the record, so `show` is a membership oracle for a DID. Membership of a community is not generally secret to an admin caller, and the caller must already hold the admin capability, so this is acceptable; do not expose this task to non-admin callers without a per-member-privacy layer.
