---
slug: vtc/endorsement-types/delete
version: "0.1"
title: VTC Endorsement-Types — Delete
summary: Remove an endorsement type from a community's registry; refused while live endorsements of the type exist.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, endorsements, endorsement-types, delete]
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
  rationale: Removing a recognised endorsement type changes what the community accepts; it MUST be attributable.
sideEffects:
  level: destructive
  rationale: "Removes an endorsement type from the registry; recoverable only by re-registering it."
subjectPath: /typeUri
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsement-types/delete:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/endorsement-types/delete:notFound
    meaning: No endorsement type with the supplied typeUri is registered.
    retryable: false
  - code: vtc/endorsement-types/delete:inUse
    meaning: At least one live endorsement of this type still exists. Revoke all live endorsements before deleting the type.
    retryable: false
---

## Abstract

The **VTC Endorsement-Types — Delete** Trust Task removes an endorsement type `typeUri` from the registry. It is refused while any live endorsement of the type remains — the no-orphans precondition.

## Conformance

Producer: supply `typeUri`. Carry a proof.

Consumer: verify the community-admin capability. Resolve the type (`notFound` if absent); if any live endorsement of it exists, return `inUse` and delete nothing. Otherwise remove the type and return `{ typeUri }`. Audit the deletion.

## Security & Privacy

**No-orphans deletion.** The `inUse` refusal guarantees no live endorsement is left pointing at a deleted type. The removal is proof-REQUIRED and audited.
