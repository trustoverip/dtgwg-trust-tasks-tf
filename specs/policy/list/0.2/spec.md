---
slug: policy/list
version: "0.2"
title: Policy — List
summary: List Rego policy modules registered on the maintainer, optionally filtered by context or enablement status.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - policy
  - rego
  - opa
  - authorization
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only inventory.
sideEffects:
  level: none
  rationale: "Read-only listing of registered policy modules."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: policy/list:permissionDenied
    meaning: The consumer lacks PolicyAdmin capability.
    retryable: false
---

## Abstract

The **Policy — List** Trust Task returns the registered Rego policy modules. Drives the policy-editor UI in the browser plugin.

## Conformance

Producer: optional filters. Consumer: verify `PolicyAdmin` (or read-only `policy-read` if such a capability is defined later). Return policies in `priority` descending order, then `updatedAt` descending.

## Security & Privacy

**Source disclosure.** Rego sources are NOT secrets, but they describe the maintainer's security posture and SHOULD be visible only to admin-class consumers.

**Auditing list calls.** Recommended at sampled rate.
