---
slug: rooms/watch
version: "0.2"
extends: "0.1"
title: "Rooms — Watch"
summary: "A member asks a room's host for a content-free signal at an address it chooses when the room moves past the version it has read to: one per arming, or, on a room that sends signals at once, live — one per change for a few minutes, while a conversation is open."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - watch
  - signal
  - wake
  - sync
  - chat
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A watch is authorized by a presentation, as a listing is, and binds the host to send to an
    address the member names. Its integrity must not depend on the transport carrying it, or a
    channel could point a member's watch at an address of its own choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A watch arms the host to send to an address, and a replayed request would re-arm one the member
    has cancelled, or arm it at a watermark the member has since read past. Placing it in time is
    what lets a host refuse a stale one.
sideEffects:
  level: mutating
  rationale: >-
    Arms, re-arms, renews or cancels the host's record of one watch: a room, an address, a
    watermark, a lease and whether it is live. Nothing in the room changes.
subjectPath: /roomId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the room, an address DID, a version, a lease and whether the watch is live. Out: whether
    the watch is armed, whether it is live, and until when. The address is a routing identifier the
    member chooses for this purpose, not data about a person.
retention:
  class: durable
  rationale: >-
    A host keeps a watch until it fires, is cancelled or its lease ends — at most 30 days, and at
    most 120 seconds for a live watch the member does not renew — and then forgets it. It is not a
    record of the room and is never part of one.
errorCodes:
  - code: rooms/watch:addressUnreachable
    meaning: "The address does not resolve to a DID document, or its document advertises no transport the host speaks. Nothing is armed."
    retryable: false
  - code: rooms/watch:limitReached
    meaning: "The host holds as many watches for this room, or for this presenter where the tier discloses one, as it allows. Nothing is armed; a member MAY cancel a watch it no longer needs and try again."
    retryable: false
  - code: rooms/watch:liveNotPermitted
    meaning: "A live watch was asked for on a room whose signal delay is not `none`, or on a room where the host serves no live watches. Nothing is armed, and a live watch the request named is left as it was; the member arms a one-shot watch instead."
    retryable: false
  - code: rooms/watch:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/watch:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
related:
  - rooms/changed
  - rooms/records/list
  - rooms/keys/read
  - rooms/signal-delay/set
  - rooms/info
  - push/wake
---

## Abstract

The **Rooms — Watch** Trust Task lets a member learn that a room has changed without polling it.
The member names an **address** — a DID it controls, reached through its own mediator — and the
version it has read the room to. The host sends a [`rooms/changed`](../../changed/0.1/spec.md)
signal to that address when the room moves past that version.

A watch is one of two kinds:

- **One-shot** (the default). The host sends one signal the first time the room moves past the
  version, and then nothing more until the member arms the watch again. However busy the room, an
  address holds at most one signal per watch.
- **Live**, for a conversation that is open on a member's screen. The host sends a signal each time
  the room moves past the version it last signalled, at most about once a second, for a lease of at
  most two minutes that the member renews — about once a minute — while the screen stays open. It saves a re-arming round
  trip per message, and it is permitted only on a room whose **signal delay** is `none`
  ([`rooms/signal-delay/set`](../../signal-delay/set/0.1/spec.md)): a room that has already chosen
  to let its signals' timing follow its writes.

The signal carries the room and its latest version, and nothing else. What changed, who changed
it, and on the sealed tiers even what kind of record it was, the member learns the way it always
has: by listing the room with `sinceVersion` ([`rooms/records/list`](../../records/list/0.7/spec.md))
and opening what it finds. A watch adds no new way to read a room; it only says when reading is
worth doing.

Delivery to a phone that is asleep needs nothing new either. The signal is a Trust Task document
addressed to a DID, so it waits in that DID's mediator queue like any other, and a mediator that
is a trigger for the device ([`push/wake`](../../../push/wake/0.2/spec.md), the
[push binding](../../../../bindings/push/0.1/spec.md)) wakes the device with the binding's
contentless doorbell. The push gateway and the platform push service see a wake; they never see
the room.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

* A request member, **`live`** (`true` only): arm, or renew, a **live** watch — a signal each time
  the room moves past the version last signalled, coalesced to at most one a second, under a lease
  of at most 120 seconds. Permitted only on a room whose signal delay is `none`; otherwise refused
  with the new code **`rooms/watch:liveNotPermitted`**.
* A response member, **`live`** (`true` only), present when the watch armed is live.
* A watch without `live` is the 0.1 watch, unchanged.

The request and response are closed objects, so a 0.1 reader would refuse the new members; a new
member on a closed payload is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules).
0.2 declares that it extends 0.1: a host that implements only 0.1 reads a live request as a one-shot
one and answers without `live`, which is how a member learns that it got a one-shot watch (Producer
item 6). A host that implements 0.2 **SHOULD** continue to answer 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### One signal per arming, or a live watch

A watch is **armed** by a request without `cancel`. A **one-shot** watch — a request without
`live` — **fires** once: the host sends one signal and disarms it. The member arms it again after
it has caught up, with the version it has now read to. This is what bounds everything downstream:
however busy a room is, and however long a device is offline, an address holds at most one signal
per one-shot watch in its mediator's queue, and a phone is woken at most once per watch until its
member has looked.

A **live** watch — a request with `live: true` — does not fire and disarm. It sends a signal each
time the room moves past the version it last signalled, coalesced (Host item 5), until its lease
ends. It exists for a screen on which the member is reading the room as it changes: there, a
one-shot watch costs a re-arming round trip per message and adds nothing, because the member reads
after every signal anyway. It gives up the bound above for the few minutes it lasts, which is why it
is short, renewed only while the screen is open, and permitted only where the room has already
given up the protection a per-change signal would undo (*Live watches and the signal delay*).

A conforming **producer** (the member) **MUST**:

1. Present a chain conferring `read` at the room's scope, exactly as for
   [`rooms/records/list`](../../records/list/0.7/spec.md).
2. Name in `address` a DID whose keys it, or its own agent, holds, and through which it receives
   Trust Task documents. On a `private` room the address **MUST** be a DID it uses for nothing
   else: not the persona it presents as, not the address of a watch on another room, and not one
   it has given anyone else (*Correlation*).
3. Set `sinceVersion` to the highest version it has read the room to, when it has one.
4. Re-arm a one-shot watch only after it has read the room since the signal it last received — or
   when it has received none within the lease — and **SHOULD NOT** re-arm more often than it
   reads.
5. Ask for a live watch only while its person has the room open and is reading it as it changes,
   and only on a room whose signal delay ([`rooms/info`](../../info/0.4/spec.md) `signalDelay`) is
   `none`. It renews the watch — the same room and address, with `live: true` — before `expiresAt`,
   about every 60 seconds, for as long as the room is on screen and the application is in the
   foreground, and **MUST** stop renewing it, and **SHOULD** cancel it, when the
   screen closes or the application leaves the foreground. It **SHOULD NOT** hold a live watch on a
   room its person is not looking at: that is what a one-shot watch is for.
6. Treat a watch as live only when the response says `live: true`. A response without it — from a
   host that implements only 0.1 — is a one-shot watch, and the producer re-arms it as one. On
   `rooms/watch:liveNotPermitted` it **MAY** arm a one-shot watch instead, and does not ask for a
   live one on that room again until the room's signal delay is `none`.

A conforming **consumer** (the host) **MUST**:

1. Apply the chain rules of [`rooms/records/put`](../../records/put/0.5/spec.md) to the
   presentation, and refuse one that does not confer `read` at this room's scope, whose chain does
   not reach the room, or whose leaf is not the party it authenticated, with the standard
   `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
2. Resolve `address` before arming, and refuse with `rooms/watch:addressUnreachable` an address
   whose document it cannot resolve or which advertises no transport it speaks. It **MUST NOT**
   send anything to the address in the course of this request.
3. Hold at most one watch per room and `address`: a request naming a watch it holds replaces its
   watermark, its lease and its kind — a request with `live` makes it live, one without makes it
   one-shot — and re-arms it if it has fired. A request that renews a live watch and omits
   `sinceVersion` keeps the watch's watermark (the version it last signalled); one that carries it
   replaces the watermark, and the host signals at once if the room is above it.
4. For a one-shot watch, send one [`rooms/changed`](../../changed/0.1/spec.md) to the address the
   first time the room's latest version is above the watermark — at once if it already is — and
   then disarm the watch. It **MUST NOT** send a second signal for a one-shot watch until it is
   armed again.
5. For a live watch, send a `rooms/changed` each time the room's latest version is above the
   watermark — at once if it already is — and then set the watermark to the version it signalled.
   Changes that land while a signal is being sent, or within the host's minimum spacing of the
   previous signal, are **coalesced** into the next signal, which carries the latest version when it
   is sent. The minimum spacing is at least one second per watch, and a host **MAY** space signals
   further apart. A host **MUST NOT** keep a queue of one signal per change, and **MUST NOT** resend
   a signal: one that is lost is recovered by the next signal, or by the member's next listing.
6. Arm or renew a live watch only on a room whose signal delay is `none`, and refuse one on any
   other room with `rooms/watch:liveNotPermitted`, leaving any watch the request named as it was. A
   host **MUST NOT** hold a live watch on a room whose signal delay is `randomised`, and **MAY**
   refuse live watches altogether with the same code. When a room's signal delay changes to
   `randomised`, the host **MUST** end every live watch on the room at once
   ([`rooms/signal-delay/set`](../../signal-delay/set/0.1/spec.md)), sending nothing to its address;
   the member learns of it when it next renews, which is refused with `liveNotPermitted`, and arms
   a one-shot watch instead.
7. Grant a live watch a lease of at most 120 seconds from the request, whatever `leaseSeconds`
   asks, and answer `live: true` with it. A renewal grants a new lease from the renewal.
8. Forget a watch when a one-shot watch fires, when it is cancelled, and at `expiresAt`, which
   **MUST NOT** be later than the earliest instant at which any credential of the presentation
   stops being valid. It **MUST** forget a watch whose presentation a `RevocationList` it accepts
   for the room revokes, as it would refuse that presentation (`AuthorityPresentation`). A live
   watch ends the same way, and sends nothing after it ends.
9. Send the same signal whatever moved the room — a put of any kind, a curation, a retraction, a
   prune, a profile written — and **MUST NOT** carry in it, or vary its timing or size by, anything
   about what changed.
10. Treat a watch as granting nothing. It is not a membership record, never consulted to authorize
    anything, and never shown to another member, to the room's owner, or to anyone else.

A host **MAY** bound how many watches it holds per room, and per presenter where the tier
discloses one, and refuses beyond that with `rooms/watch:limitReached`. A host that serves this
task **SHOULD** say so through `trust-task-discovery`, so that a member knows whether to watch or
to poll.

**A repeated request** sets a state: arming a watch already armed at the same watermark succeeds
with the same answer, renewing a live watch extends its lease, and cancelling a watch the host
does not hold answers `armed: false`. A host ignores `live`, `sinceVersion` and `leaseSeconds` on a
cancel. **Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless.

### Live watches and the signal delay

A live watch sends a signal per change, a few hundred milliseconds after it. That tells the
address's mediator, the push path, and anyone who can watch traffic to the address, how busy the
room is and when each write happened — the two things a one-shot watch's single signal and the
host's randomised wait ([`rooms/changed`](../../changed/0.1/spec.md) producer item 4) keep from
them. A room whose signal delay is `none` has already decided that its signals' timing may follow
its writes, for every member, and a member reading it on an open screen re-arms after every signal
anyway, so a live watch there adds little to what one-shot watches already show. On a room that
keeps the delay it would undo the delay for every writer in the room, and the setting is the
room's, never a watcher's: so the host refuses it.

### A watch does not move with the room

A watch is the host's record, not the room's. When a room changes host
(`vtc/rooms/storage/migrate`, `rooms/owner/transfer`) or is reset, its watches — live ones too —
stay behind and lapse. A member that reads the room at its new host arms a new watch there.

## Authorization

**Authority: `read` at the room's scope, conferred by a chain the room issued** — the same grant
a listing takes, because a signal tells the member nothing a listing would not. The host keeps no
roster and consults none: the presentation is the whole of the authority, verified as for every
other task in the family. The proof establishes who asked and that the request is unaltered,
never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A live watch takes no more authority than a one-shot one. Whether one is permitted is the room's
signal delay, which a holder of `admin` sets, never the watcher.

The address needs no authority of its own. It receives only what the presenter could read: at
most one document per one-shot arming, and for a live watch at most one a second for at most 15
minutes per renewal.

## Definitions

- **`roomId`** — the room.
- **`presentation`** — an [`AuthorityPresentation`](../../_shared/0.2/room.schema.json) conferring
  `read`.
- **`address`** — the DID the signal is sent to, chosen by the member. With `roomId`, it identifies
  the watch.
- **`sinceVersion`** — the member's watermark: the host signals once the room is above it.
- **`leaseSeconds`** — how long the member asks the watch to last if it does not fire.
- **`live`** — arm or renew a live watch.
- **`cancel`** — end the watch.
- **One-shot watch** — a watch armed without `live`: one signal, then disarmed.
- **Live watch** — a watch armed with `live: true`: a signal per change, coalesced, for a lease of
  at most 120 seconds that the member renews.
- **Response** — `armed`, `live` when the watch is live, and `expiresAt` when it is armed.

## Request

The member (`issuer`) sends the host (`recipient`) a document whose `payload` validates against
the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet arms a watch after catching up

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1001",
  "type": "https://trusttasks.org/spec/rooms/watch/0.2#request",
  "issuer": "did:webvh:QmAlice:personas.example:northwind",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2027-02-01T09:00:00Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOi...membership",
      "authority": ["eyJhbGciOi...authority"]
    },
    "address": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.SeyJ0IjoiZG0iLCJzIjoiZGlkOndlYjptZWRpYXRvci5leGFtcGxlIn0",
    "sinceVersion": 412,
    "leaseSeconds": 604800
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T09:00:00Z",
    "verificationMethod": "did:webvh:QmAlice:personas.example:northwind#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### A chat screen opens a live watch

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1004",
  "type": "https://trusttasks.org/spec/rooms/watch/0.2#request",
  "issuer": "did:webvh:QmAlice:personas.example:northwind",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2027-02-01T09:30:00Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup",
    "presentation": {
      "membership": "eyJhbGciOi...membership",
      "authority": ["eyJhbGciOi...authority"]
    },
    "address": "did:peer:2.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.Vz6MkfQwwNSZq3DwuHyBN9yS7qnQrRwJ7h4uYzaUJDEFzfW6Y.SeyJ0IjoiZG0iLCJzIjoiZGlkOndlYjptZWRpYXRvci5leGFtcGxlIn0",
    "sinceVersion": 2071,
    "leaseSeconds": 120,
    "live": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T09:30:00Z",
    "verificationMethod": "did:webvh:QmAlice:personas.example:northwind#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Cancelling it

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1003",
  "type": "https://trusttasks.org/spec/rooms/watch/0.2#request",
  "issuer": "did:webvh:QmAlice:personas.example:northwind",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2027-02-03T18:00:00Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOi...membership",
      "authority": ["eyJhbGciOi...authority"]
    },
    "address": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.SeyJ0IjoiZG0iLCJzIjoiZGlkOndlYjptZWRpYXRvci5leGFtcGxlIn0",
    "cancel": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-03T18:00:00Z",
    "verificationMethod": "did:webvh:QmAlice:personas.example:northwind#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The host answers with a document of type `https://trusttasks.org/spec/rooms/watch/0.2#response`,
whose `payload` validates against the `$anchor: "response"` sub-schema in
[`payload.schema.json`](payload.schema.json): the room and address echoed, `armed`, `live` when the
watch is live, and `expiresAt` when armed. Failures use `trust-task-error`
([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Armed

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1002",
  "type": "https://trusttasks.org/spec/rooms/watch/0.2#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:webvh:QmAlice:personas.example:northwind",
  "issuedAt": "2027-02-01T09:00:01Z",
  "threadId": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1001",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "address": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.SeyJ0IjoiZG0iLCJzIjoiZGlkOndlYjptZWRpYXRvci5leGFtcGxlIn0",
    "armed": true,
    "expiresAt": "2027-02-08T09:00:01Z"
  }
}
```

### Armed live, for two minutes

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1005",
  "type": "https://trusttasks.org/spec/rooms/watch/0.2#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:webvh:QmAlice:personas.example:northwind",
  "issuedAt": "2027-02-01T09:30:01Z",
  "threadId": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1004",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup",
    "address": "did:peer:2.Ez6LSg8zQom395jKLrGiBNruB9MM6V8PWuf2FpEy4uRFiqQBR.Vz6MkfQwwNSZq3DwuHyBN9yS7qnQrRwJ7h4uYzaUJDEFzfW6Y.SeyJ0IjoiZG0iLCJzIjoiZGlkOndlYjptZWRpYXRvci5leGFtcGxlIn0",
    "armed": true,
    "live": true,
    "expiresAt": "2027-02-01T09:32:00Z"
  }
}
```

## Security & Privacy

### Data carried

The request carries a room, a presentation the host already verifies for every read, an address
DID, a version, a lease and whether the watch is live; the response, whether the watch is armed,
whether it is live, and until when. Nothing about the room's content travels in either direction,
and nothing about it travels in the signal ([`rooms/changed`](../../changed/0.1/spec.md)) either:
the room and its latest version, which the member could have had from a listing.

**On the sealed tiers the host cannot tell a reaction from a message.** It never sees a record's
`kind` (`SealedRecordBody`, *Kind*), so every put fires an armed watch alike, and so does every
curation, retraction, prune and profile write. A watch therefore wakes a member's device for a
reaction just as for a message, and this task cannot change that without the host learning what
the sealed tiers exist to keep from it. What a member's client does about it — deciding after it
reads whether there is anything to alert its person to — is in
[`rooms/changed`](../../changed/0.1/spec.md#what-a-reader-does-with-a-signal). An `open` room's
host could tell; it sends the same signal anyway, so that the signal is one shape on every tier.

### Correlation

**The host learns no more than polling tells it.** It already sees every write and every read. A
watch adds the address and the moments the member re-arms, which are moments the member read. A
live watch adds the moments the member renews, which say that a screen was open on the room, and
for how long: a client that read the room as often as it changed would show the host the same.

**The address is a stable handle for one member at the host.** Re-arming the same address links
the member's reads over time. On `open` and `attributed` rooms the host attributes every read
anyway, so this adds nothing. On a `private` room it does: reads are presented in zero knowledge
and the host cannot otherwise tell one member's from another's, but the re-arming times of one
address are one member's reading pattern, unattributed but linked. That is why producer item 2
requires a `private` room's address to be used for nothing else, and why a member for whom even an
unattributed pattern matters **SHOULD** arm each time with a new address — a new address is a new
watch — or poll instead. A live watch keeps its address across its renewals, which links one
sitting with the room and no more; a member that minds **SHOULD** take a new address for its next
one. The address's DID document also names the mediator that receives for it, which a host learns;
a member whose mediator would identify it **SHOULD** not watch a `private` room.

**Nobody on the signal's path learns which room changed.** The signal is a Trust Task document
encrypted to the address by its transport binding: the address's mediator sees a document for the
address, the push gateway and the platform push service see a contentless wake
([push binding §2](../../../../bindings/push/0.1/spec.md)). What they all see is that something
arrived, and when. Host item 9 keeps the signal identical whatever changed. For a one-shot watch,
one signal per arming keeps their count from saying how busy the room was.

**A live watch gives that up, for as long as it lasts.** Its signals arrive one per change, at most
one a second, as the changes happen, so the address's mediator, the push path and anyone watching
traffic to the address learn how busy the room is and, to within the spacing, when each write
happened — and, by lining a write that reaches the host up with a signal that leaves it, that the
writer and the watcher share the room. A one-shot watch with no delay already shows most of this
to the same parties while a member reads after every signal; a live watch shows it without the
member's reads between, including while the member's device is not reading at all. That is why it
is permitted only where the room's signal delay is `none` (Host item 6), lasts at most two minutes
per renewal (Host item 7), and is held only while the room is on screen (Producer item 5). A
device that stops reading without cancelling — killed, or asleep — is bounded by the lease alone,
which is why the lease is short: at the minimum spacing of one second it collects at most about
120 signals in its mediator's queue before the watch ends, and usually far fewer, because a room
sends nothing while nobody writes. The renewal that keeps the lease this short costs one small
request a minute, and only while someone is looking at the room.

### Retention

A host keeps a watch only until it fires, is cancelled or lapses, and **SHOULD NOT** keep the
address afterwards, nor a log of the watches it held beyond what its operation needs: a history of
addresses and re-arming times is the reading pattern above, kept. For a live watch it keeps the
version it last signalled, as the watch's watermark, and no record of when it signalled.

### Consent/purpose

The address is given to the host for one purpose: to send it `rooms/changed` for this room. A host
**MUST NOT** send anything else to it, give it to anyone, or use it to link a member across rooms.
A member asks for a live watch to see a conversation as it happens, and a host uses it for nothing
else: not as evidence of who is reading the room, nor to tell anyone that a member is present.
