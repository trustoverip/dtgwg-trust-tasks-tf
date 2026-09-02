---
slug: rooms/records/put
version: "0.1"
title: Rooms Records — Put
summary: "A member writes a record to a data room, authorized by an authority chain the room itself issued rather than by anything the host stores."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - record
  - write
  - authority
  - attenuation
  - encryption
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
  rationale: "A write mutates durable shared state that other members will read and act on; transport-independent integrity is required."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A write is authorized by a presentation whose validity is time-bounded, and a replayed put would restore a superseded record."
sideEffects:
  level: mutating
  rationale: "Stores or replaces a record in a room."
subjectPath: /roomId
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: false
  rationale: "On an `open` room the host reads record content in the clear. On `attributed` and `private` rooms it receives ciphertext and learns only the key, version and epoch. The presentation discloses the acting member on `open` and `attributed`; on `private` it discloses only that some member acted."
retention:
  class: durable
  rationale: "A record persists until a member overwrites it or retracts it; outliving the session that wrote it is the point of the task."
errorCodes:
  - code: rooms/records/put:notAuthorized
    meaning: "The presentation does not confer `write` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/records/put:versionConflict
    meaning: "`expectedVersion` did not match. The response carries the current version and record."
    retryable: false
  - code: rooms/records/put:chainTooDeep
    meaning: "The authority chain exceeds the maximum of 8 links."
    retryable: false
  - code: rooms/records/put:subjectBindingMissing
    meaning: "A `private` room presentation omitted the required same-subject proof."
    retryable: false
  - code: rooms/records/put:epochMismatch
    meaning: "The record was sealed under an epoch that is not the room's current one."
    retryable: false
  - code: rooms/records/put:recordTooLarge
    meaning: "The record exceeds the host's per-record limit."
    retryable: false
related:
  - rooms/records/get
  - rooms/records/list
  - rooms/epoch/mint
---

## Abstract

The **Rooms Records — Put** Trust Task writes a record to a **data room**: a shared space
whose access is governed by credentials the *room itself* issues, and whose contents a host
may be unable to read.

The property that distinguishes this family from every other stored-data task in this
registry is that **a host never consults a member list of its own**. Authorization is a
presentation carrying a membership credential and an authority chain, verified against the
room's identifier. A host that speaks this family can therefore host *any* room without
knowing anything about who belongs to it, and a room can move between hosts without a
single credential being reissued.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the member, or an agent acting under an attenuated credential)
**MUST**:

1. Emit a *Trust Task document* of type `https://trusttasks.org/spec/rooms/records/put/0.1`.
2. Present the **entire** authority chain, leaf first, ending in a credential issued by the
   room. A producer MUST NOT rely on the host resolving a link it was not given.
3. Seal the record and set `sealed` on an `attributed` or `private` room, or set
   `cleartext` on an `open` one. Exactly one of the two.
4. Bind the sealed record's AEAD associated data to `roomId`, `key`, `version` and `epoch`.
5. On a `private` room, include `presentation.subjectBinding`.

A conforming **consumer** (the host) **MUST**:

1. Verify every link in the chain, and **reject the chain if any link widens** what its
   parent conferred — in actions, in scope, or in validity period. A host that verifies only
   the presented credential has verified nothing: anyone can mint a well-formed authority
   credential naming any scope, and what makes it worthless is that its chain does not reach
   the room.
2. Reject a chain of more than **8** links. Verification is linear in chain length and runs
   on every operation.
3. **Never dereference** a chain link's `parent` over the network. Doing so would make
   verification depend on availability, turn the identifier into a request the host can be
   induced to make against an address the *producer* chooses, and signal credential use to
   whoever hosts that identifier.
4. On a `private` room, **reject a presentation with no `subjectBinding`**. Without it two
   parties pool credentials — one contributes membership, the other authority — and the
   combination verifies as a single party holding both.
5. Reject a record sealed under an epoch that is not current.
6. Return the current version and record with a `versionConflict`, rather than a bare
   rejection.

A conforming host **MUST NOT** require a session or account of its own as a condition of
serving a `private` room. Authorizing by session would record which member acted on every
operation, and a period of such records reconstructs the membership the tier exists to
withhold — without breaking any cryptography.

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

On an `open` room, the record itself in the clear. On `attributed` and `private` rooms, only
ciphertext plus its key, version and epoch. The presentation carries a membership credential
and an authority chain; on `open` and `attributed` these name the acting member, on
`private` they do not.

### Correlation

On `attributed`, a host can build a per-member write history. On `private` it cannot, but it
still sees when writes happen, how large they are, and from where — enough to correlate by
timing and network origin if traffic is not routed.

### Retention

Records are durable by design and persist until overwritten or retracted. A retraction is a
tombstone: the body goes, the key and version stay, so incremental sync converges and the
audit chain holds.

### Consent/purpose

A member writes into a room they joined by accepting an invitation; the membership
credential pair is that consent. Nothing here authorizes a host to read, index, or derive
from record content beyond serving it back.
