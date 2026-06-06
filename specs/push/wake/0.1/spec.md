---
slug: push/wake
version: "0.1"
title: Push — Wake
summary: A trigger (the device's mediator or its VTA) asks the push gateway to deliver a contentless wake to a handle. The gateway authorizes against the VTA-provisioned allowlist, then fires the doorbell.
status: draft
targetFrameworkVersion: "0.1"
category: notifications
keywords:
  - push
  - wake
  - trigger
  - gateway
  - doorbell
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: trigger
    requirement: REQUIRED
    member: issuer
  - role: push gateway
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: The gateway authorizes the wake against the handle's allowlist, so the trigger's authenticated identity is load-bearing. Over the DIDComm binding the authcrypt sender provides it intrinsically; over HTTPS the caller carries a did-signed proof. A spoofed/replayed wake is harmless (a contentless doorbell — the device connects and finds the same or empty queue), so proof is RECOMMENDED, not REQUIRED.
errorCodes:
  - code: push/wake:unknown_handle
    meaning: No such handle at this gateway.
    retryable: false
  - code: push/wake:not_allowed
    meaning: The authenticated trigger DID is not on the handle's VTA-provisioned allowlist.
    retryable: false
  - code: push/wake:token_unregistered
    meaning: The push service reports the handle's token permanently unregistered. The gateway drops the token; the device must re-register (push/register). The queued message remains for the consumer's next voluntary pickup.
    retryable: false
---

## Abstract

**Push — Wake** is how a **trigger** — the device's **mediator** (queue-driven: it alone knows the device is offline with messages waiting) or its **VTA** (policy-driven: e.g. a delegated step-up) — asks the gateway to deliver a **contentless wake** to a device. The gateway authorizes the request against the handle's VTA-provisioned allowlist ([`push/provision/0.1`](../provision/0.1/spec.md)) and, if the trigger is allowed, fires the doorbell defined by the [push wake-up binding](../../../bindings/push/0.1/spec.md) §2 (gateway → device, via APNs / FCM / Web Push).

**This task carries only the binding's contentless hint fields** — `v`, and optionally `mediator` / `count` / `urgency`. It **MUST NOT** carry any Trust Task content, `reason`, relying-party identity, or task type: the wake is a doorbell, and the actual messages are drained from the mediator over the DIDComm binding after the device wakes.

Carried over the **DIDComm binding** (preferred — the authcrypt sender authenticates the trigger; this is how a `did:webvh` mediator or VTA authenticates) or HTTPS. The `recipient` is the gateway.

## Conformance

A conforming **producer** (the trigger) **MUST**:

1. Be on the handle's allowlist (a mediator or VTA the controller VTA provisioned).
2. Populate `handle` and `v`; OPTIONALLY `mediator` (so a multi-mediator consumer knows which to drain), `count`, `urgency`.
3. Carry **no** task content — only the fields above.
4. A **mediator** SHOULD fire only when the consumer's pickup queue is non-empty *and* it is offline, and SHOULD coalesce multiple queued messages into at most one wake per short window. A **VTA** fires on its own policy decision and MAY wake an already-connected device (a harmless redundant doorbell).

A conforming **consumer** (the push gateway) **MUST**:

1. Resolve `handle`; unknown → `push/wake:unknown_handle`.
2. Verify the authenticated trigger DID is on the handle's allowlist; otherwise `push/wake:not_allowed` (no push sent).
3. Deliver a push containing **only** the binding §2 contentless fields — never the handle, never task content.
4. On a push-service "permanently unregistered" report, drop the stored token and return `push/wake:token_unregistered`, leaving the queued message for the consumer's next pickup.

## Payload

`handle` (REQUIRED); `v` (REQUIRED — binding wire version, currently `1`); `mediator`, `count`, `urgency` (OPTIONAL hints). No other fields.

## Response

`status` — `delivered` (the push service accepted the wake) or `token-unregistered` (handled per the error above; included for symmetry where the gateway reports outcome in-band).

## Security & Privacy

**Allowlist-gated.** Only a DID the controller VTA put on the handle's allowlist can wake the device; the gateway authenticates the trigger (DIDComm sender / HTTPS proof) before checking membership. This bounds abuse — a party that obtains a handle still cannot wake the device.

**Contentless.** The delivered push carries no Trust Task content (binding §2). The wake reveals only *that* a wake occurred, to the push provider — deployments handling sensitive flows SHOULD coalesce + jitter and MUST NOT vary the payload by task type.

**Harmless on spoof/replay.** A forged or replayed wake, at worst, makes the device connect to its mediator and find the same or an empty queue — a wasted wake, not a security event. The framework relies on the DIDComm authcrypt envelope at pickup for confidentiality and sender authentication, never on the wake.
