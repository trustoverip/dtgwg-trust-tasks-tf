---
slug: vtc/invitations/revoke
version: "0.1"
title: VTC Invitations — Revoke
summary: Revoke an issued Invitation Credential by flipping its published status-list bit, reporting whether this call was the one that revoked it.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - invitation
  - vic
  - revoke
  - status-list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: inviter
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Withdraws a live credential. The revoking party is recorded and must be attributable.
sideEffects:
  level: mutating
  rationale: "Flips the credential's bit in the community's published BitstringStatusList. Idempotent."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/invitations/revoke:notFound
    meaning: No invitation with that `id` exists.
    retryable: false
  - code: vtc/invitations/revoke:permissionDenied
    meaning: The consumer lacks the inviter capability.
    retryable: false
---

## Abstract

The **VTC Invitations — Revoke** Trust Task withdraws an issued Invitation Credential by setting its bit in the community's published status list. Any verifier that checks the list will thereafter reject the credential, including one already in a holder's possession.

The target is the `id` returned by [`vtc/invitations/issue`](../../issue/0.1/) and listed by [`vtc/invitations/list`](../../list/0.1/).

## Conformance

Producer: supply the invitation `id`.

Consumer: verify the inviter capability. Revocation is **idempotent** — revoking an already-revoked invitation succeeds rather than erroring. Report which happened in `newlyRevoked`: `true` when this call flipped the bit, `false` when it was already set. Return the original `revokedAt`, not the time of the redundant call, so the audit record keeps naming the moment access was actually withdrawn.

Distinguishing the two is the caller's only way to tell "I revoked it" from "someone else already had" — collapsing them to a bare success would lose that, and reporting a fresh `revokedAt` on a repeat call would falsify the record.

## Security & Privacy

Revocation must remain available for the whole life of a credential and must not require holding the credential itself — the registry `id` is sufficient, and it is the one identifier `list` discloses.

Because the status list is **published**, revocation is observable to anyone who fetches it. That is the mechanism working as intended: a verifier with no access to this community's API must still be able to learn that a credential is dead.
