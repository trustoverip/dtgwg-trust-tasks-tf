---
slug: rooms/create
version: "0.1"
title: Rooms — Create
summary: "An owner registers a data room with a host, naming the room's own identifier, its visibility, and the party accountable for it."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - create
  - visibility
  - owner
  - hosting
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Room creation establishes durable state a host will serve to others; its authorization must be verifiable independently of transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Creation is time-bounded by the credential authorizing it; a replayed create could re-establish a room its owner had closed."
sideEffects:
  level: mutating
  rationale: "Registers a room with the host."
subjectPath: /roomId
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The host learns the room's identifier, its visibility, its owner, and its retention period. On `attributed` and `private` rooms it learns nothing further for the life of the room."
retention:
  class: durable
  rationale: "A room persists until it lapses without renewal past its stated retention period, or its owner closes it."
errorCodes:
  - code: rooms/create:alreadyExists
    meaning: "A room with that identifier is already registered with this host."
    retryable: false
  - code: rooms/create:visibilityNotPermitted
    meaning: "The host's governance does not permit rooms of this visibility."
    retryable: false
  - code: rooms/create:notAuthorized
    meaning: "The caller may not create rooms on this host."
    retryable: false
related:
  - rooms/epoch/mint
  - rooms/records/put
---

## Abstract

The **Rooms — Create** Trust Task registers a data room with a host.

**The owner brings the identifier; the host does not assign one.** A room identified by
something its host chose could not move to another host without changing identity, and
portability is the property the whole family rests on: because authorization is decided by
credentials the room issued rather than by host state, re-pointing a room at a different
host reissues nothing.

`visibility` is **fixed at creation and immutable thereafter**. A downgrade cannot un-see
cleartext, and an upgrade would protect only what came after while presenting as though it
protected everything. To change the visibility of some material, make another room and move
it deliberately.

`ownerDid` is visible at **every** visibility, including `private`. A room has a party
answerable for it: the controller of its identifier, the issuer of its credentials, and the
party a host addresses about quota, abuse, or lifecycle. A room whose contents no one can
read still has someone responsible for it existing.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the owner) **MUST** mint the room's identifier before calling,
and **MUST** be able to demonstrate control of it.

A conforming **consumer** (the host) **MUST** refuse a visibility its governance does not
permit, **MUST** treat `visibility` as immutable for the life of the room, and **MUST**
state its retention behaviour at creation rather than applying one discovered later. A host
**MUST NOT** require a member list, at creation or afterwards.

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

The room's identifier, its visibility, its owner, and its retention period. Nothing about
its membership, then or later.

### Correlation

A host learns that this owner runs a room. An owner running several rooms is correlatable by
that fact alone, which is a reason a party may prefer a distinct identifier per room.

### Retention

The registration is durable and governs how long the host holds the room after its epoch
lapses without renewal. Stated here rather than discovered later.

### Consent/purpose

The caller asserts control of the room identifier they bring. A host's governance decides
which visibilities it will accept, and may decline.
