---
slug: rooms/changed
version: "0.1"
title: "Rooms — Changed"
summary: "A room's host tells the address of an armed watch that the room has moved past the version its member had read to — the room and its latest version, nothing about what changed, and no response."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - watch
  - signal
  - wake
  - unread
parties:
  - role: Host
    requirement: REQUIRED
    member: issuer
  - role: Watch address
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The address's agent acts on a signal only when it comes from the host it armed the watch with.
    The transports a host sends it on authenticate the sender; a proof says the same where one does
    not. What a reader then does — list the room — is authorized and answered as every listing is.
sideEffects:
  level: none
  rationale: >-
    A hint. The recipient reads the room, or does not; nothing is stored or changed because a signal
    arrived.
subjectPath: /roomId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The room and its latest version, which a listing would have shown the member anyway. Nothing
    about what changed, who changed it, or how much.
retention:
  class: transient
  rationale: >-
    A reader uses the signal to decide to list the room and keeps nothing of it; the listing is
    what it keeps.
errorCodes: []
related:
  - rooms/watch
  - rooms/records/list
  - rooms/keys/read
  - rooms/keys/browse
  - rooms/signal-delay/set
  - push/wake
---

## Abstract

The **Rooms — Changed** Trust Task is the signal an armed [`rooms/watch`](../../watch/0.1/spec.md)
sends: *room R has moved past the version you had read to*. It names the room and the room's latest
version, and says nothing else — not what changed, not who changed it, not how many things did.
The host sends one per arming, and the member re-arms after it has read.

It is one-way: there is no `#response`. A reply would tell the host when the member's device woke,
which is more than the member asked the host to know.

The signal reaches a sleeping device the way every Trust Task document does. It waits in the
address's mediator queue, and a mediator that is a trigger for the device
([`push/wake`](../../../push/wake/0.2/spec.md)) wakes it with the
[push binding](../../../../bindings/push/0.1/spec.md)'s contentless doorbell. The device drains
the queue, finds the signal, and decides for itself — after reading the room — whether there is
anything to tell its person.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the host) **MUST**:

1. Send a signal only for a watch it holds armed, to that watch's address, as
   [`rooms/watch`](../../watch/0.1/spec.md) Host item 4 says, and send nothing else to the address.
2. Carry the room exactly as the member named it and the room's latest version when it sends, and
   **MUST NOT** carry anything about what changed — in `ext` or anywhere else — nor vary the
   signal's size or timing by it.
3. Send it over a transport binding the address's DID document advertises, preferring one that
   hides the sender from the address's mediator where both parties speak it.
4. **SHOULD** wait a short, randomised interval — a few seconds — after the change that fires the
   watch before sending, so that the signal's time does not mark one write exactly, unless the
   room's signal delay is `none` ([`rooms/signal-delay/set`](../../signal-delay/set/0.1/spec.md)),
   when it **MUST** send without a wait of its own. A host **MUST** apply the signal delay it serves
   in [`rooms/info`](../../info/0.4/spec.md), never a watcher's preference: the wait protects the
   room's writers as much as the watcher (*Correlation*). It **MUST NOT** hold a signal back to wait
   for a later change, or send a second for the same arming.
5. Treat delivery as the delivery layer's: a signal that is lost costs the member nothing it cannot
   recover, because its next listing catches up. A host **MUST NOT** queue signals per change or
   re-send one the delivery layer accepted.

A conforming **consumer** (the address's agent, for its member) **MUST**:

1. Act on a signal only when it comes from the host the member armed a watch with, for a room the
   member armed one on, and drop any other.
2. Act on it only by reading the room — never by showing anything to its person on the strength of
   the signal alone — and **MUST NOT** treat `latestVersion` as evidence of anything but a reason to
   list.
3. Re-arm the watch after it has read, with the version it has read to
   ([`rooms/watch`](../../watch/0.1/spec.md) producer item 4), if it wants the next signal.

## What a reader does with a signal

A signal says only that the room moved. Moving includes a reaction, an edit, a curation, a prune or
a profile written, as well as a message — and **on the `attributed` and `private` tiers the host
cannot tell these apart**, because it never sees a record's `kind` (`SealedRecordBody`, *Kind*). So
a signal can wake a device for a reaction exactly as for a message, and whether there is anything
*unread* is decided by the reader, after it reads:

1. **Skip a signal it has caught up past.** If `latestVersion` is at or below the reader's
   watermark, there is nothing to read; it re-arms and stops.
2. **List what changed.** [`rooms/records/list/0.6`](../../records/list/0.6/spec.md) with
   `sinceVersion` set to its watermark, to the end, applying *After a prune* where a page carries
   `lastPrune`.
3. **Learn each changed record's kind.** On an `open` room the row carries it. On the sealed tiers
   the reader opens the new and changed records through its member's key holder, several at a time
   ([`rooms/keys/read/0.2`](../../keys/read/0.2/spec.md), or
   [`rooms/keys/browse/0.4`](../../keys/browse/0.4/spec.md)), and applies the `SealedRecordBody`
   reading rules to each.
4. **Count as unread** only a record that, as the reader shows it, is a message — `kind: message`,
   or an untitled record without a kind where the reader shows those as messages
   (`SealedRecordBody`, *Untitled records*) — and that is:
   - **new to the reader**: a key it has not shown its member before; or, for a reader that keeps no
     copy of the room, one whose `createdAt` (else `updatedAt`) is after the reader's own read
     marker for the room. A new version of a message the reader has already shown is an edit, not
     an unread message. The reader uses the host's `createdAt`, not the author's `writtenAt`, so
     that a message back-dated into the past still counts as new;
   - **not its member's own**: its author, or its `createdBy` where the reader uses that, is not a
     persona the member acts as in this room;
   - **current**: not withdrawn (an empty `body` and no `file`), retracted, expired
     (`validUntil`), or refused by the reading rules.
5. **Count nothing else as unread.** A reaction is not a message: a reader **MUST NOT** count one
   as an unread message, though it **MAY** tell its member, separately, that someone reacted to a
   record the member wrote. Edits, curations, tombstones, prunes, notes and the room's profile are
   not unread messages either; a reader **MAY** show new notes in the room's library without
   alerting.
6. **Alert only on what it found.** A reader **SHOULD** raise a notification to its person only
   when step 4 found an unread message, and derives the notification's wording from the records it
   opened, never from the signal or the wake
   ([push binding §2](../../../../bindings/push/0.1/spec.md)). A signal that turns up only
   reactions or edits raises no alert. Where the platform lets it, a reader **SHOULD** arrange that
   the wake itself is silent, so that this decision is the reader's.
7. **Advance its watermark and re-arm.** The new watermark is the highest version it has read,
   which is what it gives `rooms/watch` as `sinceVersion`.

The read marker of step 4 is the member's own: where its member has seen the conversation up to.
A reader keeps it itself, or **MAY** keep it with its member's key holder through
[`rooms/keys/read-marker/set`](../../keys/read-marker/set/0.1/spec.md), so that every one of the
member's devices counts against the same marker — reading it back with
[`rooms/keys/list/0.3`](../../keys/list/0.3/spec.md) and told when another device moves it by
[`rooms/keys/read-marker/moved`](../../keys/read-marker/moved/0.1/spec.md). Either way it is
**never sent to the host, the room or any other member**: nothing in this family records it at a
host or shares it beyond the member's own clients and key holder, and a reader **MUST NOT** derive
a read receipt from it.

## Request

The host (`issuer`) sends the watch's address (`recipient`) a document whose `payload` validates
against the top-level schema in [`payload.schema.json`](payload.schema.json). There is no response.

### The room moved past version 412

```json
{
  "id": "urn:uuid:9c3e2f10-6a4b-4d8e-b1f7-2e5d0c9a7001",
  "type": "https://trusttasks.org/spec/rooms/changed/0.1",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.SeyJ0IjoiZG0iLCJzIjoiZGlkOndlYjptZWRpYXRvci5leGFtcGxlIn0",
  "issuedAt": "2027-02-01T10:14:07Z",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "latestVersion": 415
  }
}
```

## Response

None. The task is one-way, and its schema declares no `#response` form.

## Security & Privacy

### Data carried

A room identifier and a version. Both are what the member could have had from a listing, and
nothing else is admitted: the schema is closed, and producer item 2 keeps `ext` and the signal's
size and timing free of anything about the change.

### Correlation

The signal travels encrypted to its address. Its address's mediator sees a document for the
address, and the push gateway and the platform push service see a contentless wake: each learns
that something arrived, and when, and nothing of the room. One signal per arming
([`rooms/watch`](../../watch/0.1/spec.md)) keeps the number of signals from saying how busy the
room was, and producer item 4 keeps their timing from marking single writes. The host learns
nothing from sending one, because there is no reply. What the host does learn — the address, and
when the member re-arms — is described in [`rooms/watch`](../../watch/0.1/spec.md#correlation).

A room whose signal delay is `none` gives up the protection of producer item 4, for every member
at once: a party who sees a writer's request reach the host and a signal leave it soon after can
match the two, and learn when the writer wrote and that the writer and the watcher share the room.
That is why the setting is the room's, decided by a holder of `admin` and served to every member,
and why no watch request can change it
([`rooms/signal-delay/set`](../../signal-delay/set/0.1/spec.md)).

### Retention

Nothing is kept. A reader acts on a signal by listing the room and keeps the listing; the host
forgets the watch when the signal is sent.

### Consent/purpose

The member asked for exactly this signal at exactly this address. A host sends nothing else there,
and a reader uses it only to decide to read the room.
