---
slug: rooms/records/curate
version: "0.1"
title: Rooms Records — Curate
summary: "A member changes a record's standing in a data room — demote, retract, restore, pin — without rewriting it, under an authority action distinct from write."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - record
  - curation
  - tombstone
  - authority
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
  rationale: "Curation mutates durable shared state that other members and their agents rank on; a retraction whose integrity depended on the transport would let a compromised channel silence a record."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed curation would re-demote a record a member had deliberately restored."
sideEffects:
  level: mutating
  rationale: "Changes a record's status or pinning, and on `retracted` drops its body."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Names a room, a record key and a standing. Carries no content in either direction — on the sealed tiers the host cannot read the record it is curating, which is the point: curation is a statement about a record, not about its contents."
retention:
  class: durable
  rationale: "A tombstone is retained deliberately — the key and version remain so incremental sync converges. Erasing it is a separate, higher-trust act."
errorCodes:
  - code: rooms/records/curate:notAuthorized
    meaning: "The presentation does not confer `curate` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/records/curate:versionConflict
    meaning: "`expectedVersion` did not match. The response carries the current version."
    retryable: false
  - code: rooms/records/curate:alreadyRetracted
    meaning: "The record is retracted and the request asked for `active`. A retracted body is gone; a status change cannot bring it back."
    retryable: false
  - code: rooms/records/curate:roomNotLive
    meaning: "The room's epoch has lapsed. Curation is a write; a room nobody has renewed is read-only until somebody does."
    retryable: false
related:
  - rooms/records/put
  - rooms/records/list
  - rooms/epoch/mint
---

## Abstract

The **Rooms Records — Curate** Trust Task changes a record's **standing** in a data room —
demote it, retract it, restore it from demotion, pin it — without a member having to rewrite
the record to say so.

It is separate from [`rooms/records/put`](../../put/0.1/spec.md) for two reasons, and the
second is the load-bearing one.

**A record's standing is not its content.** On an `attributed` or `private` room a host
cannot read what it stores, so "replace this record with the same content marked
deprecated" would require the member to re-seal and re-upload a body the host already holds,
for a change that says nothing about the body. Curation carries no content in either
direction.

**`curate` is its own authority action.** It is deliberately not implied by `write`:
deciding what a room's shared knowledge is *worth* is a different grant from the ability to
add to it. A community can hand an agent `write` — let it record what it learns — without
handing it the standing to demote what a person wrote. Nothing prevents granting `curate` to
an agent; the default posture should not.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Behaviour

### Retraction is a tombstone, not an erasure

On `retracted` a host **MUST** drop the record's body and **MUST** retain its key, version
and epoch. Both halves matter:

- Dropping the body is what a member asking to retract actually wants.
- Retaining the key and version is what makes **incremental sync converge**. A caller
  synchronising on `sinceVersion` learns about the retraction by seeing the tombstone; a
  caller that never saw it would resurrect the record on its next full rebuild. This is why
  [`rooms/records/list`](../../list/0.1/spec.md) returns tombstones rather than filtering
  them out.

A host **MUST NOT** accept `status: "active"` for a retracted record. The body is gone, and
returning success would tell a member their record was restored when it was not —
`alreadyRetracted` says so plainly instead.

Permanent removal of a tombstone is **out of scope for this task**, and deliberately so:
removing it breaks convergence for every caller that has not synchronised past it. It
belongs to a host's retention lifecycle, not to a member's curation verb.

### Curation assigns a new version

A curated record **MUST** receive a new version from the room's counter. A demotion other
members are expected to converge on is a change like any other, and a status change that
left the version alone would be invisible to every `sinceVersion` watermark in the room.

### Pinning is orthogonal to status

`pinned` and `status` answer different questions — "what matters here" and "is this still
current" — so a record may be pinned and deprecated at once. That combination is not a
contradiction: a room may well want its superseded canonical decision kept in view.

### A lapsed room is read-only

Curation is a write. A room whose epoch has lapsed accepts none until somebody renews it
(`rooms/epoch/mint`), and a host **MUST** answer `roomNotLive` rather than curating. A room
nobody has renewed should not be quietly reorganised.

## Security & Privacy

**A retraction is not a deletion, and a member must not be told otherwise.** The tombstone
survives, the fact of the record's existence survives, and anything a member exported before
the retraction is beyond the room's reach entirely. A surface that presents retraction as
erasure is making a promise the protocol does not keep.

**Curation is a censorship surface, which is why it is a separate grant.** Whoever holds
`curate` can demote or retract anything in the room. That is the intended power — human
judgement over shared knowledge — and it is exactly why it should not arrive as a side
effect of being able to write.

### Data carried

A room identifier, a record key, a standing, and an optional reason. No content, on any
tier.

### Correlation

The same as every other room task: a host learns that a member acted on a room and, on
`open` and `attributed`, which member. On `private` it learns only that a member did. A
curation is a *more* interesting event than a read for anyone watching a log, because it is
rare and deliberate.

### Retention

The `reason` is recorded in the host's audit trail where the tier permits, and is
member-authored free text — a host **MUST** treat it as untrusted for both rendering and
any agent that reads it back.
