---
slug: messaging/admin/add
version: "0.1"
title: Messaging — Add Admins
summary: An administrator grants admin rights to one or more accounts at the mediator.
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
  rationale: "Grants admin rights at the mediator; revocable via admin/strip."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes: []
related:
  - messaging/admin/list
  - messaging/account/change-type
---

## Abstract

The **Messaging — Add Admins** Trust Task promotes one or more accounts to administrators. The requester names the target accounts in `dids`; the mediator grants each admin rights and returns the set now holding them. Only an existing administrator may perform this task.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by [`messaging/account/update`](../../../account/update/0.1/spec.md) — one `{ did, accountType: "admin" }` update per account (this task's batching is dropped; each grant becomes its own signed, auditable document). The successor is proof-**REQUIRED**, which closes an inconsistency this task embodied: it granted the same privilege as `messaging/account/change-type` — a proof-REQUIRED task — while itself only RECOMMENDING proof.

## Conformance

The normative contract is the adjacent [`payload.schema.json`](./payload.schema.json): its top-level object is the request payload and its `#/$defs/Response` (anchor `response`) is the success-response payload. The mediator **MUST** restrict this task to administrator accounts and **MUST** reject an unauthorized requester. Shared shapes are defined in [`messaging.schema.json`](../../../_shared/0.1/messaging.schema.json).
