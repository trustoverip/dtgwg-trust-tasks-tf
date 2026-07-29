---
slug: messaging/admin/strip
version: "0.1"
title: Messaging — Strip Admins
summary: An administrator removes admin rights from one or more accounts.
status: retired
supersededBy: messaging/account/update
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - admin
  - role
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Mediator
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The mediator authenticates and authorizes the request at the transport layer (the request is admin-gated); a proof is RECOMMENDED to additionally bind the request to its requester for audit, but is not required.
sideEffects:
  level: mutating
  rationale: "Removes admin rights; re-grantable via admin/add."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related:
  - messaging/admin/list
  - messaging/account/change-type
---

## Abstract

The **Messaging — Strip Admins** Trust Task demotes one or more administrator accounts back to standard. The requester names the target accounts in `dids`; the mediator removes admin rights from each and returns the set stripped. Only an existing administrator may perform this task.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by [`messaging/account/update`](../../../account/update/0.1/spec.md) — one `{ did, accountType: "standard" }` update per account (this task's batching is dropped; each demotion becomes its own signed, auditable document). The successor is proof-**REQUIRED**, which closes an inconsistency this task embodied: it performed the same role change as `messaging/account/change-type` — a proof-REQUIRED task — while itself only RECOMMENDING proof.

## Conformance

The normative contract is the adjacent [`payload.schema.json`](./payload.schema.json): its top-level object is the request payload and its `#/$defs/Response` (anchor `response`) is the success-response payload. The mediator **MUST** restrict this task to administrator accounts and **MUST** reject an unauthorized requester. Shared shapes are defined in [`messaging.schema.json`](../../../_shared/0.1/messaging.schema.json).
