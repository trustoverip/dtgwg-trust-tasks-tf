---
slug: vtc/members/request-vmc
version: "0.1"
title: VTC Members — Request VMC
summary: A community asks one of its members to issue and return a reciprocal Membership Credential naming the community as subject.
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
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
  - role: member
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The member must be able to confirm the request genuinely came from the community named as the credential's subject.
sideEffects:
  level: none
  rationale: "A request. The member decides whether to answer; nothing is persisted on receipt."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/members/request-vmc:notAMember
    meaning: The recipient does not consider itself a member of the requesting community.
    retryable: false
---

## Abstract

The **VTC Members — Request VMC** Trust Task is the message a community sends **to a member**, asking them to issue a Membership Credential whose `credentialSubject.id` is `communityDid` and to return it as [`vtc/members/vmc`](../../vmc/0.1/).

This is the member-facing half of the exchange an administrator starts with [`vtc/members/solicit-vmc`](../../solicit-vmc/0.1/). The two are separate tasks because they are separate interfaces between different pairs of parties — conflating them would put an operator surface and a wire message behind one contract.

Membership is reciprocal: the community issues the member a credential, and the member issues one back naming the community. This task solicits that second half.

## Conformance

Producer (the community): set `communityDid` to the DID the member's credential must name as its subject. `reason` is operator-supplied text surfaced to the member.

Consumer (the member): verify the request came from the community it names — the proof requirement is REQUIRED precisely because `communityDid` is the subject the member is being asked to attest to, and an unauthenticated request is an invitation to issue a credential to an impostor. Answering is **discretionary**: a member may decline, and declining is not an error. If the recipient does not consider itself a member, respond `notAMember`.

## Security & Privacy

The proof requirement is the whole security story. A member receiving this message is being asked to sign an attestation about the requester; without a proven sender, anyone could solicit a Membership Credential naming any DID they liked as subject.

Because answering is discretionary and asynchronous, a community MUST NOT treat silence as refusal or as membership lapse — the member may simply be offline.
