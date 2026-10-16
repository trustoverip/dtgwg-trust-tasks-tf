---
slug: vtc/members/update
version: "0.1"
title: VTC Members — Update
summary: Update a community member's role or non-credential metadata (consent withdrawal, departure preference, extensions); refuses promotion to admin and never grants registry consent.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - vtc
  - members
  - community
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
  rationale: Changing a member's role or metadata alters their standing in the community. The change MUST be attributable and non-repudiable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A member update overwrites the member's record, so an out-of-order copy restores details the member has already corrected.
sideEffects:
  level: mutating
  rationale: "Updates a member's role or non-credential metadata; recoverable by updating again."
consequences:
  - "Changes the member's role or metadata, effective immediately."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/update:notFound
    meaning: No member with the supplied `did` exists.
    retryable: false
  - code: vtc/members/update:adminRoleForbidden
    meaning: "`role` was `admin`. Promotion to admin is a separate, gated flow, not a metadata update."
    retryable: false
  - code: vtc/members/update:consentGrantForbidden
    meaning: "The update would set `publishConsent` to true for a member who has not consented. Registry consent is the member's own decision, given on their join request; an update may withdraw it, never grant it. Nothing in the request was applied."
    retryable: false
---

## Abstract

The **VTC Members — Update** Trust Task changes a member's role or non-credential metadata: `role`, `publishConsent`, `departurePreference`, and the opaque `extensions` bag. Every field except `did` is optional — an update carries only what changes. It returns the updated [`MemberResponse`](../../../_shared/0.1/member.schema.json).

It deliberately **cannot** promote a member to `admin`: that is a higher-trust operation with its own approval flow, not a metadata patch, so `role: admin` is refused here.

Nor can it **grant** registry consent. `publishConsent` is the member's own privacy decision: it becomes `true` only by the member's own act: `registryConsent` on [`vtc/join-requests/submit`](../../../join-requests/submit/0.3/spec.md) — including a rejoin, which writes a fresh member record — or [`vtc/members/registry-consent`](../../registry-consent/0.1/spec.md), which the member signs. Approving a join does not consent on the applicant's behalf, and an update does not either. An administrator may **withdraw** consent (`true` → `false`, for example a takedown) and may restate the current value; an update that would raise it (`false` → `true`) is refused with `consentGrantForbidden`.

`label` is an operator-facing display name for the member, editable unlike the
DID it labels. That asymmetry is the point: an operator reviewing a roster of
identifiers they cannot read needs a name they can, and the name is theirs to
correct without touching the identity underneath. `null` clears it; omitting it
leaves it unchanged.

## Conformance

Producer: supply `did` and the fields to change. Carry a proof. **MUST NOT** supply `publishConsent: true` for a member whose record does not already carry it.

Consumer: verify the community-admin capability. Resolve the member; if none, return `notFound`. If `role` is `admin`, return `adminRoleForbidden` and change nothing. If `publishConsent` is `true` and the member record's `publishConsent` is not, return `consentGrantForbidden` and change nothing — no other field in the same request is applied, so a partial update is never mistaken for a full one. Otherwise apply the supplied fields, cap `extensions` at the maintainer's limit, and return the updated `MemberResponse`. Audit the change.

An update that sets `publishConsent` from `true` to `false` withdraws the member's registry consent. The community **SHOULD** then remove the member's published trust-registry record at its next registry sync, and **MUST NOT** publish the member again until consent is given anew by the member. It **MUST** record this document's `issuedAt` as the consent change now in force, so that a member's earlier-issued [`vtc/members/registry-consent`](../../registry-consent/0.1/spec.md) grant arriving later is refused as `superseded` rather than undoing the withdrawal.

## Security & Privacy

**Standing change, so attributable.** A role or metadata change alters the member's standing in the community, so `proofRequirement: REQUIRED` and the change is audited. The `admin`-refusal keeps the highest-privilege transition off this general-purpose patch path.

**Consent runs one way.** Registry publication discloses the member's membership beyond the community, and only the member can agree to that. Letting an administrator raise `publishConsent` would publish someone who said no, or who was never asked; so an update can only lower it or leave it as it is, and the refusal applies the whole request or none of it.
