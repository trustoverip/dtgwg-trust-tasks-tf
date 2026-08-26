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
`CommunityProfile`, mirroring `vtc/community/profile/show`, alongside
`fieldsChanged`.

`relationshipIdentifierDefault` declares which identifier form the community
expects members to issue relationship credentials under — `attributed` (the
member's membership DID, so an edge names them) or `pairwise` (a relationship
DID unique to each counterparty). It is a **declaration, not an enforcement**:
the member still chooses per relationship, and a community that wants to
require one form does so in its own policy. A client reads it before minting so
it can follow the community's expectation without being told twice.

`fieldsChanged` names the members this update actually changed, in their wire
spelling, and is empty when the submitted values all matched what was stored.
It lets a caller tell a no-op from an applied change without diffing the
returned profile against the one it sent.

## Conformance

Producer: send any subset of the mutable fields. Send `null` to clear a
nullable field; omit a field to leave it unchanged.

Consumer: verify the community-admin capability. Validate supplied fields,
apply the patch, and return the full updated `CommunityProfile` (with live
`registryStatus`) together with `fieldsChanged`. Reject a
`relationshipIdentifierDefault` outside the two defined values rather than
storing it: the value is published to clients, and one they cannot interpret is
worse than no declaration at all.

## Security & Privacy

**Admin-class mutation** (`discloses: metadata`). Behind the community-admin
gate; the framework proof makes each edit attributable.
