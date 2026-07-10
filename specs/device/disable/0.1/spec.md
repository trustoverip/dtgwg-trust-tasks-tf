---
slug: device/disable
version: "0.1"
title: Device — Disable
summary: Disable a Companion or Service; the maintainer revokes its ACL entry and refuses subsequent authentication, but does not actively instruct the device to wipe its cache.
status: draft
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - device
  - disable
  - revoke
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
  requirement: REQUIRED
  rationale: Disable is a state-changing operation that withdraws access — high-trust, audited.
sideEffects:
  level: mutating
  rationale: "Revokes a device's ACL entry and refuses its authentication; recoverable by re-enrolling."
subjectPath: /deviceId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: device/disable:not_found
    meaning: No DeviceBinding with this id (or the consumer cannot see it).
    retryable: false
  - code: device/disable:permission_denied
    meaning: The consumer lacks DeviceAdmin capability.
    retryable: false
  - code: device/disable:already_disabled
    meaning: The device is already disabled (idempotent — maintainers MAY return success with the original disabledAt instead).
    retryable: false
---

## Abstract

The **Device — Disable** Trust Task revokes a registered device's access without actively wiping its cache. Use it when you want to passively neutralise a device (e.g. a stale ChromeBook you've stopped using) but don't have evidence of compromise. For active wipe, use `device/wipe/0.1`.

## Conformance

Producer: populate `deviceId`. Consumer: verify `DeviceAdmin`, set `disabledAt = now`, revoke the device's ACL entry, expire its refresh tokens, emit `sync/event/0.1` of kind `acl.changed` with `change: "device_disabled"`, return the disabledAt time.

Disable is idempotent; a retry on an already-disabled device returns the original `disabledAt`.

## Payload

`deviceId` (REQUIRED), `reason` (optional, audit-logged).

## Response

`deviceId`, `disabledAt`.

## Security & Privacy

**Defense in depth.** Disabling revokes server-side access; it does NOT instruct the device to wipe. A still-connected device sees its tokens expire and is locked out from re-authentication. If you suspect compromise, use `device/wipe/0.1`.

**Audit.** Logged with `{ who disabled, when, deviceId, reason? }`.
