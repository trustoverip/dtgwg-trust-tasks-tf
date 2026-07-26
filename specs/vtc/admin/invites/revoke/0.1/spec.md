---
slug: vtc/admin/invites/revoke
version: "0.1"
title: VTC Admin Invites — Revoke
summary: Revoke an outstanding administrator invite by its `jti`, so the install URL can no longer be redeemed.
status: draft
targetFrameworkVersion: "0.2"
category: access-control
keywords:
  - vtc
  - admin
  - invite
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
  rationale: Withdraws a pending grant of administrator access. The caller must be attributable.
sideEffects:
  level: mutating
  rationale: "Marks an issued invite unusable. Consumed invites are immutable and are refused."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/admin/invites/revoke:notFound
    meaning: No invite with that `jti` exists.
    retryable: false
  - code: vtc/admin/invites/revoke:alreadyConsumed
    meaning: The invite was already redeemed; consumed rows are immutable audit history.
    retryable: false
  - code: vtc/admin/invites/revoke:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Admin Invites — Revoke** Trust Task withdraws an outstanding administrator invite so its install URL can no longer be redeemed. The target is the `jti` returned by [`vtc/admin/invites/create`](../../create/0.1/) and listed by [`vtc/admin/invites/list`](../../list/0.1/).

## Conformance

Producer: supply the `jti` of an invite in `issued` state.

Consumer: verify the community-admin capability. Refuse a `jti` in `consumed` state with `alreadyConsumed` — a redeemed invite is audit history and MUST NOT be mutated, because rewriting it would erase the record of an administrator having been enrolled. An unknown `jti` is `notFound`. Revoking an already-`expired` invite is permitted and idempotent: the outcome the caller wants is already true.

## Security & Privacy

Revocation is the only defence against a leaked install URL, so it must stay available for the whole life of an `issued` invite and must not depend on knowing the claim code — the `jti` alone is sufficient, and the `jti` is the one part of the invite that `list` does disclose.

Refusing to revoke a consumed invite is not an inconvenience but the point: the immutability of that row is what makes the invite registry usable as evidence of how administrator access was granted.
