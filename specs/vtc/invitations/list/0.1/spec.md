---
slug: vtc/invitations/list
version: "0.1"
title: VTC Invitations — List
summary: Enumerate a Verifiable Trust Community's issued Invitation Credentials with their issuance and revocation state, without re-disclosing credential material.
status: draft
targetFrameworkVersion: "0.2"
category: credentials
keywords:
  - vtc
  - invitation
  - vic
  - list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: inviter
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only inventory of issued invitations. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the invitation registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/invitations/list:permissionDenied
    meaning: The consumer lacks the inviter capability.
    retryable: false
---

## Abstract

The **VTC Invitations — List** Trust Task enumerates the community's issued Invitation Credentials as [`InvitationSummary`](../../../_shared/0.1/invitation.schema.json) entries — who was invited, by whom, with what role, and whether the credential has been revoked.

It deliberately carries **no credential material**. The signed VIC is returned once by [`vtc/invitations/issue`](../../issue/0.1/) and never again.

Split from the former `invitations/issue` mount, which served both this and issuance under one URI distinguished only by HTTP verb. "Enumerate the registry" and "mint a bearer credential" are different contracts with different exposure — the split makes that legible rather than leaving it to a reader to infer from the method.

## Conformance

Producer: no payload members are required.

Consumer: verify the inviter capability. Return every invitation the community has issued, including revoked ones — the registry is the record of who was admitted and on whose authority. MUST NOT include the credential itself in any entry.

## Security & Privacy

The registry is admin-class metadata: it names invited DIDs, the operators who invited them, and the roles granted. That is sensitive but not secret, which is the whole reason the VIC is excluded — including it would turn a routine listing into a re-disclosure of every live bearer credential the community has ever minted.
