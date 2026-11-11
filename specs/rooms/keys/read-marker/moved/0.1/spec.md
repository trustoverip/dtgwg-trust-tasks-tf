---
slug: rooms/keys/read-marker/moved
version: "0.1"
title: "Rooms Keys — Read Marker — Moved"
summary: "A member's key holder tells the member's other clients watching a room that the member's read marker for it moved — the room, the member and the marker, so a device can clear what its member already read elsewhere — and no response."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - unread
  - read marker
  - notification
  - persona
parties:
  - role: KeyHolder
    requirement: REQUIRED
    member: issuer
  - role: Member
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The client acts on it only when it comes from its own key holder. The transports a key holder sends it on authenticate the sender; a proof says the same where one does not. A forged one could at worst clear a notification, and the client's next listing restores the marker in force.
sideEffects:
  level: none
  rationale: >-
    A hint. The client updates what it shows as unread, or does not; nothing is stored at the key holder or anywhere else because it was sent.
subjectPath: /member
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The room and member the client watches, and the member's read marker — a room version and a host timestamp — which rooms/keys/list 0.3 would have returned to the same client. Nothing about which messages were read or which device read them.
retention:
  class: transient
  rationale: >-
    The client takes the marker as the one in force and keeps nothing else of the document; the key holder keeps nothing of having sent it.
errorCodes: []
related:
  - rooms/keys/read-marker/set
  - rooms/keys/watch
  - rooms/keys/changed
  - rooms/keys/list
  - rooms/changed
  - push/wake
---

## Abstract

When one of a member's devices moves their read marker for a room ([`rooms/keys/read-marker/set`](../../set/0.1/spec.md)), the others still show what the member has now read as unread, and may still hold a notification for it. **Rooms Keys — Read Marker — Moved** is the key holder telling them: *the member's read marker for room R, as member M, is now this*. A device takes the marker, recounts what is unread, and clears a notification for a message at or before it.

It goes only to the member's own clients that are **watching the room** through the key holder ([`rooms/keys/watch`](../../../watch/0.1/spec.md)), never to the client that moved the marker, and never to anyone else. It is one-way: there is no `#response`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Producer

A conforming **producer** (the key holder):

1. **MUST** send it only after a [`rooms/keys/read-marker/set`](../../set/0.1/spec.md) that moved the marker, and only to a client that holds a watch armed through [`rooms/keys/watch`](../../../watch/0.1/spec.md) on the same room as the same member, and that may act as that member when it is sent. It **MUST NOT** send it to the client that issued the set, to a delegate, or to any other party.
2. **MUST** carry the room and the member exactly as the client named them when it armed its watch, and the marker in force when it sends — the newest, never one since replaced — and **MUST NOT** carry anything else: not which client set the marker, not when, and nothing in `ext` about either.
3. **MAY** wait a short while — a few seconds — after a set before sending, and send only the marker in force at the end of it, so that a member moving through a room on one device does not send a stream of documents to another. It **MUST NOT** send more than one for one set.
4. **MUST** send it the way it sends the client any other document unasked, as [`rooms/keys/changed`](../../../changed/0.1/spec.md) Producer item 4 does, and treat delivery as the delivery layer's: a lost one costs the client nothing its next listing does not restore. It **MUST NOT** queue or re-send one the delivery layer accepted.
5. **MUST NOT** treat sending it as consuming the client's watch. A watch is consumed by the host's signal ([`rooms/keys/watch`](../../../watch/0.1/spec.md) Recipient item 8), and this is not one.
6. **SHOULD NOT** wake the client's device itself with [`push/wake`](../../../../../push/wake/0.2/spec.md), as for `rooms/keys/changed` (its Producer item 5).

### Consumer

A conforming **consumer** (the member's client):

1. **MUST** act on it only when it comes from its own key holder, for a room and member it watches through that key holder, and drop any other silently.
2. **MUST** take the marker as the one in force — replacing its own, whether it is further on or behind, because a marker behind the client's own is the member marking the room unread on another device — and recount what is unread against it ([`rooms/changed`](../../../../changed/0.1/spec.md#what-a-reader-does-with-a-signal), step 4).
3. **SHOULD** remove any notification it has raised for a message at or before the marker, and **MUST NOT** raise a notification because one arrived. Where the platform lets it, it **SHOULD** handle it without a visible alert.
4. **MUST NOT** treat it as a signal that the room changed: it says nothing about the room's records, and **MUST NOT** re-arm or cancel its watch because of it.
5. **MUST**, when it has shown its person the room further than the marker since — a set of its own still in flight — keep its own position and set it again ([`rooms/keys/read-marker/set`](../../set/0.1/spec.md)), rather than drop what its person has seen.

### Who receives it

**The member's clients watching the room, and no others** (Producer item 1). The other choice weighed was wider:

- **Every client of the member the key holder knows** would reach a device that has not looked at the room in months, or never will — a command-line tool, a browser session long closed — with a document, and through its mediator and push path a wake, each time the member reads any room anywhere. That turns one person's reading on one device into traffic, and timing, on every other device's path, for no one's benefit. Many such clients, too, have no address the key holder sends to unasked.
**Watching clients** are exactly the ones that can have something to clear. A client that raised a notification for a room did so on a signal it received through a watch ([`rooms/keys/changed`](../../../changed/0.1/spec.md)), and re-armed after reading ([`rooms/keys/watch`](../../../watch/0.1/spec.md) Producer item 4); it has said it receives documents from its key holder unasked (that task's Producer item 5); and the key holder already keeps a record naming it, the member and the room, so sending needs no new record. A client that polls rather than watches finds the marker on its next [`rooms/keys/list`](../../../list/0.3/spec.md) 0.3.

## Definitions

- `ReadMarker` — see [`rooms/_shared/0.1/read-marker.schema.json`](../../../../_shared/0.1/read-marker.schema.json).
- **Watching client** — a client holding a watch armed through `rooms/keys/watch` that has not fired, been cancelled or lapsed.

## Request

The key holder (`issuer`) sends the client (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json). There is no response.

### The member read the room on their phone; the laptop is told

```json
{
  "id": "urn:uuid:5e2a9d10-1f3c-4b7e-9a64-0c8d2b7f4001",
  "type": "https://trusttasks.org/spec/rooms/keys/read-marker/moved/0.1",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkLaptopHolderExample",
  "issuedAt": "2027-02-01T10:20:04Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "readMarker": {
      "version": 415,
      "createdAt": "2027-02-01T10:14:02Z"
    }
  }
}
```

## Response

None. The task is one-way, and its schema declares no `#response` form.

## Security & Privacy

### Data carried

A room, a member and a marker. The room and member are what the client named when it armed its watch, and the marker is what [`rooms/keys/list`](../../../list/0.3/spec.md) 0.3 would return to it; nothing else is admitted: the schema is closed, and Producer item 2 keeps `ext` free of which device set the marker and when.

### Correlation

**The key holder learns nothing it did not know**: it is sending what a set just told it, to clients whose watches it already records. It learns nothing back, because there is no reply.

**Each recipient's path learns when the member read.** The document travels encrypted to the client; its mediator, and the push gateway and platform push service if the device is woken, see that something arrived, and when — so the arrivals are a coarse record of when the member reads rooms on their other devices. That is why only watching clients receive it (*Who receives it*), why a key holder may coalesce a burst into one (Producer item 3), and why it does not wake a device itself (Producer item 6). A member who minds it can have their clients keep their read markers to themselves, as they did before this task.

**No host and no other member learns anything.** Nothing here reaches a host or a room. A client **MUST NOT** pass the marker, or a read state derived from it, to a host, a room, a community or another member ([`rooms/keys/read-marker/set`](../../set/0.1/spec.md), *Where a marker goes*).

### Retention

Nothing is kept beyond the marker itself. The client replaces its marker with the one received and keeps nothing else of the document; the key holder keeps no record of sending it.

### Consent/purpose

The member's own devices agreeing on what the member has read. A key holder sends nothing else under this type, and a client uses it only to update what it shows as unread.
