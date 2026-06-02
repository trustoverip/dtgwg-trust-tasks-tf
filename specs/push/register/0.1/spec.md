---
slug: push/register
version: "0.1"
title: Push — Register
summary: A device registers its platform push token (APNs / FCM / Web Push) with a push gateway and receives an opaque WakeHandle in exchange. The raw token is held by the gateway only.
status: draft
targetFrameworkVersion: "0.1"
category: notifications
keywords:
  - push
  - register
  - gateway
  - wake
  - apns
  - fcm
  - webpush
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: device
    requirement: REQUIRED
  - role: push gateway
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: Over the DIDComm binding the authcrypt sender authenticates the registering device intrinsically, so a document proof is redundant. Over the HTTPS binding a caller MAY carry a did-signed proof. Registration is low-stakes — the issued handle is opaque and useless until the device's VTA provisions a trigger allowlist for it (push/provision) — so proof is RECOMMENDED, not REQUIRED.
errorCodes:
  - code: push/register:unsupported_platform
    meaning: The gateway does not implement the registration's `platform`. The device falls back to queue-and-wait (no push).
    retryable: false
  - code: push/register:invalid_registration
    meaning: The platform token / Web Push subscription is malformed.
    retryable: false
---

## Abstract

**Push — Register** is how a device hands its platform push token to a **push gateway** and gets back an opaque [`WakeHandle`](../../device/_shared/0.1/device-binding.schema.json#/$defs/WakeHandle). It is the first step of the [push wake-up binding](../../../bindings/push/0.1/spec.md)'s three-role model (gateway / trigger / device).

The gateway is the only party that holds the app's platform push credentials (APNs auth key, FCM service account, Web Push VAPID key) and therefore the only one that can deliver a push to the app. In exchange for the token it issues an opaque handle; **the raw token never leaves the gateway**. The device conveys the handle onward — to its VTA via [`device/set-wake/0.1`](../../device/set-wake/0.1/spec.md), and to triggers as the gateway address + handle — but never the token.

The device also names the **controller VTA** — the DID permitted to provision this handle's trigger allowlist ([`push/provision/0.1`](../provision/0.1/spec.md)). Possession of a handle is not authority to wake the device; the VTA-owned allowlist, enforced by the gateway, is the control.

Carried over the **DIDComm binding** (preferred — the authcrypt sender authenticates the device) or the HTTPS binding (for devices that can't speak DIDComm). The `recipient` is the gateway.

## Conformance

A conforming **producer** (the device) **MUST**:

1. Have registered its platform token with a gateway out of band, OR carry the token in `registration` here for the gateway to store.
2. Populate `registration` (the platform token) and `controllerVtaDid` (the VTA that may provision its allowlist).
3. Treat the returned `wakeHandle` as opaque; convey it (never the token) to its VTA and triggers.
4. Re-register on token rotation and convey the fresh handle (the old one is dropped).

A conforming **consumer** (the push gateway) **MUST**:

1. Reject a `platform` it does not implement → `push/register:unsupported_platform`; reject a malformed token → `push/register:invalid_registration`.
2. Store `token → handle` and issue an **opaque** handle that reveals no token. Record the `controllerVtaDid` as the only DID permitted to provision this handle's allowlist.
3. **Never** disclose the platform token to any other party (mediator, VTA, or in any response).
4. Start the handle with an **empty** trigger allowlist — a freshly-registered handle wakes no one until its VTA provisions triggers via `push/provision`.

## Payload

`registration` (REQUIRED — the [`PushRegistration`](../../device/_shared/0.1/device-binding.schema.json#/$defs/PushRegistration) platform token); `controllerVtaDid` (REQUIRED — the VTA allowed to provision this handle's allowlist).

## Response

`wakeHandle` — the opaque [`WakeHandle`](../../device/_shared/0.1/device-binding.schema.json#/$defs/WakeHandle) (`{ gateway, handle }`).

## Security & Privacy

**Token isolation.** The platform push token lives at the gateway alone, behind the opaque handle. No other party — not the VTA, not the mediator — ever holds it.

**Handle is not authority.** A handle lets a party *request* a wake, but the gateway fires one only for a DID on the handle's VTA-provisioned allowlist. A leaked handle yields, at worst, a refused wake.

**Contentless downstream.** The wake the gateway eventually delivers to the device is the [push binding](../../../bindings/push/0.1/spec.md)'s contentless doorbell — it carries no Trust Task content. This task only sets up the channel.
