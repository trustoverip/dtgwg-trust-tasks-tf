---
slug: vta/notifications/show
version: "0.1"
title: "VTA Notifications — Show"
summary: "A client reads one notification its VTA holds for the principal, with the document that delivered it exactly as received, so it can verify the originator and every relay itself."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - inbox
  - vta
  - verify
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing, normally carried by an authenticated transport between a client
    and its own VTA. The document it returns carries its own proofs, which are what the client
    verifies; the response's proof adds only the VTA's word for the record around it.
sideEffects:
  level: none
  rationale: "Read-only fetch of one held notification."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: one held notification and the signed document that delivered it — who originated and
    relayed it, where its thing is held and whom it concerns, never the thing's content. In: a
    context and a record identifier.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record; the response is the client's copy for verifying and showing it.
errorCodes:
  - code: vta/notifications/show:notificationUnknown
    meaning: "`recordId` names no notification the VTA holds in `contextId` — never held, or discarded after it lapsed."
    retryable: false
related:
  - vta/notifications/list
  - vta/notifications/dismiss
  - notifications/notify
  - notifications/relay
---

## Abstract

The **VTA Notifications — Show** Trust Task reads one notification a VTA holds for its principal,
by the `recordId` [`vta/notifications/list`](../../list/0.1/spec.md) returned. It returns the
record and the document that delivered the revision held — the
[`notifications/notify`](../../../../notifications/notify/0.1/spec.md) document, or the
[`notifications/relay`](../../../../notifications/relay/0.1/spec.md) document carrying it —
exactly as received, so the client verifies the originator's proof and every relay itself rather
than take the VTA's word for them (VTI-NTF-050). Reading changes nothing and sends nothing to
anyone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization).
3. Refuse with `vta/notifications/show:notificationUnknown` a `recordId` it holds no notification
   under in `contextId`, including one it holds in another context (VTI-NTF-051).
4. Return the record and, in `document`, the document that delivered the revision held, unchanged.
5. **MUST NOT** mark, report or act on the notification because it was read, and **MUST NOT** send
   anything to its originator, a relayer or the thing's holder (VTI-NTF-053).

A conforming **producer** (the client) that relies on `originator` or `relayedBy` without trusting
the VTA **SHOULD** verify `document` as [`notifications/relay`](../../../../notifications/relay/0.1/spec.md)
consumer item 2 does.

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/notifications/list`](../../list/0.1/spec.md#authorization).

## Definitions

- **`contextId`** — the trust context the notification is held in.
- **`recordId`** — the VTA's identifier for it.
- **`item`** (response) — the record ([`HeldNotification`](../../_shared/0.1/held-notification.schema.json)).
- **`document`** (response) — the document that delivered the revision held, whole.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read one notification

```json
{
  "id": "urn:uuid:1d2e3f40-5a6b-4c7d-8e9f-0a1b2c3d4e5f",
  "type": "https://trusttasks.org/spec/vta/notifications/show/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-02T08:01:00Z",
  "payload": {
    "contextId": "personal",
    "recordId": "hn-4Tq8Zr2Lw9Ks6Pm1"
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`. The `document` here is
abbreviated: it is the whole relay shown in
[`notifications/relay`](../../../../notifications/relay/0.1/spec.md#a-member-party-relays-an-election-notification-to-one-of-its-members),
proofs included.

### A relayed election notification

```json
{
  "id": "urn:uuid:2e3f4a50-6b7c-4d8e-9f0a-1b2c3d4e5f60",
  "type": "https://trusttasks.org/spec/vta/notifications/show/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-02T08:01:01Z",
  "threadId": "urn:uuid:1d2e3f40-5a6b-4c7d-8e9f-0a1b2c3d4e5f",
  "payload": {
    "item": {
      "recordId": "hn-4Tq8Zr2Lw9Ks6Pm1",
      "contextId": "personal",
      "addressee": "did:key:z6MkMember",
      "originator": "did:web:community.example",
      "relayedBy": ["did:web:partner.example"],
      "notification": {
        "notificationId": "ntf-7Qk2p9Xv3mR8",
        "revision": 1,
        "kind": "election",
        "subject": {
          "heldBy": "did:web:community.example",
          "id": "board-2027",
          "openWith": "https://trusttasks.org/spec/vtc/elections/show/0.5"
        },
        "audience": "did:web:partner.example",
        "status": "available",
        "relayable": true,
        "validFrom": "2027-03-08T00:00:00Z",
        "validUntil": "2027-03-15T00:00:00Z"
      },
      "state": "active",
      "receivedAt": "2027-03-01T11:20:01Z"
    },
    "document": {
      "id": "urn:uuid:5e7b2c91-0a3f-4d68-b1e4-93c0f8a27d65",
      "type": "https://trusttasks.org/spec/notifications/relay/0.1",
      "issuer": "did:web:partner.example",
      "recipient": "did:key:z6MkMember",
      "issuedAt": "2027-03-01T11:20:00Z",
      "payload": { "forwarded": { "…": "the community's notifications/notify document, whole" } },
      "proof": { "…": "the partner's proof" }
    }
  }
}
```

## Security & Privacy

### Data carried

One held notification and the signed document that delivered it: originator, relayers, the
thing's holder and identifier, the audience and period. No thing's content, because no
notification carries any (VTI-NTF-003).

### Correlation

As for [`vta/notifications/list`](../../list/0.1/spec.md#correlation): the response goes only to a
party with standing in the context, and `recordId` is meaningful only at this VTA.

### Retention

The VTA is the record. A client keeps the document only as long as it needs it to verify and show
the notification.

### Consent/purpose

Reading a notification is not acting on it. Opening the thing — for an election, reading it from
the community, which may present the member's credential — is a separate request the client makes
only at the principal's choice (VTI-NTF-053, VTI-VOT-016).
