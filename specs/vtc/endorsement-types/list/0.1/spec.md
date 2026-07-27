---
slug: vtc/endorsement-types/list
version: "0.1"
title: VTC Endorsement-Types — List
summary: List the endorsement types a community recognises.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords: [vtc, endorsements, endorsement-types, list]
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
  rationale: Read-only listing of registered endorsement types. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the endorsement-type registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsement-types/list:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Endorsement-Types — List** Trust Task returns the endorsement types the community recognises, as [`EndorsementType`](../../../_shared/0.1/endorsement-type.schema.json) entries, paged by `cursor`/`limit`.

## Conformance

Producer: optional `cursor`/`limit`.

Consumer: verify the community-admin capability. Return the registered types, `nextCursor` when more remain.

## Security & Privacy

**Registry metadata.** Discloses the recognised type URIs and their schemas — `metadata`, behind the admin gate.
