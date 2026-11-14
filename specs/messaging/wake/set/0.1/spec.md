---
slug: messaging/wake/set
version: "0.1"
title: "Messaging — Wake Set"
summary: "An account gives its own mediator the opaque wake handle its push gateway issued, or takes it back, so the mediator can ring the account's device when a sender with mail queued for it asks — without the mediator ever holding a push token."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords:
  - wake
  - push
  - doorbell
  - wake handle
  - notification
parties:
  - role: Account (the device the handle wakes)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Mediator (the account's own)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The authority is control of the account, and the mediator already authenticates the account on the session the request arrives on — the DIDComm authcrypt sender or the TSP sender VID — so a document proof adds nothing on those bindings. A proof is RECOMMENDED so that a request carried where no session authenticates the account still names it verifiably.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Setting a handle makes the account wakeable through this mediator, and clearing one makes it unwakeable. Replayed after the account changed its mind, an old document would silently reverse that choice, so a mediator must be able to place every request in its acceptance window.
sideEffects:
  level: mutating
  rationale: >-
    Stores or removes the account's wake handle at the mediator. The record is replaced whole on every request and can be set again at any time, so nothing is irreversible.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries an opaque gateway-issued handle and the gateway's address — routing infrastructure, never a platform push token. The response says only whether the mediator will ring, which the account itself asked about.
retention:
  class: durable
  rationale: >-
    The handle is live configuration: the mediator consults it on every qualifying wake request for as long as the account keeps it set. It is replaced by the next request, removed by a request without one, and dropped when the gateway reports the handle dead or the account is removed.
errorCodes: []
related:
  - messaging/wake/request
  - push/register
  - push/provision
  - push/wake
  - device/set-wake
---

## Abstract

The **Messaging — Wake Set** Trust Task lets an account give its own mediator the opaque [`WakeHandle`](../../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeHandle) that its push gateway issued at [`push/register`](../../../../push/register/0.3/spec.md), or take it back.

With a handle, the mediator can act as the account's **queue-driven trigger** in the [push wake-up binding](../../../../../bindings/push/0.1/spec.md) — the one party that knows both that mail is waiting and that the device is not connected. What it rings on is a [`messaging/wake/request`](../../request/0.1/spec.md) from a sender that has mail queued for the account. That lets a party with no standing at the device's own agent — a VTA asking a named approver to decide on a held task, say — get the device woken without ever holding the handle, the token, or any authority over the device.

The mediator learns a handle, never a token. The gateway still decides: it rings only for a trigger on the handle's allowlist ([`push/provision`](../../../../push/provision/0.2/spec.md)), so the device (or its controller) must also put this mediator on that allowlist.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the account) **MUST**:

1. Send the request to its own mediator, with itself as `issuer` and the mediator as `recipient`.
2. Carry in `wakeHandle` only a handle a push gateway issued to it, exactly as received — never a platform push token.
3. Send a new request when the gateway issues a new handle (token rotation re-registers, [`push/register`](../../../../push/register/0.3/spec.md)), and a request **without** `wakeHandle` when it no longer wants to be woken through this mediator: on sign-out, on uninstall where the platform allows, and before handing the account to another device.

A conforming **consumer** (the mediator) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Act only for the requester's own account: the account whose DID is the transport-authenticated sender, or the verified `proof`'s `issuer` where no session authenticates it. A request naming no local account is refused with `permissionDenied`.
3. Replace the handle it holds for that account with `wakeHandle`, or remove it when `wakeHandle` is absent. Exactly one handle is held per account; a second request supersedes the first.
4. Answer `wakeCapable: true` only when it now holds a handle **and** will act on it — its operator offers waking and it can reach `wakeHandle.gateway` over a transport that gateway speaks. Otherwise it answers `wakeCapable: false` and **SHOULD** hold no handle.
5. Use the handle only to send [`push/wake`](../../../../push/wake/0.2/spec.md) to `wakeHandle.gateway`, as itself, and never disclose the handle to any other party, including in any response to a sender.
6. Drop the handle when the gateway answers a wake with `push/wake:tokenUnregistered`, and stop ringing it after `push/wake:notAllowed` until the account sets a handle again.
7. Drop the handle when the account is removed.

This version defines the handle's use for [`messaging/wake/request`](../../request/0.1/spec.md). A mediator **MAY** also ring it as the binding's queue-driven trigger whenever mail is queued for an offline account ([push binding §3.4](../../../../../bindings/push/0.1/spec.md#34-triggering-a-wake)), where its operator's policy does so; the same gateway allowlist governs both.

## Authorization

The authority is **control of the account**. An account sets and clears its own handle and no other. The mediator **SHOULD** apply the capability gate it applies to the account's own queue operations; for a mediator that exposes the `local` capability, a requester without it receives `permissionDenied`. There is no administrative form: an operator who must stop waking an account removes the handle through its own administration, not by sending this task as the account.

Setting a handle confers nothing on the mediator or on anyone else. It lets the mediator *ask* the gateway to ring, and the gateway still refuses unless the mediator is on the handle's allowlist, which only the handle's controller sets ([`push/provision`](../../../../push/provision/0.2/spec.md)). Holding a handle is never authority to wake ([push binding §3.2](../../../../../bindings/push/0.1/spec.md#32-the-gateway-control-plane-is-a-trust-task-family-push)), and this task does not change that.

Verifying the transport sender or the `proof` establishes **who** set the handle ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). That the requester is the account is what entitles it; the mediator derives the account from that identity, never from anything the payload names.

## Definitions

- **`wakeHandle`** — OPTIONAL. The [`WakeHandle`](../../../../device/_shared/0.2/device-binding.schema.json#/$defs/WakeHandle) — `gateway` and an opaque `handle` — that the account's push gateway returned from `push/register`. Present: the mediator stores it. Absent: the mediator removes the handle it held. Absence is the restrictive reading on purpose: a request that names no handle leaves the account unwakeable.
- **`wakeCapable`** (response) — whether the mediator now holds a handle for the account and will act on it.

## Request

The account sends the request to its own mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A phone gives its mediator the handle it just registered

```json
{
  "id": "urn:uuid:5d0f6c1a-3b2e-4c8d-9a71-0e4b2f6c8d31",
  "type": "https://trusttasks.org/spec/messaging/wake/set/0.1",
  "issuer": "did:peer:2.Ez6LSjordanphone.Vz6Mkjordanphone",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-14T09:00:00Z",
  "payload": {
    "wakeHandle": {
      "gateway": "did:web:push.wallet.example",
      "handle": "wh_7Q2kR9vXb3LmT0pN"
    }
  }
}
```

### The phone signs out and takes the handle back

```json
{
  "id": "urn:uuid:5d0f6c1a-3b2e-4c8d-9a71-0e4b2f6c8d32",
  "type": "https://trusttasks.org/spec/messaging/wake/set/0.1",
  "issuer": "did:peer:2.Ez6LSjordanphone.Vz6Mkjordanphone",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-14T18:30:00Z",
  "payload": {}
}
```

## Response

The mediator answers whether it will ring, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### The mediator holds the handle and will ring

```json
{
  "id": "urn:uuid:5d0f6c1a-3b2e-4c8d-9a71-0e4b2f6c8d41",
  "type": "https://trusttasks.org/spec/messaging/wake/set/0.1#response",
  "threadId": "urn:uuid:5d0f6c1a-3b2e-4c8d-9a71-0e4b2f6c8d31",
  "issuer": "did:web:mediator.example",
  "recipient": "did:peer:2.Ez6LSjordanphone.Vz6Mkjordanphone",
  "issuedAt": "2026-11-14T09:00:01Z",
  "payload": {
    "wakeCapable": true
  }
}
```

`wakeCapable: true` says the mediator will *ask*. Whether the gateway rings depends on its allowlist, which the device or its controller VTA sets separately.

## Security & Privacy

### Data carried

The request carries one opaque handle and the address of the gateway that issued it. The handle reveals no platform token and lets its holder do nothing but ask that gateway for a wake, which the gateway refuses to any trigger not on the handle's allowlist. The response carries a single boolean. A producer **MUST NOT** carry the platform token, device model, or application identifier in `ext`.

### Correlation

The mediator already knows the account and when it connects. What it learns here is which push gateway serves the account, which says which wallet application the device most likely runs, and a handle that stays stable until the device's token rotates. Both are inherent in the mediator ringing the device at all. The mediator **MUST NOT** disclose either to a sender: [`messaging/wake/request`](../../request/0.1/spec.md) answers every request identically for exactly this reason, so a sender cannot use the mediator to learn whether an account is wakeable, by which gateway, or under which handle.

The mediator declares `identifierScope: public` because it must be the same DID in three places that never coordinate: the account's own session with it, the gateway allowlist the account's controller provisions (a trigger is matched by DID), and the `mediator` hint the gateway passes to the device so it knows which mediator to drain. A pairwise mediator identifier would leave the allowlist unmatchable. The account keeps `pairwise`: nothing outside its own mediator needs to recognise its DID here.

### Retention

The mediator keeps one handle per account, for as long as the account keeps it set, and replaces it whole on each request. It **MUST** delete the handle when the account clears it, when the gateway reports it dead, and when the account is removed, and **SHOULD NOT** keep superseded handles: a history of them is a history of the device's reinstalls and token rotations, which no part of this task needs.

### Consent/purpose

The handle is given for one purpose: so the mediator can ask the gateway for a contentless doorbell when mail is queued for the account and a sender with mail in that queue asks it to. Using it for anything else — telling a sender the account's gateway, waking the device for the mediator's own messages beyond what the account's queue holds, or handing it to a third party — is outside the purpose it was given for.
