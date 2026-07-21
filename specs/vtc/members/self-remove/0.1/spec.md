---
slug: vtc/members/self-remove
version: "0.1"
title: VTC Members — Self-Remove
summary: A member removes themselves from a Verifiable Trust Community, choosing how their record is disposed of.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - community
  - self-remove
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A member leaving the community is a significant, self-initiated action that MUST be attributable and non-repudiable.
sideEffects:
  level: destructive
  rationale: "With disposition purge the member's record is irreversibly erased; tombstone/historical are recoverable, but the task can destroy data, so it declares the strongest class."
consequences:
  - "Removes the caller from the community; with disposition purge this is irreversible."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: "The subject is the authenticated caller acting on their own membership — the task exercises the member's own authority over their own record."
errorCodes:
  - code: vtc/members/self-remove:notMember
    meaning: The authenticated caller is not a member of this community, so there is nothing to remove.
    retryable: false
---

## Abstract

The **VTC Members — Self-Remove** Trust Task removes the calling member from the community. The subject is the authenticated caller — there is no target field — so `actsAsSubject` is true. The optional `disposition` chooses how the record is handled (`purge` erases it, `tombstone`/`historical` retain a marker); omitting it uses the community's policy default, and the response reports the concrete disposition applied.

## Conformance

Producer: optionally supply `disposition`. Carry a proof.

Consumer: verify the caller is a member; if not, return `notMember`. Apply the disposition (resolving `policydefault` to a concrete one), remove the member, and return `{ did, disposition, removed }`. Audit the departure.

## Security & Privacy

**Self-initiated and destructive.** Because a member can `purge` their own record irreversibly, the action is proof-REQUIRED and audited. The audit row survives even a purge, so the departure is never untraceable.
