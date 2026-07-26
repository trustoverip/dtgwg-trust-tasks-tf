---
slug: vtc/recognition/check
version: "0.1"
title: VTC Recognition — Check
summary: Ask whether a Verifiable Trust Community recognises a foreign DID, and whether the trust registry backing that answer is actually configured.
status: draft
targetFrameworkVersion: "0.2"
category: reputation
keywords:
  - vtc
  - recognition
  - cross-community
  - trust-registry
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
  rationale: Read-only recognition probe. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Consults the recognition state; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/recognition/check:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Recognition — Check** Trust Task answers whether this community recognises a foreign `did` — the precondition for that identity obtaining a cross-community session via [`vtc/auth/recognise`](../../../auth/recognise/0.1/).

It returns `registryConfigured` alongside `recognised`, and that second field is the point of the task. Recognition is backed by a trust registry; if none is configured, `recognised: false` means "cannot say", not "not recognised". Collapsing the two would let a misconfiguration read as a deliberate refusal.

## Conformance

Producer: name the foreign `did` to probe.

Consumer: verify the community-admin capability. Return `recognised` from the recognition state and `registryConfigured` reflecting whether a registry is actually wired up. When `registryConfigured` is `false`, `recognised` MUST be `false` and the caller MUST treat it as indeterminate rather than negative. Surface any lookup failure in `error` rather than folding it into the boolean — a registry that is configured but unreachable is a third state again.

## Security & Privacy

This is an admin-gated probe because it discloses which foreign identities the community has a recognition relationship with — a map of its inter-community trust that is not otherwise public.

The three-way distinction (`recognised` / not configured / errored) exists so an operator debugging a failed cross-community login can tell a policy decision from an infrastructure problem. Reducing it to one boolean would make those two indistinguishable at exactly the moment the difference matters.
