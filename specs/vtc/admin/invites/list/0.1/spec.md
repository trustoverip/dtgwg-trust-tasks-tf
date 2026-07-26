---
slug: vtc/admin/invites/list
version: "0.1"
title: VTC Admin Invites — List
summary: Enumerate a Verifiable Trust Community's administrator invites, outstanding and historical, with their redemption state.
status: draft
targetFrameworkVersion: "0.2"
category: access-control
keywords:
  - vtc
  - admin
  - invite
  - passkey
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
  rationale: Read-only inventory of administrator invites. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the invite registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/admin/invites/list:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Admin Invites — List** Trust Task enumerates the community's administrator invites as [`InviteSummary`](../../../../_shared/0.1/invite.schema.json) entries — outstanding, consumed, and expired alike.

It is the read half of the pair with [`vtc/admin/invites/create`](../../create/0.1/); [`vtc/admin/invites/revoke`](../../revoke/0.1/) acts on an entry's `jti`.

This task and `create` were previously one collapsed mount (`admin/invites/manage`) distinguished only by HTTP verb. They are split here because a Trust Task URI is a contract, and "enumerate" and "mint a credential-bearing URL" are not the same contract.

## Conformance

Producer: no payload members are required.

Consumer: verify the community-admin capability. Return every invite the community holds, including terminal ones — the consumed rows are the audit trail of who was granted administrator access and when.

## Security & Privacy

Invite records name the DIDs granted administrator access, so the list is admin-class metadata. It deliberately does **not** disclose the claim code or install URL: those are minted once by `create` and never retrievable afterwards, so a leaked list cannot be replayed into an enrolment.
