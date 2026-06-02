---
slug: push
version: "0.1"
title: Push wake-up transport binding
summary: A contentless wake-up notification (APNs / FCM / Web Push) sent by a push gateway on behalf of a trigger — the device's mediator or its VTA — telling a backgrounded consumer to connect to its mediator and drain queued DIDComm-carried Trust Task documents.
status: draft
targetFrameworkVersion: "0.1"
bindingURI: https://trusttasks.org/binding/push/0.1
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how a *consumer* that cannot hold a live connection to its mediator — a mobile app suspended by the operating system, a desktop agent that is not running — is told that *Trust Task documents* are waiting for it. It is a **wake-up notification binding, not a document-carriage binding.** The documents themselves are carried by the [DIDComm binding](../../didcomm/0.1/spec.md) and retrieved from the mediator with DIDComm message-pickup (`https://didcomm.org/messagepickup/3.0`); this binding defines only the out-of-band signal that prompts the retrieval, the parties that send and authorize it, and the registration by which a consumer becomes wakeable.

The motivating case is the mobile authenticator. A phone is sent an [`auth/step-up/approve-request`](../../../specs/auth/step-up/approve-request/0.1/spec.md), shows the user the `reason`, and returns an [`approve-response`](../../../specs/auth/step-up/approve-response/0.1/spec.md). For that to work while the app is backgrounded, something must wake it. A phone cannot keep a WebSocket open across OS suspension, so a **push gateway** sends a notification through the platform's push service (APNs, FCM, Web Push); the app wakes, authenticates to its mediator, and drains its queue over the DIDComm binding as usual.

### Three roles

Background wake-up involves three roles. Keeping them distinct is the whole point of this revision:

| Role | Holds | Does |
|------|-------|------|
| **Push gateway** | the *app's* platform push credentials (APNs auth key, FCM service account, Web Push VAPID key) and the `handle → push token` map | the only party that can talk to Apple / Google / a Web Push service for this app. Issues an **opaque handle** for a registered token, enforces a VTA-provisioned allowlist, and relays a **contentless** push. Operated by the app publisher. |
| **Trigger** | a [`WakeHandle`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeHandle) (an opaque handle + the gateway's address) | decides *when* to wake the device and asks the gateway to do so. A trigger is either the device's **mediator** (queue-driven: it alone knows the device is offline with messages waiting) or its **VTA** (policy-driven: e.g. a step-up it is delegating to this device). A device MAY authorize both. |
| **Device** (consumer) | its platform push token | registers the token with the gateway, receives an opaque handle, and conveys that handle to the parties that route its wake (its mediator, via `set-device-info`; its VTA, via [`device/set-wake`](../../../specs/device/set-wake/0.1/spec.md)). |

The credential reality drives the split. APNs and FCM credentials are bound to the **app**, not to any server operator: only the holder of the app's Team key (APNs) or Firebase project (FCM) can push to it. Neither an individual VTA nor a generic shared mediator holds those keys — the **app publisher** does, via the gateway. So the gateway is a necessary third service; the open question this binding answers is only *who is authorized to ask it to wake a device*, and the answer is a per-device, **VTA-owned allowlist** ([§3.3](#33-trigger-allowlist-vta-owned)).

> This is a draft-stage revision of the `0.1` binding. The earlier draft folded the gateway into the mediator (the mediator held the token and pushed directly, per Aries RFC 0699/0734). That only works when the mediator *is* app-aware and holds the app's push key — untrue for a generic shared mediator. Separating the gateway out, and making the trigger a per-device allowlist of `{mediator, VTA}`, fixes that while preserving the contentless-doorbell guarantee unchanged.

## Status of This Document

`0.1` draft. Tracks `SPEC.md 0.1` and the [DIDComm binding `0.1`](../../didcomm/0.1/spec.md), on which it depends. The wake-handle conveyance from device to VTA is the [`device/set-wake/0.1`](../../../specs/device/set-wake/0.1/spec.md) Trust Task; the handle and allowlist shapes are [`WakeHandle`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeHandle) and [`WakeTriggerPolicy`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeTriggerPolicy).

## 1. Binding URI

| Resource           | URI                                                |
|--------------------|----------------------------------------------------|
| Binding identifier | `https://trusttasks.org/binding/push/0.1`          |

The binding URI does not appear on the wire. Like the HTTPS binding, push has no envelope `type` field; the URI is the stable identifier for this binding in registries and cross-references. Per [SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace) it is **not** a *Type URI* and **MUST NOT** appear in a *Trust Task document*'s `type` member.

## 2. What this binding carries — and what it must not

A push notification under this binding is a **doorbell**. Its payload **MUST NOT** contain any *Trust Task document*, any `payload` field, the `reason` of a step-up, the identity of a relying party, or any other task content. Push notifications transit a third-party push service (Apple, Google) and may be logged, displayed on a lock screen, or retained outside the trust boundary; treating them as a content channel leaks the very data the DIDComm authcrypt envelope exists to protect.

A conforming push notification (gateway → device) **MAY** carry only:

| Field         | Meaning                                                                                                   |
|---------------|-----------------------------------------------------------------------------------------------------------|
| `v`           | The integer `1` — this binding's wire version.                                                            |
| `mediator`    | The DID of the mediator holding the queued messages (so a consumer enrolled with several knows which to drain). |
| `count`       | OPTIONAL. An approximate count of queued messages. Advisory only — the consumer learns the true set from pickup. |
| `urgency`     | OPTIONAL. `"interactive"` or `"background"` — a hint the consumer MAY map to the platform's priority/alert behavior. |

A consumer **MUST** treat every field of the push payload as an **untrusted hint**. The authoritative message set, sender identity, and content come only from authenticated DIDComm pickup ([§4](#4-delivery-flows)). A consumer **MUST NOT** display task content from a push payload and **MUST NOT** take any framework action on the strength of a push alone.

> A consumer **MAY** show a generic, content-free local notification ("You have a pending approval") *after* it has woken and authenticated, deriving the wording from the drained, decrypted documents — never from the push payload.

The handle is **never** placed in the push payload (gateway → device); it travels only on the *trigger → gateway* request, where the gateway resolves it to a token internally ([§3.4](#34-triggering-a-wake)).

## 3. Roles and registration

### 3.1 Push gateway

The push gateway is the only component holding the app's platform push credentials, and therefore the only one that can deliver a push to the app. It:

1. Accepts a device's [`PushRegistration`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/PushRegistration) (the platform token) and issues an **opaque [`WakeHandle`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeHandle)** in exchange. It stores `handle → token` and **never discloses the token** to any other party.
2. Accepts a **trigger allowlist** for each handle, provisioned by that device's VTA ([§3.3](#33-trigger-allowlist-vta-owned)), and **enforces** it.
3. On a wake request, authenticates the requesting trigger's DID, checks it against the handle's allowlist, and — if allowed — sends the contentless push of [§2](#2-what-this-binding-carries--and-what-it-must-not) to the mapped token.
4. Is reachable by triggers and devices over either the [HTTPS binding](../../https/0.1/spec.md) (a REST gateway) **or** the [DIDComm binding](../../didcomm/0.1/spec.md) (a DID-addressable gateway). Its address — an https URL or a DID — is the `gateway` field of the handle.

The gateway is normally operated by the **app publisher**, because the push credentials are the app's (this is the [Matrix Sygnal](https://github.com/matrix-org/sygnal) topology: the app vendor runs the gateway; servers ask it to notify). It learns *that* a handle was woken and *when* — traffic-analysis metadata — but never task content (contentless) and never which trigger's *intent* lay behind the wake beyond the trigger's DID.

### 3.2 Device registration with the gateway

The device sends its platform token to the gateway and receives a handle:

```
Device ──PushRegistration (apns|fcm|webpush token)──▶ Gateway
Device ◀──────────── WakeHandle { gateway, handle } ─────────── Gateway
```

The token stays at the gateway. The device then distributes the **handle** (never the token) to the parties that route its wake:

| Recipient | Mechanism | Why |
|-----------|-----------|-----|
| Mediator | DIDComm `set-device-info` (the Aries RFC 0699/0734 exchange, now carrying a **`WakeHandle`** in place of the raw token) | so the mediator can address a wake when it sees the queue go non-empty for an offline consumer. |
| VTA | [`device/set-wake/0.1`](../../../specs/device/set-wake/0.1/spec.md) Trust Task | so the VTA can own the allowlist, provision the gateway, and itself trigger policy-driven wakes. |

Because the gateway enforces the VTA-owned allowlist ([§3.3](#33-trigger-allowlist-vta-owned)), **possession of a handle is not by itself authority to wake** — so handing the handle to the mediator over `set-device-info` is not a privilege grant. Allowlist membership, set by the VTA, is the control.

Tokens rotate. A device **MUST** re-register with the gateway on a new platform token, obtain a fresh handle, and re-convey it (new `set-device-info`; new `device/set-wake`). The gateway **MUST** treat the most recent registration as authoritative and drop the prior token; a push to a token the push service reports as permanently unregistered **MUST** cause the gateway to drop the stored token and report the handle dead, leaving any queued message for the consumer's next voluntary pickup.

### 3.3 Trigger allowlist (VTA-owned)

Which DIDs may wake a device is **VTA policy**, not a device assertion or a gateway default — all device configuration state is authoritative at the VTA. The VTA computes a [`WakeTriggerPolicy`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeTriggerPolicy) for each handle and **provisions it to the gateway**, which enforces it:

```
Device ──device/set-wake (WakeHandle)──▶ VTA
                                          │  computes allowlist (its policy)
                                          ▼
                          Gateway ◀──provision: handle → allowedTriggers──  VTA
```

The default allowlist is `{ device's mediator DID, the VTA's own DID }` — covering both the queue-driven and policy-driven wake paths. Operators MAY narrow it (e.g. mediator-only, or VTA-only for deployments that don't trust a shared mediator with even a handle) or widen it by policy. The device MAY send an advisory `suggestedTriggers` hint on `device/set-wake`; the VTA MAY ignore it.

The gateway accepts an allowlist update for a handle **only from the VTA the device named** as its controller at registration. A wake request from a DID not on a handle's allowlist is refused ([§3.4](#34-triggering-a-wake)).

### 3.4 Triggering a wake

A trigger asks the gateway to wake a device by presenting the handle and the contentless hint fields, authenticated as the trigger's DID, over REST or DIDComm:

```
Trigger ──wake { handle, v, mediator?, count?, urgency? } (authenticated as triggerDid)──▶ Gateway
```

The gateway **MUST**: authenticate the trigger's DID; look up the handle; refuse (no push) if `triggerDid` is not on the handle's allowlist; otherwise resolve the handle to a token and send the [§2](#2-what-this-binding-carries--and-what-it-must-not) contentless push. The gateway **MUST NOT** forward any field beyond those of [§2](#2-what-this-binding-carries--and-what-it-must-not), and **MUST NOT** include the handle in the device-facing push.

The two trigger kinds differ only in *when* they fire:

- **Mediator trigger** — fires when the consumer's pickup queue becomes non-empty *and* the consumer is not currently connected for live delivery. The mediator is the only party with this liveness signal, which is why mediator-triggered wake is precise (no wake for an already-foregrounded app). It **SHOULD** coalesce: multiple queued messages produce at most one wake within a short window.
- **VTA trigger** — fires on a VTA policy decision (e.g. it has just minted and queued a step-up `approve-request` for delegation to this device). The VTA has no liveness signal, so a VTA-triggered wake MAY reach an already-connected app; this is harmless (a redundant doorbell). A VTA **SHOULD** rely on the mediator trigger where it is on the allowlist, and use its own trigger for cases the mediator can't gate (e.g. urgency escalation, multi-device fan-out timing).

## 4. Delivery flows

**Mediator-triggered** (queue-driven; the common case):

```
Relying party ──approve-request──▶ Mediator (queues for did:web:alice's phone)
                                      │ 1. queue non-empty for a registered, offline consumer
                                      ▼
                  Gateway ◀──wake{handle} (auth=mediatorDid)── Mediator
                     │ 2. allowlist check: mediatorDid ∈ allowed?  → yes
                     ▼
              Push service (APNs/FCM) ──contentless wake──▶ Phone
                                                              │ 3. phone wakes
                                                              │ 4. authenticates to mediator
                                                              │ 5. messagepickup/3.0 drain
                                                              ▼
                                              unpack authcrypt → Trust Task document
                                              process per §7.2, return approve-response
```

**VTA-triggered** (policy-driven; e.g. delegated step-up to a device the VTA wants woken regardless of mediator liveness):

```
VTA  ── (a) queue approve-request via mediator (DIDComm)
     └─ (b) wake{handle} (auth=vtaDid) ──▶ Gateway
                                            │ allowlist check: vtaDid ∈ allowed? → yes
                                            ▼
                                    Push service ──contentless wake──▶ Phone
                                                                         │ wakes, drains mediator queue (as above)
```

A conforming **consumer** (push receiver) **MUST**:

1. On receiving a wake push, connect to the `mediator` named in the payload (or, if absent, every mediator it is enrolled with that it has reason to poll) and authenticate.
2. Drain its queue via message-pickup and process each document through the framework [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline over the DIDComm binding.
3. Derive any user-facing notification from the decrypted documents, never from the push payload.
4. Function correctly with no push at all: push is an optimization. A consumer that polls on foreground, or holds a live WebSocket while foregrounded, obtains the same messages. Loss, delay, or suppression of a push **MUST NOT** cause a message to be missed — only delayed until the next connection.

## 5. Identity mapping

Push carries **no** framework identity. It populates neither `issuer` nor `recipient`, derives no transport-authenticated sender, and is never the transport over which a *Trust Task document* arrives. The only identity on the wire is the *trigger's* DID on the trigger → gateway request, used by the gateway purely for allowlist enforcement — it is not a framework party and does not feed [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) precedence. All framework identity precedence is resolved by the DIDComm binding when the consumer drains the queue. This binding therefore adds no rows to the identity-mapping table — it sits strictly *below* the document layer.

## 6. Transport security profile

The push channel provides **no** integrity, confidentiality, or authentication guarantee that the framework relies on. APNs/FCM/Web Push transport security protects the push in transit to the device but the payload is visible to the push provider and potentially to the device lock screen. Consequently:

* **Confidentiality** of task content is provided entirely by the DIDComm authcrypt envelope at pickup time, never by push.
* **Authentication** of the sender is provided entirely by DIDComm `sender_kid` verification at pickup time. A spoofed or replayed push, at worst, causes a consumer to connect to its mediator and find nothing queued — a wasted wake, not a security event.
* **Freshness / replay**: a replayed wake push is harmless (idempotent: connect, find the same or empty queue). The `count` hint MUST NOT be trusted for any decision.

**Token isolation.** The raw platform push token lives only at the gateway. Triggers (mediator, VTA) and the maintainer hold an **opaque handle**, never the token. A compromised trigger or VTA leaks the handle and the allowlist — enough to *request* a (contentless, allowlist-gated) wake — but not the device's push identity (the APNs/FCM identifier). The handle abstracts the platform, so the same isolation holds for any future push method (e.g. PWA Web Push) with no change to triggers or VTA config.

**Allowlist enforcement.** Possession of a handle is not authority to wake. The gateway authenticates the trigger's DID and refuses any wake from a DID not on the handle's VTA-provisioned allowlist. This bounds abuse: a party that obtains a handle (e.g. a curious mediator outside the allowlist) still cannot wake the device.

**New trust party — the gateway.** Introducing the gateway means the app publisher's gateway sees, per handle, *that* a wake occurred and *when*, and holds the push token. This is unavoidable for native push (only the app's credential-holder can push to the app) and is the same exposure the push provider already has. Deployments that will not accept a third-party gateway holding tokens SHOULD run their own gateway within the operator's trust domain (the gateway is a deployable service, not necessarily a shared one). Because a push reveals *that* a consumer received *something*, deployments handling sensitive flows **SHOULD** consider coalescing and timing jitter to blunt rate-of-activity inference, and **MUST NOT** vary the push payload by task type in a way that re-introduces content into the metadata channel.

## 7. Error and response delivery

Push is fire-and-forget and carries no Trust Task, so it has no *error response* of its own. Failures are handled at the layer that owns them: a push-service delivery failure is the gateway's concern (the [§3.2](#32-device-registration-with-the-gateway) dead-token rule); an allowlist refusal is reported to the *trigger* over whatever transport it used (REST status / DIDComm problem-report), never to the device; a malformed or unverifiable document discovered after pickup produces a `trust-task-error/0.1` returned over the DIDComm binding, exactly as if the document had arrived without a push. Conveyance failures of the handle from device to VTA surface as `device/set-wake` error codes.

## 8. Platform profiles

Platform specifics live **inside the gateway**, behind the handle. A trigger and a VTA deal only in handles and never name a platform; adding a platform is a gateway-side change plus a new [`PushRegistration`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/PushRegistration) variant — triggers and VTA config are untouched.

| Platform | `platform` value | Token                                   | Notes                                                                                 |
|----------|------------------|-----------------------------------------|---------------------------------------------------------------------------------------|
| iOS / macOS | `apns`        | APNs device token                       | Gateway uses a background/content-available push for silent wake; an `interactive` urgency MAY use an alert push. Topic = the app bundle id. Gateway holds the Team's APNs auth key. |
| Android  | `fcm`            | FCM registration token                  | Gateway sends a data message (not a notification message) so the app controls wake and display. Gateway holds the Firebase service account. |
| Web / PWA | `webpush`       | Web Push subscription (endpoint + keys) | RFC 8030 / VAPID. The one case where the credential is **self-generated** (VAPID), so an operator-run gateway needs no platform account. Payload still contentless per [§2](#2-what-this-binding-carries--and-what-it-must-not). |

A gateway that does not implement a device's registered `platform` **MUST** report the handle as unsupported at registration rather than silently dropping wakes; the device falls back to queue-and-wait.

## 9. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision MUST remain backwards-compatible: the contentless-doorbell guarantee of [§2](#2-what-this-binding-carries--and-what-it-must-not) is preserved, and only additive hint fields, additional platform profiles, or stricter rules may be introduced. Adding content to the push payload, or making push load-bearing for delivery (so that a missed push loses a message), is a breaking change requiring a `MAJOR` bump and a new binding URI. (The gateway/trigger/allowlist structure introduced in this draft revision preserves the doorbell guarantee and is therefore a `0.1`-compatible refinement.)

## 10. References

- [DIDComm binding](../../didcomm/0.1/spec.md) — carries the Trust Task documents this binding only signals the availability of.
- [HTTPS binding](../../https/0.1/spec.md) — one of the two transports a REST gateway is reachable on.
- [`device/set-wake/0.1`](../../../specs/device/set-wake/0.1/spec.md) — the device → VTA conveyance of the `WakeHandle`.
- [`WakeHandle`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeHandle) and [`WakeTriggerPolicy`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/WakeTriggerPolicy) — the opaque handle and the VTA-owned trigger allowlist.
- [`PushRegistration`](../../../specs/device/_shared/0.1/device-binding.schema.json#/$defs/PushRegistration) — the platform token the device registers with the gateway.
- [DIDComm Messaging — Message Pickup 3.0](https://didcomm.org/messagepickup/3.0/).
- [RFC 8030 — Generic Event Delivery Using HTTP Push](https://datatracker.ietf.org/doc/html/rfc8030) (Web Push).
- [Aries RFC 0699 — Push Notifications APNs](https://github.com/hyperledger/aries-rfcs/tree/main/features/0699-push-notifications-apns) and [RFC 0734 — Push Notifications FCM](https://github.com/hyperledger/aries-rfcs/tree/main/features/0734-push-notifications-fcm) — the `set-device-info` registration exchange this binding adopts (re-purposed to carry a `WakeHandle`).
- [Matrix Sygnal](https://github.com/matrix-org/sygnal) — the app-publisher push-gateway topology this binding's gateway role mirrors.
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §9.
