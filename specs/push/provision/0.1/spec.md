---
slug: push/provision
version: "0.1"
title: Push — Provision
summary: The controller VTA sets a wake handle's trigger allowlist on the push gateway — the DIDs (its mediator and/or itself) permitted to wake the device. The gateway enforces it.
status: retired
supersededBy: push/provision/0.2
targetFrameworkVersion: "0.1"
category: notifications
keywords:
  - push
  - provision
  - allowlist
  - trigger
  - gateway
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault maintainer
    requirement: REQUIRED
    member: issuer
  - role: push gateway
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Authorization binds to the caller's authenticated identity — the gateway accepts the update only from the handle's controller VTA. Over the DIDComm binding the authcrypt sender provides that identity intrinsically; over HTTPS the caller carries a did-signed proof. Proof is therefore RECOMMENDED (redundant on DIDComm, the auth anchor on HTTPS).
sideEffects:
  level: mutating
  rationale: "Sets a wake handle's trigger allowlist on the gateway; reconfigurable."
subjectPath: /handle
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: push/provision:unknown_handle
    meaning: No such handle at this gateway.
    retryable: false
  - code: push/provision:not_controller
    meaning: The authenticated caller is not the controller VTA recorded for this handle (at push/register). Only the controller may set the allowlist.
    retryable: false
---

## Abstract

**Push — Provision** is how the **controller VTA** sets a wake handle's **trigger allowlist** on the push gateway: the DIDs permitted to request a wake for that handle (typically the device's mediator — queue-driven — and/or the VTA itself — policy-driven). The VTA is the source of truth for this policy ([`device/set-wake/0.1`](../../../device/set-wake/0.1/spec.md) conveys the device's handle to the VTA); this task is how the VTA pushes that policy to the gateway, which **enforces** it on every [`push/wake/0.1`](../../wake/0.1/spec.md).

This is the mechanism that keeps "all device config at the VTA, token at the gateway": the VTA holds the handle + allowlist and provisions the gateway, which holds the token. A wake request from a DID not on the allowlist is refused.

Carried over the **DIDComm binding** (preferred — the authcrypt sender authenticates the VTA) or HTTPS. The `recipient` is the gateway.

## Conformance

A conforming **producer** (the controller VTA) **MUST**:

1. Be the `controllerVtaDid` recorded for the handle at `push/register` — the gateway will reject otherwise.
2. Populate `handle` and `policy` (the [`WakeTriggerPolicy`](../../../device/_shared/0.1/device-binding.schema.json#/$defs/WakeTriggerPolicy) allowlist it computed by its own policy — typically `{ mediator } ∪ { self }`).
3. Re-provision whenever its policy or the device's handle changes; an empty `allowedTriggers` disables waking while the handle exists.

A conforming **consumer** (the push gateway) **MUST**:

1. Resolve `handle` → its record; unknown → `push/provision:unknown_handle`.
2. Verify the authenticated caller equals the handle's recorded `controllerVtaDid`; otherwise `push/provision:not_controller`. (The caller identity comes from the DIDComm authcrypt sender, or the HTTPS did-signed proof.)
3. Replace the handle's allowlist with `policy.allowedTriggers` and enforce it on subsequent `push/wake` requests.

## Payload

`handle` (REQUIRED — the opaque handle from `push/register`); `policy` (REQUIRED — the [`WakeTriggerPolicy`](../../../device/_shared/0.1/device-binding.schema.json#/$defs/WakeTriggerPolicy) allowlist).

## Response

`handle` and the effective `policy` the gateway recorded (so the VTA can confirm what it provisioned).

## Security & Privacy

**Only the controller may provision.** The allowlist is VTA-owned policy; the gateway accepts it only from the handle's recorded controller VTA, authenticated by the transport. A different VTA cannot widen another device's allowlist.

**Allowlist is the wake gate.** The gateway enforces `allowedTriggers` on every wake; the handle alone never authorizes a wake. An empty allowlist means no party may wake the device.

**No token exposure.** Provision deals only in the opaque handle and a list of DIDs — never the platform push token.
