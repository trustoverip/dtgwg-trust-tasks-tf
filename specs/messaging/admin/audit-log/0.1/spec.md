---
slug: messaging/admin/audit-log
version: "0.1"
title: Messaging — Audit Log
summary: An administrator pages through the mediator's privileged-change audit log, newest first.
status: retired
supersededBy: audit/list
targetFrameworkVersion: "0.2"
category: messaging
keywords:
  - messaging
  - mediator
  - admin
  - read
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
  level: none
  rationale: "Read-only paging of the privileged-change audit log."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - messaging/admin/list
  - messaging/account/change-type
---

## Abstract

The **Messaging — Audit Log** Trust Task pages through the mediator's record of privileged changes (ACL updates, account and admin changes), newest first. The requester **MAY** supply an opaque `cursor` and a `limit`; the mediator returns an array of [`AuditEntry`](../../../_shared/0.1/messaging.schema.json#/$defs/AuditEntry) records and, when more remain, a `nextCursor`. The task is read-only.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by the generic [`audit/list`](../../../../audit/list/0.1/spec.md), which offers the same newest-first cursor paging plus the filters this task never had (`from`/`to`, `action`, `actor`, `outcome`). **Payload-profile mapping:** request `cursor` → `cursor` and `limit` → `pageSize`; each [`AuditEntry`](../../../_shared/0.1/messaging.schema.json#/$defs/AuditEntry) maps onto an `AuditEnvelope` as `timestamp` → `recordedAt`, `actor` → `actor`, `target` → `target`, `action` → `action` (the mediator's `AuditAction` value becomes the maintainer-defined action name), and `detail` → `detail`; the response's `nextCursor` becomes `cursor` + `truncated: true`, and a mediator mints an `eventId` per entry (the one member the legacy shape lacked).

## Conformance

The normative contract is the adjacent [`payload.schema.json`](./payload.schema.json): its top-level object is the request payload and its `#/$defs/Response` (anchor `response`) is the success-response payload. The mediator **MUST** restrict this task to administrator accounts and **MUST** reject an unauthorized requester. Shared shapes are defined in [`messaging.schema.json`](../../../_shared/0.1/messaging.schema.json).
