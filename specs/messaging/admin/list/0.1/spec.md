---
slug: messaging/admin/list
version: "0.1"
title: Messaging — List Admins
summary: An administrator lists the mediator's administrator accounts, paginated by an opaque cursor.
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

The **Messaging — List Admins** Trust Task enumerates the administrator accounts at the mediator. The requester **MAY** supply an opaque `cursor` to continue a previous enumeration and a `limit` to bound the page; the mediator returns an array of [`AdminAccount`](../../../_shared/0.1/messaging.schema.json#/$defs/AdminAccount) views and, when more remain, a `nextCursor`. The task is read-only.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The normative contract is the adjacent [`payload.schema.json`](./payload.schema.json): its top-level object is the request payload and its `#/$defs/Response` (anchor `response`) is the success-response payload. The mediator **MUST** restrict this task to administrator accounts and **MUST** reject an unauthorized requester. Shared shapes are defined in [`messaging.schema.json`](../../../_shared/0.1/messaging.schema.json).
