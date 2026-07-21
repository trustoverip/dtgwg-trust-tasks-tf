---
slug: vtc/members/admin-remove
version: "0.1"
title: VTC Members — Admin-Remove
summary: An administrator removes a member from the community, choosing the record disposition and recording a reason.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - members
  - community
  - admin-remove
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
  rationale: An admin removing another member is a high-trust, irreversible-capable action; it MUST be attributable to the operator who ordered it.
sideEffects:
  level: destructive
  rationale: "With disposition purge the member's record is irreversibly erased; the task can destroy data, so it declares the strongest class."
consequences:
  - "Removes the target member; with disposition purge this is irreversible."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/members/admin-remove:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
  - code: vtc/members/admin-remove:notFound
    meaning: No member with the supplied did exists.
    retryable: false
---

## Abstract

The **VTC Members — Admin-Remove** Trust Task lets an administrator remove another member, identified by `did`, with an optional `disposition` and an operator `reason` recorded in the audit trail. The counterpart to [`vtc/members/self-remove`](../self-remove/0.1/) for the admin-initiated case.

## Conformance

Producer: supply `did`; optionally `disposition` and `reason`. Carry a proof.

Consumer: verify the community-admin capability. Resolve the member; if none, return `notFound`. Apply the disposition, remove the member, and return `{ did, disposition, removed }`. Audit the removal with the operator `reason`.

## Security & Privacy

**High-trust and destructive.** Removing another member — irreversibly under `purge` — is proof-REQUIRED and audited with the operator's reason. In deployments that require a second approver for member removal, the maintainer gates this task behind the community's confirm flow; that gate is applied at the enforcement point and is orthogonal to this payload.
