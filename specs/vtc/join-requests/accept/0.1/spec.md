---
slug: vtc/join-requests/accept
version: "0.1"
title: VTC Join-Requests — Accept
summary: An approved applicant completes membership by issuing the reciprocal credential back to the community.
status: retired
supersededBy: vtc/members/vmc
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - join-requests
  - onboarding
  - accept
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
  rationale: The document proof authenticates the accepting member — its signer DID is the member DID, replacing the pre-migration REST signature.
sideEffects:
  level: mutating
  rationale: "Records the member's reciprocal credential, completing the bidirectional membership edge."
subjectPath: /requestId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/join-requests/accept:notFound
    meaning: No join request with the supplied requestId exists.
    retryable: false
  - code: vtc/join-requests/accept:notApproved
    meaning: The request has not been approved, so there is nothing to reciprocate.
    retryable: false
  - code: vtc/join-requests/accept:credentialInvalid
    meaning: The reciprocal Verifiable Credential failed verification or its issuer did not match the proof signer.
    retryable: false
---

## Abstract

> **Retired.** Superseded by [`vtc/members/vmc`](../../../members/vmc/0.1/), whose optional `requestId` carries this task's request-closing semantics. Both tasks delivered the member-issued reciprocal credential to the community to close the bidirectional membership edge; accept was vmc plus a request-state transition, so the registry keeps one credential-delivery path instead of two.

The **VTC Join-Requests — Accept** Trust Task completes an approved join: the new member issues a reciprocal Verifiable Credential (`vc`) back to the community, acknowledging the membership credential (`vmcId`) the community issued them. This forms the bidirectional membership edge. The member identity is the proof signer.

## Conformance

Producer: supply `requestId`, `vmcId`, and the reciprocal `vc` (its issuer MUST equal the proof signer). Carry a proof.

Consumer: resolve the request (`notFound` if absent); if it is not approved, return `notApproved`. Verify the reciprocal credential; if it fails or the issuer mismatches the signer, return `credentialInvalid`. Otherwise store it and return `{ requestId, status: accepted, reciprocalVcId }`.

## Security & Privacy

**Reciprocity closes the edge.** The community issued the membership credential at approval; this task records the member's countersigned acknowledgement, so the trust edge is mutual and both directions are attributable to their signers.
