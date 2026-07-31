---
slug: device/list
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — List
summary: List DeviceBindings (Companions and Services) registered on the maintainer, optionally filtered by kind, capability, status, or last-seen time.
status: draft
targetFrameworkVersion: "0.2"
category: identity
keywords:
  - device
  - list
  - inventory
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
  rationale: Read-only inventory query.
sideEffects:
  level: none
  rationale: "Read-only listing of registered device bindings."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: device/list:permissionDenied
    meaning: The consumer lacks visibility into the device inventory.
    retryable: false
---

## Abstract

The **Device — List** Trust Task drives the "my devices" UI: every Companion the user has registered, plus Services authorised on their VTA. Used by the user-facing device-manager to spot unfamiliar devices, disable lost ones, and audit AI-agent presence.

## Conformance

Producer: optional filters; treat `cursor` as opaque. Consumer: scope returned devices to those the requesting consumer can see (admin sees all; Service consumers see only themselves unless explicitly granted broader visibility).

## Payload

All optional: `consumerKindFilter`, `formFactorFilter`, `serviceKindFilter`, `capabilityFilter`, `includeDisabled`, `includeWiped`, `lastSeenSince`, pagination.

## Response

`devices` — list of DeviceBinding. Recommended order: `lastSeenAt` descending.

## Security & Privacy

Same considerations as `vault/list`: enumeration of inventory is itself information. Admin-class Companions see all; Service consumers see only their own record unless granted device-admin capability.
