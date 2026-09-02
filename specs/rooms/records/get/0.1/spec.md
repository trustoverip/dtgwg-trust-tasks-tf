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
  rationale: "Returns record content — sealed on `attributed` and `private` rooms, cleartext on `open`. The host learns which record was read and, on `open` and `attributed`, which member read it."
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

A host **SHOULD** record reads on `open` and `attributed` rooms. Reads of shared material
are the interesting event — but such a log is itself a record of who was interested in
what, so it warrants a stated retention period of its own rather than inheriting a general
one.

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
