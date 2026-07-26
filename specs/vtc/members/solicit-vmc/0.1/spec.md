---
slug: vtc/members/solicit-vmc
version: "0.1"
title: VTC Members — Solicit VMC
summary: Ask the community to request a reciprocal Membership Credential from one of its members; the community dispatches the request and the member answers asynchronously.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - members
  - vmc
  - membership
  - reciprocal
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
  rationale: Triggers an outbound request to a member. Recommended for attribution.
sideEffects:
  level: mutating
  rationale: "Dispatches a request/1.0 message to the member's agent and opens a thread; no credential is created here."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/solicit-vmc:notFound
    meaning: No active member with that DID. Removed members have no membership edge to reciprocate.
    retryable: false
  - code: vtc/members/solicit-vmc:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Members — Solicit VMC** Trust Task is the *operator-facing* half of the reciprocal-membership exchange. An administrator asks the community to go and request a Membership Credential from a named member; the community then sends that member a [`vtc/members/request-vmc`](../../request-vmc/0.1/) message and the member replies, whenever they reply, with [`vtc/members/vmc`](../../vmc/0.1/).

Three tasks, three party pairs. This one is administrator → community. It is **not** the request that reaches the member, and it does not carry a credential.

`requested: true` means the request was dispatched, not that a credential arrived. The returned `threadId` is how a caller correlates the eventual `vtc/members/vmc` delivery with this solicitation.

## Conformance

Producer: name the `memberDid`. `reason` is an operator note relayed to the member ("renewal", "audit").

Consumer: verify the community-admin capability. Refuse a DID that is not an **active** member with `notFound` — a removed member has no membership edge to reciprocate. Dispatch the request and return the `threadId`. MUST NOT block on the member's reply: this task completes when the request is sent.

## Security & Privacy

`requested: true` deliberately claims only that the request left the community. Reporting anything stronger would repeat the "an `Ok` means delivered" mistake this workspace has already been bitten by — a dispatch acknowledgement is not a delivery receipt, and the member's agent may be offline for days.

The operator's `reason` is relayed verbatim to the member, so it is member-visible text, not an internal note.
