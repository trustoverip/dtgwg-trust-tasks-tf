---
slug: rooms/records/list
version: "0.8"
title: Rooms Records — List
summary: "A member lists record metadata in a data room — never bodies — oldest or newest first, between two versions, with prefix, watermark and cursor, including the blobs each record names, when each was first stored, any prune, and on an open room each record's kind."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - record
  - list
  - pagination
  - sync
  - watermark
  - order
  - load-older
authors:
  - Glenn Gore (https://github.com/stormer78)
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
  rationale: "A listing is authorized by a presentation whose integrity must not depend on the transport carrying it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A listing presentation is time-bounded; a replayed one would enumerate a room past the window its credentials describe."
sideEffects:
  level: none
  rationale: "Returns record metadata."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns keys, versions, epochs, timestamps and the BlobRefs of any files the records name; on an `open` room also titles, descriptions, authors and each record's kind. Never bodies."
retention:
  class: transient
  rationale: "The task returns existing state and stores nothing of its own."
errorCodes:
  - code: rooms/records/list:invalidCursor
    meaning: "The cursor was not issued by this host, has expired, or was issued under a different `order`, `sinceVersion` or `beforeVersion`."
    retryable: false
  - code: rooms/records/list:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
related:
  - rooms/records/prune
  - rooms/records/get
  - rooms/records/put
  - rooms/blobs/get
---

## Abstract

The **Rooms Records — List** Trust Task enumerates a room's records as **metadata**, never
bodies. A reader ranks what comes back and fetches the handful that matter with
[`rooms/records/get`](../../get/0.2/spec.md). A host that returned every body would make a
caller pay for the whole room on every listing — and on an encrypted room could not
usefully rank them anyway.

`sinceVersion` is what makes incremental sync converge, and **tombstones are returned like
any other record**. Without them a puller learns of every create and update and never of a
delete, so retracted records resurrect on the next full rebuild and disagree with peers
that saw the retraction.

Each row carries `blobs` when its record names a stored file
([`rooms/records/put/0.3`](../../put/0.3/spec.md)). A mirror syncing a room learns from a
listing which blobs each record keeps alive and fetches the ones it lacks with
[`rooms/blobs/get`](../../../blobs/get/0.2/spec.md); a tombstone carries none, which is how
a mirror learns to release them.

Version 0.3 adds `lastPrune`. A prune ([`rooms/records/prune`](../../prune/0.2/spec.md))
removes records outright — tombstones included — so there is nothing left in the room for a
`sinceVersion` reader to learn the removal from. `lastPrune` is what it learns from instead.

Version 0.6 adds each record's `kind` to an **`open`** room's rows. A reader keeping a room's
conversation — an unread count, a row preview, a page of messages — must otherwise open every
record to learn whether it is a message, a note or a reaction. On the sealed tiers that cost is
the price of the host not knowing; on an `open` room the host reads every body anyway, and
withholding `kind` there protects nothing.

Version 0.7 adds `order`. A chat client opens a conversation at its newest messages, and a room
read only oldest first makes it page through every record the room has ever held — tens of
thousands of them, on a busy room — before it can show the last one. `newestFirst` returns the
room's head first and pages backwards, with `sinceVersion` still bounding the listing from below,
so the same client can also catch up from the head down and stop at its watermark.

Version 0.8 adds `beforeVersion`, an exclusive upper bound that mirrors `sinceVersion`. A
newest-first reader showing a conversation's last screen asks for the messages before it by
passing the lowest version it holds — no cursor to keep, nothing that expires, and the same
request after the app restarts. `sinceVersion` still bounds the listing from below, so the two
together select one band of versions, in either order.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

### What changed from 0.7

One optional request member, **`beforeVersion`**: an exclusive upper bound on `version`, the
counterpart of `sinceVersion` (*Order*, below). Under `newestFirst` it is how a reader asks for
"the records older than the oldest one I hold" without holding a cursor: a cursor is the host's,
short-lived, and lost with the client's state, while a version is the reader's own and stays
meaningful for as long as the room's records are. It applies under `oldestFirst` too, which lets
a reader fetch exactly the band between two versions it already knows. Nothing else moves: the
rows, the response, `order` and its two values are those of `0.7`, and a request without
`beforeVersion` is answered exactly as `0.7` answers it. A cursor now belongs to the bounds as
well as the order it was issued under, and a host refuses it under others with `invalidCursor`.

The request is a closed object, so a `0.7` host would refuse a request carrying `beforeVersion`;
a new member on a closed payload is a `MINOR` increment per
[SPEC §5.2](/SPEC.md#52-compatibility-rules). `0.8` does not declare `extends: "0.7"`, for the
reason `0.7` did not extend `0.6`: a `0.7` host that pruned the bound away would answer a "load
older" request with the room's head, and a reader appending that page below the messages it
already shows would show the newest messages twice and the older ones never. A host serving
`0.8` **SHOULD** continue to serve `0.7` and earlier. A reader whose host answers
`unsupportedType` for `0.8` pages backwards with `0.7` and the `nextCursor` it was given.

[`rooms/keys/browse/0.6`](../../../keys/browse/0.6/spec.md) passes `beforeVersion` through for a
member whose surface reaches the room only through their key holder.

### What changed from 0.6

One optional request member, **`order`** ([`ListOrder`](../../../_shared/0.2/room.schema.json)):
`oldestFirst`, the default and what every earlier version does, or `newestFirst`, which returns
rows in descending `version` (*Order*, below). Nothing else moves: the rows are still
`RecordMetadataV3`, the response is unchanged, and a request without `order` is answered exactly
as `0.6` answers it. A cursor now belongs to the order it was issued under, and a host refuses it
under the other with the same `invalidCursor` it uses for a cursor it did not issue.

The request is a closed object, so a `0.6` host would refuse a request carrying `order`; a new
member on a closed payload is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules).
`0.7` does not declare `extends: "0.6"`: a `0.6` host that pruned the member away would answer a
`newestFirst` request oldest first, and a reader taking the first page for the room's head would
show its oldest messages as its newest. A host serving `0.7` **SHOULD** continue to serve `0.6`
and earlier. A reader that sends `newestFirst` to a host that answers `unsupportedType` for `0.7`
lists with `0.6`, oldest first, and reads to the end.

[`rooms/keys/browse/0.5`](../../../keys/browse/0.5/spec.md) passes `order` through for a member
whose surface reaches the room only through their key holder.

### What changed from 0.5

Four optional members on each row ([`RecordMetadataV3`](../../../_shared/0.2/room.schema.json)):

- **`kind`**, the record's [`RecordKind`](../../../_shared/0.2/room.schema.json) as its current
  version's `cleartext` carries it ([`rooms/records/put/0.5`](../../put/0.5/spec.md)), served
  **on an `open` room only**. On `attributed` and `private` rooms `kind` is sealed with the
  body, the host never sees it, and nothing about it is added here: a reader there opens a
  record to learn its kind, several at a time through its key holder
  ([`rooms/keys/read/0.2`](../../../keys/read/0.2/spec.md)).
- **`createdAt`**, on every tier: when the host stored the record's first version. `updatedAt`
  moves with every edit and every curation, so it cannot say when the room received a record,
  and a reader measuring an author's `writtenAt` against it would read every message edited or
  pinned a day later as backdated. `createdAt` does not move, so a reader can hold `writtenAt`
  to it (`SealedRecordBody`, *When it was written*). It is host metadata, not body, so the
  sealed tiers carry it too.
- **`createdBy`**, on `open` and `attributed` rooms: the author of the record's first version,
  as the host recorded it. A reader seeing a record for the first time has no earlier version
  to compare a rewrite with; `createdBy` lets it apply the rules about a rewrite by another
  member anyway (`SealedRecordBody`, *The record's original author*). A `private` room's host
  records no author, and serves none.
- **`bodyStoredAt`**, on every tier: when the host stored the body the record's current version
  carries — the time of the `rooms/records/put` that wrote it, which a curation leaves alone. A
  reader verifies the author's proof against the DID document as it stood then
  (`SealedRecordBody`, *Which DID document*), so a key the author rotated away before it cannot
  sign a version that verifies. The host knows which task wrote each version, so the sealed
  tiers, `private` included, carry it too, and it says nothing about the body.

`RecordMetadataV2` is `additionalProperties: false`, so a `0.5` reader would refuse a row
carrying any of them; `0.6` returns `RecordMetadataV3`, `RecordMetadataV2` plus these four. It
does not declare `extends: "0.5"`: the rows are a differently named generated type, which the
declaration does not admit, although every `0.5` member keeps its meaning. A new member on a
closed payload is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A host
serving `0.6` **SHOULD** continue to serve `0.5` and earlier, omitting all four.

The other ways a member reads a record carry the same metadata from these versions on:
[`rooms/records/get/0.3`](../../get/0.3/spec.md) (one record, from the host),
[`rooms/keys/browse/0.4`](../../../keys/browse/0.4/spec.md) (this listing, through a member's key
holder) and [`rooms/keys/read/0.2`](../../../keys/read/0.2/spec.md) (one record or several, opened
by the key holder). A reader that learns a record only through an earlier version of one of them
has no `createdAt`, `createdBy` or `bodyStoredAt`, and falls back as `SealedRecordBody` says.

### What changed from 0.2 to 0.3

One optional response member, `lastPrune` ([`RecordPrune`](../../../_shared/0.2/room.schema.json)),
and the reader rule that goes with it (*After a prune*, below). The response is a closed
object, so a `0.2` reader would refuse a page carrying it — and serving `0.2` a listing
**without** it, after a prune, is exactly the case that cannot be fixed at `0.2`: such a reader
keeps every pruned record it synchronised before the prune, and one retracted while it was
away comes back as current, because the tombstone that would have told it is gone. A host
serving `0.3` **SHOULD** continue to serve `0.2` and `0.1`, and a reader that synchronises
incrementally **SHOULD** move to `0.3`; a `0.2` reader of a pruned room is correct only after a
full rebuild. A new member on a closed payload is a `MINOR` increment per
[SPEC §5.2](/SPEC.md#52-compatibility-rules).

### Why 0.2 was a new version

`RecordMetadata` is `additionalProperties: false`, so a `0.1` reader
would reject a row carrying `blobs`. `0.2` returns `RecordMetadataV2` — `RecordMetadata`
plus `blobs` — targets framework `0.6.0`, and states the authorization refusal as the
standard `permissionDenied`. A host serving `0.2` **SHOULD** keep serving `0.1` to readers
that ask for it, omitting `blobs`.

## Changes from 0.4

* The shared `RecordPrune` member (response `lastPrune`) **`at`** → **`prunedAt`**. Same type (`string`, `date-time`, required) and meaning.

The rename brings this member into line with the registry's naming convention — an instant that records an event is named `<event>At` — so the same concept has one spelling across the registry. All of the schema's `$ref`s into `rooms/_shared/0.1/room.schema.json` move to `rooms/_shared/0.2/room.schema.json`, which differs from 0.1 only by this rename; nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.4` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **consumer** (the host) **MUST** apply the chain rules of
[`rooms/records/put`](../../put/0.3/spec.md); **MUST** return retracted records to a caller
supplying `sinceVersion`, so that a deletion propagates; **MUST NOT** accept a cursor it did
not issue; and **MUST NOT** return record bodies from this task.

A host **MUST** return `blobs` on each row exactly as the stored version of that record names
it, and **MUST** omit it on a row whose record names none, on a tombstone, and on a record
written by `rooms/records/put/0.1`. A presentation that does not confer `read` at this room's
scope, whose chain does not reach the room, or whose leaf is not the party the host
authenticated is refused with the standard `permissionDenied`
([SPEC §8.3](/SPEC.md#83-standard-error-codes)); this version declares no task-specific code
for it.

On an `open` room a host **MUST** return `kind` on each row exactly as the stored version's
`cleartext.kind` carries it, and **MUST** omit it on a row whose record carries none and on a
tombstone, which keeps nothing of its body. It **MUST NOT** derive `kind` from anything else, or
default it: absent is how a reader learns that the record named none, and reads it as a note.
On an `attributed` or `private` room a host **MUST NOT** return `kind`. A reader **MUST** treat
`kind` on a row as it treats every other member of an open room's record — the writer's claim
as the host stores it — and applies the same rules to it as to the record's own
(`SealedRecordBody`, *Kind*).

A host **MUST** return `createdAt` on each row, on every tier, as the instant it stored the
record's first version — unchanged by every later version and every curation, retraction
included ([`rooms/records/put/0.5`](../../put/0.5/spec.md) Host item 16). For a record stored
before it kept that time it **MUST** omit `createdAt` rather than supply a guess, and a reader
then judges `writtenAt` against `updatedAt` as `SealedRecordBody` says.

On `open` and `attributed` rooms a host **MUST** return `createdBy` on each row as the author it
recorded for the record's first version, kept like `createdAt` (Host item 17 of
[`rooms/records/put/0.5`](../../put/0.5/spec.md)), and **MUST** omit it for a record stored before
it kept that author. On a `private` room it **MUST NOT** return `createdBy`.

A host **MUST** return `bodyStoredAt` on each row, on every tier, as the time it stored the body
the record's current version carries (Host item 16 of
[`rooms/records/put/0.5`](../../put/0.5/spec.md)), and **MUST** omit it for a version stored
before it kept that time.

### Order

`order` chooses which end of the room a listing starts from. Absent, it is `oldestFirst`.

A host:

1. **MUST**, under `oldestFirst`, return rows in ascending `version` — as every earlier version
   of this task does.
2. **MUST**, under `newestFirst`, return rows in descending `version`, and continue in descending
   `version` on each page its `nextCursor` leads to, so that a reader paging to the end has read
   the room from its head down.
3. **MUST** apply `sinceVersion` under either order as a lower bound — only rows whose `version`
   is greater than it — and `prefix` and `limit` exactly as it does under `oldestFirst`. A
   `newestFirst` listing with `sinceVersion` therefore ends at the reader's watermark rather than
   at the room's first record.
4. **MUST** apply `beforeVersion` under either order as an exclusive upper bound — only rows
   whose `version` is less than it — on every page of the listing, together with `sinceVersion`
   where both are present. A `newestFirst` listing with `beforeVersion` therefore starts at the
   highest version below the bound rather than at the room's head. A request whose
   `beforeVersion` is not greater than `sinceVersion + 1` selects no version, and the host
   answers it with an empty page, not an error.
5. **MUST** return tombstones under `newestFirst` exactly as under `oldestFirst`, and `lastPrune`
   on every page as *After a prune* says, whichever order the page is in.
6. **MUST** refuse with `invalidCursor` a `cursor` issued under a different `order`,
   `sinceVersion` or `beforeVersion` from the request carrying it. A cursor marks a position in
   one direction between two bounds; read in the other direction it would skip records or repeat
   them, and read between other bounds it would return rows outside them, and nothing on the
   page would show which.
7. **MUST** compute `dataCommitment`, `recordCount` and `headVersion` over the whole room as
   *The data commitment* says, whatever the order and the bounds. They change which rows a page carries,
   never what the room holds.

**What a newest-first page holds.** A record's `version` is the version of its latest change, and
an edit, a curation or a retraction moves it to the top. So a `newestFirst` listing is not a list
of records by when they were written: an old message edited this morning appears beside this
morning's messages. What the order does guarantee is this: **a newest-first page — together with
the pages before it in the same listing — whose lowest `version` is `V` holds every record whose
current version is above `V`.** Creating a record is a change, so every record created after the
room reached `V` is in it, and so is every older record changed since. A reader opening a
conversation at its head therefore has every message newer than the oldest row it holds, and
**MAY** also hold some older ones that were edited or curated recently. It **MUST** order the
messages it shows by the host's `createdAt` (`RecordMetadataV3`; else `updatedAt`, for a record
stored before the host kept `createdAt`), never by the order of the rows, and **SHOULD** show an
older record it met this way only where it falls among the messages it is already showing, or
once the reader pages back to it. The same holds for what a reader counts as unread: it applies
[`rooms/changed/0.1`](../../../changed/0.1/spec.md) *What a reader does with a signal*, which
judges a record new by its key and `createdAt`, not by where the listing put it.

**Loading older messages.** A reader showing a conversation from its head, whose lowest row is at
version `V`, asks for the page before it with `order: "newestFirst"` and `beforeVersion: V`, and
with `sinceVersion` where it keeps a watermark below which it already holds the room. The answer
starts at the highest version below `V` — the next older row, whatever was written meanwhile — and
each such request is complete in itself, so a reader **MAY** use `beforeVersion` instead of the
previous page's `nextCursor`, and **MUST** use it once that cursor is lost or refused. The
guarantee above holds within the bound: a listing with `beforeVersion` `B` whose lowest version is
`V` holds every record whose current version is above `V` and below `B`. A record edited or
curated after the reader fetched it has moved above `B`, out of the band, and the reader learns
of it the way it learns of every change at the head — from a listing with its watermark as
`sinceVersion` — never by assuming it is gone because the band no longer holds it.

**A room that moves while it is listed.** Pages are read one at a time, and a record changed
between two of them takes a new version above the listing's first row. Oldest first, such a record
is met again further on; newest first, the listing has already passed that point and does not
return it. So the guarantee above holds for every record except one changed while the listing was
being read — and that one now has a version above the highest the listing returned, where the
reader's next listing with that version as `sinceVersion` finds it. A reader **MUST NOT** treat a
newest-first listing as covering any version above the highest version on its first page.

A reader that keeps a copy of the room and has synchronised it to watermark `W` **MAY** catch up
with a `newestFirst` listing and `sinceVersion` `W`, and show the newest messages from its first
page. It **MUST NOT** take a new watermark until it has read that listing to its end: a
`newestFirst` page that is not the last one says nothing of the versions between its lowest row
and `W`, and a reader that advanced its watermark to the page's highest version would never fetch
them. Having read it to the end, it takes as its new watermark the highest version on the
listing's **first** page, or the latest `lastPrune.version` where that is higher (*After a prune*)
— never a version from a later page, which would skip a record that moved while it read.

A reader looking for its authors' names (`MemberProfileRecord`,
[`rooms/_shared/0.3`](../../../_shared/0.3/member-profile.schema.json), *Near the room's head*)
reads, under `newestFirst`, at least 500 rows below the earliest record it shows, or to the
listing's end. Each row is a distinct version, so 500 rows reach at least 500 versions down, and
hold every name a writer following that rule keeps near its writing. A host **MAY** return fewer
rows than `limit`, so a reader counts rows, not pages.

### After a prune

A host **MUST** return `lastPrune` on every page of every listing of a room whose records have
ever been pruned — whatever its `prefix` or `sinceVersion` — as the latest
[`rooms/records/prune`](../../prune/0.2/spec.md) left it, and **MUST** omit it otherwise. It
**MUST NOT** return a pruned record, or any trace of one beyond the marker: a pruned tombstone
is not a tombstone.

A reader that keeps its own copy of the room, synchronised to some version `W`:

1. **MUST**, on a page whose `lastPrune.version` is greater than `W`, stop treating its copy as
   current: list the room again with no `sinceVersion` and no `beforeVersion`, to the end, and drop every record its
   copy holds that the full listing does not. Merging the incremental page instead keeps every
   record the prune removed.
2. **MUST** then take as its new watermark the highest version it has seen — which is at least
   `lastPrune.version` — so that the same prune does not trigger a second rebuild.
3. **MUST NOT** read the rebuild's disappearances as records it may restore from its own copy,
   push back to the room, or show as current. They were removed by a holder of `admin`, under
   the room's stated record retention.

A reader with no copy — one that lists to look and keeps nothing — has nothing to rebuild.

A `newestFirst` page carries `lastPrune` exactly as an `oldestFirst` one does, so a reader catching
up from the head down learns of a prune on its first page and rebuilds before it advances its
watermark.

## The data commitment

A listing has **no completeness property of its own**. Records are signed and
room-bound, so a host cannot forge one, alter one, or move it between rooms — but
omitting one from a response costs nothing and looks like a room that never held
it. `dataCommitment` is what turns that silence into something checkable.

### What it is worth, and when

**A commitment read once, in isolation, proves nothing.** It is the host's own
assertion, and a host willing to omit a record is willing to assert the root of
the set it chose to send. It becomes evidence the moment a reader can compare it
against a copy the host did not choose for them:

- the root the same host gave **another member**;
- the root it gave **the same member earlier**, against a listing that has only
  grown;
- the **witnessed anchor**, once a room anchors one.

That is the same mechanism Certificate Transparency relies on: a signed tree head
is not proof of non-equivocation, comparing them is.

### A root alone is not comparable — `recordCount` and `headVersion`

An earlier revision of this section ended *"a host that shows two members two
different roots for the same room has been caught, and cannot claim a transient."*
**That was wrong**, and the error came from taking half of the analogy above.

A room moves. Every put, curate and retraction changes the tree, so two roots taken
at two moments differ for the most ordinary reason there is. A host shown to have
served two different roots answers *there was a write between your reads* — and
with a bare root there is nothing that contradicts it. The comparison the whole
member is built on could not be performed by anybody.

Certificate Transparency does not have this problem because **an STH is a root and
a tree size**, and a size names the state the root describes. This family shipped
the root and dropped the rest. So:

- **`headVersion`** — the highest version among the records the root covers, or the
  version of the room's latest prune where that is higher (`HeadVersion`). A prune removes
  leaves and takes a version like any other change to the tree, so a root before it and a
  root after it are never labelled alike. A
  version is assigned by exactly the mutations that change the tree, so the highest
  of them names the state. Two roots at the **same** `headVersion` that differ is a
  host caught, with no write to attribute the difference to. Two roots at different
  ones are two moments, and a reader must draw nothing from them.

  It is taken **from the committed set**, not read from the room's own counter, and
  a host **MUST** derive it that way. The two agree for any host that has never
  erased a record, but they are not interchangeable: a root and a counter are two
  reads, and two reads are not a snapshot. A write landing between them labels a
  root with a version from a different moment, and two members would then hold
  roots over different trees under one version — which reads as equivocation and is
  not. **A false accusation discredits the mechanism rather than the host**, which
  is the worst outcome this machinery has.
- **`recordCount`** — how many records the room held, tombstones included. A reader
  cannot recompute the root from a listing (below), but it can **count**. A host
  that omits a record from a listing while committing to a tree holding it now
  contradicts itself inside one response, with no second party and no anchor.

Both are computed from the snapshot the root was taken over, and both are OPTIONAL
only in the sense the root is: a host that maintains no tree asserts none of it.
A host that serves `dataCommitment` **SHOULD** serve both, and a reader that
receives a root **without** them **MUST NOT** compare it against another root. Such
a root is still usable against a **witnessed anchor**, where the epoch pins the
state — which is why it is not simply refused.

A host can of course understate the count and the head together. That is the point
rather than a hole: the omission stops being *silence* and becomes a specific claim
about how many records the room holds and how far it has been written, which any
other member's view — and any writer's signed put acknowledgement — contradicts
directly. Making an omission **attributable** is the whole of what this machinery
buys; it never claimed to make one impossible.

### Why it is OPTIONAL

A host that does not maintain the tree cannot honestly assert a root, and a
fabricated one is worse than none — its **absence is itself informative**, saying
"this host offers no completeness guarantee", which is true and which a member is
entitled to know. A member who requires the guarantee can decline such a host;
one who does not can carry on.

A conforming consumer **MUST NOT** treat an absent `dataCommitment` as a failure,
and **MUST NOT** treat a present one as a completeness proof on its own.

### Whole room, never the page

The commitment is over every record the room holds, not over the records being
returned. A page-scoped root is one a host satisfies by construction and could
never fail, which would make the member feel checked while checking nothing.

The consequence is deliberate: a reader **cannot** recompute the root from a
single page, and is not meant to. Comparison is what this member is for; proving
a *particular* record sits under a particular root is what a
[trace](../../get/0.2/spec.md#traces) is for, and it lives on the single-record
read.

### A listing cannot be reconciled against the root

Not even a complete, unfiltered one — and this is worth stating plainly, because
"compare the roots" invites the assumption that a reader who has every entry can
recompute one. A leaf commits to a **whole record**, its metadata and its stored
content together, while this task returns `RecordMetadata`, a projection that
deliberately omits the body. A reader holding every entry in the room cannot
rebuild a single leaf.

That is a cost of committing to whole records, and it is the right cost. A leaf
over metadata alone would make a listing self-verifying — and would leave an
`open` room's body, which is cleartext the host stores and nobody signs, outside
the commitment entirely. On the sealed tiers less would have been lost, since the
AEAD already binds each ciphertext to `roomId`, `key`, `version` and `epoch`; on
`open` there is nothing else holding the body at all.

Reconciling a listing against the root therefore means reading the records — one
[`rooms/records/get`](../../get/0.2/spec.md) each, verifying each trace. That is a
deliberate audit, not something a client does behind every listing, and this
member is not an invitation to make it one.

### It is only evidence because the response is signed

Every specification declaring a `proof` requirement on its request declares one
on its response too ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 7),
so this value arrives attributable. An unsigned root would be a number from
nobody in particular, and none of the comparisons above would mean anything — a
host could deny having said it.

## Request

A **Member** (`issuer`) sends this to the room's **Host** (`recipient`). The payload is the
top-level schema in [`payload.schema.json`](payload.schema.json).

### A reader loads the messages before the oldest one it shows

The reader opened the conversation newest first and holds rows down to version `30411`. It asks
for the 50 before that, and keeps no cursor between the two requests.

```json
{
  "id": "urn:uuid:7c0e4b6a-3f1d-4a8e-9b2c-5d6e7f8a9b01",
  "type": "https://trusttasks.org/spec/rooms/records/list/0.8#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-01-01T00:05:00Z",
  "threadId": "urn:uuid:7c0e4b6a-3f1d-4a8e-9b2c-5d6e7f8a9bff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-root.sig"
      ]
    },
    "order": "newestFirst",
    "beforeVersion": 30411,
    "limit": 50
  }
}
```

## Response

The **Host** answers with the sub-schema reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Failures use a `trust-task-error` document rather
than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### The page before it

Shortened to two rows. The first is the highest version below the bound; `nextCursor` says more
remain, and the reader may follow it or send `beforeVersion: 30406` next time.

```json
{
  "id": "urn:uuid:7c0e4b6a-3f1d-4a8e-9b2c-5d6e7f8a9b02",
  "type": "https://trusttasks.org/spec/rooms/records/list/0.8#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-01-01T00:05:01Z",
  "threadId": "urn:uuid:7c0e4b6a-3f1d-4a8e-9b2c-5d6e7f8a9bff",
  "payload": {
    "records": [
      {
        "key": "Xk3nP0vQ8sRb2YtLw5HcJg",
        "version": 30409,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2025-12-31T23:58:10Z",
        "createdAt": "2025-12-31T23:58:10Z"
      },
      {
        "key": "m7Tz1QaE4uWc9DkRy2NvHs",
        "version": 30406,
        "epoch": 7,
        "status": "active",
        "updatedAt": "2025-12-31T23:51:44Z",
        "createdAt": "2025-12-31T23:51:44Z"
      }
    ],
    "nextCursor": "djE6ZGVzYzpiNDozMDQwNg"
  }
}
```

Versions `30410`, `30408` and `30407` are absent because they belong to records that have since
changed again and now sit above the bound — at the head, where the reader's next listing from its
watermark finds them.

## Security & Privacy

**A host verifies chains; it does not keep a roster.** Authorization is decided entirely by
credentials the room issued. A host that consults state of its own has made the room
unmovable and has made itself part of the membership.

**Chain verification is the security of this family.** Anyone can mint a well-formed
authority credential naming any scope and any actions; what makes it worthless is that its
chain does not reach the room. A host that verifies only the credential it was handed
accepts a self-issued grant of arbitrary authority. Every link is verified, and any link
that widens actions, scope, or validity beyond its parent invalidates the chain.

**Chain depth is a denial-of-service surface.** Verification is linear in length and runs on
every operation, hence the maximum of 8. The known uses need 2 to 3.

**Parents are never dereferenced.** A producer presents every link. Resolving one over the
network would make verification depend on availability, turn an identifier into a request
the host can be induced to make against an address the *producer* chooses, and signal
credential use to whoever hosts that identifier. Identifiers in a chain are identifiers, not
locators.

**Credential pooling.** Where membership and authority are presented with the subject
withheld, a host that does not require proof that both describe the same subject lets two
parties combine one's membership with the other's authority and present as a single party
holding both.

**What a host learns anyway.** Sealing content does not hide activity. A host observes room
identifiers, owners, record counts, sizes, epochs, and timing, and — unless traffic is
routed — the network origin of whoever is acting. A room whose adversary can correlate on
those wants a different host, not a different visibility.

### Data carried

Metadata only: keys, versions, epochs, timestamps, the BlobRefs of any files the records
name, when each record was first stored, and — on `open` rooms — titles, descriptions,
authors, kinds and first authors. `lastPrune` adds when the
room was last pruned and how many records went, never which. Never bodies, and nothing
about a file beyond which blob it is.

`order` and `beforeVersion` add nothing to what crosses the wire. A host already serves every row to a reader
entitled to list the room; `newestFirst` changes only the order in which it serves them, and
`beforeVersion` only which of them a page holds, so neither changes what a host discloses nor
to whom.

### Correlation

A listing reveals a room's size and change rate to whoever may read it. A `newestFirst` listing
that stops after its first page tells the host that the reader looked only at the room's recent
records, which is no more than the `sinceVersion` an incremental reader already sends it. A
`beforeVersion` tells the host how far back a reader has scrolled, which a reader following
`nextCursor` tells it equally, one page at a time. Opaque keys on
encrypted rooms keep the listing itself from describing the material.

### Retention

The task stores nothing. Cursors are short-lived and host-issued, under either order;
`beforeVersion` is the reader's own and needs nothing kept by the host.

### Consent/purpose

Authorized by a chain conferring `read`, and scoped to one room. The order and the bounds are
the reader's choice and need no authority of their own: each returns only what `read` already
confers.
