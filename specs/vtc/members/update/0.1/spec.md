---
slug: vtc/members/update
version: "0.1"
title: VTC Members — Update
summary: Update a community member's role or non-credential metadata (consent, departure preference, extensions); refuses promotion to admin.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - community
  - update
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
  rationale: Changing a member's role or metadata alters their standing in the community. The change MUST be attributable and non-repudiable.
sideEffects:
  level: mutating
  rationale: "Updates a member's role or non-credential metadata; recoverable by updating again."
consequences:
  - "Changes the member's role or metadata, effective immediately."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/update:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/members/update:notFound
    meaning: No member with the supplied `did` exists.
    retryable: false
  - code: vtc/members/update:adminRoleForbidden
    meaning: "`role` was `admin`. Promotion to admin is a separate, gated flow, not a metadata update."
    retryable: false
---

## Abstract

The **VTC Members — Update** Trust Task changes a member's role or non-credential metadata: `role`, `publishConsent`, `departurePreference`, and the opaque `extensions` bag. Every field except `did` is optional — an update carries only what changes. It returns the updated [`MemberResponse`](../../_shared/0.1/member.schema.json).

It deliberately **cannot** promote a member to `admin`: that is a higher-trust operation with its own approval flow, not a metadata patch, so `role: admin` is refused here.

## Conformance

Producer: supply `did` and the fields to change. Carry a proof.

Consumer: verify the community-admin capability. Resolve the member; if none, return `notFound`. If `role` is `admin`, return `adminRoleForbidden` and change nothing. Otherwise apply the supplied fields, cap `extensions` at the maintainer's limit, and return the updated `MemberResponse`. Audit the change.

## Security & Privacy

**Standing change, so attributable.** A role or metadata change alters the member's standing in the community, so `proofRequirement: REQUIRED` and the change is audited. The `admin`-refusal keeps the highest-privilege transition off this general-purpose patch path.
