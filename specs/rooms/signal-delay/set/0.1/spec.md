---
slug: rooms/signal-delay/set
version: "0.1"
title: "Rooms — Signal Delay — Set"
summary: "A room's administrator changes whether the room's host waits a randomised interval before it sends a watch signal — on by default, off for a room where members expect to see a write at once, such as a conversation."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - watch
  - signal
  - latency
  - chat
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Turning the delay off changes what the timing of every member's signals reveals. The change must be attributable independently of the transport it arrived on, because members are entitled to know who decided it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed change would restore a setting an administrator has since replaced — possibly turning the delay off again after the room decided to keep it. Placing it in time is what lets a host refuse a stale one.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the room's signal delay. Applies to every signal the host sends from then on; a signal already waiting is sent as it would have been. A change to `randomised` also ends every live watch on the room.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a setting and a reason; the response, the setting now in force and the one it replaced. A setting every member of the room can read anyway (`rooms/info`).
retention:
  class: durable
  rationale: >-
    The setting is kept for the life of the room, and each change to it — old setting, new setting, reason, requester where the tier discloses one — is kept in the room's audit trail.
errorCodes:
  - code: rooms/signal-delay/set:signalDelayNotPermitted
    meaning: "The host's governance does not permit this signal delay — commonly a host that keeps the delay for every room it holds. The host records nothing rather than a setting other than the one asked for."
    retryable: false
  - code: rooms/signal-delay/set:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/signal-delay/set:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
related:
  - rooms/changed
  - rooms/watch
  - rooms/info
  - rooms/records/retention/set
---

## Abstract

A room's **administrator** changes the room's **signal delay**
([`SignalDelay`](../../../_shared/0.2/room.schema.json)): whether its host waits a short,
randomised interval before sending a [`rooms/changed`](../../../changed/0.1/spec.md) signal to a
member's armed [`rooms/watch`](../../../watch/0.2/spec.md).

The delay is the default, and it is there so that a signal's time does not mark one write
exactly. It costs a few seconds on every signal. In a room used as a document store that is
nothing; in a room used as a conversation it is a few seconds between one member writing and
another seeing it, on every message. This task lets a room that is a conversation say so.

The setting is the room's, not a member's, and it can be turned back on at any time. A room whose
signal delay is `none` also lets its members hold **live** watches
([`rooms/watch`](../../../watch/0.2/spec.md)) while a conversation is open on screen: a signal per
change rather than one per arming.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **host** (`recipient`):

1. **MUST** verify the presentation as [`rooms/records/put`](../../../records/put/0.5/spec.md)
   does and require that it confer `admin` at this room's scope; a presentation that does not is
   refused with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
2. **MUST** refuse with `signalDelayNotPermitted`, recording nothing, a setting its governance
   will not apply, and **MUST NOT** record one it will not apply: a room whose members read
   `none` while the host still waits would only be slower than it says, but a room whose members
   read `randomised` while the host sends at once has lost a protection it believes it has.
3. **MUST** record `signalDelay` as the room's signal delay, and from then on **MUST** serve it in
   [`rooms/info`](../../../info/0.4/spec.md) 0.4 and apply it to every `rooms/changed` signal it
   sends for the room: with `randomised`, the wait of `rooms/changed` producer item 4; with
   `none`, no wait of its own.
4. **MUST** answer with the setting now in force and the one it replaced, `randomised` for a room
   that had never set one. A request that names the setting already in force succeeds and changes
   nothing, so that a retry after a lost response is harmless.
5. **MUST** keep in the room's audit trail the setting replaced, the one recorded, the `reason`,
   and the requester where the tier discloses one.
6. **MUST NOT** change, because of this task, anything but the timing of signals and whether live
   watches may be held: not which one-shot watches are armed, not how many signals a one-shot watch
   sends, and not what a signal carries. On a change to `randomised` it **MUST** end every live
   watch on the room at once ([`rooms/watch`](../../../watch/0.2/spec.md) Host item 6), sending
   nothing to their addresses; each member learns of it when it next renews its live watch, which
   is refused, and arms a one-shot watch instead. On a change to `none`, no watch changes until a
   member asks for a live one.

A conforming **producer** **SHOULD**, before sending `none`, show whoever is deciding what it gives
up (*What `none` gives up*, below), and **SHOULD** show the room's members the signal delay in
force where it shows the room's other terms.

## Authorization

`admin` at the room's scope, the grant
[`rooms/records/retention/set`](../../../records/retention/set/0.1/spec.md) takes. The setting
changes what the timing of every member's signals reveals, including signals the member asked for
and the administrator did not; that is a term of the room, decided by whoever decides its other
terms, and not something `write` or `curate` should reach.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), a verified `proof` establishes who
asked, not that they may. The entitlement is the chain conferring `admin`, which the room issued.

## Why the room decides, not each watcher

A watcher might reasonably think the delay is its own business: it is the one waiting. It is not.
Someone who can see requests reach the host and signals leave it — the host's network, or a
party watching both ends — can line up a write with a signal that follows it at once. That tells
them when the writer wrote, and that the writer and the watcher share a room. The delay blurs both
for every writer in the room, and a single watcher with no delay would undo it for all of them.
So the room makes the choice, every member can read it, and no watch request can change it.

## What `none` gives up

In a busy room, little: when members write every few seconds, the rhythm of the room is visible
from its writes alone, and a few seconds of jitter on each signal hides little more. In a quiet
room, more: one write followed by one signal at once is easy to match. An administrator choosing
`none` for a room that is a conversation is making a fair trade; one choosing it for a room that
is written to twice a week is making a poor one, and a surface **SHOULD** say so.

`none` removes only the host's own wait. A signal still crosses the host's transport, its
mediator and, for a sleeping device, a push gateway, each of which takes the time it takes.

`none` also permits **live** watches ([`rooms/watch`](../../../watch/0.2/spec.md)): while a member
has the room open, a signal per change, coalesced to at most one a second, rather than one per
arming. A member reading an open conversation re-arms after every signal anyway, so what the
address's mediator and push path see changes little; but a live watch keeps sending while the
member's device is not reading, for up to two minutes per renewal, and so shows them how busy the
room is and when each write happened. An administrator choosing `none` is choosing that as well.

## Definitions

**Signal delay** — [`SignalDelay`](../../../_shared/0.2/room.schema.json): `randomised` (the
default) or `none`. A room that has never set one has `randomised`.

**`signalDelay`** — the setting from now on. REQUIRED.

**`reason`** — why, for the room's audit trail. REQUIRED.

**`previous`** (response) — the setting replaced.

## Request

An **Administrator** sends this to the room's **Host**. The payload is the top-level schema in
[`payload.schema.json`](payload.schema.json).

### A team's chat room turns the delay off

```json
{
  "id": "urn:uuid:7a1c3e52-9b4d-4f6a-8e20-1d5c7b9a3e01",
  "type": "https://trusttasks.org/spec/rooms/signal-delay/set/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-11-11T09:00:00Z",
  "threadId": "urn:uuid:7a1c3e52-9b4d-4f6a-8e20-1d5c7b9a3eff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.admin-leaf.sig",
        "eyJhbGciOiJFZERTQSJ9.authority-root.sig"
      ]
    },
    "signalDelay": "none",
    "reason": "This room is the team's live conversation; messages should arrive as they are sent."
  }
}
```

## Response

The **Host** responds with the sub-schema reachable via `$anchor: "response"`. Failures use a
`trust-task-error` document rather than a `#response`, per
[SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Recorded

```json
{
  "id": "urn:uuid:7a1c3e52-9b4d-4f6a-8e20-1d5c7b9a3e02",
  "type": "https://trusttasks.org/spec/rooms/signal-delay/set/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-11T09:00:01Z",
  "threadId": "urn:uuid:7a1c3e52-9b4d-4f6a-8e20-1d5c7b9a3eff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup",
    "signalDelay": "none",
    "previous": "randomised"
  }
}
```

## Security & Privacy

### Data carried

A room identifier, a setting and a reason in; the setting in force and the one it replaced out.
No record, key or content crosses. A producer **MUST NOT** put a member identifier or record
content in `reason`, which the host keeps for the life of the room.

### Correlation

To the host this is an administrator deciding how the room's signals are timed; on `open` and
`attributed` rooms, which one. With `none` in force, the correlation described in
[`rooms/changed`](../../../changed/0.1/spec.md#correlation) is no longer blurred by the host's wait:
a signal's time is close to the time of the write that fired it. That is the whole of what the
setting changes, and it is why the setting is the room's and is served to every member.

### Retention

The setting is kept for the life of the room, and each change to it in the room's audit trail.
Nothing a member wrote is kept or removed by this task.

### Consent/purpose

The setting exists so that a room used as a conversation can behave like one without every room
giving up the default. A change to it is a change to what every member's signals reveal, which is
why it takes the grant that decides the room's other terms, carries a reason, and is served to
every member.

This specification describes purpose only. Per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that
consent, approval or a step-up is required.
