---
slug: rooms/keys/changed
version: "0.1"
title: "Rooms Keys — Changed"
summary: "A member's key holder passes a room host's content-free signal on to the client that asked it to watch the room — the room, the member, the host and the room's latest version, nothing about what changed, and no response."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - watch
  - signal
  - wake
  - unread
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
    The client acts on a signal only when it comes from its own key holder. The transports a key
    holder sends it on authenticate the sender; a proof says the same where one does not. What the
    client then does — read the room through its key holder — is authorized and answered as every
    read is.
sideEffects:
  level: none
  rationale: >-
    A hint. The client reads the room, or does not; nothing is stored or changed because a signal
    arrived.
subjectPath: /member
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The room, the member and host the client named when it armed the watch, and the room's latest
    version, which a listing would have shown it anyway. Nothing about what changed, who changed
    it, or how much.
retention:
  class: transient
  rationale: >-
    The client uses the signal to decide to read the room and keeps nothing of it; what it reads is
    what it keeps.
errorCodes: []
related:
  - rooms/keys/watch
  - rooms/changed
  - rooms/watch
  - rooms/keys/browse
  - rooms/keys/read
  - push/wake
---

## Abstract

The **Rooms Keys — Changed** Trust Task is the last hop of a watch armed through a key holder.
A client asks its key holder to watch a room with [`rooms/keys/watch`](../../watch/0.2/spec.md);
the key holder arms [`rooms/watch`](../../../watch/0.2/spec.md) at the room's host, as the member,
with an address of its own; the host sends [`rooms/changed`](../../../changed/0.1/spec.md) to that
address; and the key holder sends **this** to the client: *room R, which you watch as member M at
host H, has moved past the version you had read to*. It adds to the host's signal only what the
client named when it armed — the member and the host — and says nothing about what changed. For a
one-shot watch the key holder sends one; for a live watch, which a client holds while a
conversation is on screen, it sends one for every signal the host sends, until the watch ends.

It is one-way: there is no `#response`. A reply would tell the key holder when the client's device
woke, which is more than it needs to know, and the host learns nothing from this hop at all.

The signal reaches a sleeping device the way every Trust Task document does: it waits in the
client's mediator queue, and a mediator that is a trigger for the device
([`push/wake`](../../../../push/wake/0.2/spec.md)) wakes it with the
[push binding](../../../../../bindings/push/0.1/spec.md)'s contentless doorbell.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the key holder) **MUST**:

1. Send it only as [`rooms/keys/watch`](../../watch/0.2/spec.md) Recipient item 8 says: for a
   `rooms/changed` from the host a watch was armed at, at that watch's address, for that watch's
   room, while the watch is armed, to the client that armed it — once for a one-shot watch, and then
   forget the watch; for each signal of a live watch, keeping the watch until it ends.
2. Carry the signal's `roomId` and `latestVersion` unaltered, and the watch's `member` and `host`
   as the client named them, and **MUST NOT** carry anything else — in `ext` or anywhere else —
   nor the address the watch was armed with.
3. Send it without delay of its own: the host has already applied the room's signal delay
   ([`rooms/changed`](../../../changed/0.1/spec.md) producer item 4), and the key holder neither
   holds it back for a later signal nor batches it with others — for a live watch no more than for
   a one-shot one, since the host has already coalesced its signals.
4. Send it to the client the way it sends that client any other document unasked — for a client
   whose DID advertises no service, through the mediator at which the client collects from its key
   holder — and treat delivery as the delivery layer's: a lost signal costs the client nothing it
   cannot recover by reading. It **MUST NOT** queue or re-send one the delivery layer accepted.
5. **SHOULD NOT** also wake the client's device itself with `push/wake`. The document waiting in
   the client's mediator queue is what a mediator wakes it for, and a key-holder wake per signal
   would show the push gateway how often the member's rooms move.

A conforming **consumer** (the member's client) **MUST**:

1. Act on a signal only when it comes from its own key holder, for a watch it armed with
   `rooms/keys/watch` — the same `member`, `host` and `roomId` — and drop any other silently.
2. Act on it only by reading the room — never by showing anything to its person on the strength of
   the signal alone — and **MUST NOT** treat `latestVersion` as evidence of anything but a reason to
   read.
3. Re-arm a one-shot watch with `rooms/keys/watch` after it has read, with the version it has read
   to, if it wants the next signal: one signal per arming holds end to end. A live watch needs no
   re-arming between signals; the client renews it while the room is on screen.

## What a reader does with a signal

Exactly what [`rooms/changed`](../../../changed/0.1/spec.md#what-a-reader-does-with-a-signal) says
a reader does, read as `member` at `host` through the key holder: skip a signal at or below its
watermark; list what changed with [`rooms/keys/browse/0.6`](../../browse/0.6/spec.md) and
`sinceVersion`; open the new and changed records with
[`rooms/keys/read/0.2`](../../read/0.2/spec.md) where the tier seals them; count as unread only a
current message, new to the reader, that is not the member's own; alert only on what it found, with
wording derived from the records it opened; then advance its watermark and, for a one-shot watch, re-arm. A signal can
mean a reaction, an edit, a curation, a prune or a profile written as well as a message, and on the
sealed tiers nobody before the reader can tell which.

**Records stored before their host kept `createdAt`.** A reader that keeps no copy of the room
counts a record as new when its `createdAt`, else its `updatedAt`, is after its read marker. A
record stored before the host recorded `createdAt` has only `updatedAt`, which moves when the record
is edited, so such a reader counts an edit of an old message as unread. That is accepted: it is
bounded to records older than the host's `createdAt`, errs toward showing the member something
rather than hiding it, and a reader that keeps a copy of the room tells an edit from a new message
by the key it has already shown.

## Request

The key holder (`issuer`) sends the client (`recipient`) a document whose `payload` validates
against the top-level schema in [`payload.schema.json`](payload.schema.json). There is no response.

### The room moved past version 412

```json
{
  "id": "urn:uuid:4a9e7c21-0d3b-4f5a-8e62-1b7c9d0f2001",
  "type": "https://trusttasks.org/spec/rooms/keys/changed/0.1",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkWalletHolderExample",
  "issuedAt": "2027-02-01T10:14:08Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "latestVersion": 415
  }
}
```

## Response

None. The task is one-way, and its schema declares no `#response` form.

## Security & Privacy

### Data carried

A room, a member, a host and a version. The first three are what the client named when it armed
the watch, and the version is what a listing would have shown it; nothing else is admitted: the
schema is closed, and producer item 2 keeps `ext` free of anything about the change and keeps the
address with the key holder.

### Correlation

The signal travels encrypted to the client. Its mediator sees a document from the key holder, and
the push gateway and the platform push service see a contentless wake: each learns that something
arrived, and when, and nothing of the room. For a one-shot watch, one signal per arming keeps their
number from saying how busy the room was, and the host's randomised delay, which this hop does not
undo, keeps their timing from marking single writes. A live watch gives up both while it lasts —
its signals follow the room's changes one for one, at most one a second — which is why it is held
only on a room whose signal delay is `none`, and only while the room is on screen
([`rooms/keys/watch`](../../watch/0.2/spec.md#correlation)). The key holder learns nothing from
sending one, because there is no reply. What the host learns is described in [`rooms/watch`](../../../watch/0.2/spec.md#correlation),
and what the key holder learns in [`rooms/keys/watch`](../../watch/0.2/spec.md#correlation).

### Retention

Nothing is kept. A reader acts on a signal by reading the room and keeps what it reads; the key
holder forgets a one-shot watch when it passes the signal on, and keeps a live watch's record —
never its signals — until the watch ends.

### Consent/purpose

The client asked for exactly this signal, for exactly these rooms. A key holder sends nothing else
under this type, and a reader uses it only to decide to read the room.
