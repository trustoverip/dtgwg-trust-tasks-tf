---
slug: rooms/records/get
version: "0.1"
title: Rooms Records — Get
summary: "A member reads one record from a data room, presenting the same authority chain a write presents and needing no host session."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - record
  - read
  - authority
  - recall
  - commitment
  - trace
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
  rationale: "A read returns material the room's members rely on; the presentation authorizing it must be verifiable independently of transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A read presentation is time-bounded, and accepting a stale one would extend access past the window its credentials describe."
sideEffects:
  level: none
  rationale: "Returns one record."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns record content — sealed on `attributed` and `private` rooms, cleartext on `open`. The host learns which record was read and, on `open` and `attributed`, which member read it. The response also names the record's author on those two tiers: the same member `rooms/records/list` already returns there, now carried on a single read because it is part of what the record commits to."
retention:
  class: transient
  rationale: "The task returns existing state and stores nothing of its own."
errorCodes:
  - code: rooms/records/get:notAuthorized
    meaning: "The presentation does not confer `read` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/records/get:notFound
    meaning: "No record with that key in this room."
    retryable: false
  - code: rooms/records/get:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/records/get:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
related:
  - rooms/records/put
  - rooms/records/list
---

## Abstract

The **Rooms Records — Get** Trust Task returns one record from a data room.

**A read presents exactly as a write does.** This is the point of the task, not an
implementation detail: authorizing reads by host session would hand the host a member
identifier on every access, and a period of access logs reconstructs the membership a
`private` room exists to withhold — recovered without breaking any cryptography. So on a
`private` room a host **MUST NOT** require a session, and learns only that *a* member read
*a* record.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** present the entire authority chain conferring `read`,
and on a `private` room **MUST** include `presentation.subjectBinding`.

A conforming **consumer** (the host) **MUST** apply the chain-verification, depth, and
no-dereference rules of
[`rooms/records/put`](../../put/0.1/spec.md), and **MUST NOT** condition service of a
`private` room on a session of its own.

The response has its own two parties, and they are the other way round. A host that
serves `trace` **MUST** compute it and `dataCommitment` from the same snapshot of the
room, and **MUST** serve every committed member it holds — an omitted `status`,
`updatedAt`, `pinned` or `author` leaves the reader hashing a different object and
reaching a leaf that is in no tree. A host that maintains no record tree **MUST NOT**
serve either member.

A reader **MUST** verify a `trace` against the `dataCommitment` carried in the same
response and no other; **MUST** treat a `trace` arriving without `dataCommitment`,
`status` or `updatedAt` as *invalid* rather than merely unverified; and **MUST NOT**
read a verified `trace` as evidence that the room holds no record the host withheld.

A host **SHOULD** record reads on `open` and `attributed` rooms. Reads of shared material
are the interesting event — but such a log is itself a record of who was interested in
what, so it warrants a stated retention period of its own rather than inheriting a general
one.

## The data commitment

`dataCommitment` carries the room's record-tree root at the moment this record was
read — the same value and the same construction as
[`rooms/records/list`](../../list/0.1/spec.md), whose section on it governs.

It is here, on a single-record read, for two reasons. A reader taking several
reads can tell **whether the room moved between them**, which a per-read root
answers and a listing taken once does not. And it is the root a returned record is
proved to sit under — `trace` reaches it, and is defined to reach nothing else.

As on a listing: OPTIONAL, because a host that maintains no tree must not invent
a root; not a completeness proof on its own, because a single root is the host's
own assertion until it is compared against one the host did not choose.

## Traces

`trace` is the path from this record's leaf to `dataCommitment`. Verifying it answers
exactly one question — *is this record under the root the host just asserted?* — and a
reader that takes it to answer any other has been misled by the word "proof", which is
why the member is not called one. In this framework `proof` is the document's
data-integrity proof ([SPEC §7.3](/SPEC.md#73-specification-requirements)); a payload
member of the same name in the same document invites reading one for the other, and the
confusion would be silent.

### The leaf preimage is this response

**The leaf is `SHA-256(0x00 || JCS(record))`, where the record is this response payload
with `dataCommitment`, `trace` and `ext` removed.** That object is `CommittedRecord` in
the [shared schema](../../../_shared/0.1/room.schema.json), and a reader MAY validate
what it assembles against that definition before hashing it.

Reassembly is therefore a **deletion**, not a reconstruction — the reader already holds
every member. The obvious alternative, carrying the committed record as a member of its
own, would put the ciphertext on the wire twice: paid for on every read, and free to
disagree with itself on the read where it mattered.

That is what the four members this response gained are for. `status`, `updatedAt`,
`pinned` and `author` are committed, and a response omitting them was one no reader could
hash. They are OPTIONAL so that adding them to a published `0.1` breaks nothing;
`dependentRequired` makes `status`, `updatedAt` and `dataCommitment` mandatory wherever
`trace` is present, which is the only place their absence can do harm.

The definition used to be looser than it reads. `DataCommitment` described its leaf as
"`RecordMetadata` plus its stored content" — exact-sounding, and not reproducible:
`RecordMetadata` is the *projection* a listing returns, which renders `updatedAt` as a
timestamp, lifts `epoch` to the top level, carries `title` and `description` pulled out of
an `open` room's body, and has no `pinned` at all. A host hashing what it stores and a
reader hashing that projection commit to different roots, and nothing said which counted.
Traces are what made the gap matter: a commitment only has to be *comparable between two
of the same implementation*, while a trace has to be *computable by someone else*.

### Verification

  1. `h = SHA-256(0x00 || JCS(record))` — the leaf.
  2. For each step in order: `h = SHA-256(0x01 || sibling || h)` when `siblingIsLeft` is
     true, and `SHA-256(0x01 || h || sibling)` when it is false.
  3. `h` **MUST** equal `dataCommitment` **from this same response**. Not one kept from an
     earlier read and not one from a listing: a room moves, and a trace is only ever a
     statement about the tree it was cut from.

An **empty `trace` is valid** — the room holds one record and its leaf is the root. That
is not the same as an absent `trace`, which says the host offered none.

A trace is **not** always `ceil(log₂ n)` steps: a level that promotes an odd node
unchanged (`DataCommitment` step 4) contributes no step for it. Follow the steps given
rather than counting them against a tree size you assumed.

### What a trace does not prove

A trace binds a record to a root. It says **nothing** about whether that root is the
room's. A reader that verifies a trace against a root the same host handed it a moment
earlier has checked the host's arithmetic and nothing more: a host serving a private view
of the room builds a consistent tree over that view and traces every record in it
perfectly.

The root becomes evidence the way it does on a listing — by comparison against one the
host did not choose: the root it gave another member, the root it gave this member
earlier, or a witnessed anchor. **The two mechanisms answer different questions and
neither substitutes for the other.** The commitment catches a host that equivocates; the
trace binds one record to what that host committed to. Completeness needs both.

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

Returns one record — sealed or cleartext by visibility. The request names the room and key.

### Correlation

A read discloses which record was read and, on `open` and `attributed`, by whom. A read log
is a record of who was interested in what, which is why it warrants its own retention period
rather than inheriting a general one. On `private` the host learns only that a member read.

### Retention

The task stores nothing. Any read record a host keeps is its own audit decision.

### Consent/purpose

Reading is authorized by an authority chain conferring `read`. A host serves the record and
draws no further inference from having served it.
