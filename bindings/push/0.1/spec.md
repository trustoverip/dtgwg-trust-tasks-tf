---
slug: push
version: "0.1"
title: Push wake-up transport binding
summary: A contentless wake-up notification (APNs / FCM) that tells a backgrounded consumer — typically a mobile Companion — to connect to its mediator and drain queued DIDComm-carried Trust Task documents.
status: draft
targetFrameworkVersion: "0.1"
bindingURI: https://trusttasks.org/binding/push/0.1
authors:
  - Glenn Gore (https://github.com/stormer78)
---

## Abstract

This binding specifies how a *consumer* that cannot hold a live connection to its mediator — a mobile app suspended by the operating system, a desktop agent that is not running — is told that *Trust Task documents* are waiting for it. It is a **wake-up notification binding, not a document-carriage binding.** The documents themselves are carried by the [DIDComm binding](../../didcomm/0.1/spec.md) and retrieved from the mediator with DIDComm message-pickup (`https://didcomm.org/messagepickup/3.0`); this binding defines only the out-of-band signal that prompts the retrieval, and the registration by which a consumer tells its mediator where to send that signal.

The motivating case is the mobile authenticator. A phone receives an [`auth/step-up/approve-request`](../../../specs/auth/step-up/approve-request/0.1/spec.md), shows the user the `reason`, and returns an [`approve-response`](../../../specs/auth/step-up/approve-response/0.1/spec.md). For that to work while the app is backgrounded, the mediator must be able to wake it. A phone cannot keep a WebSocket open across OS suspension, so the mediator sends a push notification through the platform's push service (Apple Push Notification service, Firebase Cloud Messaging); the app wakes, authenticates to the mediator, and drains its queue over the DIDComm binding as usual.

## Status of This Document

`0.1` draft. Tracks `SPEC.md 0.1` and the [DIDComm binding `0.1`](../../didcomm/0.1/spec.md), on which it depends.

## 1. Binding URI

| Resource           | URI                                                |
|--------------------|----------------------------------------------------|
| Binding identifier | `https://trusttasks.org/binding/push/0.1`          |

The binding URI does not appear on the wire. Like the HTTPS binding, push has no envelope `type` field; the URI is the stable identifier for this binding in registries and cross-references. Per [SPEC §9.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#93-binding-namespace) it is **not** a *Type URI* and **MUST NOT** appear in a *Trust Task document*'s `type` member.

## 2. What this binding carries — and what it must not

A push notification under this binding is a **doorbell**. Its payload **MUST NOT** contain any *Trust Task document*, any `payload` field, the `reason` of a step-up, the identity of a relying party, or any other task content. Push notifications transit a third-party push service (Apple, Google) and may be logged, displayed on a lock screen, or retained outside the trust boundary; treating them as a content channel leaks the very data the DIDComm authcrypt envelope exists to protect.

A conforming push notification **MAY** carry only:

| Field         | Meaning                                                                                                   |
|---------------|-----------------------------------------------------------------------------------------------------------|
| `v`           | The integer `1` — this binding's wire version.                                                            |
| `mediator`    | The DID of the mediator holding the queued messages (so a consumer enrolled with several knows which to drain). |
| `count`       | OPTIONAL. An approximate count of queued messages. Advisory only — the consumer learns the true set from pickup. |
| `urgency`     | OPTIONAL. `"interactive"` or `"background"` — a hint the consumer MAY map to the platform's priority/alert behavior. |

A consumer **MUST** treat every field of the push payload as an **untrusted hint**. The authoritative message set, sender identity, and content come only from authenticated DIDComm pickup ([§4](#4-delivery-flow)). A consumer **MUST NOT** display task content from a push payload and **MUST NOT** take any framework action on the strength of a push alone.

> A consumer **MAY** show a generic, content-free local notification ("You have a pending approval") *after* it has woken and authenticated, deriving the wording from the drained, decrypted documents — never from the push payload.

## 3. Push registration (`set-device-info`)

Before a mediator can wake a consumer, the consumer tells the mediator where to push. This binding **adopts the established DIDComm push-notification protocols** — [Aries RFC 0699 `push-notifications-apns`](https://github.com/hyperledger/aries-rfcs/tree/main/features/0699-push-notifications-apns) and [RFC 0734 `push-notifications-fcm`](https://github.com/hyperledger/aries-rfcs/tree/main/features/0734-push-notifications-fcm), carried as DIDComm v2 messages — rather than defining a new mechanism.

Registration is a **device → mediator** exchange over the same authenticated DIDComm channel the consumer already uses for mediation and pickup. After mediation coordination (`coordinate-mediation/2.0`: `mediate-request` → `mediate-grant`, `keylist-update`), the consumer sends:

| Message            | `type`                                                                                                           | Body                                                                                                                                                                                                 |
|--------------------|-----------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Set device info    | `https://didcomm.org/push-notifications-apns/1.0/set-device-info` (or `…/push-notifications-fcm/1.0/set-device-info`) | A `PushRegistration` ([device-binding shared schema](../../../specs/device/_shared/0.1/device-binding.schema.json) `#/$defs/PushRegistration`): platform, device token, and platform-specific routing (APNs topic/environment, Web Push endpoint + keys). |
| Delete device info | `…/1.0/delete-device-info`                                                                                       | Empty. Unregisters the consumer (e.g. on logout).                                                                                                                                                    |

The mediator stores the registration **keyed to the consumer's recipient DID** — the DID it already routes for, established at `keylist-update`. The token is **never** sent to the VTA: a push channel is a transport detail owned by the mediator, not vault state. This is the correction to placing the token in a `device/register` Trust Task — the party that *uses* the token (the mediator) is the party that *holds* it.

Push tokens rotate. A consumer **MUST** re-send `set-device-info` whenever the platform issues a new token; the mediator **MUST** treat the most recent registration as authoritative and discard the prior token. A push to a token the push service reports as permanently unregistered **MUST** cause the mediator to drop the stored token and fall back to queue-and-wait (the message remains queued for the consumer's next voluntary pickup).

> A consumer's authority to register push **is** its authenticated DIDComm session with the mediator — the same credential that authorized mediation. No separate grant is needed, and a `set-device-info` from an unauthenticated sender is rejected at the DIDComm layer before this binding sees it.

A maintainer/VTA that wants device-management visibility of which devices are wakeable reads the non-secret `pushCapable` flag on the [`DeviceBinding`](../../../specs/device/_shared/0.1/device-binding.schema.json) (surfaced in `device/list`); it never sees the token itself.

## 4. Delivery flow

```
Relying party ──approve-request──▶ Mediator (queues for did:web:alice's phone)
                                      │
                                      │ 1. queue non-empty for a registered, offline consumer
                                      ▼
                              Push service (APNs/FCM) ──contentless wake──▶ Phone
                                      ▲                                        │
                                      │ 2. phone wakes                         │
                                      │ 3. authenticates to mediator (ATM challenge-response)
                                      │ 4. messagepickup/3.0 deliver/drain  ◀──┘
                                      │ 5. unpack DIDComm authcrypt → Trust Task document
                                      ▼
                              Phone processes per §7.2, returns approve-response
                              (DIDComm forward → mediator → relying party)
```

A conforming **mediator** (push sender) **MUST**:

1. Send a push only when a registered consumer has queued messages and is not currently connected for live delivery.
2. Populate the push payload with no more than the fields in [§2](#2-what-this-binding-carries--and-what-it-must-not).
3. Authenticate the consumer on its subsequent pickup connection exactly as it would for any DIDComm session — the push grants no standing; it is not a credential.
4. Coalesce: multiple queued messages SHOULD produce at most one wake push within a short window, not one push per message.
5. Drop a token the push service reports as permanently unregistered, and leave the message queued.

A conforming **consumer** (push receiver) **MUST**:

1. On receiving a wake push, connect to the `mediator` named in the payload (or, if absent, every mediator it is enrolled with that it has reason to poll) and authenticate.
2. Drain its queue via message-pickup and process each document through the framework [§7.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#72-consumer-requirements) pipeline over the DIDComm binding.
3. Derive any user-facing notification from the decrypted documents, never from the push payload.
4. Function correctly with no push at all: push is an optimization. A consumer that polls on foreground, or holds a live WebSocket while foregrounded, obtains the same messages. Loss, delay, or suppression of a push **MUST NOT** cause a message to be missed — only delayed until the next connection.

## 5. Identity mapping

Push carries **no** framework identity. It populates neither `issuer` nor `recipient`, derives no transport-authenticated sender, and is never the transport over which a *Trust Task document* arrives. All [§4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity) identity precedence is resolved by the DIDComm binding when the consumer drains the queue. This binding therefore adds no rows to the identity-mapping table — it sits strictly *below* the document layer.

## 6. Transport security profile

The push channel provides **no** integrity, confidentiality, or authentication guarantee that the framework relies on. APNs/FCM transport security protects the push in transit to the device but the payload is visible to the push provider and potentially to the device lock screen. Consequently:

* **Confidentiality** of task content is provided entirely by the DIDComm authcrypt envelope at pickup time, never by push.
* **Authentication** of the sender is provided entirely by DIDComm `sender_kid` verification at pickup time. A spoofed or replayed push, at worst, causes a consumer to connect to its mediator and find nothing queued — a wasted wake, not a security event.
* **Freshness / replay**: a replayed wake push is harmless (idempotent: connect, find the same or empty queue). The `count` hint MUST NOT be trusted for any decision.

Because a push reveals *that* a consumer received *something* (traffic-analysis metadata visible to the push provider), deployments handling sensitive flows **SHOULD** consider coalescing and timing jitter to blunt rate-of-activity inference, and **MUST NOT** vary the push payload by task type in a way that re-introduces content into the metadata channel.

## 7. Error and response delivery

Push is fire-and-forget and carries no Trust Task, so it has no *error response* of its own. Failures are handled at the layer that owns them: a push-service delivery failure is the mediator's concern ([§3](#3-push-token-registration) token-drop rule); a malformed or unverifiable document discovered after pickup produces a `trust-task-error/0.1` returned over the DIDComm binding, exactly as if the document had arrived without a push.

## 8. Platform profiles

| Platform | `platform` value | Token                                   | Notes                                                                                 |
|----------|------------------|-----------------------------------------|---------------------------------------------------------------------------------------|
| iOS / macOS | `apns`        | APNs device token                       | Use a background/content-available push for silent wake; an `interactive` urgency MAY use an alert push. Topic = the app bundle id. |
| Android  | `fcm`            | FCM registration token                  | Use a data message (not notification message) so the app controls wake and display. |
| Web      | `webpush`        | Web Push subscription (endpoint + keys) | RFC 8030 / VAPID. Payload still contentless per [§2](#2-what-this-binding-carries--and-what-it-must-not). |

A mediator that does not implement a consumer's registered `platform` **MUST** fall back to queue-and-wait for that consumer rather than failing the message.

## 9. Versioning

This binding follows the framework's `MAJOR.MINOR` versioning ([SPEC §5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#5-versioning)). A `MINOR` revision MUST remain backwards-compatible: the contentless-doorbell guarantee of [§2](#2-what-this-binding-carries--and-what-it-must-not) is preserved, and only additive hint fields, additional platform profiles, or stricter rules may be introduced. Adding content to the push payload, or making push load-bearing for delivery (so that a missed push loses a message), is a breaking change requiring a `MAJOR` bump and a new binding URI.

## 10. References

- [DIDComm binding](../../didcomm/0.1/spec.md) — carries the Trust Task documents this binding only signals the availability of.
- [DIDComm Messaging — Message Pickup 3.0](https://didcomm.org/messagepickup/3.0/).
- [RFC 8030 — Generic Event Delivery Using HTTP Push](https://datatracker.ietf.org/doc/html/rfc8030) (Web Push).
- [Aries RFC 0699 — Push Notifications APNs](https://github.com/hyperledger/aries-rfcs/tree/main/features/0699-push-notifications-apns) and [RFC 0734 — Push Notifications FCM](https://github.com/hyperledger/aries-rfcs/tree/main/features/0734-push-notifications-fcm) — the `set-device-info` registration protocol this binding adopts.
- [Trust Tasks framework specification](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md), §4.8.1, §7.2, §9.
- [`device/_shared` DeviceBinding](../../../specs/device/_shared/0.1/device-binding.schema.json) — defines the `PushRegistration` `set-device-info` body shape and the non-secret `pushCapable` visibility flag.
