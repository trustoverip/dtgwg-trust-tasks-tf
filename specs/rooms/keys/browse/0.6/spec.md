---
slug: rooms/keys/browse
version: "0.6"
title: "Rooms Keys — Browse"
summary: "A member asks their own agent to list a room's records at its host, oldest or newest first, between two versions, a page at a time if asked, check it against what the host committed to, and say whether records were pruned; each row says when, and by whom, it was first stored."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - record
  - list
  - commitment
  - completeness
  - order
authors:
  - Glenn Gore (https://github.com/stormer78)
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
    The request makes the recipient act outward in its principal's name — presenting their room credentials to a party this request names. One whose origin depended on the transport would let a compromised channel aim a member's credentials at a host of somebody else's choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The response carries a verdict about a host's honesty, and a verdict that cannot be placed in time cannot be compared with a later one — which is the whole mechanism. An undated listing is also one a duplicate-suppression window cannot absorb.
sideEffects:
  level: none
  rationale: >-
    Returns metadata. The recipient makes an outbound request and MAY record what the host asserted, but the room is unchanged.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` is the point: the recipient presents its principal's membership and authority to the named host, as them. `discloses: metadata` rather than `secret` because a listing carries no bodies — which is a property of the task and not of the tier, and is why browsing and reading are two tasks.
retention:
  class: transient
  rationale: >-
    The listing is returned and not kept; the presentation minted for the fetch is single-purpose. What the recipient MAY retain is what the host asserted about the room — see `verification.priorRoots`.
errorCodes:
  - code: rooms/keys/browse:notAMember
    meaning: "The recipient holds no group state for this room, so it has nothing to present."
    retryable: false
  - code: rooms/keys/browse:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer."
    retryable: true
  - code: rooms/keys/browse:hostRefused
    meaning: "The host answered and declined. Its own code and reason are carried in `details` — commonly that it does not serve this room."
    retryable: false
related:
  - rooms/records/prune
  - rooms/keys/read
  - rooms/keys/present
  - rooms/records/list
---

## Abstract

A member asks their **key holder** to list a room's records at its host, and to check the listing against what the host committed to.

It is [`rooms/records/list`](../../../records/list/0.8/spec.md) made from the party that can both present and check. The presentation only the agent can mint; the checking needs the room's state and, for the part that matters most, a memory of what this host has said before — which a tab does not have and a CLI does not keep.

Bodies are never returned. That is a property of this task rather than of the tier: [`rooms/keys/read`](../../read/0.2/spec.md) is how a member opens something, and it is also the only shape a trace can be produced for. Browsing and reading are two tasks so that looking at a room's shelf does not decrypt the room.

Version 0.2 passes on the room's `lastPrune`. A prune removes records outright, tombstones
included, so a member's agent syncing a room incrementally through this task has no other way
to learn that records it holds are gone.

Version 0.5 lets a member's surface open a conversation at its newest messages. It passes
`order` through to the host, so a browse can read the room from its head down, and adds
`maxPages` and `cursor`, so the surface can take the newest page and come back for the next one
rather than have its agent read a room of tens of thousands of records to the end.

Version 0.6 passes `beforeVersion` through, so a surface showing a conversation from its head can
ask for the messages before the oldest one it shows by naming that message's version, without
keeping the host's cursor between requests.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

One optional response member, `lastPrune` ([`RecordPrune`](../../../_shared/0.2/room.schema.json)),
which a recipient fills from [`rooms/records/list`](../../../records/list/0.3/spec.md) `0.3`. The
response is a closed object, so a `0.1` caller would refuse it; a new member on a closed
payload is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A recipient
implementing `0.2` **SHOULD** continue to serve `0.1`; answering a `0.1` request carrying
`sinceVersion` for a room it knows has been pruned past it, it **SHOULD** ignore `sinceVersion`
and list the whole room, so that a `0.1` caller is at least handed the full set — though it
cannot be told to drop what is missing from it.

## Changes from 0.5

One optional request member, **`beforeVersion`**, passed through to the host's
[`rooms/records/list/0.8`](../../../records/list/0.8/spec.md): an exclusive upper bound on
`version`, the counterpart of `sinceVersion`. A surface paging a conversation backwards sends
`order: "newestFirst"` and the lowest version it holds, and gets the page before it whether or not
it kept the previous browse's `nextCursor` — which it cannot once the app restarts, and which the
host may have expired. Nothing else changes, and a request without `beforeVersion` is answered
exactly as `0.5` answers it. `0.6` targets framework `0.6.0` (`0.5` targeted `0.5.0`), and its
`ext` members reference the framework `0.4` schema's `Ext`, as the other current rooms
specifications do; neither changes what a conforming document carries.

The request is a closed object, so a `0.5` recipient would refuse a request carrying
`beforeVersion`; a new member on a closed payload is a `MINOR` increment per
[SPEC §5.2](/SPEC.md#52-compatibility-rules). `0.6` does not declare `extends: "0.5"`: a `0.5`
recipient that pruned the bound away would answer a "load older" browse with the room's head. A
recipient implementing `0.6` **SHOULD** continue to serve `0.5`.

## Changes from 0.4

Three optional request members and one optional response member:

- **`order`** ([`ListOrder`](../../../_shared/0.2/room.schema.json)), passed through to the host's
  [`rooms/records/list/0.7`](../../../records/list/0.7/spec.md): `oldestFirst`, the default, or
  `newestFirst`.
- **`maxPages`**, the most host pages the recipient reads before it stops.
- **`cursor`**, a `nextCursor` from an earlier browse, passed through to the host so the recipient
  continues where that browse stopped.
- **`nextCursor`** (response), the host's continuation, present exactly when `complete` is `false`.

`order` alone would not have been enough. A `0.4` recipient reads the host's listing to its end, so
a newest-first browse of a large room with no `sinceVersion` would still read the whole room before
answering, and a surface opening a conversation would wait for every record it was trying not to
read. `maxPages` is how the caller asks it to stop, and `cursor` and `nextCursor` how the caller
comes back for the next page; until now a recipient could stop only of its own accord, and the
caller could not continue from where it did.

The request and response are closed objects, so a `0.4` recipient would refuse a request carrying
any of these, and a `0.4` caller a response carrying `nextCursor`; a new member on a closed payload
is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). `0.5` does not declare
`extends: "0.4"`: a `0.4` recipient that pruned `order` and `maxPages` away would read the whole
room oldest first, and a caller taking its first rows for the room's head would show its oldest
messages as its newest. A recipient implementing `0.5` **SHOULD** continue to serve `0.4`.

## Changes from 0.3

The rows are [`RecordMetadataV3`](../../../_shared/0.2/room.schema.json), which
[`rooms/records/list/0.6`](../../../records/list/0.6/spec.md) returns, rather than
`RecordMetadata`: each row may carry `createdAt` and `bodyStoredAt` (every tier), `createdBy`
(`open` and `attributed`) and `kind` (`open` only), as well as `blobs`, which `RecordMetadata` never carried
here. A member whose surface reaches the room only through their key holder needs them for the
same reason a direct reader does — to judge a record's `writtenAt`, to verify a version's proof
against the author's DID document as it stood when its body was stored, to tell a rewrite by another
member from the author's own, and on an open room to tell a message from a reaction without
opening it (`SealedRecordBody`).

A recipient serving `0.4` **MUST** list with `rooms/records/list/0.6` where the host serves it,
and **MUST** pass every row through exactly as the host served it, adding and removing nothing;
where the host serves only an earlier version, it lists with that and the rows carry what that
version carries. The response is a closed object whose rows are a differently named generated
type, so a `0.3` caller would refuse it and `0.4` does not declare `extends: "0.3"`; it is a
`MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A recipient implementing
`0.4` **SHOULD** continue to serve `0.3`, passing the rows through without the members
`RecordMetadata` does not have.

## Changes from 0.2 to 0.3

* The shared `RecordPrune` member (response `lastPrune`) **`at`** → **`prunedAt`**. Same type (`string`, `date-time`, required) and meaning.

The rename brings this member into line with the registry's naming convention — an instant that records an event is named `<event>At` — so the same concept has one spelling across the registry. All of the schema's `$ref`s into `rooms/_shared/0.1/room.schema.json` move to `rooms/_shared/0.2/room.schema.json`, which differs from 0.1 only by this rename; nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.2` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

**The entitlement is being the recipient's own principal.** A key holder holds one member's room keys; this asks it to use their credentials to look.

A conforming consumer **MUST** accept this only from the party whose keys it holds, **MUST** refuse it for a room it holds no group state for, and **MUST** present a `read` grant scoped to this room and granted to its principal, in `rooms/records/list` documents issued and signed by the principal — never one conferring more. As in [`rooms/keys/read`](../../read/0.2/spec.md), the recipient acts as its principal and its own identifier **MUST NOT** reach the host.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

Everything [`rooms/keys/read`](../../read/0.2/spec.md) says about a caller-named `host` applies unchanged: presenter binding means naming a host transfers no standing, what it transfers is **sight** of the presentation, and a recipient **SHOULD** bound the outbound request to transports the host's DID document advertises.

## The room's prunes

A recipient serving `0.2` **MUST** list with `rooms/records/list` `0.3` where the host serves
it, and **MUST** return `lastPrune` exactly as the host served it, omitting it when the host
did. A caller that keeps its own copy of the room **MUST** apply the reader rules of
[`rooms/records/list`](../../../records/list/0.8/spec.md) `0.3` (*After a prune*) to it: a
`lastPrune.version` above its watermark means rebuild from a browse with no `sinceVersion`,
and drop what that no longer holds.

## Order and paging

A recipient serving `0.6`:

1. **MUST** list with [`rooms/records/list/0.8`](../../../records/list/0.8/spec.md) where the host
   serves it, passing `order` and `beforeVersion` through exactly as the caller gave them, and
   omitting each where the caller did. Where the host serves only an earlier version it **MAY**
   list with that version for a request without `beforeVersion` — with `0.7` for one carrying
   `order`, and with an earlier version for one without `order` or with `order: "oldestFirst"`.
   For a request with `beforeVersion`, or with `order: "newestFirst"` to a host serving neither
   `0.8` nor `0.7`, it **MUST** answer `hostRefused`, carrying the host's refusal in `details`,
   rather than list without the bound or in the other order. A recipient that dropped the bound
   would hand a surface asking for older messages the room's head, and one that returned an
   oldest-first listing to a caller that asked for the newest would have its member's surface
   show the room's first messages as its latest.
2. **MUST** return the rows in the order the host served them, passing every row through as
   `0.4` requires, so that a `newestFirst` browse returns rows in descending `version`.
3. **MUST** stop after reading `maxPages` host pages, where the caller set it, and **MAY** stop
   earlier of its own accord as before.
4. **MUST**, when it stops with the host's `nextCursor` in hand, answer `complete: false` and
   return that cursor, unchanged, as `nextCursor`; and **MUST** omit `nextCursor` when it read the
   listing to its end. `complete` and `nextCursor` therefore always agree.
5. **MUST**, on a request carrying `cursor`, pass it to the host as its listing's `cursor` on the
   first page, with the request's own `prefix`, `sinceVersion`, `beforeVersion`, `order` and
   `limit`. It **MUST NOT**
   read, change or check the cursor itself: it is the host's, and the host refuses one it did not
   issue or issued under another order or other bounds (`rooms/records/list/0.8` *Order*), which
   the recipient
   answers as `hostRefused`.
6. **MUST** treat a listing begun from a `cursor` as not comparable against `recordCount`, however
   it ends: it did not start at the room's first record in its order, so its length is not the
   room's.

A caller that pages a conversation backwards sends `order: "newestFirst"` with `maxPages` — one is
enough for a surface's first screen — and on each later request the `nextCursor` it was given, with
the same `prefix`, `sinceVersion` and `order` — or, holding no cursor, the same request with
`beforeVersion` set to the lowest version it holds. What a newest-first listing holds, and how a
reader orders and counts the messages in it, is stated by `rooms/records/list/0.8` *Order*: a reader
**MUST** apply it to a browse as to a listing it made itself, and orders messages by `createdAt`,
never by the order of the rows.

## What the recipient checks

1. **The reply is signed by the host that was addressed** — verified, and the proven signer bound to the `host` named. An unsigned or misattributed reply is `hostRefused`, never a success with a caveat.
2. **The root against what it has seen before, at this `headVersion`** — `verification.priorRoots`.
3. **The length against the count the host committed to** — `verification.count`, and this is the check that only exists here.
4. **The room's own witnessed anchor**, where it has published one — `verification.anchor`. The only one of these a first-time reader can make, and the only place a **rollback** is visible: a host serving a state older than the room's own published statement is `behind`.

### Counting is the check a listing can actually do

A reader **cannot** recompute a room's root from a listing. A leaf commits to a whole record and a listing returns a projection without the body, so the arithmetic is unavailable however complete the listing is.

What is available is the count. A host that omits a record from a listing while committing to a tree that holds it **contradicts itself inside one exchange** — no second party, no anchor, no other member. That is the omission the commitment exists to make detectable, caught at the cheapest possible moment.

The condition is narrow and a consumer **MUST** respect it. The comparison is meaningful only for a listing with **no `prefix`**, **no `sinceVersion`**, **no `beforeVersion`**, **no `cursor`**, and `complete: true`. The order does not matter: a whole room read newest first holds the same records as one read oldest first. Anything else legitimately holds fewer records, and comparing it produces a discrepancy the reader manufactured. `notComparable` is the honest answer and will be the common one.

**`complete` is why a page bound is not a short listing.** A recipient that stopped at a `limit` and a host that withheld a record produce the same shorter array, and a consumer that could not tell them apart would either cry wolf on every paged listing or learn to ignore the one that mattered. So a recipient **MUST** set `complete: false` when it stopped for any reason of its own, and **MUST NOT** report `count: "short"` in that case.

### A verdict is never an error

As on [`rooms/keys/read`](../../read/0.2/spec.md), and for the same reason: **the consequence of catching a host belongs on the write path, not on the member's ability to look at their own room.** A consumer **MUST NOT** treat any value in `verification` as a failed listing, **MUST** surface an adverse one in words that name what was observed, and **MUST NOT** reduce it to a status colour — a member who dismisses an icon here is a member who later reads a refused write as their own agent malfunctioning.

## Definitions

**`roomId`** — the room to list. The recipient **MUST** already hold group state for it.

**`host`** — the host to list from, as a DID. Named by the caller because nothing maps a room to its host; see [`rooms/keys/read`](../../read/0.2/spec.md).

**`prefix`**, **`sinceVersion`**, **`beforeVersion`** — passed through to the host. Any one makes the listing non-comparable against the room's `recordCount`, which is stated here because a caller setting a prefix has usually not thought about the commitment at all.

**`order`** — passed through to the host: `oldestFirst` (the default) or `newestFirst`. See *Order and paging*.

**`limit`** — the **page size to ask the host for**, never a cap on the result. A recipient **MUST** follow the host's cursor to the end unless the caller's `maxPages` stopped it or it stopped of its own accord, in which case `complete` is `false`. A short page is not the end of a listing, and a producer that infers exhaustion from one has invented a shorter room.

**`maxPages`** — the most host pages to read for this request. The caller's way of asking the recipient to stop; see *Order and paging*.

**`cursor`** — a `nextCursor` from an earlier browse of the same room at the same host, with the same `prefix`, `sinceVersion`, `beforeVersion` and `order`. Opaque to the caller and to the recipient; only the host reads it.

**`complete`** (response) — whether the recipient reached the end. See above; it is what separates a page bound from a withheld record.

**`nextCursor`** (response) — the host's continuation, present exactly when `complete` is `false`, and absent at the end.

## Request

A **Member** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member looks at a room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community"
  }
}
```

### A member's surface opens a conversation at its newest messages

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "order": "newestFirst",
    "limit": 50,
    "maxPages": 1
  }
}
```

### The same surface, later, loads the messages before them

The surface was restarted and kept no cursor. It holds rows down to version `30411`, from the
newest page below, and asks for the page before it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000007",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#request",
  "issuer": "did:example:member",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:10:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fc",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "order": "newestFirst",
    "beforeVersion": 30411,
    "limit": 50,
    "maxPages": 1
  }
}
```

## Response

The **KeyHolder** responds, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### A complete listing that reconciles

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [
      {
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "version": 412,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2026-01-01T00:00:00Z"
      }
    ],
    "complete": true,
    "verification": {
      "priorRoots": "agree",
      "count": "agrees",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 1,
        "headVersion": 412
      }
    }
  }
}
```

### A host that served fewer records than it committed to

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [],
    "complete": true,
    "verification": {
      "priorRoots": "noneHeld",
      "count": "short",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 118,
        "headVersion": 412
      }
    }
  }
}
```

The listing is still returned, and `priorRoots: "noneHeld"` shows this was the first read — the host was caught **without** the agent having any history at all, which is the point of carrying a count.

### A room whose records have been pruned

The host removed fourteen records at version 413. A caller whose copy of the room was
synchronised below 413 rebuilds it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-10-02T00:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [
      {
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "version": 412,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2026-09-30T00:00:00Z"
      }
    ],
    "complete": true,
    "lastPrune": { "version": 413, "prunedAt": "2026-10-01T00:00:05Z", "removed": 14 },
    "verification": {
      "priorRoots": "noneHeld",
      "count": "agrees",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 1,
        "headVersion": 413
      }
    }
  }
}
```

`headVersion` is the prune's version, not the highest record's: a prune takes a version like
any other change to the tree (`HeadVersion`).

### The newest page of a conversation

The answer to the newest-first request above, shortened to two rows. The recipient stopped at
`maxPages`, so the listing is not complete, the count is not comparable, and `nextCursor` is how
the surface asks for the page before it.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/rooms/keys/browse/0.6#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "records": [
      {
        "key": "q4bZ0sYw9LkVnT2xHcRe1A",
        "version": 30412,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2026-01-01T00:00:00Z",
        "createdAt": "2026-01-01T00:00:00Z"
      },
      {
        "key": "giXFLTGBdnnQJRoIsktuIg",
        "version": 30411,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2025-12-31T23:59:41Z",
        "createdAt": "2025-12-30T09:12:05Z"
      }
    ],
    "complete": false,
    "nextCursor": "djE6ZGVzYzozMDQxMQ",
    "verification": {
      "priorRoots": "noneHeld",
      "count": "notComparable",
      "head": {
        "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "recordCount": 30120,
        "headVersion": 30412
      }
    }
  }
}
```

The second row was written two days earlier and edited a moment ago, which is why it sits beside
the newest message. A surface shows it where its `createdAt` places it, not second.

## Security & Privacy

### Data carried

The request carries identifiers and a query. What crosses the network *because of it* is the recipient presenting its principal's membership and authority to the named host.

`order`, `maxPages` and `cursor` add nothing to what the host learns or serves: the host already
serves every row to the member's presentation, and these choose only the order and how much of it
one request reads. `nextCursor` is the host's own token, handed to the member it was issued for.

The response carries **metadata, never bodies**. On the sealed tiers the keys are opaque by construction, so a listing discloses how much the room holds and when it moved rather than what it says. A producer **MUST NOT** put record content in `ext`.

### Correlation

To the host, this is a member of that room surveying it — which is what the presentation already said. The event is *who has seen what this room holds*, and it is the event a host **SHOULD** log with a retention period of its own rather than one inherited.

A browse that stops after a newest-first page tells the host that this member looked only at the room's recent records — no more than the `sinceVersion` an incremental reader already sends. A `beforeVersion` tells it how far back the member has scrolled, which following `nextCursor` tells it equally.

Browsing is cheaper to repeat than reading and therefore easier to turn into a poll. A recipient that polls converts an access pattern into a timeline held by the host, so a consumer **SHOULD NOT** poll.

### Retention

The listing is returned and not kept; the presentation minted for the fetch is single-purpose.

What a recipient **MAY** retain, and what `verification.priorRoots` depends on, is `(headVersion, dataCommitment)` — a few dozen bytes per observation. It carries no record content, but it is a record of *when this member looked at this room*, so a recipient **SHOULD** keep it bounded and **SHOULD NOT** keep it beyond the member's membership.

### Consent/purpose

The listing is fetched so that a member's agent can show them the room its principal is a member of. It confers nothing the member did not already hold.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
