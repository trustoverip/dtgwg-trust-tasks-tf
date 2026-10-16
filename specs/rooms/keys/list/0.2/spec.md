---
slug: rooms/keys/list
version: "0.2"
title: "Rooms Keys — List"
summary: "An agent asks which rooms its principal's key holder can open, as which member, how far back each one reads — and, if it asks, what the member has named each one for themselves."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - persona
  - label
parties:
  - role: Agent
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Oracle
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The response names every room this key holder can open for the members the caller acts as, and — when asked — what each member privately calls each one. A request whose origin depended on the transport would let a compromised channel enumerate both."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The answer changes as commits arrive, chains are delivered and labels are written, so an undated request cannot be placed against the state it described."
sideEffects:
  level: none
  rationale: >-
    A read of what this key holder already holds. Opens nothing and changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses which rooms the members the caller acts as hold keys for, as which member, and how far back each reads; with `includePersonal`, also each member's personal label — free text they wrote for themselves, returned only to a caller that acts as them. No record content and no key material.
retention:
  class: transient
  rationale: >-
    A snapshot of state the recipient already holds. Worth nothing once acted on, and stale the moment a commit arrives or a label changes.
errorCodes: []
related:
  - rooms/keys/label
  - rooms/keys/open
  - rooms/keys/act
  - rooms/keys/chain
  - rooms/records/list
---

## Abstract

An agent asks its principal's **key holder** which rooms it can open, **as which member**, and how far back each one reads — and, when it asks, the **personal label** the member keeps for each with [`rooms/keys/label`](../../label/0.1/spec.md).

This is what a surface needs before it can show anything at all. Every other room task takes a `roomId` the caller already knows; this is where it comes from.

**It is key custody, not membership.** The two differ, and the difference is the useful part — see below.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

- **Each entry names its `member`** — the DID the room admitted, whose group state the entry is. 0.1 answered for "the principal" with one entry per room, which a key holder serving one person as several personas cannot do: a person who holds a room both as themselves and as the place they hold for an organisation has two sets of group state for it, at different epochs, and a 0.1 response either listed the room twice with nothing saying whose each entry was, or listed it once and hid the other. Implementations carried the member in `ext` to cope. The entry is now one per **room and member**, and the member it names is what a client passes to `rooms/keys/label` as `member` and to [`rooms/keys/act`](../../act/0.3/spec.md) as `issuer` to act on that entry and not the other.
- **`includePersonal`**, and the optional `personal` it returns: the member's own label for the room. Off by default.
- **The entries are those of the members the caller may act as**, stated as a requirement (Recipient item 1), rather than "its principal's".

Every 0.1 member keeps its meaning. A recipient serving 0.1 and 0.2 answers each at its own version.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Recipient

A conforming **recipient** (the key holder):

1. **MUST** answer with one entry for each room and member for which it holds group state and its caller may act as that member, and with no other. A caller to which a member has delegated narrower standing ([`rooms/keys/present`](../../present/0.2/spec.md)) does not act as the member.
2. **MUST** include `personal` in an entry when, and only when, the request carries `includePersonal: true` and the entry's member has a label for the room. It **MUST NOT** include one otherwise — in particular not for a request without `includePersonal`.
3. **MUST** return a label exactly as it is stored ([`rooms/keys/label`](../../label/0.1/spec.md) Recipient item 7).

### Consumer of the response

A conforming **consumer** (the caller):

1. **MUST NOT** present this list as an authority to act, only as what can be opened.
2. **MUST NOT** include any part of a `personal` label in anything it sends to a room's host, a room's owner, a community or a room.
3. **SHOULD NOT** pass a `personal` label to a website — a community's member portal, or any page asking which rooms the member holds — unless the member has chosen to show it to that site. A wallet answering such a page **SHOULD** request this list without `includePersonal`, so that it never holds labels it is not going to show.

## Definitions

**`includePersonal`** — asks for each entry's personal label. Absent means `false`.

**`rooms[].roomId`** — the room's identifier.

**`rooms[].member`** — the member whose group state this entry is: the DID the room admitted. Often a persona the key holder holds for the principal in one of their contexts.

**`rooms[].epoch`** — the epoch this key holder is at, for this member.

**`rooms[].earliestReadableEpoch`** — the earliest epoch it can derive a key for, for this member, walking the epoch key chain it holds.

**`rooms[].personal`** — the member's personal label for the room: see [`rooms/keys/label`](../../label/0.1/spec.md) and `PersonalLabel` in [`rooms/_shared/0.1/personal-label.schema.json`](../../../_shared/0.1/personal-label.schema.json).

### What the two epochs tell a surface, and why both are here

Neither number alone says what a member can read.

- `epoch` **behind the room's own** means a commit has not been delivered. Records written since will not open, and the repair is delivery, not anything the member does.
- `earliestReadableEpoch` **equal to `epoch`** means this key holder reads only from where it joined. The repair is [`rooms/keys/chain`](../../chain/0.1/spec.md).
- `earliestReadableEpoch` of **`1`** means the room's whole retained history is reachable.
- Anything **between** is a chain delivered in part, or one deliberately severed below that point — and a surface cannot tell those apart, because they are the same fact about reachability.

### Membership is not custody

This lists rooms whose **group state** the recipient holds. A member may hold a perfectly good membership credential for a room whose Welcome never arrived — that room is absent here, and correctly so: the key holder cannot open it.

The converse also holds and matters more. A key holder that has not been told about a removal still holds keys for a room its member has been removed from. It cannot write there — the host checks credentials, and theirs no longer verify — and it can still open what it already has, which is exactly what removal has always meant. Its personal label stays with it ([`rooms/keys/label`](../../label/0.1/spec.md) Recipient item 8): a room a member can no longer write in is the one they most need to recognise.

### Which name a surface shows

*This subsection is informative.* A surface listing a member's rooms prefers the member's **personal label**, then the room's own **profile** name (written by its curators, read by its members), then the label of the community's **offer** ([`vtc/rooms/offers/list`](../../../../vtc/rooms/offers/list/0.2/spec.md)), then the room's DID — and keeps the room's own name reachable beside a personal one. [`rooms/keys/label`](../../label/0.1/spec.md) gives the reasons.

## Request

An **Agent** sends this to its principal's **Oracle**.

### Rooms, with the member's own names for them

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/list/0.2#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-10-16T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": { "includePersonal": true }
}
```

### Rooms only — what a wallet asks before sharing a room with a page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/list/0.2#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-10-16T09:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

The **Oracle** answers with one entry per room and member, using the sub-schema reachable via `$anchor: "response"`.

Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### One room held as two personas, and two others

The Northwind room is held both as the principal's work persona and as the place they hold for their organisation; each entry is its own member, with its own reach back into the room's history, and only the first has a personal label. The book club is held as a third persona. The archive is missing a commit, and wants delivery.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/list/0.2#response",
  "issuer": "did:example:oracle",
  "recipient": "did:example:agent",
  "issuedAt": "2026-10-16T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "rooms": [
      {
        "roomId": "did:webvh:example.com:rooms:northwind",
        "member": "did:example:alice-work",
        "epoch": 7,
        "earliestReadableEpoch": 1,
        "personal": {
          "label": "Northwind tender",
          "tags": ["work", "bids"],
          "updatedAt": "2026-10-16T08:12:00Z"
        }
      },
      {
        "roomId": "did:webvh:example.com:rooms:northwind",
        "member": "did:example:acme-seat",
        "epoch": 7,
        "earliestReadableEpoch": 7
      },
      {
        "roomId": "did:webvh:example.com:rooms:bookclub",
        "member": "did:example:alice-personal",
        "epoch": 4,
        "earliestReadableEpoch": 1,
        "personal": {
          "label": "Book club",
          "notes": "Next: chapter 9.",
          "updatedAt": "2026-09-30T19:40:00Z"
        }
      },
      {
        "roomId": "did:webvh:example.com:rooms:archive",
        "member": "did:example:alice-work",
        "epoch": 2,
        "earliestReadableEpoch": 1
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Room identifiers, member DIDs and two epoch numbers each; with `includePersonal`, each member's personal label. No record content, no key material.

**The identifiers are sensitive**, and they are more than any single room operation discloses: together they are the members' room membership as key custody sees it, and the `member` beside each says which of a person's personas holds which room — a link between personas that no room and no host ever sees. A recipient answers only for the members its caller acts as (Recipient item 1) and **MUST NOT** offer a form of this that answers for anyone else. A caller acting for one of a person's contexts therefore never learns that their other personas hold a room at all.

**A personal label is personal data** — free text a person wrote for themselves, often naming others. It is returned only when asked for (Recipient item 2), and a consumer sends it nowhere but to the member (Consumer items 2 and 3). It is plain text, rendered as text and never treated as instructions, by a person's surface or by an agent.

A producer **MUST NOT** put a filter in `ext`. A filter would be a way to ask "is this member in *that* room", which is a different question with a different answer and no authorization story.

### Correlation

The recipient learns nothing new — it is answering from its own storage.

The epochs are a weak history signal to anyone who sees several responses over time: a room whose `epoch` advances quickly has an active membership, and `earliestReadableEpoch` moving to `1` says a chain has just been delivered, which usually means its member has just joined or restored. A surface that polls this on a timer publishes that rhythm to whatever it polls over; refreshing on use rather than on a clock says less.

### Retention

Transient on both sides. It is a snapshot of state the recipient already holds, worth nothing once acted on and stale the moment a commit arrives or a label changes. A caller that caches it **SHOULD** treat a failure to open as the authority, not the cached list, and **SHOULD** keep a cached label no longer, and with no less protection, than it keeps the member's session.

### Consent/purpose

The list is read so that a principal's own surfaces can show them their own rooms, under the names they chose. It confers no authority: a room appearing here says the key holder can decrypt its records, never that its member may still act in it — the host decides that, from credentials the room issued.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
