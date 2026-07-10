---
slug: device/set-wake
version: "0.1"
title: Device — Set Wake
summary: A device tells its VTA the opaque WakeHandle it obtained from a push gateway, so the VTA can own the trigger allowlist and provision the gateway. Idempotent; carries no platform push token.
status: draft
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - device
  - push
  - wake
  - notification
  - gateway
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: device
    requirement: REQUIRED
    member: issuer
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Setting the wake channel determines who can cause this device to be woken and what the VTA provisions to the gateway. It is security-significant and infrequent (only on token rotation), so — unlike the high-volume device/heartbeat — it carries a REQUIRED holder proof and is fully audited.
sideEffects:
  level: mutating
  rationale: "Sets the device's opaque WakeHandle on the VTA; idempotent config write."
exposure:
  discloses: none
  actsAsSubject: false
related:
  - device/register
  - device/heartbeat
errorCodes:
  - code: device/set-wake:not_registered
    meaning: The issuer's DID has no DeviceBinding. The device MUST complete device/register before setting a wake channel.
    retryable: false
  - code: device/set-wake:invalid_handle
    meaning: The supplied WakeHandle is malformed, or the named gateway rejected it as unknown/expired when the VTA attempted to provision the allowlist.
    retryable: false
  - code: device/set-wake:gateway_unreachable
    meaning: The VTA could not reach or authenticate to the named push gateway to provision the trigger allowlist. The handle is not recorded; the device SHOULD retry.
    retryable: true
---

## Abstract

The **Device — Set Wake** Trust Task is how a Companion conveys its **WakeHandle** to its VTA. Background wake-up uses three roles ([push wake-up binding](../../../../bindings/push/0.1/spec.md)): a **push gateway** that holds the platform push token (APNs/FCM/Web Push) and is the only party that can talk to Apple/Google; one or more **triggers** (the device's mediator and/or its VTA) that decide *when* to wake it; and the **device** itself. The device registers its push token with the gateway and receives an **opaque handle** in return — the token never leaves the gateway.

This task carries that handle — never the token — from the device to its VTA. The VTA is the source of truth for device configuration, so it owns the **trigger allowlist** (which DIDs may wake this device) and **provisions it to the gateway**, which enforces it. Putting the handle on the VTA (config) while the token stays on the gateway (transport) gives the VTA full policy ownership without ever holding the push token.

The task is **idempotent**: a device re-sends it whenever its platform token rotates (and the gateway issues a fresh handle), or with no handle to disable wake. Unlike [`device/register`](../register/0.1/spec.md) — which is one-shot and rejects re-registration — set-wake is the steady-state update path for the wake channel.

## Conformance

A conforming **producer** (the device) **MUST**:

1. Have completed [`device/register`](../register/0.1/spec.md) — the issuer's DID MUST already have a DeviceBinding, else `device/set-wake:not_registered`.
2. Have registered its platform push token with a push gateway and obtained a [`WakeHandle`](../_shared/0.1/device-binding.schema.json#/$defs/WakeHandle) **before** issuing this task. The device **MUST NOT** place any platform push token in this payload — only the opaque handle.
3. Supply `wakeHandle` to set or replace the wake channel, or omit it to clear the channel (the device becomes non-wakeable; the VTA empties the gateway allowlist).
4. Carry a `proof`.
5. Re-issue this task whenever the gateway issues a new handle (token rotation).

A conforming **consumer** (the VTA / vault maintainer) **MUST**:

1. Verify proof; the producer's DID MUST be in the ACL with a DeviceBinding. If not → `device/set-wake:not_registered`.
2. Compute the [`WakeTriggerPolicy`](../_shared/0.1/device-binding.schema.json#/$defs/WakeTriggerPolicy) from its own configuration — **this is VTA-owned policy, not device-supplied.** The default allowlist is the device's mediator DID (queue-driven wake) together with the VTA's own DID (policy-driven wake); operators MAY narrow or widen it by policy. A device-supplied `suggestedTriggers` hint, if present, is advisory only and the VTA MAY ignore it.
3. Provision the allowlist to the gateway named in the handle, authenticating as the VTA. The gateway records `handle → allowedTriggers`. On unreachable/refused gateway → `device/set-wake:gateway_unreachable` (retryable) or `device/set-wake:invalid_handle` (terminal) per the gateway's response.
4. Record the handle against the DeviceBinding and set `pushCapable = true` (or `false` when cleared). The VTA stores the **handle and the allowlist, never the token**.
5. Return the effective `triggerPolicy` it provisioned, so the device can see who is authorized to wake it.

A consumer **MUST NOT** treat the wake channel as a security boundary for any framework action: a wake is a contentless doorbell ([push binding §2](../../../../bindings/push/0.1/spec.md)). Authorization for the operation the device performs *after* waking (e.g. an `auth/step-up/approve-response`) rests on that document's own proof, not on the fact that a wake occurred.

## Payload

`wakeHandle` (OPTIONAL — present sets/replaces, absent clears); `pushPlatform` (OPTIONAL — the abstract platform kind, advisory, for `device/list` visibility); `suggestedTriggers` (OPTIONAL — advisory hint the VTA MAY ignore).

## Response

`triggerPolicy` — the effective allowlist the VTA provisioned to the gateway; `pushCapable` — whether the device now has a usable wake channel.

## Security & Privacy

**Token isolation.** The platform push token never appears in this task, on the VTA, or on the mediator — it is held by the gateway alone, behind the opaque handle. A compromised VTA leaks the handle and the allowlist, not the device's push identity (the token / APNs-FCM identifier).

**VTA owns the allowlist.** Who may wake the device is VTA policy, not device assertion. The device proposes a handle; the VTA decides the triggers. This keeps all device configuration state authoritative at the VTA and prevents a device from authorizing an arbitrary third party to wake it.

**Gateway enforcement.** The gateway refuses a wake from a DID not on the provisioned allowlist, and authenticates the trigger's DID first. The handle alone does not authorize a wake — allowlist membership does.

**Rotation and revocation.** Re-issuing with a fresh handle atomically supersedes the prior one (the VTA re-provisions; the old handle SHOULD be dropped at the gateway). Clearing (omitting `wakeHandle`) empties the gateway allowlist so no party can wake the device. `device/disable` and `device/wipe` SHOULD also clear the wake channel as part of decommissioning.

**Replay.** The `id` is the maintainer's idempotency key; a retry of the same id within the idempotency window returns the same result without re-provisioning.
