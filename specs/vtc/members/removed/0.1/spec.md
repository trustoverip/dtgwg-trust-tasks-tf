---
slug: vtc/members/removed
version: "0.1"
title: VTC Members — Removed
summary: Enumerate the tombstones of members who have left a Verifiable Trust Community, with when they left and their published revocation slot.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - removed
  - tombstone
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
  rationale: Read-only inventory of departed members. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads membership tombstones; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/removed:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Members — Removed** Trust Task enumerates the community's membership tombstones — the records left behind when a member is removed by an administrator or leaves of their own accord.

Each entry carries the DID, when removal happened, and the `statusListIndex` of the member's revoked credential, so an operator can correlate a departure with the published status list a verifier would consult.

Live members come from [`vtc/members/list`](../../list/0.1/); [`vtc/members/purge`](../../purge/0.1/) erases a tombstone entirely.

## Conformance

Producer: no payload members are required.

Consumer: verify the community-admin capability. Return one entry per tombstone. A DID that has been purged MUST NOT appear — purge means the community no longer holds the record, and a listing that still named it would defeat the erasure.

## Security & Privacy

Tombstones are the record of who left and when, which is exactly the information a right-to-be-forgotten request asks to have destroyed. The task therefore reads only what has not been purged, and offers no way to recover a purged record.

The `statusListIndex` is community-internal correlation metadata. The status list it points into is public by design — that is how a verifier learns a credential is dead — but the mapping from index back to a departed member's DID is not, and this admin-gated task is the only thing that discloses it.
