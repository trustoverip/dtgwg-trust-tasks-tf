---
slug: rooms/keys/label
version: "0.1"
title: "Rooms Keys — Label"
summary: "A member keeps their own private name, description, notes and tags for a room they hold, with their key holder, for their eyes only — never sent to the room, its host or a community."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - label
  - personal
  - rename
  - notes
  - persona
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
  rationale: >-
    The request writes the member's own notes about a room into storage only they can read back. A request whose origin depended on the transport would let a compromised channel rewrite what a member is shown a room as — the name they trust most, because they believe they wrote it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Mutating, so SPEC §7.3 item 17 sets REQUIRED as the floor. Each detail is replaced wholesale, so a replayed earlier request would silently put back a name or notes the member has since changed.
sideEffects:
  level: mutating
  rationale: >-
    Replaces or removes the given details of the member's personal label for one room, in the key holder's own storage. Reversible by another request; nothing reaches the room, its host, its group or any credential.
subjectPath: /member
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    The request carries free text a person wrote about a room for themselves — what it is to them, who is in it, what they agreed there — which is personal data about them and often about others. The response returns the same label to the caller that wrote it. The key holder keeps it for the member alone and sends it nowhere.
retention:
  class: durable
  rationale: >-
    A label is worth having only if it is there next time: it persists with the key holder's state for the room and the member, is backed up with it, and goes when that state goes.
errorCodes:
  - code: rooms/keys/label:notHeld
    meaning: "The recipient holds no group state for `roomId` as `member` — or `member` is not one the caller may act as, which is answered the same way so that the error does not say which members of which rooms the recipient serves. A room absent from rooms/keys/list for this member cannot be labelled."
    retryable: false
related:
  - rooms/keys/list
  - rooms/keys/act
  - rooms/info
  - vtc/rooms/offers/list
  - vtc/rooms/community/update
---

## Abstract

A member who holds several rooms has to be able to tell them apart, and a room's DID says nothing about it. The names others give a room — the room's own profile, written by its curators; the label a community puts on an offer — are theirs: they may be missing, generic, in another language, or chosen for an audience the member is not. **Rooms Keys — Label** lets the member keep **their own** name for a room, with a description, notes and tags, held by their **key holder** beside the room's keys and shown back to them by [`rooms/keys/list`](../../list/0.2/spec.md) 0.2.

The label is **private to the member**. It is not part of the room, not sent to the room's host, its owner or any community, and not shared with the room's other members. It is the complement of the room's profile: a profile is one room's name for itself, written by its curators and read by every member; a personal label is one member's name for one room, written and read by that member alone.

It is kept per **room and member**. A person who holds one room as two personas — their own, and the place they hold for an organisation — keeps one label for each, and each is shown only where that persona is.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Producer

A conforming **producer** (the member's client):

1. **MUST** name in `member` the member whose label it means, as [`rooms/keys/list`](../../list/0.2/spec.md) 0.2 returns it beside the room.
2. **SHOULD** trim leading and trailing white space from every detail before sending it. The key holder stores what it receives.
3. **MUST NOT** put a credential, a key, a password or a secret of any kind in a detail. A label is shown on every surface that lists the member's rooms, and is not protected as a secret.

### Recipient

A conforming **recipient** (the key holder):

1. **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) a caller to which the member has delegated narrower standing — an agent holding a chain attenuated to itself by [`rooms/keys/present`](../../present/0.2/spec.md). A label is the member's, and is read and written only by a caller that may act as the member.
2. **MUST** refuse with `notHeld` a request whose `member` it holds no group state for in `roomId`, **and** one whose `member` is not a member its caller may act as. It **MUST** give the same answer to both, and **MUST** check no detail of the request against the stored label before it has decided neither applies.
3. **MUST** apply every change the request names together or not at all: each detail given replaces the stored one, each `removeX` removes one, each detail absent is unchanged. `tags` replaces the whole set. It **MUST** refuse with the standard `malformedRequest` a request that names no change — one whose only members besides `roomId` and `member` are in `ext`, which the schema's `minProperties` cannot tell from a change.
4. **MUST** treat the label as removed when no detail remains — after a request removing the last one, it holds no label for the room and member, and answers without `personal`.
5. **MUST**, when every value given already matches the stored one and every detail named for removal is already absent, answer with `changed: false` and write nothing; `updatedAt` does not move. Strings are compared exactly, as given; tags as a set.
6. **MUST**, when anything changed, set `updatedAt` to the time of the change and answer with `changed: true`.
7. **MUST NOT** case-fold, normalise, translate or otherwise rewrite a detail. What a member wrote is what they are shown.
8. **MUST** keep the label with its group state for the room and the member, protected as it protects that state, and **MUST** include it in a backup of that state. It **MUST** discard the label when it discards the group state for that room and member — and **MUST NOT** discard it for any other reason: not when the member is removed from the room (the key holder still opens what it holds, and an archived room is the one a member most needs to recognise), not when the room lapses, and not when its host or owner changes.
9. **MUST NOT** send a label, or any part of one, to anyone but a caller that may act as its member: not to the room, its host, its owner or a community; not in any document it completes, signs or seals for the member ([`rooms/keys/act`](../../act/0.3/spec.md), [`rooms/keys/seal`](../../seal/0.2/spec.md), [`rooms/keys/present`](../../present/0.2/spec.md)); and not in any record it writes to a room.
10. **MUST NOT** write a detail's text to a log or an audit record. It **MAY** record that the label for a room and member changed, and when.

## Authorization

**The entitlement is acting as the member** — being, or being the client of, the key holder's own principal, with standing for the persona named in `member`. That is the same question every `rooms/keys/*` task asks before it uses a member's state: whether this caller acts as this member. A client authorised for one of a person's contexts acts as the personas of that context and no other, so it reads and writes the labels of the rooms those personas hold and cannot reach another's — or learn, from `notHeld`, whether another persona holds the room at all (Recipient item 2).

A delegate is refused (Recipient item 1). [`rooms/keys/present`](../../present/0.2/spec.md) hands a delegate a narrow authority to act in one room; it does not make the delegate the member, and a label — what the member calls the room and what they wrote about it — is not part of what a delegate is given.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome. The decision is the key holder's, from its own record of whom its caller acts as.

**The model this task is designed for.** The key holder executes a change on the caller's authority alone, without a step-up and without anyone's consent. It confers nothing, reaches no other party, changes nobody's access, and is undone by the same task. A deployment whose policy asks for more applies it as it does to any act; whether any is required is that policy's, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

**No optimistic concurrency.** Two of the member's own devices changing the same detail at once: the later write wins. Details are replaced one by one, so a rename on one device does not overwrite notes written on another, and a label is one person's, rarely edited and always visible to them — a lost edit is seen and repaired. A precondition would cost every client a read before every write to guard against a race between a person and themselves.

## Definitions

- `PersonalLabel`, `PersonalLabelName`, `PersonalLabelDescription`, `PersonalLabelNotes`, `PersonalLabelTags` — see [`rooms/_shared/0.1/personal-label.schema.json`](../../../_shared/0.1/personal-label.schema.json).
- **Details** — a label's `label` (the member's name for the room), `description`, `notes` and `tags`.
- **Member** — the DID a room admitted: the subject of the membership credential it issued. Often a persona the key holder holds for a person in one of their contexts.

### Which name a surface shows

*This subsection is informative.*

A room can be known by several names, written by different parties for different readers. A surface showing a member their rooms is expected to prefer, in order:

1. the member's **personal label** (this task, read through [`rooms/keys/list`](../../list/0.2/spec.md) 0.2) — the only name nobody else wrote;
2. the room's own **profile** name — written by the room's curators, sealed in the room and read by its members, and the same for every member;
3. the label of the community's **offer** of the room ([`vtc/rooms/offers/list`](../../../../vtc/rooms/offers/list/0.2/spec.md)) — what the community called it when it offered it, which the member may have seen only once;
4. the room's **DID**.

A surface that shows a personal label **SHOULD** make the room's own name reachable beside it — as a second line, or on the room's detail view — so that a member who named a room long ago can see that its curators have since renamed it, and **SHOULD** mark the personal label as theirs ("your name for it"). Every one of these names is plain text and is rendered as text: names written by others come from parties a member does not control, and even a personal label is shown on surfaces — an agent, a terminal — where markup or control characters would be acted on.

## Request

An **Agent** — the member's client — sends this to the member's key holder (**Oracle**). The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Name a room, and say what it is

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/label/0.1#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-10-16T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:example:alice-work",
    "label": "Northwind tender",
    "description": "Pricing working room for the Northwind bid. Dana invited me.",
    "tags": ["work", "bids"]
  }
}
```

### Add notes, and remove the description

Only the details named change; the name and tags set above stay.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/label/0.1#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-10-16T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000002ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:example:alice-work",
    "notes": "Final numbers are in the March folder.\nAsk Dana before sharing outside the team.",
    "removeDescription": true
  }
}
```

### Forget the label altogether

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/rooms/keys/label/0.1#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-10-16T11:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:example:alice-work",
    "removeLabel": true,
    "removeDescription": true,
    "removeNotes": true,
    "removeTags": true
  }
}
```

## Response

The **Oracle** answers with the sub-schema reachable via `$anchor: "response"`: the label as it now stands, absent when none remains, and `changed`. A refusal is a `trust-task-error` ([SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification)).

### Named

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/label/0.1#response",
  "issuer": "did:example:oracle",
  "recipient": "did:example:agent",
  "issuedAt": "2026-10-16T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:example:alice-work",
    "personal": {
      "label": "Northwind tender",
      "description": "Pricing working room for the Northwind bid. Dana invited me.",
      "tags": ["work", "bids"],
      "updatedAt": "2026-10-16T09:00:01Z"
    },
    "changed": true
  }
}
```

### Forgotten

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/rooms/keys/label/0.1#response",
  "issuer": "did:example:oracle",
  "recipient": "did:example:agent",
  "issuedAt": "2026-10-16T11:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:example:alice-work",
    "changed": true
  }
}
```

### Refused: a room this member does not hold, or a member this caller does not act as

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:oracle",
  "recipient": "did:example:agent",
  "issuedAt": "2026-10-16T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "code": "rooms/keys/label:notHeld",
    "message": "No group state for this room is held for a member you act as.",
    "retryable": false
  }
}
```

## Retry safety

The task is **idempotent**: a request repeated after a lost response leaves the label as the first left it and answers `changed: false` (Recipient item 5). A client **MAY** therefore retry it as it is, under the same document `id` or a fresh one, without reading the label first.

## Security & Privacy

### Data carried

A room identifier, a member DID, and up to four details the member wrote, in; the same label back. The details are free text and tags a person wrote for themselves, and they are **personal data** — about the member, and often about the people they name ("Dana invited me"). They are plain text: a consumer renders them as text, never as markup, and an agent reading them — an assistant summarising a member's rooms — **MUST NOT** treat them as instructions. A producer **MUST NOT** put a secret in one (Producer item 3).

### Where a label goes

**To the key holder, and back to the member's own clients — nowhere else.** The key holder sends it to no room, host, owner or community and puts it in nothing it signs or seals (Recipient item 9). That is what makes it safe for a member to write what they actually think a room is.

A **client** that receives a label (through [`rooms/keys/list`](../../list/0.2/spec.md) 0.2) holds the same obligation: it **MUST NOT** include a label in anything it sends to a room's host, a community or a room. A wallet answering a **website** — a community's member portal, or any page asking which rooms the member holds — **SHOULD NOT** hand that page a label unless the member has chosen to show it to that site; by default it shares the room, never what the member calls it. A portal is the community's, and a label is exactly what the member did not tell the community.

### Correlation

The label is held by the member's own key holder, which already knows every room it holds keys for; storing a name beside each tells it nothing about the room it did not know, and tells it what the member thinks of the room, which it is trusted to keep. A label is never seen by any party that could join it to the room or to the member. The member's identifier in `member` is one the key holder already holds; the Agent's is pairwise, as in every `rooms/keys/*` task.

### Retention

Durable, with the key holder's state for the room and member: kept until the member changes or removes it, backed up with that state, and discarded when that state is discarded — never earlier (Recipient item 8). A key holder **MUST NOT** keep a label's earlier values: there is no history to read back, and no audit record carries its text (Recipient item 10).

### Consent/purpose

The member describing a room to themselves. Nothing is disclosed to anyone else, so no other party's consent is involved. This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
