---
slug: vtc/relationships/list
version: "0.1"
title: VTC Relationships — List
summary: List the Verifiable Relationship Credentials published about a community member.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
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
  rationale: Read-only listing of a member's relationships. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the relationship store; persists nothing."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/list:permissionDenied
    meaning: The consumer lacks the capability to read this member's relationships.
    retryable: false
  - code: vtc/relationships/list:notFound
    meaning: No member with the supplied did exists.
    retryable: false
---

## Abstract

The **VTC Relationships — List** Trust Task returns the Verifiable Relationship Credentials recorded for a member `did` — each with its `id`, `issuerDid`/`subjectDid`, the `vrcJsonld` body, a `vrcSha256`, and `createdAt`. Paged by `cursor`/`limit`.

## Conformance

Producer: supply `did`; optionally `cursor`/`limit`.

Consumer: resolve the member (`notFound` if absent). Return the relationships where the member is issuer or subject, clamping `limit` to 1..=200 and setting `nextCursor` when more remain.

## Security & Privacy

**Relationship metadata.** The entries name the related DIDs and carry the credential bodies — community-visible metadata behind the read gate.
