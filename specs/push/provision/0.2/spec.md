---
slug: push/provision
version: "0.2"
wireCompatibleWith: "0.1"
title: Push — Provision
summary: The controller VTA sets a wake handle's trigger allowlist on the push gateway — the DIDs (its mediator and/or itself) permitted to wake the device. The gateway enforces it.
status: draft
targetFrameworkVersion: "0.5"
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
    identifierScope: public
  - role: push gateway
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: Authorization binds to the caller's authenticated identity — the gateway accepts the update only from the handle's controller VTA. Over the DIDComm binding the authcrypt sender provides that identity intrinsically; over HTTPS the caller carries a did-signed proof. Proof is therefore RECOMMENDED (redundant on DIDComm, the auth anchor on HTTPS).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Provisioning a push channel binds a device token the service will subsequently deliver to. A replayed provision reinstates a token the owner has retired, sending notifications to a device that should no longer receive them.
sideEffects:
  level: mutating
  rationale: "Sets a wake handle's trigger allowlist on the gateway; reconfigurable."
subjectPath: /handle
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response returns `policy.allowedTriggers` — the effective allowlist of
    DIDs the gateway recorded as permitted to wake this handle. `handle` is
    echoed from the request and discloses nothing the caller did not already
    supply; the allowlist is descriptive infrastructure data about the device, so
    `metadata`. Inbound is the same material in the same direction: an opaque
    `handle` the gateway itself minted, and a list of trigger DIDs naming the
    device's mediator and its VTA. Infrastructure descriptors rather than data
    about a person, and no platform push token — which this task never carries —
    so `ingests` is `metadata`.
retention:
  class: durable
  rationale: >-
    The allowlist is live configuration, not a message: the gateway stores it
    and consults it on every `push/wake` for the life of the handle. It is
    replaced wholesale by the next provision and emptied when the VTA sends an
    empty `allowedTriggers`; nothing expires on its own. A gateway that dropped
    it would fail closed — with no list there is nothing for a trigger to match
    and the device becomes unwakeable — so the record is what keeps the wake
    channel working at all.
errorCodes:
  - code: push/provision:unknownHandle
    meaning: No such handle at this gateway.
    retryable: false
  - code: push/provision:notController
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
2. Populate `handle` and `policy` (the [`WakeTriggerPolicy`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeTriggerPolicy) allowlist it computed by its own policy — typically `{ mediator } ∪ { self }`).
3. Re-provision whenever its policy or the device's handle changes; an empty `allowedTriggers` disables waking while the handle exists.

A conforming **consumer** (the push gateway) **MUST**:

1. Resolve `handle` → its record; unknown → `push/provision:unknownHandle`.
2. Verify the authenticated caller equals the handle's recorded `controllerVtaDid`; otherwise `push/provision:notController`. (The caller identity comes from the DIDComm authcrypt sender, or the HTTPS did-signed proof.)
3. Replace the handle's allowlist with `policy.allowedTriggers` and enforce it on subsequent `push/wake` requests.

## Payload

`handle` (REQUIRED — the opaque handle from `push/register`); `policy` (REQUIRED — the [`WakeTriggerPolicy`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeTriggerPolicy) allowlist).

## Response

`handle` and the effective `policy` the gateway recorded (so the VTA can confirm what it provisioned).

## Security & Privacy

### Data carried

Two members, and neither describes a person. `handle` is the opaque reference
the gateway itself minted at [`push/register`](../../register/0.2/spec.md), so
the caller is telling the gateway nothing it did not already know.
`policy.allowedTriggers` is the substance: a list of DIDs, typically the
device's mediator and the VTA itself, naming who may cause this device to be
woken.

What that list amounts to is a small map of the device's routing
infrastructure, handed to the gateway. It is not secret, but it is the one thing
the gateway learns here, and it is worth being precise that the gateway needs it
only for a comparison, never for contact.

The absence of entries is content rather than omission: an empty
`allowedTriggers` disables waking for that handle entirely, which is the
supported way to turn a device's push channel off without surrendering the
handle.

Provision deals only in the opaque handle and a list of DIDs. The platform push
token appears nowhere in this task, and the VTA issuing it has never held one —
so the party configuring the wake channel is structurally incapable of using it
directly.

### Correlation

The gateway learns which mediator serves this device, and it learns that for
every handle it hosts. Nothing here names a principal, but the associations are
stable and they accumulate: a gateway ends up holding a map of which mediators
its device population routes through, and — via the `controllerVtaDid` recorded
at registration — which handles a single VTA controls. Those clusters are a
by-product of the design rather than a purpose of it, and a gateway operator has
them whether or not it wants them.

Both parties declare `identifierScope: public`, and neither declaration is
optional given how the family fits together. The **push gateway** must be
addressable by the same value in three different hands — the device that
registered, this VTA, and every trigger that later calls `push/wake` — because
`wakeHandle.gateway` is conveyed onward through `device/set-wake` to parties
that never spoke to the device; a pairwise gateway identifier would leave the
trigger unable to establish it was talking to the right gateway at all. The
**controller VTA** is public for the same structural reason in the other
direction: the identifier it authenticates as here **MUST** be the same value
the device named as `controllerVtaDid` at registration, and the same value a
trigger presents at wake time when the VTA is on its own allowlist. Recognition
of one identifier by parties that share no pairwise relationship is exactly what
a pairwise identifier cannot supply.

The cost of both is the same and should be stated rather than assumed: a
gateway, and anyone who compromises one, can see which handles belong to which
VTA, and which VTAs use which mediators. That is a topology, not a set of
people, but topologies are stable and people are attached to them.

### Retention

Durable, because the allowlist is configuration the gateway must consult on
every wake rather than a message it processes once. It lives for the life of the
handle, is replaced wholesale by the next provision, and is emptied — not
expired — when the VTA sends an empty list.

Superseded allowlists **SHOULD NOT** be retained. Keeping them would give the
gateway a dated history of a device's changing mediator relationships, which is
a record this task never asked to create and which the current list already
supersedes for every operational purpose.

### Consent/purpose

Only the controller may provision. The allowlist is VTA-owned policy, and the
gateway accepts an update to it only from the handle's recorded controller VTA,
authenticated by the transport — over DIDComm intrinsically by the authcrypt
sender, over HTTPS by a did-signed proof. A different VTA cannot widen another
device's allowlist, and that single rule is what makes a handle safe to
circulate at all.

The allowlist is the wake gate rather than a hint. The gateway enforces
`allowedTriggers` on every `push/wake`, so the handle alone never authorises a
wake, and an empty allowlist means no party may wake the device however many
handles they hold.

The purpose limit follows from what the gateway needs: these DIDs are supplied
so it can perform one string comparison at wake time. A gateway that read the
same list as a directory — using it to discover which mediators to approach, or
selling the map of which VTAs use which infrastructure — would be reusing an
access-control list as an intelligence source, which is outside what a
controller VTA sends it for.
