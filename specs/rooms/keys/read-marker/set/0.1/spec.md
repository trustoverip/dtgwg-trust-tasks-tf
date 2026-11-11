---
slug: rooms/keys/read-marker/set
version: "0.1"
title: "Rooms Keys — Read Marker — Set"
summary: "A member's client tells their key holder where the member has seen a room's conversation up to, so that every one of their devices agrees on what is unread — kept for the member alone, never sent to the room, its host or another member."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - unread
  - read marker
  - chat
  - persona
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request moves what every one of the member's devices treats as read. A request whose origin depended on the transport would let a compromised channel mark a room read on all of them — hiding a message from its member everywhere at once, which is worse than hiding it on one device.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Mutating, so SPEC §7.3 item 17 sets REQUIRED as the floor. A forward set is harmless to replay, but a replayed rewind would mark a room unread again after the member has read past it.
sideEffects:
  level: mutating
  rationale: >-
    Moves the member's read marker for one room, in the key holder's own storage, and may tell the member's other clients that it moved. Nothing reaches the room, its host, its owner, another member or a community.
subjectPath: /member
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a room, a member, a room version and a host timestamp — where the member has read to. Out: the marker now in force, to the caller and to the member's own clients watching the room. Over time, the moments it moves are when the member reads each room; the key holder keeps only the current marker, never that history.
retention:
  class: durable
  rationale: >-
    A marker is useful only if the member's next device finds it: it persists with the key holder's state for the room and the member, and goes when that state goes. Only the current marker is kept; each set replaces it.
errorCodes:
  - code: rooms/keys/read-marker/set:notHeld
    meaning: "The recipient holds no group state for `roomId` as `member` — or `member` is not one the caller may act as, which is answered the same way so that the error does not say which members of which rooms the recipient serves. A room absent from rooms/keys/list for this member has no read marker."
    retryable: false
related:
  - rooms/keys/read-marker/moved
  - rooms/keys/list
  - rooms/keys/label
  - rooms/keys/watch
  - rooms/changed
---

## Abstract

A member reads a room on their phone, and their laptop still shows it unread; a notification they opened on one device stays on the other. That is because the **read marker** a reader counts unread messages against ([`rooms/changed`](../../../../changed/0.1/spec.md#what-a-reader-does-with-a-signal), step 4) has so far been each client's own.

**Rooms Keys — Read Marker — Set** lets the member keep one read marker per room with their **key holder**, beside the room's keys and the member's [personal label](../../../label/0.1/spec.md). A client tells the key holder where the member has now seen the conversation up to; the key holder keeps the marker, answers with the one now in force, returns it to every client that lists the member's rooms ([`rooms/keys/list`](../../../list/0.3/spec.md) 0.3), and tells the member's other clients watching the room that it moved ([`rooms/keys/read-marker/moved`](../../moved/0.1/spec.md)), so a device can clear a notification its member has already dealt with elsewhere.

**A marker only moves forward**, unless the client asks to rewind it ("mark as unread"). Two devices setting a marker at once therefore agree without either reading first: whichever set arrives, the further marker stands.

The marker is **private to the member**. It is not part of the room, and is never sent to the room's host, its owner, another member or a community: a host never learns who has read what, and no member learns when another read their message. This task provides no read receipts, and is not a way to build them.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Producer

A conforming **producer** (the member's own client):

1. **MUST** name in `member` the member whose marker it means, as [`rooms/keys/list`](../../../list/0.3/spec.md) 0.3 returns it beside the room.
2. **MUST** set `readMarker` to where its person has actually seen the conversation: `version`, the highest room version it has shown them the room to; `createdAt`, the host's `createdAt` of the newest message it has shown them, exactly as the host returned it. It **MUST NOT** advance the marker for records it has fetched and not shown — a notification raised is not a message seen, and neither is a room listed in the background.
3. **SHOULD** set the marker when its person leaves the room or the view of it settles, rather than on every message scrolled past, and **SHOULD NOT** set one that is not further on than the marker it last saw in force: the key holder would keep the one it has.
4. **MUST** set `rewind` only when its person asks for the room, or a message in it, to be unread again, with the marker just before the first message to show as unread — `version` `0` and no `createdAt` for the whole room.
5. **MUST** adopt the marker the response returns as the one in force, whether or not this request moved it.

### Recipient

A conforming **recipient** (the key holder):

1. **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) a caller to which the member has delegated narrower standing — an agent holding a chain attenuated to itself by [`rooms/keys/present`](../../../present/0.2/spec.md). A read marker is the member's, as their [label](../../../label/0.1/spec.md) is (its Recipient item 1), and is read and moved only by a caller that may act as the member.
2. **MUST** refuse with `notHeld` a request whose `member` it holds no group state for in `roomId`, **and** one whose `member` is not a member its caller may act as. It **MUST** give the same answer to both, and **MUST NOT** compare the request with the stored marker before it has decided neither applies.
3. **MUST**, without `rewind`, keep the further of the stored marker and the requested one: the greater `version`; at equal `version`, the later `createdAt`, an absent `createdAt` being earlier than any present one. When the requested marker is further on, it replaces the stored one whole and the answer is `moved: true`. Otherwise nothing is written, the answer is `moved: false`, and `readMarker` is the stored marker. With no stored marker, the requested one is stored and the answer is `moved: true`.
4. **MUST**, with `rewind: true`, replace the stored marker with the requested one whether it is further on or not, and answer `moved: true` unless it is equal to the stored one.
5. **MUST** compare `createdAt` values as instants, and store and return each exactly as received. It **MUST NOT** check a marker against the room or its host: it does not ask the host anything to serve this task, and a marker beyond the room's latest version is the member's to correct with `rewind`.
6. **SHOULD**, when the marker moved, send [`rooms/keys/read-marker/moved`](../../moved/0.1/spec.md) to the member's other clients as that task's Producer item 1 bounds them — never to the caller, and never to anyone but a client that may act as `member`.
7. **MUST** keep only the marker in force, one per room and member, with its group state for that room and member, protected as it protects that state, and **MUST** discard it when it discards that state. It **MUST NOT** keep earlier markers, when each was set, or which client set it.
8. **MUST NOT** send a read marker, or anything derived from one, to anyone but a client that may act as its member: not to the room, its host, its owner, another member or a community; not in any document it completes, signs or seals for the member ([`rooms/keys/act`](../../../act/0.12/spec.md), [`rooms/keys/seal`](../../../seal/0.3/spec.md), [`rooms/keys/present`](../../../present/0.2/spec.md)); and not in any record it writes to a room.
9. **MUST NOT** write a marker's `version` or `createdAt` to a log or an audit record, nor when it was set. It **MAY** count, for its own operation, that markers were set.

A marker is the member's position in a conversation, not a credential: it is never consulted to authorize anything.

## Authorization

**The entitlement is acting as the member** — being, or being the client of, the key holder's own principal, with standing for the persona named in `member` — exactly as for [`rooms/keys/label`](../../../label/0.1/spec.md), and for the same reasons. A client authorised for one of a person's contexts moves the markers of the rooms its personas hold and no other, and cannot learn from `notHeld` whether another persona holds a room at all (Recipient item 2).

A delegate is refused (Recipient item 1). An agent acting in one room under a chain the member attenuated to it reads for itself; what it has read is not what the member has seen, and moving the member's marker on its reading would hide messages from the member on every device.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. The decision is the key holder's, from its own record of whom its caller acts as.

**The model this task is designed for.** The key holder executes a set on the caller's authority alone, without a step-up and without anyone's consent. It confers nothing, reaches no other party, changes nobody's access, and is undone by a rewind. Whether a deployment's policy asks for more is that policy's, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

**No precondition, by design.** Two of the member's devices setting a marker at once both succeed, and the further marker stands (Recipient item 3) whichever arrives first, so they converge without a read before every write. A rewind is the one move that is not merged: the member asked for it, on one device, and the later of a rewind and a forward set wins — which is what a person who marked a room unread and then read it expects.

## Definitions

- `ReadMarker` — see [`rooms/_shared/0.1/read-marker.schema.json`](../../../../_shared/0.1/read-marker.schema.json).
- **Further on** — of two markers, the one with the greater `version`; at equal `version`, the one with the later `createdAt`, an absent `createdAt` being earlier than any present one. Markers are totally ordered this way, so "keep the further" gives the same result whatever order sets arrive in.
- **Member** — the DID a room admitted: the subject of the membership credential it issued. Often a persona the key holder holds for a person in one of their contexts.
- **Watermark** and **read marker** — two different positions. A client's watermark ([`rooms/changed`](../../../../changed/0.1/spec.md), step 7) is how far *that client* has read the room from its host, and stays the client's own. The read marker is how far *the member* has seen the conversation, and is what this task shares between the member's devices.

## Request

The member's client (`issuer`) sends its key holder (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The member has read the room on their phone

```json
{
  "id": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3001",
  "type": "https://trusttasks.org/spec/rooms/keys/read-marker/set/0.1#request",
  "issuer": "did:key:z6MkPhoneHolderExample",
  "recipient": "did:webvh:QmVta:vta.example",
  "issuedAt": "2027-02-01T10:20:00Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "readMarker": {
      "version": 415,
      "createdAt": "2027-02-01T10:14:02Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T10:20:00Z",
    "verificationMethod": "did:key:z6MkPhoneHolderExample#z6MkPhoneHolderExample",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Mark a message unread again

The member marks the message stored at version 409 as unread: the marker goes back to the message before it.

```json
{
  "id": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3003",
  "type": "https://trusttasks.org/spec/rooms/keys/read-marker/set/0.1#request",
  "issuer": "did:key:z6MkLaptopHolderExample",
  "recipient": "did:webvh:QmVta:vta.example",
  "issuedAt": "2027-02-01T11:02:00Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "readMarker": {
      "version": 408,
      "createdAt": "2027-02-01T09:58:41Z"
    },
    "rewind": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T11:02:00Z",
    "verificationMethod": "did:key:z6MkLaptopHolderExample#z6MkLaptopHolderExample",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The key holder answers with a document of type `https://trusttasks.org/spec/rooms/keys/read-marker/set/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json): the marker now in force, and whether this request moved it. A refusal is a `trust-task-error` ([SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification)).

### Moved

```json
{
  "id": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3002",
  "type": "https://trusttasks.org/spec/rooms/keys/read-marker/set/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkPhoneHolderExample",
  "issuedAt": "2027-02-01T10:20:01Z",
  "threadId": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3001",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "readMarker": {
      "version": 415,
      "createdAt": "2027-02-01T10:14:02Z"
    },
    "moved": true
  }
}
```

### Not moved: another device had already read further

The laptop, which last showed the room to version 412, sets that marker; the phone's set has already arrived. The laptop adopts the phone's (Producer item 5).

```json
{
  "id": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3004",
  "type": "https://trusttasks.org/spec/rooms/keys/read-marker/set/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkLaptopHolderExample",
  "issuedAt": "2027-02-01T10:21:05Z",
  "threadId": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3005",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "member": "did:webvh:QmAlice:personas.example:northwind",
    "readMarker": {
      "version": 415,
      "createdAt": "2027-02-01T10:14:02Z"
    },
    "moved": false
  }
}
```

### Refused: a room this member does not hold, or a member this caller does not act as

```json
{
  "id": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3006",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmVta:vta.example",
  "recipient": "did:key:z6MkPhoneHolderExample",
  "issuedAt": "2027-02-01T10:20:01Z",
  "threadId": "urn:uuid:3b1d6f2a-8c4e-4e9a-a7d2-5f0b9e1c3007",
  "payload": {
    "code": "rooms/keys/read-marker/set:notHeld",
    "message": "No group state for this room is held for a member you act as.",
    "retryable": false
  }
}
```

## Retry safety

The task is **idempotent**: a set repeated after a lost response finds the marker already where it put it, writes nothing and answers `moved: false` with that marker (Recipient items 3 and 4), and a repeated rewind finds the marker equal to the one it asks for. A client **MAY** therefore retry it as it is, under the same document `id` or a fresh one, without reading the marker first. **Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless.

A forward set that arrives late — after the member has rewound — is the one case a retry can change: it moves the marker forward past the rewind. A client **SHOULD NOT** retry a forward set issued before a rewind it has since sent.

## Security & Privacy

### Data carried

A room identifier, a member DID and a marker — a room version and a host timestamp — in; the marker in force and `moved` back. Nothing about which messages were read, who wrote them, or what they said: the version and `createdAt` are values the host itself assigned, which the member's clients already hold. The schema is closed and admits no other member, so no request carries a destination for the marker.

### Where a marker goes

**To the key holder, and back to the member's own clients — nowhere else.** The key holder sends it to no room, host, owner, member or community, and puts it in nothing it signs or seals (Recipient item 8). A **client** that receives a marker — in this response, through [`rooms/keys/list`](../../../list/0.3/spec.md) 0.3, or in [`rooms/keys/read-marker/moved`](../../moved/0.1/spec.md) — holds the same obligation: it **MUST NOT** include it, or a "read" state derived from it, in anything it sends to a room's host, a community or a room. A member who wants others to know they have read something says so in the room.

### Correlation

**The key holder learns when the member reads each room** — a reading pattern, room by room, over time. It largely knows this already, because it opens the records the member reads on every sealed tier, but a marker states it precisely and on every tier. That is bounded three ways: the key holder keeps only the marker in force, with no history of earlier ones, of when each was set, or of which device set it (Recipient item 7); it writes none of them to a log or audit record (Recipient item 9); and a producer sets a marker when a view settles rather than per message (Producer item 3), so the moments it learns are coarse. A key holder **MUST NOT** use the marker for anything but returning it to the member's clients — not to rank, recommend or report on the member's rooms.

**No host and no other member learns anything.** Nothing in this task reaches a host, so a host cannot tell from it which member has read which record, or when; the marker is built only from values the host already served. The member's identifier in `member` is one the key holder already holds; the client's is pairwise, as in every `rooms/keys/*` task.

**The member's other devices learn that it moved**, through `rooms/keys/read-marker/moved` — documents that travel encrypted to each client and whose arrival its mediator and push path see. [`rooms/keys/read-marker/moved`](../../moved/0.1/spec.md#correlation) bounds who receives them and what that path learns.

### Retention

Durable, with the key holder's state for the room and member: one marker, replaced by each set that moves it, discarded when that state is discarded (Recipient item 7). There is no history to read back. A key holder **MAY** include the marker in a backup of that state; a member restoring without it has every room's messages shown as their clients counted them before, which loses nothing.

### Consent/purpose

The member keeping their own place in their own conversations, across their own devices. Nothing is disclosed to anyone else, so no other party's consent is involved. This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
