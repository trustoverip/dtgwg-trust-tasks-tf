---
slug: vtc/relationships/revoke
version: "0.1"
title: VTC Relationships — Revoke
summary: The issuer revokes a Verifiable Relationship Credential they previously published.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - revoke
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
  rationale: Only the credential's issuer may revoke it; the proof signer is checked against the stored issuer, and the revocation must be attributable.
sideEffects:
  level: mutating
  rationale: "Revokes a relationship credential; reversible by publishing a fresh one."
subjectPath: /id
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/revoke:notFound
    meaning: No relationship with the supplied id exists, or the proof signer is not its issuer.
    retryable: false
---

## Abstract

The **VTC Relationships — Revoke** Trust Task revokes a relationship credential `id` previously published via [`vtc/relationships/publish`](../publish/0.1/). Only the original issuer may revoke.

## Conformance

Producer: supply `id`. Carry a proof.

Consumer: resolve the relationship; if absent, or the proof signer is not its issuer, return `notFound` (the same code for both, so a caller cannot probe for relationships it did not issue). Otherwise revoke it and return `{ id }`. Audit the revocation.

## Security & Privacy

**Issuer-gated.** Unknown id and not-your-relationship collapse to one `notFound`, so revoke is not an oracle over others' relationships. Only the issuer, proven by the signer match, can revoke their own attribution.
