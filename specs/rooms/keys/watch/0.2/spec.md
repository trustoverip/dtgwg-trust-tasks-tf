---
slug: rooms/keys/watch
version: "0.2"
extends: "0.1"
title: "Rooms Keys — Watch"
summary: "A member's client asks its key holder to watch rooms at one host for it — arming rooms/watch as the member at an address the key holder controls, one-shot or live while a conversation is open — and to pass the signals on, so a wallet need not poll."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - watch
  - signal
  - wake
  - persona
  - chat
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
subjectPath: /member
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request makes the recipient act outward in its member's name — presenting the member's
    room credentials to a host this request names — and binds the recipient to send to the caller
    whatever that host signals. One whose origin depended on the transport would let a compromised
    channel aim a member's credentials at a host of somebody else's choosing, or point the member's
    signals at a client that is not theirs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed request would re-arm a watch the caller has cancelled, or arm one at a watermark the
    caller has since read past. Placing it in time is what lets the recipient refuse a stale one.
sideEffects:
  level: mutating
  rationale: >-
    Arms, re-arms, renews or cancels watches at the host (rooms/watch), and keeps one small record
    per armed watch at the recipient until it fires, is cancelled or lapses. Nothing in any room
    changes.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    `actsAsSubject`: the recipient presents the member's own `read` authority to the host, as the
    member. In: a member, a host, rooms, watermarks and whether to watch live. Out: whether each
    watch is armed, whether it is live, and until when. The address the recipient arms with stays with the recipient; the caller never learns it.
retention:
  class: durable
  rationale: >-
    The recipient keeps one record per armed watch — the caller, the member, the room, the host,
    the address and when the watch lapses — because it must know, when a signal arrives at the
    address, whose it is and where to pass it on. It forgets the record when the watch fires, is
    cancelled or lapses — at most 30 days, and at most two minutes for a live watch the caller does
    not renew — and keeps nothing of the watermarks or the signals.
errorCodes:
  - code: rooms/keys/watch:memberNotPrincipal
    meaning: "`member` is not a DID the recipient holds keys for and the caller may act as. Nothing is armed or cancelled. `details.member` repeats it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member]
      properties:
        member:
          type: string
          maxLength: 2048
related:
  - rooms/watch
  - rooms/changed
  - rooms/keys/changed
  - rooms/keys/act
  - rooms/keys/browse
  - rooms/keys/read
  - rooms/keys/list
  - push/wake
---

## Abstract

[`rooms/watch`](../../../watch/0.2/spec.md) lets a member ask a room's host for a content-free
signal — [`rooms/changed`](../../../changed/0.1/spec.md) — at an address the member names, when the
room moves past the version the member has read to: one per arming, or, on a room whose signal
delay is `none`, one per change while a **live** watch lasts. Two things stop a wallet whose
member keys are held by its **key holder** (its agent) from using it:

- **It cannot present as the member.** The member is a persona the key holder holds, and the
  wallet must not sign the member's host request with a key of its own, because the host would
  take that key as the member. [`rooms/keys/act`](../../act/0.12/spec.md) completes a
  `rooms/watch` as the member, but leaves the address to the client.
- **It has no address.** A browser wallet's only identifier is the one its key holder knows it
  by — a `did:key`, which advertises no service a host could send to — and a phone that holds a
  DID behind its own mediator would have to give the host that DID, and on a `private` room a
  new one for every watch.

This task asks the key holder to do both. The client names the member, the host and the rooms,
with the version it has read each to. The key holder arms each watch at the host as the member —
its own presentation, its own signature — with an **address the key holder controls**, used for
that one watch and nothing else. When the host's signal arrives at that address, the key holder
passes it on to the client that asked with [`rooms/keys/changed`](../../changed/0.1/spec.md): the
room, its latest version, and nothing more. A client with a conversation open asks for the watch
**live**, and the key holder passes on every signal until the client stops renewing it. The host sees the member and an address; it never sees
the client or the key holder. The client sees the signal; it never sees the address.

**This is the intended shape for a mobile wallet whose member keys are at its agent.** The signal
it receives is a Trust Task document from its own agent, waiting in its own mediator's queue, and
a sleeping phone is woken for it by the [push binding](../../../../../bindings/push/0.1/spec.md)'s
contentless doorbell like any other. The phone holds no address a host knows, and registers no DID
per watch: its agent bears that cost.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

* A request member, **`live`** (`true` only): arm or renew each named room's watch at the host as a
  live [`rooms/watch/0.2`](../../../watch/0.2/spec.md) watch, and pass on **every** signal it sends
  rather than the first. It applies to every room the request names; a client with one open
  conversation sends it for that room alone.
* An outcome member, **`live`** (`true` only), present when the room's watch is armed live.
* A host's refusal of a live watch — `rooms/watch:liveNotPermitted`, on a room whose signal delay
  is not `none` — is reported, like any host refusal, as `hostRefused` with the code in `hostCode`.
* The key holder arms with [`rooms/watch/0.2`](../../../watch/0.2/spec.md). A request without
  `live` is the 0.1 request, unchanged.

The request and the outcome are closed objects, so a 0.1 reader would refuse the new members; a
new member on a closed payload is a `MINOR` increment per
[SPEC §5.2](/SPEC.md#52-compatibility-rules). 0.2 declares that it extends 0.1: a key holder that
implements only 0.1 reads a live request as a one-shot one and answers without `live`, which is how
a client learns that it got one-shot watches (Producer item 7). A key holder that implements 0.2
**SHOULD** continue to answer 0.1.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### One signal per arming, end to end — unless the watch is live

`rooms/watch` sends one signal per one-shot arming, and this task keeps that property across the
extra hop: for a one-shot watch the key holder passes on **at most one** `rooms/keys/changed` per
arming, and arms again only when the client asks it to. However busy a room is, and however long a
device is offline, the client's mediator queue holds at most one signal per one-shot watch, and
the client re-arms after it has read.

A **live** watch is the exception, for a conversation open on the client's screen: the host sends a
signal per change, coalesced to at most one a second, and the key holder passes on every one, as it
arrives, until the watch ends. It lasts at most two minutes per renewal, is permitted only on a room
whose signal delay is `none` ([`rooms/watch/0.2`](../../../watch/0.2/spec.md), *Live watches and the
signal delay*), and is held only while the room is on screen (Producer item 6).

### Producer

A conforming **producer** (the member's own client) **MUST**:

1. Use this task only when it acts **as** the member, as for [`rooms/keys/act`](../../act/0.12/spec.md)
   Producer item 1. A party acting under a chain the member attenuated to it is an agent, and
   watches for itself.
2. Name the member in `member` — one of the members [`rooms/keys/list`](../../list/0.3/spec.md)
   says it reads the rooms as — and name each room at most once.
3. Set each room's `sinceVersion` to the highest version it has read the room to, when it has one.
4. Re-arm a room's one-shot watch only after it has read the room since the signal it last
   received — or when it has received none before `expiresAt` — and **SHOULD NOT** re-arm more
   often than it reads.
5. Receive Trust Task documents from its key holder at the identifier it sends this task from, the
   way it receives any other document the key holder sends it unasked (for example
   [`task-consent/granted`](../../../../task-consent/granted/0.1/spec.md)): that is where the
   signals go.
6. Ask for a live watch (`live: true`) only for a room its person has open and is reading as it
   changes, and only where the room's signal delay ([`rooms/info`](../../../info/0.4/spec.md)
   `signalDelay`) is `none`. It renews the watch — a request naming the room with `live: true` —
   before `expiresAt`, about every 60 seconds, for as long as the room is on screen and the
   application is in the foreground, **MUST** stop renewing it when the screen closes
   or the application leaves the foreground, and **SHOULD** then cancel it. It **SHOULD NOT** hold
   a live watch on a room its person is not looking at, which is what a one-shot watch is for: each
   live watch is a stream of signal times to the host and to the mediators that carry them.
7. Treat a room's watch as live only when its outcome says `live: true`, and otherwise as one-shot.
   On a `hostRefused` outcome with `hostCode` `rooms/watch:liveNotPermitted` it **MAY** arm the room
   one-shot instead, and does not ask for a live watch on that room again until the room's signal
   delay is `none`. A live watch that the host ends — when the room's signal delay changes back to
   `randomised` — is learned of at the next renewal, which is refused that way.

### Recipient

A conforming **recipient** (the key holder) **MUST**:

1. Refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) a
   caller to which the member has delegated narrower standing, as
   [`rooms/keys/act`](../../act/0.12/spec.md) Recipient item 3 does: watching as the member is
   acting as the member.
2. Refuse with `memberNotPrincipal` a `member` it holds no keys for, or that the caller may not act
   as, and refuse with the standard `malformedRequest` a request naming a room twice. It **MUST
   NOT** pick a member for the caller. **Only the member's own personas are watched**: a watch
   is armed for a DID whose keys the recipient holds for its own principal, at the request of a
   client of that principal, and for no one else.
3. For each room, arm, renew or cancel the watch at `host` with [`rooms/watch/0.2`](../../../watch/0.2/spec.md),
   issued and signed by the member with a key of the member's DID document listed for
   `authentication`, carrying a presentation that confers **exactly** `read` at the room's scope,
   granted to the member — leaf first, and on a `private` room the same-subject binding — as
   [`rooms/keys/act`](../../act/0.12/spec.md) Recipient item 4 would insert it. Each room is a
   request of its own, with its own presentation and its own address: the recipient **MUST NOT**
   let the host see two rooms in one request, or one address on two rooms. On a request with
   `live`, it arms each room's watch with `live: true`, and **MUST NOT** fall back to a one-shot
   watch on the caller's behalf: whether to do that is the caller's decision (Producer item 7).
4. Report a room it cannot watch in that room's outcome rather than failing the request:
   `notAMember` when it holds no credential the room issued to the member, `actionNotHeld` when
   the member's credentials do not confer `read`, `hostUnreachable` when the host cannot be
   resolved, advertises no transport it speaks or does not answer, `limitReached` when it holds as
   many watches as it allows, and `hostRefused`, with the
   host's code unaltered in `hostCode`, when the host refuses — including
   `rooms/watch:liveNotPermitted` when the host refuses a live watch.
5. Arm with an **address it controls**, at which it receives Trust Task documents over a transport
   its DID document advertises, and which it uses for that one watch and nothing else. The address
   **MUST NOT** be the recipient's own DID — that would tell the host which agent serves the
   member, which its identifier reaching the host is forbidden to ([`rooms/keys/browse`](../../browse/0.6/spec.md)) —
   nor the member's, nor the caller's, nor the address of any other watch, past or present, nor
   one it gives anyone else. A request that re-arms a one-shot watch that has not fired keeps its
   address, so that it replaces the host's watch rather than adding one, and a request that renews
   a live watch keeps its address for as long as the caller renews it; any other arming takes a new
   address. A request that arms one-shot a room whose watch is live, or live a room whose watch is
   one-shot, arms the new watch at a new address and, once the host has armed it, cancels the old
   one at the host and forgets it — so that a live sitting and the reads around it are not linked
   by one address, and a refused live watch leaves the one-shot watch in place.
6. Keep, for each armed watch, only: the caller, the member, the room, the host, the address and
   the key material it receives at, when the watch lapses, and whether it is live. It **MUST NOT**
   keep the watermarks it was given, the versions it was signalled, or the times signals arrived —
   for a live watch no more than for a one-shot one — and
   **SHOULD NOT** keep an address, or a log of the watches it held, once it has forgotten the
   watch: a history of addresses and arming times is the reading pattern
   [`rooms/watch`](../../../watch/0.2/spec.md#correlation) warns of, kept. It **MAY** record, for
   its member's own audit, that it armed or cancelled a watch on a room for a caller.
7. Answer `expiresAt` no later than the `expiresAt` the host answered, and `live: true` only when
   the host answered it, and forget the watch — and stop receiving at its address — when a one-shot
   watch fires (item 8), when it is cancelled, at `expiresAt`,
   when the caller's standing to act as the member is withdrawn, when it no longer holds the
   member's credentials for the room, and, for a live watch, when the host refuses its renewal
   with `rooms/watch:liveNotPermitted` — the room's signal delay is no longer `none`, and the host
   has ended the watch. On a withdrawal it **SHOULD** also cancel the watch at the
   host.
8. On a [`rooms/changed`](../../../changed/0.1/spec.md) at a watch's address, pass it on only when
   the signal's sender is authenticated — by the transport or by a proof — as the `host` the watch
   was armed at, its `roomId` is the watch's room, and the watch is armed. It then sends **one**
   [`rooms/keys/changed`](../../changed/0.1/spec.md) to the caller that armed the watch, carrying
   the signal's `roomId` and `latestVersion` unaltered. For a one-shot watch it then forgets the
   watch; for a live watch it keeps it, and passes on each later signal the same way, as it
   arrives — never holding one back or batching it, and never comparing it with an earlier one,
   which would mean keeping the versions item 6 forbids. It **MUST** drop any
   other document at the address, and any signal for a watch it has forgotten, silently: a
   `rooms/changed` has no response.
9. Treat a watch as granting nothing. It is never consulted to authorize anything, and is never
   shown to anyone but the caller that armed it.

A recipient **MAY** bound how many watches it holds per member and per caller, and reports a room
beyond that bound as refused with `limitReached` — its own bound, distinct from a host's
`rooms/watch:limitReached`, which it reports as `hostRefused`. A caller polls a room it could not
watch.

**A repeated request** sets a state, as `rooms/watch`'s does: re-arming a watch already armed at
the same watermark succeeds with the same answer, renewing a live watch extends its lease, and
cancelling one the recipient does not hold answers `armed: false`. A recipient ignores `live` on a
cancel. **Retry class** (VTI-OPS-060): the second — a durable effect whose repeat
is harmless.

### Many rooms at one host

A member in many rooms at one host arms them all in one request, and one request re-arms all the
rooms it has just read. The batching stops at the key holder. `rooms/watch` names one room, and
this task does not ask for a host request naming several: on a `private` room presentations are
made in zero knowledge so that a host cannot tell one member's reads from another's, and a request
— or an address — covering several rooms would tell the host that one party reads all of them,
which is the linkage `rooms/watch` forbids the host to make. Each room is armed with its own
request, presentation and address (Recipient items 3 and 5), so the host sees what it would have
seen from one member arming each room separately.

A phone woken once for several rooms needs no batching of the signals either: each
`rooms/keys/changed` waits in its mediator's queue, a mediator coalesces queued documents into one
wake ([`push/wake`](../../../../push/wake/0.2/spec.md) producer item 4), and the phone drains them
together.

### A watch does not move with the room

As [`rooms/watch`](../../../watch/0.2/spec.md) says, a watch is the host's record, not the room's,
and lapses when the room moves to another host. The key holder's record lapses with it. A client
that reads the room at its new host arms there.

## Authorization

**The entitlement is acting as the member** — being, or being the client of, the key holder's own
principal — as for [`rooms/keys/act`](../../act/0.12/spec.md), and it is refused to delegates for
the same reason (Recipient item 1). The member's standing in each room is the room's decision: the
recipient presents `read` granted to the member, exactly as a listing does, and the host authorizes
the watch from that presentation as it authorizes any `rooms/watch`.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`,
`recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they
were entitled to the outcome.

Naming a host transfers no standing, as on [`rooms/keys/browse`](../../browse/0.6/spec.md): what
it transfers is **sight** of the member's presentation. A recipient **SHOULD** bound the outbound
request to transports the host's DID document advertises.

## Definitions

- **`member`** — the member to watch as: a persona the key holder holds for its principal. This
  task's subject.
- **`host`** — the host to arm at.
- **`rooms`** — the rooms to watch there, each with the caller's watermark (`sinceVersion`).
- **`leaseSeconds`** — how long the caller asks each watch to last if it does not fire.
- **`live`** — arm or renew the named rooms' watches live, for a conversation on screen.
- **`cancel`** — end the watches on the named rooms.
- **Address** — the DID the key holder arms a watch with: its own, for that watch only, never
  given to the caller.
- **Response** — the member and host echoed, and one outcome per room: `armed`, `live` when the
  watch is live, `expiresAt` when it is armed, and `refused` (with `hostCode`) when it was refused.

## Request

The member's client (`issuer`) sends its key holder (`recipient`) a document whose `payload`
validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet re-arms three rooms after reading them

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1001",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.2#request",
  "issuer": "did:key:z6MkWalletHolderExample",
  "recipient": "did:webvh:QmVta:vta.example",
  "issuedAt": "2027-02-01T09:00:00Z",
  "payload": {
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "rooms": [
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal", "sinceVersion": 412 },
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:board", "sinceVersion": 88 },
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:social", "sinceVersion": 1290 }
    ],
    "leaseSeconds": 604800
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T09:00:00Z",
    "verificationMethod": "did:key:z6MkWalletHolderExample#z6MkWalletHolderExample",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### A conversation opens on screen

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1004",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.2#request",
  "issuer": "did:key:z6MkWalletHolderExample",
  "recipient": "did:webvh:QmVta:vta.example",
  "issuedAt": "2027-02-01T09:30:00Z",
  "payload": {
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "rooms": [
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup", "sinceVersion": 2071 }
    ],
    "leaseSeconds": 120,
    "live": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T09:30:00Z",
    "verificationMethod": "did:key:z6MkWalletHolderExample#z6MkWalletHolderExample",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Cancelling one

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1003",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.2#request",
  "issuer": "did:key:z6MkWalletHolderExample",
  "recipient": "did:webvh:QmVta:vta.example",
  "issuedAt": "2027-02-03T18:00:00Z",
  "payload": {
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "rooms": [
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:social" }
    ],
    "cancel": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-03T18:00:00Z",
    "verificationMethod": "did:key:z6MkWalletHolderExample#z6MkWalletHolderExample",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The key holder answers with a document of type
`https://trusttasks.org/spec/rooms/keys/watch/0.2#response`, whose `payload` validates against the
`$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json): one outcome per
room, in the order requested. A refusal of the whole request — a delegate, a member the caller may
not act as — uses `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)); a refusal of one room
is that room's outcome.

### Two armed, one refused by the host

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1002",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkWalletHolderExample",
  "issuedAt": "2027-02-01T09:00:02Z",
  "threadId": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1001",
  "payload": {
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "watches": [
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal", "armed": true, "expiresAt": "2027-02-08T09:00:01Z" },
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:board", "armed": true, "expiresAt": "2027-02-08T09:00:01Z" },
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:social", "armed": false, "refused": "hostRefused", "hostCode": "rooms/watch:limitReached" }
    ]
  }
}
```

### Armed live

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1005",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkWalletHolderExample",
  "issuedAt": "2027-02-01T09:30:01Z",
  "threadId": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1004",
  "payload": {
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "watches": [
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup", "armed": true, "live": true, "expiresAt": "2027-02-01T09:32:00Z" }
    ]
  }
}
```

### A live watch refused: the room keeps its signal delay

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1006",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkWalletHolderExample",
  "issuedAt": "2027-02-01T09:30:01Z",
  "threadId": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1004",
  "payload": {
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "host": "did:web:rooms.northwind.example",
    "watches": [
      { "roomId": "did:webvh:QmRoom:rooms.northwind.example:standup", "armed": false, "refused": "hostRefused", "hostCode": "rooms/watch:liveNotPermitted" }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a member, a host, rooms, watermarks and whether to watch live; the response,
whether each watch is armed, whether it is live, and until when, or why not. Neither carries an address: the key holder chooses it and keeps
it. At the host, each watch is an ordinary `rooms/watch` — the member's presentation, an address,
a watermark and a lease — and the signal that comes back, and the `rooms/keys/changed` that passes
it on, carry the room and its latest version and nothing about what changed.

### Correlation

**The host learns what `rooms/watch` says it learns, and nothing of the client or the key
holder.** It sees the member, as on any presentation on a tier that discloses one, and an address
used for that watch only. Because any arming after a signal takes a new address (Recipient item 5),
the re-arming times of one address are never a member's reading pattern over time — on a `private`
room the property `rooms/watch` asks a member who minds it to buy with a new address per arming,
here provided by the key holder, which can afford it, rather than by a phone, which would register
a new DID with its mediator each time. The address's DID document names the mediator the key
holder receives at, which a host learns; a key holder whose mediator serves only its own principal
identifies the member by it, and on a `private` room such a key holder **SHOULD** tell its client
to poll rather than watch.

**The key holder learns which rooms its member's clients watch, and when they re-arm** — when its
member read them, which it largely knows already from serving those reads. Of a live watch it
learns that the room was on screen, for how long, and — because it passes each signal on — when
the room changed, to within the host's spacing: what any party carrying the signals sees. Recipient item 6 keeps it
from turning that into a stored history.

**Nobody on the pass-on's path learns which room changed.** The `rooms/keys/changed` is encrypted to
the client by its transport binding; the client's mediator, the push gateway and the platform push
service see that something arrived, and when. For a one-shot watch that is once per arming. For a
live watch it is once per change, at most once a second: the client's mediator, the push path and
the key holder's own mediator learn how busy the room is while it is on screen, and when each write
happened. That is what a live watch gives up, and why it is held only for a room on screen (Producer
item 6), lasts at most two minutes per renewal — so a client that stops reading without cancelling
collects at most about 120 signals, and usually far fewer, before it lapses — and is permitted only on a room that has already let
its signals' timing follow its writes. A live sitting keeps one address at the host across its
renewals; once it ends, the next arming of the room, live or not, takes another (Recipient item 5).

**A phone on a `private` room is expected to poll, unless its agent watches for it.** A phone that
arms with an address of its own ([`rooms/keys/act/0.12`](../../act/0.12/spec.md) Producer item 6)
would need a new DID registered with its mediator for each arming to keep the property above;
polling with [`rooms/keys/browse`](../../browse/0.6/spec.md) is the expected choice for it on a
`private` room. Through this task the key holder bears that cost instead.

### Retention

The key holder keeps one record per armed watch and forgets it when a one-shot watch fires, or any
watch is cancelled or lapses (Recipient items 6 and 7). It keeps no watermark, no signal and no
history of addresses, for a live watch as for a one-shot one.

### Consent/purpose

The member's client asks for exactly this: a signal for these rooms, passed on to it. The key
holder uses the address for nothing else, sends nothing to the host but the watch and its cancel,
and passes on nothing but the signal. A host's obligations toward the address are
[`rooms/watch`](../../../watch/0.2/spec.md#consentpurpose)'s. A live watch is asked for to see a
conversation as it happens, and the key holder uses it for nothing else — not as a record of when
its member was reading.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
