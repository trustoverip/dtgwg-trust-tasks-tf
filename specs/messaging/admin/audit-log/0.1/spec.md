---
slug: messaging/admin/audit-log
version: "0.1"
title: Messaging — Audit Log
summary: An administrator pages through the mediator's privileged-change audit log, newest first.
status: draft
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
errorCodes: []
related:
  - messaging/admin/list
  - messaging/account/change-type
---

## Abstract

The **Messaging — Audit Log** Trust Task pages through the mediator's record of privileged changes (ACL updates, account and admin changes), newest first. The requester **MAY** supply an opaque `cursor` and a `limit`; the mediator returns an array of [`AuditEntry`](../../../_shared/0.1/messaging.schema.json#/$defs/AuditEntry) records and, when more remain, a `nextCursor`. The task is read-only.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The normative contract is the adjacent [`payload.schema.json`](./payload.schema.json): its top-level object is the request payload and its `#/$defs/Response` (anchor `response`) is the success-response payload. The mediator **MUST** restrict this task to administrator accounts and **MUST** reject an unauthorized requester. Shared shapes are defined in [`messaging.schema.json`](../../../_shared/0.1/messaging.schema.json).
