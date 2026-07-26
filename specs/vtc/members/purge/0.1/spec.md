---
slug: vtc/members/purge
version: "0.1"
title: VTC Members — Purge
summary: Irreversibly delete a removed member's tombstone and residual records from a Verifiable Trust Community, refusing when it would orphan the last administrator.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - purge
  - rtbf
  - erasure
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: super administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Irreversible erasure of a member record. The caller must be attributable.
sideEffects:
  level: destructive
  rationale: "Deletes the membership record and its tombstone. Not recoverable."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/purge:notFound
    meaning: No member or tombstone exists for that DID.
    retryable: false
  - code: vtc/members/purge:lastAdministrator
    meaning: Purging would leave the community with no administrator.
    retryable: false
  - code: vtc/members/purge:permissionDenied
    meaning: The consumer lacks the community super-admin capability.
    retryable: false
---

## Abstract

The **VTC Members — Purge** Trust Task irreversibly deletes a member's record and tombstone. It is the terminal step after removal: [`vtc/members/admin-remove`](../../admin-remove/0.1/) and [`vtc/members/self-remove`](../../self-remove/0.1/) leave a tombstone that [`vtc/members/removed`](../../removed/0.1/) can enumerate; purge erases it.

This is the community's right-to-be-forgotten lever, and the only one that leaves nothing behind.

## Conformance

Consumer: verify the **super-admin** capability — a step above the community-admin gate the removal tasks use, because removal is reversible in effect and this is not. Refuse with `notFound` when there is neither a member nor a tombstone.

Refuse with `lastAdministrator` when the purge would leave the community with no administrator. That check is not advisory: a community with no admin cannot grant one, so the operation would be unrecoverable from inside the community.

## Security & Privacy

Purge is the one member operation with no undo, which is why it sits behind super-admin rather than admin and why the last-administrator guard has no override flag. A destructive operation with a `force` escape hatch is a destructive operation that will eventually be forced.

Erasure is deliberately scoped to the community's own records. Credentials the member holds, and any the community published to a status list, are not reachable from here — a purge removes the community's memory of a member, not the member's evidence of having been one.
