---
slug: vtc/relationships/publish
version: "0.1"
title: VTC Relationships — Publish
summary: A member publishes a Verifiable Relationship Credential asserting a relationship to another community member.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - publish
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
  rationale: The publisher is the document proof signer, which MUST equal the VRC's issuer; publishing a relationship attribution must be attributable.
sideEffects:
  level: mutating
  rationale: "Stores a relationship credential; reversible via relationships/revoke."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/publish:vrcInvalid
    meaning: The VRC failed verification, or its issuer did not match the proof signer.
    retryable: false
  - code: vtc/relationships/publish:subjectNotMember
    meaning: The credentialSubject.id is not a member of this community.
    retryable: false
---

## Abstract

The **VTC Relationships — Publish** Trust Task records a member-issued Verifiable Relationship Credential (`vrc`) asserting a relationship to another member. The community verifies and stores it, returning its `id`, the `issuerDid`/`subjectDid`, and a `vrcSha256` for out-of-band integrity. Revoke via [`vtc/relationships/revoke`](../../revoke/0.1/).

## Conformance

Producer: supply the signed `vrc`; its issuer MUST equal the proof signer. Carry a proof.

Consumer: verify the proof and the VRC; if the VRC fails or its issuer mismatches the signer, return `vrcInvalid`. Confirm the `credentialSubject.id` is a member (`subjectNotMember` otherwise). Store the VRC and return `{ id, issuerDid, subjectDid, vrcSha256 }`.

## Security & Privacy

**Self-attested, attributable.** A relationship is a claim by its issuer, bound to the proof signer, so it cannot be forged in another member's name. The subject is a member but does not countersign here — the credential is the issuer's assertion.
