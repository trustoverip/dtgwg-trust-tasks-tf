---
slug: device/set-wake
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — Set Wake
summary: A device tells its VTA the opaque WakeHandle it obtained from a push gateway, so the VTA can own the trigger allowlist and provision the gateway. Idempotent; carries no platform push token.
status: draft
targetFrameworkVersion: "0.2"
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
    identifierScope: pairwise
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Setting the wake channel determines who can cause this device to be woken and what the VTA provisions to the gateway. It is security-significant and infrequent (only on token rotation), so — unlike the high-volume device/heartbeat — it carries a REQUIRED holder proof and is fully audited.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A wake schedule is an overwrite, so an out-of-order copy silently reverts the device to a schedule the owner has already changed. Ordering the two requires a timestamp.
sideEffects:
  level: mutating
  rationale: "Sets the device's opaque WakeHandle on the VTA; idempotent config write."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response returns `triggerPolicy.allowedTriggers` — the DIDs the VTA
    computed and provisioned as permitted to wake this device, typically its
    mediator and the VTA itself — plus whether a usable wake channel now exists.
    That names part of the device's infrastructure to the caller; it is
    descriptive rather than a bearer secret, so `metadata`. Inbound the request
    carries the opaque `wakeHandle`, an advisory `pushPlatform` hint, and a list
    of suggested trigger DIDs — infrastructure descriptors rather than data
    about a person, and deliberately not the platform push token, which never
    leaves the gateway. `ingests` is therefore `metadata`: the handle is a
    bearer reference to *request* a wake, gated at the gateway by the allowlist,
    not confidential material the VTA holds on the device's behalf.
retention:
  class: durable
  rationale: >-
    The handle and the allowlist derived from it are device configuration, held
    at the VTA until the device supersedes them with a fresh handle or clears
    the channel by omitting `wakeHandle`. Nothing here expires on its own. A
    consumer that dropped the handle would leave the device unwakeable and would
    lose the VTA's authoritative copy of who is permitted to wake it — the
    record the gateway's enforcement is provisioned from.
related:
  - device/register
  - device/heartbeat
errorCodes:
  - code: device/set-wake:notRegistered
    meaning: The issuer's DID has no DeviceBinding. The device MUST complete device/register before setting a wake channel.
    retryable: false
  - code: device/set-wake:invalidHandle
    meaning: The supplied WakeHandle is malformed, or the named gateway rejected it as unknown/expired when the VTA attempted to provision the allowlist.
    retryable: false
  - code: device/set-wake:gatewayUnreachable
    meaning: The VTA could not reach or authenticate to the named push gateway to provision the trigger allowlist. The handle is not recorded; the device SHOULD retry.
    retryable: true
---

## Abstract

The **Device — Set Wake** Trust Task is how a Companion conveys its **WakeHandle** to its VTA. Background wake-up uses three roles ([push wake-up binding](../../../../bindings/push/0.1/spec.md)): a **push gateway** that holds the platform push token (APNs/FCM/Web Push) and is the only party that can talk to Apple/Google; one or more **triggers** (the device's mediator and/or its VTA) that decide *when* to wake it; and the **device** itself. The device registers its push token with the gateway and receives an **opaque handle** in return — the token never leaves the gateway.

This task carries that handle — never the token — from the device to its VTA. The VTA is the source of truth for device configuration, so it owns the **trigger allowlist** (which DIDs may wake this device) and **provisions it to the gateway**, which enforces it. Putting the handle on the VTA (config) while the token stays on the gateway (transport) gives the VTA full policy ownership without ever holding the push token.

The task is **idempotent**: a device re-sends it whenever its platform token rotates (and the gateway issues a fresh handle), or with no handle to disable wake. Unlike [`device/register`](../../register/0.1/spec.md) — which is one-shot and rejects re-registration — set-wake is the steady-state update path for the wake channel.

## Conformance

A conforming **producer** (the device) **MUST**:

1. Have completed [`device/register`](../../register/0.1/spec.md) — the issuer's DID MUST already have a DeviceBinding, else `device/set-wake:notRegistered`.
2. Have registered its platform push token with a push gateway and obtained a [`WakeHandle`](../../_shared/0.2/device-binding.schema.json#/$defs/WakeHandle) **before** issuing this task. The device **MUST NOT** place any platform push token in this payload — only the opaque handle.
3. Supply `wakeHandle` to set or replace the wake channel, or omit it to clear the channel (the device becomes non-wakeable; the VTA empties the gateway allowlist).
4. Carry a `proof`.
5. Re-issue this task whenever the gateway issues a new handle (token rotation).

A conforming **consumer** (the VTA / vault maintainer) **MUST**:

1. Verify proof; the producer's DID MUST be in the ACL with a DeviceBinding. If not → `device/set-wake:notRegistered`.
2. Compute the [`WakeTriggerPolicy`](../../_shared/0.2/device-binding.schema.json#/$defs/WakeTriggerPolicy) from its own configuration — **this is VTA-owned policy, not device-supplied.** The default allowlist is the device's mediator DID (queue-driven wake) together with the VTA's own DID (policy-driven wake); operators MAY narrow or widen it by policy. A device-supplied `suggestedTriggers` hint, if present, is advisory only and the VTA MAY ignore it.
3. Provision the allowlist to the gateway named in the handle, authenticating as the VTA. The gateway records `handle → allowedTriggers`. On unreachable/refused gateway → `device/set-wake:gatewayUnreachable` (retryable) or `device/set-wake:invalidHandle` (terminal) per the gateway's response.
4. Record the handle against the DeviceBinding and set `pushCapable = true` (or `false` when cleared). The VTA stores the **handle and the allowlist, never the token**.
5. Return the effective `triggerPolicy` it provisioned, so the device can see who is authorized to wake it.

A consumer **MUST NOT** treat the wake channel as a security boundary for any framework action: a wake is a contentless doorbell ([push binding §2](../../../../bindings/push/0.1/spec.md)). Authorization for the operation the device performs *after* waking (e.g. an `auth/step-up/approve-response`) rests on that document's own proof, not on the fact that a wake occurred.

## Payload

`wakeHandle` (OPTIONAL — present sets/replaces, absent clears); `pushPlatform` (OPTIONAL — the abstract platform kind, advisory, for `device/list` visibility); `suggestedTriggers` (OPTIONAL — advisory hint the VTA MAY ignore).

## Response

`triggerPolicy` — the effective allowlist the VTA provisioned to the gateway; `pushCapable` — whether the device now has a usable wake channel.

## Security & Privacy

### Data carried

What this task carries is best described by what it deliberately does not. The
platform push token — an APNs device token, an FCM registration token, or a Web
Push endpoint with its `p256dh` and `auth` keys — never appears in this document,
never reaches the VTA, and never reaches the mediator. It is held by the push
gateway alone. What crosses this wire instead is `wakeHandle`, a pair of
`gateway` (the DID or https URL of the service that issued it) and `handle` (an
opaque string that reveals no token). A compromised VTA therefore leaks the
handle and the allowlist, not the device's push identity.

The handle is a bearer reference, but a weak one, and the distinction is worth
stating precisely because it is what keeps this task out of the `secret` class:
it lets a party *request* a wake, never read the channel, and the gateway
refuses a request from a DID outside the provisioned allowlist. Possession alone
yields, at worst, a refused wake.

`pushPlatform` is advisory and explicitly non-authoritative — it exists so
`device/list` can show a platform family without the VTA ever seeing a token —
but it does narrow the device: `apns` says Apple hardware. `suggestedTriggers`
is a list of DIDs, typically the device's own mediator, so a producer that
populates it is naming part of its routing infrastructure to its VTA. Both are
optional and both are hints.

The one member whose *absence* is content is `wakeHandle` itself. Omitting it
does not mean "no change"; it clears the wake channel and empties the gateway
allowlist. A producer sending an empty payload is issuing a command, and a
consumer **MUST** read it as one.

### Correlation

Between rotations, `handle` is a stable identifier for this device's push
channel at the gateway. Every wake request the gateway ever receives for this
device names that same handle, so the gateway can assemble a complete traffic
history — how often this device is woken, at what hours, in what bursts — from
the handle alone, without decrypting anything, because there is nothing to
decrypt.

The architecture's answer is a split of knowledge rather than a secret. The VTA
holds the handle and the allowlist but not the token; the gateway holds the
token and the handle but not what the wakes were for. Neither side alone
reconstructs "whose phone was woken, and why". That separation is a deployment
property, not a wire property: nothing in this document prevents one operator
from running both the VTA and the gateway, and an operator that does has
collapsed the split without changing a single member.

Rotation is the other correlation surface. A fresh handle supersedes the prior
one atomically — the VTA re-provisions and the old handle **SHOULD** be dropped
at the gateway — and a VTA that retained superseded handles instead would hold a
dated history of the device's push-channel rotations, which tracks app
reinstalls, device restores, and OS migrations rather than anything this task
needs.

The device party declares `identifierScope: pairwise`. Its DID must be stable
enough for the VTA to attach the handle to the right `DeviceBinding`, but no
third party — not the gateway, which sees only the handle — is asked to
recognise it, and a device reusing one identifier across VTAs would let them
align their views of its wake configuration.

### Retention

Durable, because the handle is configuration rather than a message. The VTA
holds it until the device replaces it or clears it, and holds the derived
`allowedTriggers` for as long as it is the source of truth the gateway is
provisioned from. Nothing here carries an expiry.

Clearing is the deletion path and it is complete by design: omitting
`wakeHandle` empties the gateway allowlist, so no party can wake the device.
`device/disable` and `device/wipe` **SHOULD** clear the wake channel as part of
decommissioning, since a decommissioned device that remains wakeable is a
channel nobody is watching. Superseded handles **SHOULD NOT** be retained after
rotation. The document `id` is the maintainer's idempotency key, so a retry
inside the idempotency window returns the same result rather than
re-provisioning the gateway.

### Consent/purpose

The handle moves for one reason: so that the VTA, not the device, owns the
question of who may wake this device. The device proposes; the VTA decides. That
is why `suggestedTriggers` is advisory and why the schema says the VTA **MAY**
ignore it entirely — the purpose limit is written into the member's own
definition. Without it, a device could authorise an arbitrary third party to
wake it, and device configuration would stop being authoritative at the VTA.

The gateway is the backstop rather than the decision-maker: it authenticates a
trigger's DID and then checks allowlist membership, so the handle alone never
authorises a wake. Reuse of the handle for anything other than provisioning that
allowlist — as a device identifier in an unrelated record, say — is outside the
purpose it was conveyed for, and would give the VTA a correlation key it was
handed for a narrower job.
