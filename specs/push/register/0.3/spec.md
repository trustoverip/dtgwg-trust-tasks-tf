---
slug: push/register
version: "0.3"
title: Push — Register
summary: A device registers its platform push token with a push gateway and receives an opaque WakeHandle, naming a controller VTA or — where the gateway admits it — itself as the party that sets who may wake it. The token stays at the gateway.
status: draft
targetFrameworkVersion: "0.6.0"
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
    member: issuer
    identifierScope: pairwise
  - role: push gateway
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Over the DIDComm and TSP bindings the sender is authenticated intrinsically, so a document proof is redundant there. A handle naming a controller VTA is inert until that VTA provisions it, so registration is low-stakes and proof stays RECOMMENDED. A self-controlled handle makes the registering device its controller, so the gateway must know who registered: where no binding authenticates the sender (HTTPS without a DID-bound session), a self-controlled registration carries a proof or is refused (Consumer rule 3).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Registration attaches a push endpoint to the account. Replayed after the endpoint was removed it re-attaches it, and the account holder has no signal that an old document rather than a new decision caused it.
sideEffects:
  level: mutating
  rationale: "Registers a device's push token and mints an opaque WakeHandle, recording its controller and, for a self-controlled handle, its initial trigger allowlist."
exposure:
  discloses: none
  ingests: secret
  actsAsSubject: false
  rationale: >-
    The response hands back only the opaque `wakeHandle`, which reveals no
    token, so nothing confidential is disclosed. Inbound is the exception in
    this family: `registration` is the raw platform push channel — an APNs
    device token with its topic, an FCM registration token, or an RFC 8030 Web
    Push endpoint with its `p256dh` and `auth` secrets. That is credential-grade
    material the gateway must protect on the device's behalf for the life of the
    handle, which is exactly what `ingests: secret` denotes.
retention:
  class: durable
  rationale: >-
    Holding the token *is* the service. The gateway retains the registration
    behind the handle for as long as the handle is live, because a wake it
    cannot deliver is no wake at all. The deletion path is rotation rather than
    expiry: when the platform issues a new token the device re-registers, and a
    gateway that retained superseded registrations instead of dropping them
    would accumulate a dated history of the device's reinstalls and restores.
errorCodes:
  - code: push/register:unsupportedPlatform
    meaning: The gateway does not implement the registration's `platform`. The device falls back to queue-and-wait (no push).
    retryable: false
  - code: push/register:invalidRegistration
    meaning: The platform token / Web Push subscription is malformed.
    retryable: false
  - code: push/register:selfControlRefused
    meaning: The gateway's policy does not admit a self-controlled handle from this device, or no authenticated sender could be established for it. The device registers naming a controller VTA, or falls back to queue-and-wait.
    retryable: false
---

## Abstract

**Push — Register** is how a device hands its platform push token to a **push gateway** and gets back an opaque [`WakeHandle`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeHandle). It is the first step of the [push wake-up binding](../../../../bindings/push/0.1/spec.md)'s three-role model (gateway / trigger / device).

The gateway is the only party that holds the app's platform push credentials (APNs auth key, FCM service account, Web Push VAPID key) and therefore the only one that can deliver a push to the app. In exchange for the token it issues an opaque handle; **the raw token never leaves the gateway**. The device conveys the handle onward — to its VTA via [`device/set-wake/0.2`](../../../device/set-wake/0.2/spec.md), to its own mediator via [`messaging/wake/set`](../../../messaging/wake/set/0.1/spec.md) — but never the token.

The device also names the handle's **controller**, the one DID permitted to set its trigger allowlist ([`push/provision/0.2`](../../provision/0.2/spec.md)). Possession of a handle is not authority to wake the device; the controller's allowlist, enforced by the gateway, is the control. The controller is either:

- **a controller VTA** (`controllerVtaDid`) — the device's own agent, which holds its configuration and provisions the allowlist, as in `0.2`; or
- **the device itself** (`selfControl`) — for a device that has no VTA of its own. A phone that a family member's VTA names as a trusted approver, for example, has a mediator and a wallet but no agent to provision it. It registers as its own controller and names its mediator as a trigger in the same request, so that mediator can ring it when a sender with mail queued for it asks ([`messaging/wake/request`](../../../messaging/wake/request/0.1/spec.md)). A gateway admits a self-controlled handle only where its policy does.

Carried over the **TSP** or **DIDComm** binding (preferred — the sender is authenticated intrinsically) or the HTTPS binding (for devices that can speak neither). The `recipient` is the gateway.

## Changes from 0.2

- **`selfControl`**, new: the device registers as its own controller, with an initial trigger allowlist. Exactly one of `controllerVtaDid` and `selfControl` is present.
- **`controllerVtaDid`** is no longer required (it is one of the two), and is now bounded and constrained to a DID.
- **`push/register:selfControlRefused`**, new: the gateway's policy does not admit a self-controlled handle.

A `0.2` registration is a valid `0.3` registration. The change is not declared an extension of `0.2` because a gateway reading `0.3` as `0.2` would drop `selfControl` and record no controller at all.

## Conformance

A conforming **producer** (the device) **MUST**:

1. Populate `registration` (the platform token) and exactly one of `controllerVtaDid` (the VTA that may provision its allowlist) and `selfControl`.
2. With `selfControl`, send the request so the gateway can authenticate it — over a binding that authenticates the sender, or with a `proof` — and name in `allowedTriggers` only parties it means to let wake it, never itself. Its own mediator is the expected entry.
3. Treat the returned `wakeHandle` as opaque; convey it (never the token) to its controller VTA, or to its mediator, as its configuration requires.
4. Re-register on token rotation and convey the fresh handle (the old one is dropped).

A conforming **consumer** (the push gateway) **MUST**:

1. Reject a `platform` it does not implement → `push/register:unsupportedPlatform`; reject a malformed token → `push/register:invalidRegistration`; reject a document carrying both or neither of `controllerVtaDid` and `selfControl` → `malformedRequest`.
2. Store `token → handle` and issue an **opaque** handle that reveals no token.
3. For `controllerVtaDid`, record it as the only DID permitted to provision this handle's allowlist, and start the handle with an **empty** allowlist — a handle wakes no one until its controller provisions triggers.
4. For `selfControl`:
    1. Establish the registering device's DID from the transport-authenticated sender, or from the verified `proof` where no binding authenticates the sender. Where it can establish neither, refuse → `push/register:selfControlRefused`.
    2. Apply its policy for self-controlled handles, and refuse one it does not admit → `push/register:selfControlRefused`. The policy is the gateway's: it may admit any authenticated device, only devices of its own application, only up to a rate, or none.
    3. Record the device's DID as the handle's controller — so that a later `push/provision` for the handle is accepted from the device and from nobody else — and start the handle with `selfControl.allowedTriggers`, less any entry naming the device itself.
5. **Never** disclose the platform token to any other party (mediator, VTA, or in any response).

## Authorization

**Registration itself presupposes no authorization evidence.** Any device that
can reach the gateway may register a platform token and receive an opaque
handle; the gateway's own policy — rate limiting, platform admission, whatever
it applies — is the only gate, and this specification neither defines nor
constrains it. [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15 asks that this be said explicitly rather than left to be
inferred from silence.

With `controllerVtaDid`, that is safe because a freshly-registered handle is
inert. It starts with an **empty** trigger allowlist and wakes nobody until its
controller provisions triggers, so registration confers reachability, not the
ability to cause anything to happen. The `controllerVtaDid` is authorization
evidence for a **different task**: the gateway records it as the only DID
permitted to provision this handle's allowlist under `push/provision`. Naming it
does not authorize this document — the producer asserts it, and per
[SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 10 an
assertion by the party that benefits from it is not evidence of entitlement.

With `selfControl`, the handle is not inert: it can be rung at once by the
triggers the device named. What makes that acceptable is **whose device it is**.
The authority is the device's control of the push channel it is registering —
evidenced by the platform token it presents, which only the app on that device
can obtain — and the gateway binds the handle to the DID it **authenticated**
for the registration, never to one the payload names. The allowlist lets the
named triggers *ask*; each wake still carries no content and causes nothing but
a device connecting to its own mediator. Whether to admit self-controlled
handles at all is the gateway operator's decision, exercised in Consumer rule
4.2.

## Payload

`registration` (REQUIRED — the [`PushRegistration`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/PushRegistration) platform token); exactly one of `controllerVtaDid` (the VTA allowed to provision this handle's allowlist) and `selfControl` (`{ allowedTriggers }`, the device's initial allowlist, at most four DIDs).

## Request

### A phone with no VTA of its own registers as its own controller

```json
{
  "id": "urn:uuid:a1d4f0c2-7b3e-4f61-9c08-5e2b7d1a9f40",
  "type": "https://trusttasks.org/spec/push/register/0.3",
  "issuer": "did:peer:2.Ez6LSjordanphone.Vz6Mkjordanphone",
  "recipient": "did:web:push.wallet.example",
  "issuedAt": "2026-11-14T08:59:58Z",
  "payload": {
    "registration": {
      "platform": "apns",
      "token": "7c1e5a2b9d4f6e8a0c3b5d7f9e1a3c5b7d9f1e3a5c7b9d1f3e5a7c9b1d3f5e7a",
      "topic": "example.wallet",
      "environment": "production"
    },
    "selfControl": {
      "allowedTriggers": ["did:web:mediator.example"]
    }
  }
}
```

## Response

`wakeHandle` — the opaque [`WakeHandle`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeHandle) (`{ gateway, handle }`).

### The gateway admits the self-controlled handle

```json
{
  "id": "urn:uuid:a1d4f0c2-7b3e-4f61-9c08-5e2b7d1a9f41",
  "type": "https://trusttasks.org/spec/push/register/0.3#response",
  "threadId": "urn:uuid:a1d4f0c2-7b3e-4f61-9c08-5e2b7d1a9f40",
  "issuer": "did:web:push.wallet.example",
  "recipient": "did:peer:2.Ez6LSjordanphone.Vz6Mkjordanphone",
  "issuedAt": "2026-11-14T08:59:59Z",
  "payload": {
    "wakeHandle": {
      "gateway": "did:web:push.wallet.example",
      "handle": "wh_7Q2kR9vXb3LmT0pN"
    }
  }
}
```

The phone then gives the handle to its mediator with [`messaging/wake/set`](../../../messaging/wake/set/0.1/spec.md).

## Security & Privacy

### Data carried

This is the one task in the push family where the platform push token actually
crosses a wire, and it crosses exactly once, to exactly one party. `registration`
is a tagged union over `platform`: for `apns`, the Apple-issued device `token`
with its `topic` and `environment`; for `fcm`, the Firebase registration
`token`; for `webpush`, an RFC 8030 subscription `endpoint` together with the
RFC 8291 `keys.p256dh` and `keys.auth` secrets. Every variant is credential-grade
— a party holding an APNs token and the matching provider credential can push to
that device, and a party holding a Web Push endpoint with its keys can encrypt to
it — which is why the gateway is being asked to *protect* this rather than merely
to record it.

Two members leak more than their purpose requires, and neither is avoidable. An
APNs `topic` is conventionally the application's bundle identifier, and a Web
Push `endpoint` names the browser vendor's push service by hostname, so the
registration tells the gateway which application and which browser the device
runs before any wake has ever been delivered. `controllerVtaDid` is not secret,
but it tells the gateway which VTA this device answers to. `selfControl` instead
tells it the triggers the device allows — normally its mediator — which the
gateway needs for the allowlist it enforces, and the device's own DID, which it
needs to recognise the device as controller later.

The response is deliberately thin: only `wakeHandle`, a pair of `gateway` and an
opaque `handle` that reveals no token. The exchange exists to make that trade —
a secret in, an opaque reference out — so that every downstream party in the
architecture can talk about this device's push channel without holding the
means to push to it. The platform token lives at the gateway alone; no other
party, not the VTA and not the mediator, ever holds it.

### Correlation

The gateway is the correlation point of the entire push design, and it is the
only one. It alone holds both the platform token and the handle, so it alone can
join "this handle" to "this physical device on this vendor's push network".
Every other arrangement in the architecture — the VTA holding a handle without a
token, a mediator triggering by handle — exists so that no second party can make
that join. `controllerVtaDid` gives the gateway a coarser join as well: handles
naming the same controller form a cluster, so a gateway can see that several
devices belong to one principal's VTA without learning whose. A self-controlled
handle gives it the device's DID and its mediator instead, so the gateway can
group the devices that use one mediator. A device that wants to limit that join
**SHOULD** register with a DID it uses for nothing but its mediator account and
this gateway.

The push gateway party declares `identifierScope: public`, and this is the place
in the family where a public identifier is required rather than merely
convenient. Three mutually unrelated parties must address the same gateway by
the same value: the device registering here, the controller VTA that later calls
[`push/provision`](../../provision/0.2/spec.md), and whichever trigger calls
[`push/wake`](../../wake/0.2/spec.md). `wakeHandle.gateway` is that value, and it
travels onward through `device/set-wake` to parties that never spoke to the
device. A pairwise gateway identifier would break the handoff outright: the VTA
would receive a `gateway` it could not resolve to the service the device
actually registered with, and a trigger would have no way to establish it was
talking to the right gateway at all.

That choice is taken with its cost understood. A public gateway identifier makes
the population of devices using a given gateway a visible group, and a device's
*choice* of gateway is itself a signal — a self-hosted gateway distinguishes its
users from the crowd on a large shared one, which is the familiar anonymity-set
trade rather than a defect in this task.

The device declares `identifierScope: pairwise`. Nothing downstream names the
device by its DID; the handle is what later tasks carry, and proof on this
request is only RECOMMENDED, so the gateway has no need to recognise this device
anywhere else.

### Retention

Durable, because retention is the service being purchased. The gateway holds the
registration for as long as the handle is live; a token it discarded would leave
the handle pointing at nothing.

Rotation, not expiry, is the deletion path. When the platform issues a new token
the device re-registers and receives a fresh handle, and the gateway **SHOULD**
drop the superseded registration rather than keep it — a gateway that accumulated
them would hold a dated sequence of a device's reinstalls, restores, and OS
migrations, which no part of this task needs.

The device retains only its handle. The controller VTA never receives the token
at all, so there is nothing there to retain, expire, or leak: the strongest
retention limit in this specification is the one implemented by not sending the
data.

### Consent/purpose

The token is handed over for one purpose — so that the gateway can deliver a
contentless doorbell when an authorised trigger asks for one. The
[push binding](../../../../bindings/push/0.1/spec.md) is explicit that the
delivered wake carries no Trust Task content; this task only sets up the
channel. A gateway that used a registered token to deliver anything else would
be acting outside the purpose the material was given for, and the device would
have no way to detect it. That is why the choice of gateway is a trust decision
rather than a configuration detail.

Registration naming a controller VTA does not, by itself, make the device
wakeable, and the handle it returns is not authority. A handle lets a party
*request* a wake; the gateway fires one only for a DID on the allowlist the
controller provisions, so a leaked handle yields at worst a refused wake, and a
device whose VTA never calls `push/provision` cannot be woken by anyone. A
self-controlled registration makes the device wakeable at once, but only by the
triggers it named itself, and the device can narrow or empty that list at any
time with `push/provision`.
