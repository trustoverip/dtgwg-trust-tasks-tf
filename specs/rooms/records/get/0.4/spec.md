---
slug: rooms/records/get
version: "0.4"
extends: "0.3"
title: Rooms Records — Get
summary: "A member reads one record from a data room, presenting the same authority chain a write presents and needing no host session; the record names the stored blobs it keeps alive, and when it was first stored and by whom."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - record
  - read
  - authority
  - recall
  - commitment
  - trace
  - file
  - blob
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
  - code: rooms/records/get:notFound
    meaning: "No record with that key in this room."
    retryable: false
  - code: rooms/records/get:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/records/get:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
  - code: rooms/records/get:purposeNotGranted
    meaning: "The request names a purpose the presented authority chain does not grant — a chain stating no purposes grants `any`, which never includes `modelTraining` — or names none where the chain lists more than one. The refusal discloses no content and is distinct from `permissionDenied`, which refuses access itself (VTI-ACL-104)."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [purposeNamed]
      properties:
        purposeNamed:
          type: boolean
          description: Whether the request named a purpose. False when it named none where more than one is granted.
        purpose:
          type: string
          description: The purpose the request named, when it named one.
related:
  - rooms/records/put
  - rooms/records/list
  - rooms/blobs/get
---

## Abstract

The **Rooms Records — Get** Trust Task returns one record from a data room.

**A read presents exactly as a write does.** This is the point of the task, not an
implementation detail: authorizing reads by host session would hand the host a member
identifier on every access, and a period of access logs reconstructs the membership a
`private` room exists to withhold — recovered without breaking any cryptography. So on a
`private` room a host **MUST NOT** require a session, and learns only that *a* member read
*a* record.

**A record can carry a file.** Since [`rooms/records/put/0.2`](../../put/0.2/spec.md) a
record may name a stored blob — the file's ciphertext, held outside the record — in its
cleartext `blobs` member. This version returns that member, so a reader learns which blob
to fetch with [`rooms/blobs/get`](../../../blobs/get/0.2/spec.md), and a mirror learns which
blobs the record keeps alive.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

### What changed from 0.3

- **`purpose`** (optional) is added to the request: the one purpose the presenter will use the record for
  (VTI-ACL-100 – 110), `RecordPurpose` in [`rooms/_shared`](../../../_shared/0.2/room.schema.json).
- **`rooms/records/get:purposeNotGranted`** is new: a host refuses a read whose purpose the presented
  authority chain does not grant (VTI-ACL-110); a chain stating none grants `any`.
- `extends: "0.3"`: a 0.3 consumer reads a 0.4 get as a 0.3 get, ignoring `purpose`. The response is unchanged.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** present the entire authority chain conferring `read`,
and on a `private` room **MUST** include `presentation.subjectBinding`.

A conforming **consumer** (the host) **MUST** apply the chain-verification, depth, and
no-dereference rules of
[`rooms/records/put`](../../put/0.3/spec.md), and **MUST NOT** condition service of a
`private` room on a session of its own.

A host **MUST** return `createdAt`, `createdBy` and `bodyStoredAt` as [`rooms/records/list/0.6`](../../list/0.6/spec.md)
returns them on its rows — the same values for the same record, `createdBy` never on a `private`
room — and as [`rooms/records/put/0.5`](../../put/0.5/spec.md) Host items 16 and 17 record them.

A host **MUST** return `blobs` exactly as the stored version of the record names it, and
**MUST** omit it for a record that names none, for a retracted record (a tombstone keeps no
body, and releases its blobs), and on any record written by `rooms/records/put/0.1`.

A presentation that does not confer `read` at this room's scope, whose chain does not reach
the room, or whose leaf is not the party the host authenticated is refused with the standard
`permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)); this version declares no
task-specific code for it.

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
[`rooms/records/list`](../../list/0.6/spec.md), whose section on it governs.

It is here, on a single-record read, for two reasons. A reader taking several
reads can tell **whether the room moved between them**, which a per-read root
answers and a listing taken once does not. And it is the root a returned record is
proved to sit under — `trace` reaches it, and is defined to reach nothing else.

As on a listing: OPTIONAL, because a host that maintains no tree must not invent
a root; not a completeness proof on its own, because a single root is the host's
own assertion until it is compared against one the host did not choose.

`recordCount` and `headVersion` travel with it and are what make that comparison
possible at all — see [`rooms/records/list`](../../list/0.6/spec.md), whose section
on them governs. The short of it: a root without a `headVersion` names no state, so
two of them differing is a room that moved as readily as a host that equivocated,
and a reader **MUST NOT** compare it against another root.

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
with its verification members — `dataCommitment`, `recordCount`, `headVersion` and
`trace` — `ext`, `blobs`, `createdAt`, `createdBy` and `bodyStoredAt` removed.** `blobs`,
`createdAt`, `createdBy` and `bodyStoredAt` are removed because `CommittedRecord` predates them and does not commit to them; a reader hashing a payload that still held `blobs`
would reach a leaf in no tree. What the commitment therefore does not cover, and why a file's
integrity does not depend on it, is stated in
[`rooms/records/put/0.3`](../../put/0.3/spec.md) under Security & Privacy. That object is `CommittedRecord` in
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

## Mirrors of rooms that hold files

A **mirror** — a second host, or a member's own replica, that keeps a copy of a room by
reading it — learns from `blobs` which stored blobs each record keeps alive, and fetches
each one it lacks with [`rooms/blobs/get`](../../../blobs/get/0.2/spec.md) and
[`rooms/blobs/chunk`](../../../blobs/chunk/0.1/spec.md), under the same `read` chain it reads
records with. A blob is identified by its BlobRef — the digest of its ciphertext manifest —
so a mirror verifies every chunk and the whole blob against the manifest, and stores nothing
it cannot verify.

A mirror keeps a blob for as long as some record it holds names it, and releases it when the
last such record drops it or is retracted, as the host does. It never needs a file key: the
blob is ciphertext, and a mirror that is not a member could not open it anyway.

What a mirror cannot learn from the record tree is whether the host dropped `blobs` from a
record: the commitment does not cover it (see the leaf preimage above). A reader that opens
the record finds the `blobRef` in the sealed, author-signed `FileManifest` regardless.

### Purpose of use

Every read is **purpose-bound** (VTI-ACL-102, VTI-ACL-110): the presented authority chain is a grant
written as a credential, and the purposes it grants bound what the presenter may use the record for. A
chain stating none grants `any`, which does not grant `modelTraining`, so a read is held to the chain's
purposes whether or not any link states them.

The purposes the chain **grants** are those its leaf states, read as
the chain rule in [`rooms/owner/issue-authority`](../../../owner/issue-authority/0.3/spec.md) 0.3, *Purposes along a chain* describes: a link that states none takes
its parent's, and a chain in which a link states purposes its parent does not grant is refused whole, as
any other chain that does not verify (VTI-ACL-107).

`any` grants every purpose but `modelTraining` (VTI-ACL-105); a list grants exactly its members, an
unrecognised purpose equalling only itself (VTI-ACL-109). Where exactly one purpose is granted, a request
naming none is read as naming it (VTI-ACL-103).

A producer **SHOULD** name in `purpose` the one purpose it will use the record for, and **MUST** name one
where the chain it presents lists more than one.

A conforming **consumer** (the host) **MUST**, in addition:

1. Refuse with `rooms/records/get:purposeNotGranted`, after the chain verifies and before returning any
   record, a request naming a purpose the chain does not grant, or naming none where the chain lists more
   than one. The refusal carries no content, and says nothing about whether the record exists.
2. Where it keeps a record of reads, record each read's purpose, or that none was named (VTI-ACL-106).

A host that serves a sealed tier returns ciphertext it cannot open: the purpose binds the presenter all the
same, and the key holder that opens the record holds the caller to it again (`rooms/keys/read` 0.3).

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

Returns one record — sealed or cleartext by visibility — and, when it carries a file, the
`blobRef` of the stored blob in the clear; when the host stored the record's first version and
its current body; and,
on `open` and `attributed` rooms, who wrote it. The request names the room and key.

### Correlation

A read discloses which record was read and, on `open` and `attributed`, by whom. A read log
is a record of who was interested in what, which is why it warrants its own retention period
rather than inheriting a general one. On `private` the host learns only that a member read.

### Retention

The task stores nothing. Any read record a host keeps is its own audit decision.

### Consent/purpose

Reading is authorized by an authority chain conferring `read`. A host serves the record and
draws no further inference from having served it.
