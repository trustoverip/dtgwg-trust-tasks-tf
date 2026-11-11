---
slug: rooms/keys/watch
version: "0.1"
title: "Rooms Keys — Watch"
summary: "A member's client asks its key holder to watch rooms at one host for it — arming rooms/watch as the member, at an address the key holder controls — and to pass each signal on, so a wallet with no member keys and no address of its own need not poll."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - watch
  - signal
  - wake
  - persona
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
    Arms, re-arms or cancels watches at the host (rooms/watch), and keeps one small record per
    armed watch at the recipient until it fires, is cancelled or lapses. Nothing in any room
    changes.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    `actsAsSubject`: the recipient presents the member's own `read` authority to the host, as the
    member. In: a member, a host, rooms and watermarks. Out: whether each watch is armed and until
    when. The address the recipient arms with stays with the recipient; the caller never learns it.
retention:
  class: durable
  rationale: >-
    The recipient keeps one record per armed watch — the caller, the member, the room, the host,
    the address and when the watch lapses — because it must know, when a signal arrives at the
    address, whose it is and where to pass it on. It forgets the record when the watch fires, is
    cancelled or lapses — at most 30 days — and keeps nothing of the watermarks or the signals.
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

[`rooms/watch`](../../../watch/0.1/spec.md) lets a member ask a room's host for one content-free
signal — [`rooms/changed`](../../../changed/0.1/spec.md) — at an address the member names, the
first time the room moves past the version the member has read to. Two things stop a wallet whose
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
room, its latest version, and nothing more. The host sees the member and an address; it never sees
the client or the key holder. The client sees the signal; it never sees the address.

**This is the intended shape for a mobile wallet whose member keys are at its agent.** The signal
it receives is a Trust Task document from its own agent, waiting in its own mediator's queue, and
a sleeping phone is woken for it by the [push binding](../../../../../bindings/push/0.1/spec.md)'s
contentless doorbell like any other. The phone holds no address a host knows, and registers no DID
per watch: its agent bears that cost.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### One signal per arming, end to end

`rooms/watch` sends one signal per arming, and this task keeps that property across the extra
hop: the key holder passes on **at most one** `rooms/keys/changed` per arming, and arms again only
when the client asks it to. However busy a room is, and however long a device is offline, the
client's mediator queue holds at most one signal per watch, and the client re-arms after it has
read.

### Producer

A conforming **producer** (the member's own client) **MUST**:

1. Use this task only when it acts **as** the member, as for [`rooms/keys/act`](../../act/0.12/spec.md)
   Producer item 1. A party acting under a chain the member attenuated to it is an agent, and
   watches for itself.
2. Name the member in `member` — one of the members [`rooms/keys/list`](../../list/0.3/spec.md)
   says it reads the rooms as — and name each room at most once.
3. Set each room's `sinceVersion` to the highest version it has read the room to, when it has one.
4. Re-arm a room's watch only after it has read the room since the signal it last received — or
   when it has received none before `expiresAt` — and **SHOULD NOT** re-arm more often than it
   reads.
5. Receive Trust Task documents from its key holder at the identifier it sends this task from, the
   way it receives any other document the key holder sends it unasked (for example
   [`task-consent/granted`](../../../../task-consent/granted/0.1/spec.md)): that is where the
   signals go.

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
3. For each room, arm or cancel the watch at `host` with [`rooms/watch/0.1`](../../../watch/0.1/spec.md),
   issued and signed by the member with a key of the member's DID document listed for
   `authentication`, carrying a presentation that confers **exactly** `read` at the room's scope,
   granted to the member — leaf first, and on a `private` room the same-subject binding — as
   [`rooms/keys/act`](../../act/0.12/spec.md) Recipient item 4 would insert it. Each room is a
   request of its own, with its own presentation and its own address: the recipient **MUST NOT**
   let the host see two rooms in one request, or one address on two rooms.
4. Report a room it cannot watch in that room's outcome rather than failing the request:
   `notAMember` when it holds no credential the room issued to the member, `actionNotHeld` when
   the member's credentials do not confer `read`, `hostUnreachable` when the host cannot be
   resolved, advertises no transport it speaks or does not answer, `limitReached` when it holds as
   many watches as it allows, and `hostRefused`, with the
   host's code unaltered in `hostCode`, when the host refuses.
5. Arm with an **address it controls**, at which it receives Trust Task documents over a transport
   its DID document advertises, and which it uses for that one watch and nothing else. The address
   **MUST NOT** be the recipient's own DID — that would tell the host which agent serves the
   member, which its identifier reaching the host is forbidden to ([`rooms/keys/browse`](../../browse/0.4/spec.md)) —
   nor the member's, nor the caller's, nor the address of any other watch, past or present, nor
   one it gives anyone else. A request that re-arms a watch that has not fired keeps its address,
   so that it replaces the host's watch rather than adding one; any other arming takes a new
   address.
6. Keep, for each armed watch, only: the caller, the member, the room, the host, the address and
   the key material it receives at, and when the watch lapses. It **MUST NOT** keep the
   watermarks it was given, the versions it was signalled, or the times signals arrived, and
   **SHOULD NOT** keep an address, or a log of the watches it held, once it has forgotten the
   watch: a history of addresses and arming times is the reading pattern
   [`rooms/watch`](../../../watch/0.1/spec.md#correlation) warns of, kept. It **MAY** record, for
   its member's own audit, that it armed or cancelled a watch on a room for a caller.
7. Answer `expiresAt` no later than the `expiresAt` the host answered, and forget the watch — and
   stop receiving at its address — when it fires (item 8), when it is cancelled, at `expiresAt`,
   when the caller's standing to act as the member is withdrawn, and when it no longer holds the
   member's credentials for the room. On a withdrawal it **SHOULD** also cancel the watch at the
   host.
8. On a [`rooms/changed`](../../../changed/0.1/spec.md) at a watch's address, pass it on only when
   the signal's sender is authenticated — by the transport or by a proof — as the `host` the watch
   was armed at, its `roomId` is the watch's room, and the watch is armed. It then sends **one**
   [`rooms/keys/changed`](../../changed/0.1/spec.md) to the caller that armed the watch, carrying
   the signal's `roomId` and `latestVersion` unaltered, and forgets the watch. It **MUST** drop any
   other document at the address, and any signal for a watch it has forgotten, silently: a
   `rooms/changed` has no response.
9. Treat a watch as granting nothing. It is never consulted to authorize anything, and is never
   shown to anyone but the caller that armed it.

A recipient **MAY** bound how many watches it holds per member and per caller, and reports a room
beyond that bound as refused with `limitReached` — its own bound, distinct from a host's
`rooms/watch:limitReached`, which it reports as `hostRefused`. A caller polls a room it could not
watch.

**A repeated request** sets a state, as `rooms/watch`'s does: re-arming a watch already armed at
the same watermark succeeds with the same answer, and cancelling one the recipient does not hold
answers `armed: false`. **Retry class** (VTI-OPS-060): the second — a durable effect whose repeat
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

As [`rooms/watch`](../../../watch/0.1/spec.md) says, a watch is the host's record, not the room's,
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

Naming a host transfers no standing, as on [`rooms/keys/browse`](../../browse/0.4/spec.md): what
it transfers is **sight** of the member's presentation. A recipient **SHOULD** bound the outbound
request to transports the host's DID document advertises.

## Definitions

- **`member`** — the member to watch as: a persona the key holder holds for its principal. This
  task's subject.
- **`host`** — the host to arm at.
- **`rooms`** — the rooms to watch there, each with the caller's watermark (`sinceVersion`).
- **`leaseSeconds`** — how long the caller asks each watch to last if it does not fire.
- **`cancel`** — end the watches on the named rooms.
- **Address** — the DID the key holder arms a watch with: its own, for that watch only, never
  given to the caller.
- **Response** — the member and host echoed, and one outcome per room: `armed`, `expiresAt` when
  it is, and `refused` (with `hostCode`) when it was refused.

## Request

The member's client (`issuer`) sends its key holder (`recipient`) a document whose `payload`
validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet re-arms three rooms after reading them

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1001",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.1#request",
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

### Cancelling one

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1003",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.1#request",
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
`https://trusttasks.org/spec/rooms/keys/watch/0.1#response`, whose `payload` validates against the
`$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json): one outcome per
room, in the order requested. A refusal of the whole request — a delegate, a member the caller may
not act as — uses `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)); a refusal of one room
is that room's outcome.

### Two armed, one refused by the host

```json
{
  "id": "urn:uuid:7d2c1e0a-3b4f-4a6e-9d81-2c5f0e7a1002",
  "type": "https://trusttasks.org/spec/rooms/keys/watch/0.1#response",
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

## Security & Privacy

### Data carried

The request carries a member, a host, rooms and watermarks; the response, whether each watch is
armed and until when, or why not. Neither carries an address: the key holder chooses it and keeps
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
member read them, which it largely knows already from serving those reads. Recipient item 6 keeps it
from turning that into a stored history.

**Nobody on the pass-on's path learns which room changed.** The `rooms/keys/changed` is encrypted to
the client by its transport binding; the client's mediator, the push gateway and the platform push
service see that something arrived, and when.

**A phone on a `private` room is expected to poll, unless its agent watches for it.** A phone that
arms with an address of its own ([`rooms/keys/act/0.12`](../../act/0.12/spec.md) Producer item 6)
would need a new DID registered with its mediator for each arming to keep the property above;
polling with [`rooms/keys/browse`](../../browse/0.4/spec.md) is the expected choice for it on a
`private` room. Through this task the key holder bears that cost instead.

### Retention

The key holder keeps one record per armed watch and forgets it when the watch fires, is cancelled
or lapses (Recipient items 6 and 7). It keeps no watermark, no signal and no history of addresses.

### Consent/purpose

The member's client asks for exactly this: a signal for these rooms, passed on to it. The key
holder uses the address for nothing else, sends nothing to the host but the watch and its cancel,
and passes on nothing but the signal. A host's obligations toward the address are
[`rooms/watch`](../../../watch/0.1/spec.md#consentpurpose)'s.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
