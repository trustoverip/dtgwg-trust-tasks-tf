---
slug: push/register
version: "0.2"
wireCompatibleWith: "0.1"
title: Push — Register
summary: A device registers its platform push token (APNs / FCM / Web Push) with a push gateway and receives an opaque WakeHandle in exchange. The raw token is held by the gateway only.
status: draft
targetFrameworkVersion: "0.5"
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
  rationale: Over the DIDComm binding the authcrypt sender authenticates the registering device intrinsically, so a document proof is redundant. Over the HTTPS binding a caller MAY carry a did-signed proof. Registration is low-stakes — the issued handle is opaque and useless until the device's VTA provisions a trigger allowlist for it (push/provision) — so proof is RECOMMENDED, not REQUIRED.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Registration attaches a push endpoint to the account. Replayed after the endpoint was removed it re-attaches it, and the account holder has no signal that an old document rather than a new decision caused it.
sideEffects:
  level: mutating
  rationale: "Registers a device's push token and mints an opaque WakeHandle."
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
---

## Abstract

**Push — Register** is how a device hands its platform push token to a **push gateway** and gets back an opaque [`WakeHandle`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeHandle). It is the first step of the [push wake-up binding](../../../../bindings/push/0.1/spec.md)'s three-role model (gateway / trigger / device).

The gateway is the only party that holds the app's platform push credentials (APNs auth key, FCM service account, Web Push VAPID key) and therefore the only one that can deliver a push to the app. In exchange for the token it issues an opaque handle; **the raw token never leaves the gateway**. The device conveys the handle onward — to its VTA via [`device/set-wake/0.1`](../../../device/set-wake/0.1/spec.md), and to triggers as the gateway address + handle — but never the token.

The device also names the **controller VTA** — the DID permitted to provision this handle's trigger allowlist ([`push/provision/0.1`](../../provision/0.1/spec.md)). Possession of a handle is not authority to wake the device; the VTA-owned allowlist, enforced by the gateway, is the control.

Carried over the **DIDComm binding** (preferred — the authcrypt sender authenticates the device) or the HTTPS binding (for devices that can't speak DIDComm). The `recipient` is the gateway.

## Conformance

A conforming **producer** (the device) **MUST**:

1. Have registered its platform token with a gateway out of band, OR carry the token in `registration` here for the gateway to store.
2. Populate `registration` (the platform token) and `controllerVtaDid` (the VTA that may provision its allowlist).
3. Treat the returned `wakeHandle` as opaque; convey it (never the token) to its VTA and triggers.
4. Re-register on token rotation and convey the fresh handle (the old one is dropped).

A conforming **consumer** (the push gateway) **MUST**:

1. Reject a `platform` it does not implement → `push/register:unsupportedPlatform`; reject a malformed token → `push/register:invalidRegistration`.
2. Store `token → handle` and issue an **opaque** handle that reveals no token. Record the `controllerVtaDid` as the only DID permitted to provision this handle's allowlist.
3. **Never** disclose the platform token to any other party (mediator, VTA, or in any response).
4. Start the handle with an **empty** trigger allowlist — a freshly-registered handle wakes no one until its VTA provisions triggers via `push/provision`.

## Authorization

*Stated in anticipation of [SPEC.md §7.3](/SPEC.md#73-specification-requirements)
item 15, which binds specifications targeting framework 0.4; this specification
targets 0.2, where the declaration is not yet required.*

**Registration itself presupposes no authorization evidence.** Any device that
can reach the gateway may register a platform token and receive an opaque
handle; the gateway's own policy — rate limiting, platform admission, whatever
it applies — is the only gate, and this specification neither defines nor
constrains it. Item 15 asks that this be said explicitly rather than left to be
inferred from silence.

That is safe only because a freshly-registered handle is inert. It starts with
an **empty** trigger allowlist and wakes nobody until its VTA provisions
triggers, so registration confers reachability, not the ability to cause
anything to happen.

The `controllerVtaDid` carried here is authorization evidence for a **different
task**: the gateway records it as the only DID permitted to provision this
handle's allowlist under `push/provision`. Naming it in this document does not
authorize this document — the producer asserts it, and per
[SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 10 an
assertion by the party that benefits from it is not evidence of entitlement.
It is stored as the gate for a later exchange, and that exchange is where it is
checked.

## Payload

`registration` (REQUIRED — the [`PushRegistration`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/PushRegistration) platform token); `controllerVtaDid` (REQUIRED — the VTA allowed to provision this handle's allowlist).

## Response

`wakeHandle` — the opaque [`WakeHandle`](../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeHandle) (`{ gateway, handle }`).

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
but it tells the gateway which VTA this device answers to.

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
devices belong to one principal's VTA without learning whose.

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

Registration does not, by itself, make the device wakeable, and the handle it
returns is not authority. A handle lets a party *request* a wake; the gateway
fires one only for a DID on the allowlist the controller VTA provisions
separately, so a leaked handle yields at worst a refused wake, and a device that
registers here but whose VTA never calls `push/provision` cannot be woken by
anyone.
