---
slug: messaging/admin/config
version: "0.1"
title: Messaging — Get Configuration
summary: An administrator reads the mediator's current configuration and software version.
status: retired
supersededBy: config/show
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
  rationale: "Read-only read of the mediator's configuration and version."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - messaging/admin/list
  - messaging/account/change-type
---

## Abstract

The **Messaging — Get Configuration** Trust Task returns the mediator's software `version` and its current `config` as a JSON object. The configuration members are mediator-implementation-specific and opaque to this specification. The task is read-only.

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels): it is no longer recommended for new use and is preserved so already-issued documents remain verifiable. It is superseded by the generic [`config/show`](../../../../config/show/0.1/spec.md), which is strictly stronger: per-key values with a source layer and restart semantics, plus a `keys` filter, versus this task's opaque whole-config blob. **Payload-profile mapping:** an empty request maps to `config/show` with `keys` omitted (every registered key); each member of this task's `config` object becomes a `ConfigField` (`key` = the member path, `value` = its value, `source` = a mediator-defined label such as `mediator`, `requiresRestart` per the key's reload semantics); the `version` member is exposed as an ordinary key (e.g. `mediator.version`). The successor's secret-redaction rule applies to secret-bearing keys the blob would have leaked.

## Conformance

The normative contract is the adjacent [`payload.schema.json`](./payload.schema.json): its top-level object is the request payload and its `#/$defs/Response` (anchor `response`) is the success-response payload. The mediator **MUST** restrict this task to administrator accounts and **MUST** reject an unauthorized requester. Shared shapes are defined in [`messaging.schema.json`](../../../_shared/0.1/messaging.schema.json).
