---
slug: vtc/admin/events/subscribe
version: "0.5"
title: "VTC Admin Events — Subscribe"
summary: "An administrator's console opens one live channel per session over which a Verifiable Trust Community sends hints — which topic changed, and for badge topics a count — never records; the console re-reads through its signed reads, and polling stays the fallback."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [vtc, admin, console, events, live, subscribe, server-sent-events, sse, push, badge, notifications, directory, groups, rooms, community-rooms]
parties:
  - role: Administrator (console)
    requirement: REQUIRED
    member: issuer
  - role: Community maintainer (VTC)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: OPTIONAL
  rationale: >-
    The community decides which topics to stream from the caller's administrative standing, so it needs the caller from
    a proof over the request, not from whatever session carried it. A bearer session proves who opened a connection,
    and a stream opened on a session alone would tell whoever holds that session how busy another administrator's queue
    is. Every reconnection is a freshly signed request, so the channel is re-authorized each time it is opened. The
    response needs no proof: it arrives on the stream the caller just opened, grants nothing, and only reports which
    topics the community will hint about. A forged one could at worst make the console poll a topic it could have been
    told about.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A captured subscribe document, replayed, would open a stream of the signer's hints on the replayer's connection.
    Requiring issuedAt lets the community refuse any subscribe older than a short acceptance window, so a captured
    document is worthless within minutes.
sideEffects:
  level: none
  rationale: >-
    Opens a delivery and persists nothing. The stream exists only while its connection is open, and ending it — from
    either side — leaves no state behind. The resume position is a token the console holds, not a record the community
    keeps on its behalf.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    The stream discloses when the community's administrative state changes, per topic, and for the badge topics the
    caller's counts: how many actions await them, how many acknowledgements, how many join requests await a decision.
    It never discloses a record, an identifier of a record, or any member's personal data. Those reach the console only
    through the reads the hints prompt, each of which authorizes itself.
retention:
  class: transient
  rationale: >-
    Nothing is kept. The community forgets the stream when it closes. The console keeps only the latest resume token,
    and only so that it can resume.
errorCodes:
  - code: vtc/admin/events/subscribe:notAdministrator
    meaning: The proven caller holds no administrative role at this community.
    retryable: false
  - code: vtc/admin/events/subscribe:streamUnavailable
    meaning: >-
      The request did not arrive over a delivery that can carry a stream — over HTTPS, a request without
      `Accept: text/event-stream`, or a binding that defines no streamed response. The console falls back to polling.
    retryable: false
  - code: vtc/admin/events/subscribe:tooManyStreams
    meaning: >-
      The caller already holds as many open streams as the community allows. Close one (another tab or device), or
      poll.
    retryable: true
related:
  - vtc/admin/events/event
  - vtc/admin/actions/list
  - vtc/join-requests/list
  - vtc/members/list
  - vtc/config/export
  - vtc/directory/query
  - vtc/rooms/community/list
  - vtc/rooms/community/get
  - vtc/rooms/community/members/list
  - messaging/monitor/subscribe
---

## Abstract

An administrator's console shows things that change while it is open: actions waiting on the administrator, acknowledgements still owed, join requests to decide, the member list, the community's configuration, and whether the community is in single-administrator mode. Until now the console could only learn of a change by polling every read on a timer.

**VTC Admin Events — Subscribe** opens **one live channel per console session** over which the community sends **hints**: [`vtc/admin/events/event`](../../event/0.4/spec.md) documents that say *this topic changed* and, for the badge topics, *your count is now n*. A hint carries no record, no record identifier, and no personal data. The console reacts to a hint by re-fetching that topic through the read it already uses — [`vtc/admin/actions/list`](../../../actions/list/0.2/spec.md), [`vtc/join-requests/list`](../../../../join-requests/list/0.1/spec.md), and so on — and each of those reads is signed and authorized on its own. Authorization therefore stays on every read. The stream can neither leak a record nor grant a read: anyone who could read the stream learns only *when* to ask, and must still be entitled to the answer.

Over HTTPS the channel is a Server-Sent Events stream, defined by the streamed response of [HTTPS binding 0.3 §2.1](../../../../../../bindings/https/0.3/spec.md). Polling remains the fallback. Whenever the stream is down, unsupported or refused, the console polls exactly as it did before, and it shows the administrator which of the two it is doing.

### What 0.4 added

One topic, for the console's community-rooms pages:

- **`communityRooms`** — the data rooms the community itself **owns**, as their owner sees them: a community room created, or its label, description or tags changed; an admission offered, withdrawn, accepted or renewed; a removal owed, retried, settled or failed, including one owed because a member left the community; whether the room's profile write is pending or written; and the community's own authority over the room being renewed, or failing to be. Read through [`vtc/rooms/community/list`](../../../../rooms/community/list/0.2/spec.md), [`vtc/rooms/community/get`](../../../../rooms/community/get/0.2/spec.md) and [`vtc/rooms/community/members/list`](../../../../rooms/community/members/list/0.1/spec.md), by holders of `vtc.rooms.manage`.

It is not a count topic. It is separate from `rooms` because the two are different authorities over different views. `rooms` is the community's view as a **host** — storage, limits and lifecycle over every room it stores, for holders of `vtc.rooms.admin` — and carries no room's details. `communityRooms` is its view as an **owner** of some of those rooms — their details and who is admitted — for holders of `vtc.rooms.manage`. A community may grant either without the other, so folding the owner's view into `rooms` would either withhold its hints from a `vtc.rooms.manage` holder who lacks `vtc.rooms.admin`, or send them to a `vtc.rooms.admin` holder who cannot read them. A `communityRooms` hint names no room, member, DID, offer or credential, in any member including `ext`: which room changed, and whose admission, is the owner's data, and reaches the console only through the reads. A 0.1, 0.2 or 0.3 subscription is unchanged: the topic set is closed per version, so a console asks for `communityRooms` only at 0.4 or later, and a community serving several versions answers each at its own.

### What 0.3 added

Two topics, for the console's member-directory and groups pages:

- **`directory`** — the community's member directory as administrators see it: a field defined, changed or retired; an entry's card or an administrator-sourced value changed; an entry added or removed; an administrators' note appended or redacted. Read through the `vtc/directory/*` reads, by holders of `vtc.directory.admin` or `vtc.directory.notes`.
- **`groups`** — the community's groups: a group created, changed or deleted; a member added or removed; a binding added or removed; a join request opened or decided. Read through the `vtc/groups/*` reads, by holders of `vtc.groups.manage`.

Neither is a count topic. The hints-only rule is unchanged and matters more here than anywhere: a directory or groups hint names no entry, no field, no group, no note and no member, in any member including `ext`. A hint that said *which* entry changed would tell an administrator who cannot read that entry that it exists. A 0.1 or 0.2 subscription is unchanged: the topic set is closed per version, so a console asks for `directory` or `groups` only at 0.3, and a community serving several versions answers each at its own.

### What 0.2 added

The `rooms` topic, for the console's data-rooms pages: rooms created or removed, a room's lifecycle state changing, a room or a storage config crossing 80, 95 or 100 percent of a limit or of capacity, and a storage config's state changing. A 0.1 subscription is unchanged: the topic set is closed per version, so a console asks for `rooms` only at 0.2, and a community serving both answers each at its own version.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.4

- Request `since` → **`resumeToken`**. Same type (the shared [`ResumeToken`](../../_shared/0.4/topic.schema.json), unchanged), same optionality, same meaning: the latest `resumeToken` the console received on an earlier stream. An unknown, expired or foreign token is still not an error and still yields `resumed: false`.

The reason is the registry-wide convention that `since` only ever names an inclusive RFC 3339 lower-bound time filter; a stream resume token is named `resumeToken`. The request now uses the same name as the response member and the event member it echoes. The topic set, the response (`topics`, `heartbeatSeconds`, `resumed`, `resumeToken`) and the events are unchanged.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.4` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The shape of the exchange

This task's success response opens a stream. The community sends the `#response` document first, and then sends `vtc/admin/events/event/0.4` documents on the same delivery until the stream ends. The `#response` closes the subscribe exchange as an ordinary success ([SPEC §4.4.1](/SPEC.md#441-request-and-response-variants)). Each event that follows is a separate fire-and-forget *Trust Task document* from the community to the console, not a further reply to the subscribe. A failure is a `trust-task-error` and opens no stream.

A delivery that cannot carry a stream cannot carry this task. Over HTTPS the stream is the streamed response of [binding 0.3 §2.1](../../../../../../bindings/https/0.3/spec.md). No other published binding defines one today, and a community answers a subscribe arriving any other way with `streamUnavailable`.

### Producer (the console)

A conforming **producer** **MUST**:

1. Emit a document whose `type` is `https://trusttasks.org/spec/vtc/admin/events/subscribe/0.5`, with itself as `issuer`, the community as `recipient`, an `issuedAt`, and a `proof` made with the console's key.
2. Over HTTPS, send it with `Accept: text/event-stream` as [binding 0.3 §2.1](../../../../../../bindings/https/0.3/spec.md) requires.
3. Sign a **fresh** document, with a new `id`, `issuedAt` and `proof`, every time it opens or re-opens the stream. It **MUST NOT** re-send a document it has already used. To resume, it puts the latest `resumeToken` it holds in the request's `resumeToken`.
4. Hold at most one open stream per console session. All the topics it renders go in one subscription.
5. Treat the response's `topics` as the whole of what it will hear about. For a requested topic the response omits, it **MUST NOT** show live status. It either polls that topic, if its read is still answered, or does not show it.
6. When `resumed` is `false`, re-fetch every effective topic before relying on hints, because changes may have been missed.
7. React to an event only by re-fetching that topic's read. It **MUST NOT** treat an event as data, as authority, or as evidence that a record exists. It **MAY** show an event's `count` on a badge until the re-fetch returns, and **MUST** prefer the read's own answer once it does.

### Live or offline: no latched status

A console **MUST** show whether it is live or offline, and the indicator **MUST** be able to go false again. It **MUST NOT** derive "live" from having once received the `#response`.

8. The console shows **live** only while bytes are arriving on the stream — an event or a heartbeat — at intervals no longer than twice `heartbeatSeconds`.
9. When the stream ends, fails, or stays silent for twice `heartbeatSeconds`, the console **MUST** show **offline** at once, **MUST** close the connection if it is still open, and **MUST** fall back to polling every topic it shows. It **SHOULD** then re-subscribe with exponential backoff and jitter, taking any `retry` hint the binding carries as the floor.
10. On a refusal (`streamUnavailable`, `notAdministrator`, `permissionDenied`, `unsupportedType`), the console stays offline and polls. It **MUST NOT** retry `streamUnavailable` or `unsupportedType` on the same transport during the session. It retries `tooManyStreams` with backoff.

### Consumer (the community)

A conforming **consumer** **MUST**:

1. Take the caller from the verified `proof`, and refuse a caller holding no administrative role with `notAdministrator`. Over HTTPS it **SHOULD** also require the bearer-authenticated sender to be the proven `issuer` ([SPEC §4.8.1](/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)).
2. Refuse a document whose `issuedAt` falls outside an acceptance window of no more than **five minutes**. A subscribe is signed at the moment of connecting, so a wider window serves only a replay.
3. Refuse with `streamUnavailable` a subscribe that arrived over a delivery unable to carry the stream. It **MUST NOT** answer a `#response` it cannot follow with events.
4. Compute the **effective topics**: the requested topics the caller may read now (see [Authorization](#authorization)). If there are none, refuse with `permissionDenied`. Otherwise answer with them in `topics`.
5. Resume when it can. If the request's `resumeToken` is a token it minted for this caller, still within its retention, it answers `resumed: true`. It then sends, straight after the `#response`, one hint for every effective topic that changed after that position. Otherwise it answers `resumed: false`. An unknown, expired or foreign `resumeToken` is **never** an error, so it cannot serve as an oracle for tokens.
6. Send a hint for an effective topic whenever the state behind that topic's read changes for the caller, as [`vtc/admin/events/event`](../../event/0.4/spec.md) defines. It **MAY** coalesce several changes to one topic into one hint, and **SHOULD** send no more than one hint per topic per second.
7. Never send a hint for a topic the caller cannot read at the moment of sending. When the caller's readable topics shrink, or the caller stops being an administrator, it **MUST** end the stream rather than silently stop sending one topic. When they grow, it **MAY** end the stream. In either case the console's re-subscribe reports the new set.
8. Send something — an event, or the binding's heartbeat — at least every `heartbeatSeconds`, and choose `heartbeatSeconds` short enough to survive the idle timeouts of the intermediaries it knows about. **25** is a good default.
9. End the stream no later than the expiry or revocation of the authentication it was opened on, and **SHOULD** bound any one stream's life, for example to an hour. Each end costs the console a freshly signed subscribe, which re-checks authorization.
10. **MAY** end the stream at any time and for any reason. Ending it is not an error and carries no reason ([binding 0.3 §2.1](../../../../../../bindings/https/0.3/spec.md)). The console falls back to polling and re-subscribes.
11. Refuse with `tooManyStreams` a subscribe that would exceed its per-caller limit on open streams. The limit **SHOULD** allow at least a few concurrent consoles, such as several tabs or devices.

### Why hints, not records

A stream that carried the changed records would need its own authorization model: per-record, re-evaluated on every send, and consistent with each read's model as both evolve. In practice it would diverge from the reads, and the gap would be a leak. A hint carries nothing that needs authorizing beyond the topic, and the topic is checked against the very read it prompts. The cost is one round trip per hint. The console makes it anyway to render the change, and coalescing bounds it.

## Authorization

The authority is **an administrative role at the community, held by the proven caller**, and, per topic, **the standing to perform that topic's read**:

| Topic | Read it prompts | Who receives its hints |
|---|---|---|
| `actions` | [`vtc/admin/actions/list`](../../../actions/list/0.2/spec.md) | Every administrator. Each sees only the actions that read would show them. |
| `acknowledgements` | [`vtc/admin/actions/list`](../../../actions/list/0.2/spec.md) (acknowledge actions) | Every administrator. A hint is sent only for acknowledge actions the caller would see. |
| `joinRequests` | [`vtc/join-requests/list`](../../../../join-requests/list/0.1/spec.md) | Callers the community would answer on that read. |
| `members` | [`vtc/members/list`](../../../../members/list/0.1/spec.md) | Callers the community would answer on that read. |
| `singleAdminMode` | the community's administrative status | Every administrator. A community in single-administrator mode shows that to every administrator, so knowing when it changes is open to all of them. |
| `config` | [`vtc/config/export`](../../../../config/export/0.1/spec.md) | Callers the community would answer on that read. |
| `rooms` | [`vtc/rooms/list`](../../../../rooms/list/0.1/spec.md), [`vtc/rooms/usage`](../../../../rooms/usage/0.1/spec.md), [`vtc/storage/configs/list`](../../../../storage/configs/list/0.1/spec.md) | Holders of `vtc.rooms.admin`, who are answered on all three. |
| `directory` | the `vtc/directory/*` reads: the field schema, entries, and administrators' notes | Holders of `vtc.directory.admin` or of `vtc.directory.notes`. A holder of only one is still sent every `directory` hint, because a hint names nothing and the read it prompts shows each holder only what that capability lets it read. |
| `groups` | the `vtc/groups/*` reads: groups, their members, bindings and join requests | Holders of `vtc.groups.manage`, including one qualified to a single group (`group:<id>`): such a holder is sent a hint only for a change to a group it may manage. |
| `communityRooms` | [`vtc/rooms/community/list`](../../../../rooms/community/list/0.2/spec.md), [`vtc/rooms/community/get`](../../../../rooms/community/get/0.2/spec.md), [`vtc/rooms/community/members/list`](../../../../rooms/community/members/list/0.1/spec.md) | Holders of `vtc.rooms.manage`, who are answered on all three. Holding `vtc.rooms.admin` alone does not qualify: that is the host's authority, and its reads are the `rooms` topic's. |

Entitlement is evaluated against the community's own records: when the stream opens, and again for every hint sent. A hint for a topic the caller cannot read is **never** sent. Within a topic, a change the caller's read would not show them produces no hint: a new action they cannot see does not move their `actions` topic. A hint is not the authorization for anything. The read it prompts decides afresh, on its own proof.

The `proof` on the subscribe establishes *who* is asking. Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements) it is not the entitlement. It lets the community look the caller up in its administrative records, where the entitlement lives. Nothing about subscribing presupposes consent, approval or a step-up. Whether a community demands more before it streams is its own policy.

## Definitions

- **Hint** — a [`vtc/admin/events/event`](../../event/0.4/spec.md) document: a topic, a time, an optional count, and a resume token. It carries no record.
- **Topic** — one of the closed set in the shared [`Topic`](../../_shared/0.4/topic.schema.json). Each topic is bound to one read.
- **Count topic** — `actions`, `acknowledgements` or `joinRequests`, whose hints carry the caller's badge count.
- **Effective topics** — the requested topics the caller may read, as reported in the response.
- **`topics`** (REQUIRED) — one to ten distinct topics the console renders.
- **`resumeToken`** (request, OPTIONAL) — the latest [`ResumeToken`](../../_shared/0.4/topic.schema.json) the console holds, to resume from.
- **`heartbeatSeconds`** (response) — 5–60. The longest the community lets the stream go silent.
- **`resumed`** (response) — whether the request's `resumeToken` was honoured. When `false`, the console re-reads every effective topic.
- **`resumeToken`** (response) — the stream position as the stream opens.

## Request

The console signs a subscribe for every topic it renders and sends it to the community. Over HTTPS it sends it with `Accept: text/event-stream`. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A console opens its channel, resuming from an earlier stream

```json
{
  "id": "urn:uuid:195bd22b-70cc-4f96-88d4-73040fff978b",
  "type": "https://trusttasks.org/spec/vtc/admin/events/subscribe/0.5",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-03T09:15:00Z",
  "payload": {
    "topics": ["actions", "acknowledgements", "joinRequests", "members", "singleAdminMode", "config"],
    "resumeToken": "evt.000141"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-03T09:15:00Z",
    "verificationMethod": "did:web:bob.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "zghVe8AMy6UYa9pvgx9rvbqJ28MhYuHwcfTzAow1n9pSBCQXmr8LfS5AQWQ8bk4AzTHsqCLX1zi7B8JxNjW8wD9d"
  }
}
```

## Response

The community answers with the subscription as granted, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. It is the first document on the stream, and hints follow it. Failures use `trust-task-error`, not a `#response` document, and open no stream.

### Granted, narrowed and resumed

Bob is an administrator without access to the community's configuration export, so `config` is not among his effective topics. His console polls nothing for it, because Bob could not read it either way. The community recognised the request's `resumeToken` and follows the response with a hint for the one topic that changed while Bob was away.

```json
{
  "id": "urn:uuid:76152941-349c-4382-954b-ddeaa3b3b282",
  "type": "https://trusttasks.org/spec/vtc/admin/events/subscribe/0.5#response",
  "threadId": "urn:uuid:195bd22b-70cc-4f96-88d4-73040fff978b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-03T09:15:01Z",
  "payload": {
    "topics": ["actions", "acknowledgements", "joinRequests", "members", "singleAdminMode"],
    "heartbeatSeconds": 25,
    "resumed": true,
    "resumeToken": "evt.000141"
  }
}
```

On the HTTPS binding the opening of that stream reads as follows (non-normative; each `data:` line is one complete document, abbreviated here):

```text
HTTP/1.1 200 OK
Content-Type: text/event-stream
Cache-Control: no-store

id: evt.000141
data: {"id":"urn:uuid:76152941-…","type":"https://trusttasks.org/spec/vtc/admin/events/subscribe/0.5#response",…}

id: evt.000142
data: {"id":"urn:uuid:8b5f6424-…","type":"https://trusttasks.org/spec/vtc/admin/events/event/0.4",…,"payload":{"topic":"actions","at":"2026-10-03T09:12:40Z","count":2,"resumeToken":"evt.000142"}}

: heartbeat

```

### Refused: the delivery cannot stream

The console sent the subscribe without `Accept: text/event-stream`, so the community has no stream on which to follow a `#response`. The console polls.

```json
{
  "id": "urn:uuid:070239b1-133f-4a17-a6a7-8485dc5920aa",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:529a5d8b-11aa-4d0b-804f-5da7a621057b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-03T09:20:01Z",
  "payload": {
    "code": "vtc/admin/events/subscribe:streamUnavailable",
    "message": "This request did not ask for an event stream.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a list of topic names and an opaque resume token, and nothing personal. The stream carries, per hint, a topic, a timestamp and, for the count topics, one integer. A producer **MUST NOT** place record data, record identifiers, DIDs or other personal data in a hint or in `ext`. That rule is the design: the moment a hint says *which* join request arrived, the stream needs the per-record authorization the reads already have, and the two will diverge. The smallest payload that answers the task is "re-read this topic", and the counts are the one deliberate addition, because a badge is the thing a console renders most and a count spares a read on every tick.

### Correlation

The stream discloses activity **volume and timing**. Counts say how many actions await this administrator and how many join requests await a decision. Hint timing says when an administrator acted, when an applicant applied, and when the configuration changed. This is justified because every recipient is an administrator already entitled to the read that reveals the same number and more. A hint reveals only what polling that read would reveal, earlier. It is not nothing: the count on `actions` reflects other administrators' activity, since an action appears when a colleague requests it and leaves when a colleague decides it. A community with reason to hide that rhythm can coalesce more aggressively. An observer of the connection who cannot read the TLS content still sees hint timing and size, which roughly tracks administrative activity. Heartbeats at a fixed interval reveal only that a console is open. The resume token is per caller and opaque, and **SHOULD** be unlinkable across callers. The `directory` and `groups` topics add timing about members, not only about administration: a `directory` hint can follow a member editing their own card, and one shortly after an administrator opens a member's page can follow a note being written about that member. A recipient is entitled to the reads that show the same changes, so this is again earlier, not additional; but because a hint names no entry, group or member, an administrator who could not read the change learns nothing about whom it concerned, and a community **SHOULD** coalesce `directory` hints so that one member's edits are not legible as a rhythm. The `communityRooms` topic adds timing about members too: a hint can follow a member accepting an admission, or leaving the community and so being owed a removal from the community's rooms. Its recipients hold `vtc.rooms.manage` and are entitled to the reads that show those changes and whom they concern, so this is again earlier, not additional; the hint itself names no room and no member.

### Retention

Nothing is retained by design. The community holds the stream only while it is open, plus whatever bounded history it keeps to honour a request's `resumeToken`, which **SHOULD** be minutes, not days. It **MAY** log the opening of a stream, as it would any signed read. The console holds only the latest resume token. Hints are transient display triggers, and a console that persisted them would be building an activity log of the community's administration, which is no part of this task's purpose.

### Consent/purpose

The purpose is to keep an administrator's open console current without polling, so that decisions they owe are seen promptly and the screen does not show stale state. Using the stream to profile administrators' responsiveness, or other administrators' activity, is outside that purpose. This specification describes the channel. Whether a community offers it, to whom beyond the topic rules, and whether anything more is required before streaming is the community's policy.
