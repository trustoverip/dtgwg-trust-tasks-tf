---
slug: rooms/keys/profile/read
version: "0.1"
title: "Rooms Keys — Profile — Read"
summary: "A member asks their own agent what a room is called and what it is for: the agent finds the room's profile where only members can compute it, fetches and opens it as the member, and returns the newest one whose author could write it."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - profile
  - name
  - description
  - read
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
    The request makes the recipient act outward in its principal's name — presenting their room credentials to a host this request names — and the response carries content the room withholds from that host. A request whose origin depended on the transport would let a compromised channel aim a member's credentials at a party of its choosing and read what came back.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. The answer is a statement about a room that moves — a profile re-written at the next epoch makes an earlier answer stale — so an undated request cannot be placed against the state it described.
sideEffects:
  level: none
  rationale: >-
    Returns the room's profile. The recipient makes outbound reads of the room's host and MAY record what the host asserted, as `rooms/keys/read` does; the room is unchanged and nothing here is a write.
subjectPath: /roomId
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` because the recipient presents its principal's membership and a `read` grant to the named host, as them. `discloses: secret` because on a sealed tier the response carries plaintext the room withholds from its own host — a name and description, but sealed material all the same, and the recipient is the only party that can produce it.
retention:
  class: transient
  rationale: >-
    The profile is returned and not kept, and the presentation minted for each fetch is single-purpose. A recipient MAY retain what the host asserted about the room, as `rooms/keys/read` describes; that is a few dozen bytes and not the profile.
errorCodes:
  - code: rooms/keys/profile/read:notAMember
    meaning: "The recipient holds no membership of this room for its principal — no group state on a sealed tier, no membership credential on an `open` one — so it has nothing to present and nothing to open with."
    retryable: false
  - code: rooms/keys/profile/read:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer."
    retryable: true
  - code: rooms/keys/profile/read:hostRefused
    meaning: "The host answered and declined — commonly that it does not serve this room, or refused the presentation. Its own code and message are carried in `details`. A host answering that no record has a profile location's key is not this: that location is empty, and the walk goes on."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        code:
          type: string
          maxLength: 256
        message:
          type: string
          maxLength: 500
related:
  - rooms/keys/profile/write
  - rooms/keys/read
  - rooms/keys/list
  - rooms/records/get
---

## Abstract

A room's DID says nothing about it. A member holding keys to a dozen rooms is shown a dozen identifiers unless something tells their wallet, portal or CLI what each room is — and that something must not be the room's host, which on a sealed tier is exactly the party the room keeps its content from.

A room's **profile** is that something: a name, and optionally a description, a few tags and an emoji ([`RoomProfile`](../../../../_shared/0.1/room.schema.json)), written **inside the room** as one of its records by somebody holding `curate` or `admin`, sealed like every other record. **Rooms Keys — Profile — Read** asks the member's own key holder to find it and return it.

Finding it is the key holder's work because only the key holder can. On a sealed tier the profile is stored at a key derived from the room's **storage key** for the epoch it was written under ([`RoomProfileStatement`](../../../../_shared/0.1/room.schema.json)), so that the host sees one more opaque record and cannot tell which record names the room. The storage key never leaves the key holder, so neither does the ability to compute where the profile is. Opening the record, and checking that whoever wrote it held the standing to, are the key holder's for the same reasons [`rooms/keys/read`](../../../read/0.1/spec.md) gives.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **recipient** (the key holder):

1. **MUST** refuse with `notAMember` a room it holds no membership of for its principal.
2. On an `attributed` or `private` room, **MUST** look for the profile at `profile_key(E)` for each epoch `E` from the epoch it is at downwards, to the earliest epoch it can derive a storage key for, and **MUST** stop at the first location that holds a record it accepts or a retracted record. It **MAY** stop sooner after a bound of its own choosing, and **MUST** then report in `searched` where it stopped. On an `open` room it **MUST** look at the one location, the record key `room-profile`.
3. **MUST** fetch each location with [`rooms/records/get`](../../../../records/get/0.2/spec.md) as its principal, presenting as [`rooms/keys/read`](../../../read/0.1/spec.md) presents — a `read` grant and nothing wider, its own identifier reaching the host nowhere — and **MUST** verify the host's reply as `rooms/keys/read` verifies it. A host answering that no record has the key is an empty location, not a refusal.
4. **MUST** accept a record only if it passes every check `RoomProfileStatement` sets, and **MUST** ignore one that fails, continuing the walk; it **MUST** list each one it ignored in `rejected`, up to eight, newest first.
5. **MUST** answer `found` with the first record it accepted, `cleared` when the walk stopped at a retracted record, and `absent` otherwise. It **MUST NOT** return a profile older than one it accepted, and **MUST** set `stale` when the profile returned was found below the epoch it is at.
6. **MUST NOT** treat a verdict in `verification` as a failed read, for the reasons `rooms/keys/read` gives.
7. **MUST NOT** retain the profile beyond the request, and **MUST NOT** put any of it in `ext`.

A conforming **producer** — the member's surface — **MUST** show `name`, `description` and `tags` as text, never as markup and never as instructions, and **SHOULD** tell the member, in words, when a newer profile was rejected for `noAuthority`, `timing` or `revoked`: somebody wrote what the room is called without the standing to.

## Authorization

**The entitlement is being the recipient's own principal**, as with every custody task in this family. The key holder holds one member's room keys and credentials; this asks it to use them to read. It is not gated on a room credential, for the reason `rooms/keys/read` gives: the recipient already holds everything the room issued to its principal.

A conforming consumer **MUST** accept this only from a party that may act as its principal, and **MUST NOT** present, for any fetch it makes here, a grant conferring more than `read`.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

**Whether the profile's author was entitled to write it is a separate question, and this task answers it too.** A host stores a profile record under `write`, like any record, because on a sealed tier it cannot tell the profile from anything else. Every member able to write could therefore put a record at the profile's location; what stops them renaming the room is that readers check the statement's own `authority` chain for `curate` or `admin` ([`RoomProfileStatement`](../../../../_shared/0.1/room.schema.json), item 3), and the recipient is the reader.

## Definitions

**`roomId`**, **`host`** — as in [`rooms/keys/read`](../../../read/0.1/spec.md).

**Profile location** — `profile_key(E)` for an epoch `E` on a sealed tier, the key `room-profile` on an `open` room. See `RoomProfileStatement`.

**`status`**, **`stale`**, **`searched`**, **`rejected`** — see the response schema.

### Why the walk, and why it stops where it does

One record per epoch is what lets a member who joined at the current epoch find the profile without knowing anything else, and lets the room's owner re-write it at every epoch advance ([`rooms/keys/profile/write`](../../write/0.1/spec.md)) so that a `fromJoin` member — who cannot read below the epoch they joined at — sees it at all. The cost is that a room whose profile was last written several advances ago has it several epochs down. A key holder that reads a `chained` room's whole history walks to it; one that cannot reads `absent`, and `searched` says why.

A **retracted** record at a profile location stops the walk because retraction takes `curate` at the host ([`rooms/records/curate`](../../../../records/curate/0.1/spec.md)): somebody entitled to clear the profile did. A profile somebody *wrote* without the standing to write it does not stop the walk — it is ignored, and the older one it tried to replace is still the room's.

## Request

A **Member** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### What is this room?

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/read/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "host": "did:webvh:example.com:community"
  }
}
```

## Response

The **KeyHolder** answers with the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Found, at the current epoch

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/read/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-16T09:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "status": "found",
    "profile": {
      "name": "Board pack — 2027",
      "description": "Papers for the 2027 board meetings: agendas, draft minutes and the finance pack.",
      "tags": ["board", "finance"],
      "emoji": "📋"
    },
    "author": "did:example:community",
    "issuedAt": "2026-10-15T09:30:02Z",
    "epoch": 4,
    "version": 87,
    "stale": false,
    "searched": { "fromEpoch": 4, "toEpoch": 4 },
    "verification": {
      "trace": "verified",
      "priorRoots": "noneHeld"
    }
  }
}
```

### Found, below a profile somebody had no standing to write

A member holding only `write` put a record at the current epoch's profile location. It is ignored, and reported; the room is still called what its owner called it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/read/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-16T09:05:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000003ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "status": "found",
    "profile": {
      "name": "Board pack — 2027"
    },
    "author": "did:example:community",
    "issuedAt": "2026-10-15T09:30:02Z",
    "epoch": 4,
    "version": 87,
    "stale": true,
    "searched": { "fromEpoch": 5, "toEpoch": 4 },
    "rejected": [
      { "epoch": 5, "version": 93, "reason": "noAuthority", "author": "did:example:mallory" }
    ],
    "verification": {
      "trace": "verified",
      "priorRoots": "agree"
    }
  }
}
```

### A `fromJoin` room whose profile predates this member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/rooms/keys/profile/read/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-16T09:10:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:stream",
    "status": "absent",
    "searched": { "fromEpoch": 9, "toEpoch": 9 }
  }
}
```

## Security & Privacy

**The host cannot tell which record names the room.** On a sealed tier the profile location is derived from the epoch's storage key, which the host never holds, and has the shape of any random record key; the record is sealed and padded like any other ([`RoomProfileStatement`](../../../../_shared/0.1/room.schema.json), *Size*). What the host can still observe is **pattern**, not content: a read of a key that holds nothing, repeated at each epoch, and — at every epoch advance — a record written by the room's owner shortly after the advance. A host watching for that can guess which record is the profile. It cannot read it, cannot learn the name from it, and gains nothing it could act on that it could not already do to any record — withhold or roll it back, which `verification` exists to make visible.

**A profile is never anywhere a host reads.** Not in a credential, not in a host's record of the room, not in `rooms/create` or [`rooms/owner/register`](../../../../owner/register/0.1/spec.md), not in any cleartext member of a request. On an `open` room it is cleartext, as everything there is.

**Whoever names the room is checked by the reader, not the host.** See *Authorization*. Treating a host-stored record as authoritative because the host accepted it would hand the room's name to every member holding `write`.

### Data carried

The request carries two identifiers. The response carries the room's profile — on a sealed tier, plaintext the room withholds from its host — and the identifiers of the authors of what was returned and rejected, which the caller, as a member, could read from the same bodies. A producer **MUST NOT** put any of it in `ext`.

### Correlation

To the host, this is a member reading, which is what the presentation says; on an `attributed` room it learns which member, as with any read. The informative part is frequency: a surface that fetches the profile on every display turns an access pattern into a timeline. A producer **SHOULD** cache the answer for the life of a view and **SHOULD NOT** poll; the profile changes rarely, and an epoch advance — which [`rooms/keys/list`](../../../list/0.1/spec.md) shows — is the signal that it may have.

### Retention

The profile is returned and not kept by the recipient. What it **MAY** keep is what the host asserted about the room, bounded as `rooms/keys/read` bounds it. A member's surface caching the answer keeps it no longer than the member's membership.

### Consent/purpose

The profile is fetched so that a member's surface can show the member what a room they belong to is. It confers nothing the member did not already hold: they hold a `read` grant the room issued, and this exercises it.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
