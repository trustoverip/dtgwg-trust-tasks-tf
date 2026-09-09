---
slug: rooms/records/list
version: "0.1"
title: Rooms Records — List
summary: "A member lists record metadata in a data room, with prefix, watermark and cursor — never bodies, which are fetched individually."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - record
  - list
  - pagination
  - sync
  - watermark
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
  rationale: "Returns keys, versions, epochs and timestamps; on an `open` room also titles, descriptions and authors. Never bodies."
retention:
  class: transient
  rationale: "The task returns existing state and stores nothing of its own."
errorCodes:
  - code: rooms/records/list:notAuthorized
    meaning: "The presentation does not confer `read` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/records/list:invalidCursor
    meaning: "The cursor was not issued by this host, or has expired."
    retryable: false
  - code: rooms/records/list:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
related:
  - rooms/records/get
  - rooms/records/put
---

## Abstract

The **Rooms Records — List** Trust Task enumerates a room's records as **metadata**, never
bodies. A reader ranks what comes back and fetches the handful that matter with
[`rooms/records/get`](../../get/0.1/spec.md). A host that returned every body would make a
caller pay for the whole room on every listing — and on an encrypted room could not
usefully rank them anyway.

`sinceVersion` is what makes incremental sync converge, and **tombstones are returned like
any other record**. Without them a puller learns of every create and update and never of a
delete, so retracted records resurrect on the next full rebuild and disagree with peers
that saw the retraction.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **consumer** (the host) **MUST** apply the chain rules of
[`rooms/records/put`](../../put/0.1/spec.md); **MUST** return retracted records to a caller
supplying `sinceVersion`, so that a deletion propagates; **MUST NOT** accept a cursor it did
not issue; and **MUST NOT** return record bodies from this task.

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

- **`headVersion`** — the highest version among the records the root covers. A
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
[trace](../../get/0.1/spec.md#traces) is for, and it lives on the single-record
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
[`rooms/records/get`](../../get/0.1/spec.md) each, verifying each trace. That is a
deliberate audit, not something a client does behind every listing, and this
member is not an invitation to make it one.

### It is only evidence because the response is signed

Every specification declaring a `proof` requirement on its request declares one
on its response too ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 7),
so this value arrives attributable. An unsigned root would be a number from
nobody in particular, and none of the comparisons above would mean anything — a
host could deny having said it.

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

Metadata only: keys, versions, epochs, timestamps, and — on `open` rooms — titles,
descriptions and authors. Never bodies.

### Correlation

A listing reveals a room's size and change rate to whoever may read it. Opaque keys on
encrypted rooms keep the listing itself from describing the material.

### Retention

The task stores nothing. Cursors are short-lived and host-issued.

### Consent/purpose

Authorized by a chain conferring `read`, and scoped to one room.
