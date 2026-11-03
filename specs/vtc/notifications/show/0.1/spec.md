---
slug: vtc/notifications/show
version: "0.1"
title: "VTC Notifications — Show"
summary: "An administrator reads one notification the community holds as its addressee, with the document that delivered it exactly as received, what a relay of it discloses, and the relay decisions made — the read that precedes a relay decision."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - inbox
  - vtc
  - relay
  - verify
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the addressee)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The read discloses what the community has been told and by whom, so it must be attributable to
    the administrator who made it. The document it returns carries its own proofs, which are what the
    administrator verifies.
sideEffects:
  level: none
  rationale: "Read-only fetch of one received notification."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: one received notification and the signed document that delivered it — who originated and
    relayed it, where its thing is held and whom it concerns, never the thing's content. In: a record
    identifier.
retention:
  class: exchange
  rationale: >-
    The community remains the record; the response is the administrator's copy for verifying and
    deciding on it.
errorCodes:
  - code: vtc/notifications/show:notificationUnknown
    meaning: "`recordId` names no notification the community holds — never held, or discarded after it lapsed."
    retryable: false
related:
  - vtc/notifications/list
  - vtc/notifications/relay
  - notifications/notify
  - notifications/relay
---

## Abstract

The **VTC Notifications — Show** Trust Task reads one notification a community holds as its
addressee, by the `recordId` [`vtc/notifications/list`](../../list/0.1/spec.md) returned. It returns
the record — with whether the community may relay it, what a relay of it discloses (VTI-NTF-046)
and the relay decisions already made — and the document that delivered the revision held, exactly
as received, so an administrator can verify the originator's proof and every relay before deciding
a relay with [`vtc/notifications/relay`](../../relay/0.1/spec.md) (VTI-NTF-070). Reading changes
nothing and sends nothing to anyone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the community) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without the authority in [Authorization](#authorization),
   read from the signer's live access-control entry at execution time.
3. Refuse with `vtc/notifications/show:notificationUnknown` a `recordId` it holds no notification
   under.
4. Return the record, with `mayRelay`, `relayDisclosure` and `relayDecisions` as
   [`vtc/notifications/list`](../../list/0.1/spec.md) consumer item 4 requires, and, in `document`, the
   document that delivered the revision held, unchanged.
5. **MUST NOT** mark, report or act on the notification because it was read, and **MUST NOT** send
   anything to its originator, a relayer or the thing's holder (VTI-NTF-053).

A conforming **producer** (the administrator's client) that relies on `originator` or `relayedBy`
without trusting the community **SHOULD** verify `document` as
[`notifications/relay`](../../../../notifications/relay/0.1/spec.md) consumer item 2 does, and
**MUST** show the administrator `relayDisclosure`, where present, before offering a relay decision
(VTI-NTF-046).

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

The authority is the **`vtc.notifications.read` capability at the community**, as for
[`vtc/notifications/list`](../../list/0.1/spec.md#authorization).

## Definitions

- **`recordId`** — the community's identifier for the received notification.
- **`item`** (response) — the record ([`ReceivedNotification`](../../_shared/0.1/received-notification.schema.json)).
- **`document`** (response) — the document that delivered the revision held, whole; the document a
  relay of it carries in `forwarded`.

## Request

The administrator sends the community the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read one received notification

```json
{
  "id": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a5b",
  "type": "https://trusttasks.org/spec/vtc/notifications/show/0.1",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:partner.example",
  "issuedAt": "2027-03-01T10:01:00Z",
  "payload": {
    "recordId": "rn-4Tq8Zr2Lw9Ks6Pm1"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T10:01:00Z",
    "verificationMethod": "did:web:carol.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2Cd..."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. The `document` here
is abbreviated: it is the whole notification shown in
[`notifications/notify`](../../../../notifications/notify/0.1/spec.md#a-community-tells-a-member-party-of-an-election-put-to-its-members),
proof included.

### An election notification addressed to the community

```json
{
  "id": "urn:uuid:2f3a4b5c-6d7e-4f8a-9b0c-1d2e3f4a5b6c",
  "type": "https://trusttasks.org/spec/vtc/notifications/show/0.1#response",
  "issuer": "did:web:partner.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2027-03-01T10:01:01Z",
  "threadId": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a5b",
  "payload": {
    "item": {
      "recordId": "rn-4Tq8Zr2Lw9Ks6Pm1",
      "originator": "did:web:community.example",
      "relayedBy": [],
      "notification": {
        "notificationId": "ntf-7Qk2p9Xv3mR8",
        "revision": 1,
        "kind": "election",
        "subject": {
          "heldBy": "did:web:community.example",
          "id": "board-2027",
          "openWith": "https://trusttasks.org/spec/vtc/elections/show/0.6"
        },
        "audience": "did:web:partner.example",
        "status": "available",
        "relayable": true,
        "validFrom": "2027-03-08T00:00:00Z",
        "validUntil": "2027-03-15T00:00:00Z"
      },
      "receivedAt": "2027-03-01T09:00:02Z",
      "mayRelay": true,
      "relayDisclosure": "Relaying this notification shows each member it reaches that this community is a member of the community holding the election, which this community's own listing in that community's directory may otherwise keep from them.",
      "relayDecisions": []
    },
    "document": {
      "id": "urn:uuid:2f8c1a40-7d1e-4b8e-9a51-0c6f3e2d7a11",
      "type": "https://trusttasks.org/spec/notifications/notify/0.1",
      "issuer": "did:web:community.example",
      "recipient": "did:web:partner.example",
      "issuedAt": "2027-03-01T09:00:00Z",
      "payload": { "…": "the notification, as above" },
      "proof": { "…": "the community's proof" }
    }
  }
}
```

## Security & Privacy

### Data carried

One received notification and the signed document that delivered it: originator, relayers, the
thing's holder and identifier, the audience and period, what a relay would disclose and the relay
decisions made. No thing's content, because no notification carries any (VTI-NTF-003).

### Correlation

The community's `identifierScope` is `public`: it is named by its public DID because it is the party notifications are addressed to
and the party its administrators already know: a pairwise identifier would name no community the
notifications could have reached. The administrator's identifier is pairwise, as for the community's
other administrative tasks.

As for [`vtc/notifications/list`](../../list/0.1/spec.md#correlation): the response goes only to an
administrator holding `vtc.notifications.read`, and `recordId` is meaningful only at this community.

### Retention

The community is the record. A client keeps the document only as long as it needs it to verify and
show the notification.

### Consent/purpose

Reading a notification is not relaying it, and not opening the thing. The read exists so that
whoever decides a relay has seen the notification and what relaying it discloses (VTI-NTF-046).
