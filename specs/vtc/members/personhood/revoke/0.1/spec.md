---
slug: vtc/members/personhood/revoke
version: "0.1"
title: VTC Members Personhood — Revoke
summary: An administrator revokes a member's personhood, re-issuing their credentials with the personhood flag cleared.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - personhood
  - revoke
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
  rationale: Revoking a member's personhood downgrades their standing; it MUST be attributable to the operator who ordered it.
sideEffects:
  level: mutating
  rationale: "Clears the member's personhood flag and re-issues their credentials; reversible via a fresh personhood/assert."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/members/personhood/revoke:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/members/personhood/revoke:notFound
    meaning: No member with the supplied did exists.
    retryable: false
---

## Abstract

The **VTC Members Personhood — Revoke** Trust Task clears the personhood flag for member `did`, the admin-driven counterpart to [`vtc/members/personhood/assert`](../../assert/0.1/). On success the community re-issues the member's credentials carrying `personhood: false`; a revoke of an already-unset member is an idempotent no-op (the credentials are then omitted).

## Conformance

Producer: supply `did`. Carry a proof.

Consumer: verify the community-admin capability. Resolve the member (`notFound` if absent). Clear personhood and re-issue the credentials with `personhood: false`, returning `{ did, personhood: false, vmc, roleVec }`; on a no-op omit `vmc`/`roleVec`. Audit the revocation.

## Security & Privacy

**Admin downgrade, so attributable.** Revoking personhood lowers a member's standing, so it is proof-REQUIRED and audited. It is reversible — the member can assert personhood again — but each transition is recorded.
