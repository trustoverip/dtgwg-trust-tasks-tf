---
slug: vtc/community/profile/update
version: "0.1"
title: VTC Community Profile — Update
summary: Update the community's public profile (name, description, contact, language, extensions); a partial patch, refuses to set the read-only registry status.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - community
  - profile
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
  rationale: Editing the community's public identity is a governance change and MUST be attributable and non-repudiable.
sideEffects:
  level: mutating
  rationale: "Overwrites the supplied profile fields; recoverable by updating again."
consequences:
  - "Changes the community's public-facing profile, effective immediately."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/community/profile/update:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/community/profile/update:validationFailed
    meaning: A supplied field failed validation (e.g. empty name, malformed URL).
    retryable: false
---

## Abstract

The **VTC Community Profile — Update** Trust Task edits the community's public
[`CommunityProfile`](../../../../_shared/0.1/community.schema.json). It is a
**partial patch**: only the fields present in the request are changed; omitted
fields are left as-is. The nullable fields (`logoUrl`, `publicUrl`,
`contactEmail`) may be explicitly set to `null` to clear them.

`registryStatus` is **not** accepted in the request — it is live operational
state, not a settable profile field. The response returns the full updated
`CommunityProfile`, mirroring `vtc/community/profile/show`.

## Conformance

Producer: send any subset of the mutable fields. Send `null` to clear a
nullable field; omit a field to leave it unchanged.

Consumer: verify the community-admin capability. Validate supplied fields,
apply the patch, and return the full updated `CommunityProfile` (with live
`registryStatus`).

## Security & Privacy

**Admin-class mutation** (`discloses: metadata`). Behind the community-admin
gate; the framework proof makes each edit attributable.
