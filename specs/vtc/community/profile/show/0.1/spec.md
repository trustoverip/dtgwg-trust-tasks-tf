---
slug: vtc/community/profile/show
version: "0.1"
title: VTC Community Profile — Show
summary: Fetch the Verifiable Trust Community's public profile (name, description, contact, language) plus live registry-reachability status.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - community
  - profile
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
  rationale: Read-only fetch of the community's own profile. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the community profile record; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/community/profile/show:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Community Profile — Show** Trust Task returns the community's public
[`CommunityProfile`](../../../../_shared/0.1/community.schema.json) — its name,
description, optional logo / public URL / contact, display language, and any
community-defined `extensions`. The response additionally carries the
read-only `registryStatus` (`active` | `degraded`) reflecting current
trust-registry reachability.

This is the read half of the former `community/profile/manage` operation;
`vtc/community/profile/update` is the write half.

## Conformance

Producer: send an empty request (only the framework `ext` point is allowed).

Consumer: verify the community-admin capability, then return the current
`CommunityProfile`, populating `registryStatus` from live registry health.

## Security & Privacy

**Admin-class metadata** (`discloses: metadata`). The profile is largely
public-facing, but `contactEmail` and `registryStatus` are operational
detail; the task sits behind the community-admin gate.
