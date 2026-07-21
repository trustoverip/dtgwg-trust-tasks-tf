---
slug: vtc/endorsement-types/register
version: "0.1"
title: VTC Endorsement-Types — Register
summary: Register an endorsement type a community will recognise, optionally with a claim schema.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, endorsements, endorsement-types, register]
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
  rationale: Registering a recognised endorsement type changes what the community will accept; it MUST be attributable.
sideEffects:
  level: mutating
  rationale: "Adds an endorsement type to the community's registry."
subjectPath: /typeUri
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsement-types/register:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/endorsement-types/register:reserved
    meaning: The typeUri is a workspace-reserved URI (e.g. CommunityRole) and cannot be registered.
    retryable: false
  - code: vtc/endorsement-types/register:exists
    meaning: An endorsement type with this typeUri is already registered.
    retryable: false
  - code: vtc/endorsement-types/register:invalidUri
    meaning: The typeUri is empty or exceeds 512 bytes.
    retryable: false
---

## Abstract

The **VTC Endorsement-Types — Register** Trust Task adds an endorsement type the community will recognise, keyed by `typeUri`, with an optional `description` and `claimSchema`. It returns the stored [`EndorsementType`](../../_shared/0.1/endorsement-type.schema.json).

## Conformance

Producer: supply `typeUri`; optionally `description` and `claimSchema`. Carry a proof.

Consumer: verify the community-admin capability. Refuse reserved URIs (`reserved`), duplicates (`exists`), and empty/oversized URIs (`invalidUri`). Otherwise store the type and return the full `EndorsementType`.

## Security & Privacy

**Widens what the community accepts.** Registering a type expands the endorsements the community will honour, so the change is proof-REQUIRED and audited.
