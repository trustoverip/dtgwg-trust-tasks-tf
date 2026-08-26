---
slug: vtc/website/generations/list
version: "0.1"
title: VTC Website — Generations List
summary: List a managed-mode website's deploy generations, marking the current one.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords: [vtc, website, generations, list]
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
  rationale: Read-only listing of deploy generations. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Enumerates the deploy generations; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/website/generations/list:notManaged
    meaning: The website is in live mode, which has no generations. Managed mode only.
    retryable: false
---

## Abstract

The **VTC Website — Generations List** Trust Task enumerates a managed-mode website's deploy generations, marking the one `current` resolves to. Managed mode only.

## Conformance

Producer: send with no parameters.

Consumer: verify the community-admin capability. In live mode return `notManaged`. Otherwise return each generation with a `current` flag.

## Security & Privacy

**Deploy metadata.** Discloses generation labels, not content — `metadata`, behind the admin gate.
