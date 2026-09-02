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
