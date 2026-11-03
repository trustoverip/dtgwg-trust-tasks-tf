---
slug: vtc/notifications/list
version: "0.1"
title: "VTC Notifications — List"
summary: "An administrator lists the notifications the community holds as their addressee — filtered by kind or holder — with whether each may be relayed, what a relay of it discloses, and the relay decisions already made."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - inbox
  - vtc
  - relay
  - administrator
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
    The list says what the community has been told of and by whom — which communities, groups and
    networks it belongs to — so the read must be attributable to the administrator who made it, as
    the community's other administrative reads are.
sideEffects:
  level: none
  rationale: "Read-only enumeration of received notifications."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: for each received notification, who originated and relayed it, where the thing is held, the
    audience it concerns, what a relay of it would disclose and the relay decisions made — never the
    thing's content, because no notification carries any. In: filters.
retention:
  class: exchange
  rationale: >-
    The community remains the record; the page is the administrator's working copy for the view it
    renders.
errorCodes: []
related:
  - vtc/notifications/show
  - vtc/notifications/relay
  - notifications/notify
  - notifications/relay
---

## Abstract

The **VTC Notifications — List** Trust Task returns the notifications a community holds as their
addressee: those it accepted, directly by [`notifications/notify`](../../../../notifications/notify/0.1/spec.md)
or by [`notifications/relay`](../../../../notifications/relay/0.1/spec.md), under its stated acceptance
policy (VTI-NTF-034). It is how the community's administrators see what was addressed to the
community — an election another community puts to this community's members, for instance — and is
the starting point of a relay decision with [`vtc/notifications/relay`](../../relay/0.1/spec.md)
(VTI-NTF-070). It filters by kind and by the DID holding the thing, and leaves out lapsed
notifications unless asked for them.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

A filter suffices for this collection without a single-record twin for existence: the split pair is
this task and [`vtc/notifications/show`](../../show/0.1/spec.md), which reads one record by `recordId`
and refuses an unknown one.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the community) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without the authority in [Authorization](#authorization),
   read from the signer's live access-control entry at execution time, and answer nothing about any
   notification to it (VTI-NTF-070).
3. Return the received notifications matching every filter given, most recently received first: of
   `kind` only, where given; of `subject.heldBy` equal to `heldBy` only, where given; and withdrawn
   ones and those whose `validUntil` has passed only where `includeLapsed` is `true`.
4. State, for each, `mayRelay` and `relayDisclosure` as
   [`ReceivedNotification`](../../_shared/0.1/received-notification.schema.json) defines them, and the
   relay decisions made for it — the selection and count of each, never the parties it reached
   (VTI-NTF-073).
5. Page with `cursor` and `limit`, refusing with `malformedRequest` a `cursor` it did not issue for
   the same filters.
6. **MUST NOT** send anything to an originator, a relayer or a thing's holder because an
   administrator listed the community's notifications (VTI-NTF-053).

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

The authority is the **`vtc.notifications.read` capability at the community** (VTI-NTF-070), read
from the signer's access-control entry at execution time. The proof establishes who asked, never that
they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The capability is apart from
`vtc.notifications.relay` so that a community can let an administrator see what it has been told
without letting them pass it to members.

## Definitions

- **`kind`**, **`heldBy`** — optional filters.
- **`includeLapsed`** — optional widening; absent reads as `false`.
- **`cursor`**, **`limit`** — paging.
- **`items`** (response) — the received notifications
  ([`ReceivedNotification`](../../_shared/0.1/received-notification.schema.json)).
- **`nextCursor`** (response) — present when more remain.

## Request

The administrator sends the community the top-level schema in [`payload.schema.json`](payload.schema.json).

### The elections other communities have put to this community's members

```json
{
  "id": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e2f",
  "type": "https://trusttasks.org/spec/vtc/notifications/list/0.1",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:partner.example",
  "issuedAt": "2027-03-01T10:00:00Z",
  "payload": {
    "kind": "election"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T10:00:00Z",
    "verificationMethod": "did:web:carol.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3Ab..."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`.

### One election, not yet relayed

```json
{
  "id": "urn:uuid:0d1e2f3a-4b5c-4d6e-9f7a-8b9c0d1e2f3a",
  "type": "https://trusttasks.org/spec/vtc/notifications/list/0.1#response",
  "issuer": "did:web:partner.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2027-03-01T10:00:01Z",
  "threadId": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e2f",
  "payload": {
    "items": [
      {
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
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Each item says who originated a notification, who relayed it, where its thing is held, whom it
concerns, and what relaying it would disclose — together, a map of the communities, groups and
networks this community belongs to and what they have told it of. No item carries a thing's
content, because no notification does (VTI-NTF-003). The relay decisions carry the selection and a
count, never the members reached.

### Correlation

The community's `identifierScope` is `public`: it is named by its public DID because it is the party notifications are addressed to
and the party its administrators already know: a pairwise identifier would name no community the
notifications could have reached. The administrator's identifier is pairwise, as for the community's
other administrative tasks.

The response joins every notification the community holds into one view, which is its purpose, and
goes only to an administrator holding `vtc.notifications.read`. It is never shown to members: what
this community has been told is not theirs to see until an administrator decides to relay it
(VTI-NTF-070).

### Retention

The community is the record. A client keeps the page for the view it renders and **SHOULD NOT** keep
it past that, since a withdrawal at the community does not reach a copy.

### Consent/purpose

The list serves the administrators' decision whether to pass a notification on, and nothing else.
Listing a notification is not acting on it: the community does not open the thing because it was
told of it (VTI-NTF-053).
