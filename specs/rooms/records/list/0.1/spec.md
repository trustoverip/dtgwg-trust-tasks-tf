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

A host that shows two members two different roots for the same room has been
caught, and cannot claim a transient. That is the whole mechanism, and it is the
same one Certificate Transparency relies on: a signed tree head is not proof of
non-equivocation, comparing them is.

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
a *particular* record sits under a particular root is what a trace will be for,
and that is a separate member in a later version.

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
