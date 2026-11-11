---
slug: rooms/watch
version: "0.1"
title: "Rooms — Watch"
summary: "A member asks a room's host for one content-free signal, sent to an address the member chooses, the first time the room moves past the version the member has read to — so a wallet learns a room changed without polling, and nobody on the way learns more."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - watch
  - signal
  - wake
  - sync
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
    Arms, re-arms or cancels the host's record of one watch: a room, an address, a watermark and a
    lease. Nothing in the room changes.
subjectPath: /roomId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the room, an address DID, a version and a lease. Out: whether the watch is armed and until
    when. The address is a routing identifier the member chooses for this purpose, not data about a
    person.
retention:
  class: durable
  rationale: >-
    A host keeps a watch until it fires, is cancelled or its lease ends — at most 30 days — and then
    forgets it. It is not a record of the room and is never part of one.
errorCodes:
  - code: rooms/watch:addressUnreachable
    meaning: "The address does not resolve to a DID document, or its document advertises no transport the host speaks. Nothing is armed."
    retryable: false
  - code: rooms/watch:limitReached
    meaning: "The host holds as many watches for this room, or for this presenter where the tier discloses one, as it allows. Nothing is armed; a member MAY cancel a watch it no longer needs and try again."
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
  - push/wake
---

## Abstract

The **Rooms — Watch** Trust Task lets a member learn that a room has changed without polling it.
The member names an **address** — a DID it controls, reached through its own mediator — and the
version it has read the room to. The host sends one [`rooms/changed`](../../changed/0.1/spec.md)
signal to that address the first time the room moves past that version, and then nothing more
until the member arms the watch again.

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

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### One signal per arming

A watch is **armed** by a request without `cancel`, and **fires** once: the host sends one signal
and disarms it. The member arms it again after it has caught up, with the version it has now read
to. This is what bounds everything downstream: however busy a room is, and however long a device
is offline, an address holds at most one signal per watch in its mediator's queue, and a phone is
woken at most once per watch until its member has looked.

A conforming **producer** (the member) **MUST**:

1. Present a chain conferring `read` at the room's scope, exactly as for
   [`rooms/records/list`](../../records/list/0.7/spec.md).
2. Name in `address` a DID whose keys it, or its own agent, holds, and through which it receives
   Trust Task documents. On a `private` room the address **MUST** be a DID it uses for nothing
   else: not the persona it presents as, not the address of a watch on another room, and not one
   it has given anyone else (*Correlation*).
3. Set `sinceVersion` to the highest version it has read the room to, when it has one.
4. Re-arm the watch only after it has read the room since the signal it last received — or when
   it has received none within the lease — and **SHOULD NOT** re-arm more often than it reads.

A conforming **consumer** (the host) **MUST**:

1. Apply the chain rules of [`rooms/records/put`](../../records/put/0.5/spec.md) to the
   presentation, and refuse one that does not confer `read` at this room's scope, whose chain does
   not reach the room, or whose leaf is not the party it authenticated, with the standard
   `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
2. Resolve `address` before arming, and refuse with `rooms/watch:addressUnreachable` an address
   whose document it cannot resolve or which advertises no transport it speaks. It **MUST NOT**
   send anything to the address in the course of this request.
3. Hold at most one watch per room and `address`: a request naming a watch it holds replaces its
   watermark and lease, and re-arms it if it has fired.
4. Send one [`rooms/changed`](../../changed/0.1/spec.md) to the address the first time the room's
   latest version is above the watermark — at once if it already is — and then disarm the watch.
   It **MUST NOT** send a second signal for a watch until it is armed again.
5. Forget the watch when it fires, when it is cancelled, and at `expiresAt`, which **MUST NOT** be
   later than the earliest instant at which any credential of the presentation stops being valid.
   It **MUST** forget a watch whose presentation a `RevocationList` it accepts for the room revokes,
   as it would refuse that presentation (`AuthorityPresentation`).
6. Send the same signal whatever moved the room — a put of any kind, a curation, a retraction, a
   prune, a profile written — and **MUST NOT** carry in it, or vary its timing or size by, anything
   about what changed.
7. Treat a watch as granting nothing. It is not a membership record, never consulted to authorize
   anything, and never shown to another member, to the room's owner, or to anyone else.

A host **MAY** bound how many watches it holds per room, and per presenter where the tier
discloses one, and refuses beyond that with `rooms/watch:limitReached`. A host that serves this
task **SHOULD** say so through `trust-task-discovery`, so that a member knows whether to watch or
to poll.

**A repeated request** sets a state: arming a watch already armed at the same watermark succeeds
with the same answer, and cancelling a watch the host does not hold answers `armed: false`.
**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless.

### A watch does not move with the room

A watch is the host's record, not the room's. When a room changes host
(`vtc/rooms/storage/migrate`, `rooms/owner/transfer`) or is reset, its watches stay behind and
lapse. A member that reads the room at its new host arms a new watch there.

## Authorization

**Authority: `read` at the room's scope, conferred by a chain the room issued** — the same grant
a listing takes, because a signal tells the member nothing a listing would not. The host keeps no
roster and consults none: the presentation is the whole of the authority, verified as for every
other task in the family. The proof establishes who asked and that the request is unaltered,
never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

The address needs no authority of its own. It receives only what the presenter could read, and at
most one document per arming.

## Definitions

- **`roomId`** — the room.
- **`presentation`** — an [`AuthorityPresentation`](../../_shared/0.2/room.schema.json) conferring
  `read`.
- **`address`** — the DID the signal is sent to, chosen by the member. With `roomId`, it identifies
  the watch.
- **`sinceVersion`** — the member's watermark: the host signals once the room is above it.
- **`leaseSeconds`** — how long the member asks the watch to last if it does not fire.
- **`cancel`** — end the watch.
- **Response** — `armed`, and `expiresAt` when it is.

## Request

The member (`issuer`) sends the host (`recipient`) a document whose `payload` validates against
the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet arms a watch after catching up

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1001",
  "type": "https://trusttasks.org/spec/rooms/watch/0.1#request",
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

### Cancelling it

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1003",
  "type": "https://trusttasks.org/spec/rooms/watch/0.1#request",
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

The host answers with a document of type `https://trusttasks.org/spec/rooms/watch/0.1#response`,
whose `payload` validates against the `$anchor: "response"` sub-schema in
[`payload.schema.json`](payload.schema.json): the room and address echoed, `armed`, and
`expiresAt` when armed. Failures use `trust-task-error`
([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Armed

```json
{
  "id": "urn:uuid:5b1f6c2e-0a7d-4e3b-9c41-8d2f7a6e1002",
  "type": "https://trusttasks.org/spec/rooms/watch/0.1#response",
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

## Security & Privacy

### Data carried

The request carries a room, a presentation the host already verifies for every read, an address
DID, a version and a lease; the response, whether the watch is armed and until when. Nothing about
the room's content travels in either direction, and nothing about it travels in the signal
([`rooms/changed`](../../changed/0.1/spec.md)) either: the room and its latest version, which the
member could have had from a listing.

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
watch adds the address and the moments the member re-arms, which are moments the member read.

**The address is a stable handle for one member at the host.** Re-arming the same address links
the member's reads over time. On `open` and `attributed` rooms the host attributes every read
anyway, so this adds nothing. On a `private` room it does: reads are presented in zero knowledge
and the host cannot otherwise tell one member's from another's, but the re-arming times of one
address are one member's reading pattern, unattributed but linked. That is why producer item 2
requires a `private` room's address to be used for nothing else, and why a member for whom even an
unattributed pattern matters **SHOULD** arm each time with a new address — a new address is a new
watch — or poll instead. The address's DID document also names the mediator that receives for it,
which a host learns; a member whose mediator would identify it **SHOULD** not watch a `private`
room.

**Nobody on the signal's path learns which room changed.** The signal is a Trust Task document
encrypted to the address by its transport binding: the address's mediator sees a document for the
address, the push gateway and the platform push service see a contentless wake
([push binding §2](../../../../bindings/push/0.1/spec.md)). What they all see is that something
arrived, and when. Host item 6 keeps the signal identical whatever changed, and one signal per
arming keeps its count from saying how busy the room was.

### Retention

A host keeps a watch only until it fires, is cancelled or lapses, and **SHOULD NOT** keep the
address afterwards, nor a log of the watches it held beyond what its operation needs: a history of
addresses and re-arming times is the reading pattern above, kept.

### Consent/purpose

The address is given to the host for one purpose: to send it `rooms/changed` for this room. A host
**MUST NOT** send anything else to it, give it to anyone, or use it to link a member across rooms.
