---
slug: vtc/policies/test
version: "0.1"
title: VTC Policies — Test
summary: Evaluate a stored policy module against caller-supplied input without activating it, returning the raw result of the queried rule.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - policy
  - rego
  - dry-run
  - evaluate
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
  rationale: Read-only evaluation against an uploaded module. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Evaluates a stored module in isolation; the active policy and community state are untouched."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vtc/policies/test:notFound
    meaning: No policy module with that id.
    retryable: false
  - code: vtc/policies/test:evaluationFailed
    meaning: The module failed to evaluate — a compile error, or the queried rule does not exist.
    retryable: false
  - code: vtc/policies/test:permissionDenied
    meaning: The consumer lacks the community-admin capability.
    retryable: false
---

## Abstract

The **VTC Policies — Test** Trust Task evaluates an uploaded policy module against input the caller supplies, and returns whatever the queried rule produced. It is the dry-run that belongs between [`policy/upsert`](../../../../policy/upsert/0.2/spec.md) and activation.

Two things distinguish it from canonical policy evaluation. `query` names **any** rule in the module, not just an `allow` decision — an author debugging a policy needs to see intermediate rules. And `input` is deliberately schema-free: a community's policies decide over community-shaped facts (a membership application, a removal request), and those do not fit a fixed evaluation model.

## Conformance

Producer: name the module `id`, the `query` rule, and the `input` document to evaluate against.

Consumer: verify the community-admin capability. Evaluate the **stored** module identified by `id` — never the active policy, and never a module supplied inline. Return the rule's raw result in `result` without interpreting it: a policy under test may legitimately return an object, a list, or nothing at all, and coercing that to a verdict would defeat the purpose. Report a compile error or a missing rule as `evaluationFailed` rather than an empty result, which a caller would read as a deliberate deny.

Testing MUST NOT activate the module or affect any live decision.

## Security & Privacy

The isolation is the safety property. An administrator must be able to run an untrusted, half-written policy against realistic input without any chance of it deciding a real membership question — so evaluation reads a stored module and writes nothing.

`result` is returned uninterpreted, which means a policy that leaks data into its output will leak it here. That is acceptable for an admin-gated authoring tool and is exactly what makes it useful for debugging, but it is the reason this task is not exposed more widely.
