---
slug: rooms/owner/claim
version: "0.1"
title: Rooms Owner — Claim
summary: "A nominated successor claims ownership of a room whose owner has stopped renewing it — an act, never an automatic promotion."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - succession
  - ownership
  - liveness
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Successor
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A claim moves ownership of a shared space. One whose origin depended on the transport would let a compromised channel hand a room to anyone holding a nomination."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed claim would re-take a room the original owner has since recovered by renewing it."
sideEffects:
  level: mutating
  rationale: "Changes the room's recorded owner."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Names a room and returns its new owner. The nomination discloses that this party was named a successor, which on every tier is already true of whoever the room addresses about its lifecycle."
retention:
  class: durable
  rationale: "The recorded owner persists; it is who the host addresses about quota, abuse and lifecycle."
errorCodes:
  - code: rooms/owner/claim:notNominated
    meaning: "The nomination is missing, invalid, not issued by this room, or does not name the claimant."
    retryable: false
  - code: rooms/owner/claim:roomStillLive
    meaning: "The room has not been dormant long enough to be claimed. The owner is renewing it."
    retryable: false
  - code: rooms/owner/claim:notAMember
    meaning: "The claimant presented no valid membership credential for this room, and so could not renew what they are claiming."
    retryable: false
  - code: rooms/owner/claim:notAuthorized
    meaning: "The presentation does not verify against this room, or its chain does not reach it."
    retryable: false
related:
  - rooms/owner/transfer
  - rooms/epoch/mint
---

## Abstract

The **Rooms Owner — Claim** Trust Task lets a nominated successor take ownership of a room
whose owner has stopped renewing it.

It exists because ownership is **load-bearing for liveness, not just administration**. A
room's owner is its sole committer, so a room with no reachable owner cannot advance an
epoch — and a room that cannot advance an epoch cannot be renewed, cannot admit anyone, and
lapses to read-only on the schedule in [`rooms/epoch/mint`](../../../epoch/mint/0.1/spec.md).
Without succession, one person becoming unreachable ends a shared space.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Behaviour

### A claim is an act

A successor **MUST** claim. A host **MUST NOT** promote one automatically, however long a
room has been dormant and however unambiguous the nomination.

The distinction is not ceremony. An automatic promotion is an ownership change nobody
performed: there is no actor to audit, no moment to point at, and an owner who returns finds
the room changed hands with no event to examine. Requiring a claim makes every ownership
change something a party did, at a time, for a reason they stated.

### Three conditions, and all three

A host **MUST** refuse unless all of the following hold. Each closes a different way a claim
could become a takeover:

1. **A nomination the room issued.** It names the claimant, it is issued by `roomId`, its
   proof verifies, and it is within its validity window. The owner decided this in advance;
   the host is checking a decision, not making one.
2. **The room is dormant.** Not merely lapsed — dormant, meaning the grace window after the
   epoch expired has also passed and the owner has had their notice. An epoch expiring is
   often somebody on holiday; a room still unrenewed after the grace window is a room whose
   owner has stopped.
3. **The claimant holds a membership credential for the room.** A successor who is not a
   member cannot commit, and so inherits a room they cannot renew — the one thing ownership
   exists to do. Answering `notAMember` is kinder than handing someone a room that will
   lapse again in a year with nobody able to save it.

   **A host cannot see the MLS group**, and this is worth being exact about rather than
   implying otherwise. It holds no roster and no group state; what it can check is the VMC
   the room itself issued, which is the room's own statement that this party is a member.
   That is a proxy, and a good one — it is the same membership every other room task
   presents and the same one the room's authority chains are rooted beside — but a party
   removed from the MLS group while still holding an unexpired VMC would pass this check.
   Closing that gap is the room's job, by revoking the credential, not the host's.

### Renewing cancels a pending claim

A room returned to live by [`rooms/epoch/mint`](../../../epoch/mint/0.1/spec.md) is no longer
claimable, and a host **MUST** answer `roomStillLive`.

This is the property worth noticing: **the defence against a hostile claim is the same act as
ordinary use.** An owner who was merely away fixes it by doing what they would have done
anyway. Nothing has to be revoked, no dispute has to be raised, and an owner who is present
is structurally safe from succession without ever thinking about it.

### The nomination lives outside the host

The host stores nothing about who a room's successors are. A nomination is a credential the
room issued and the claimant presents — the same shape as the invitation in
[`rooms/keys/welcome`](../../../keys/welcome/0.1/spec.md), for the same reason: a host that
kept a roster of successors would hold part of the room's authority structure, and the room
could no longer move hosts without rebuilding it.

A consequence worth stating: to a host, "this room has no successor" and "the successor has
not claimed yet" are the same observation. Both resolve identically — retention runs out and
the room becomes reclaimable — so the host never needs to tell them apart.

## Security & Privacy

**A claim is a takeover surface, and it is supposed to be.** Whoever holds a valid nomination
can take a dormant room. That is the intended power: the alternative is a shared space that
dies with one person's availability.

What bounds it is that all three conditions are required together. A nomination alone does
nothing while the owner renews. A dormant room alone does nothing without a nomination. And
neither helps a claimant who cannot commit.

**The host is not an arbiter.** If a former owner disputes a claim, the host has no way to
adjudicate and **MUST NOT** try. It recorded what an authorized party told it, and it
recorded that in the audit trail. Ownership of the room's identifier is settled by whoever
controls that identifier — a matter for the DID's own controller and its witnesses, not for a
service that stores ciphertext.

**Nominate carefully, and expire nominations.** A nomination is a standing right to take the
room the moment its owner stops. An owner who nominates broadly has distributed that right
broadly. Nominations **SHOULD** carry a validity window and be re-issued rather than left
open-ended.

### Data carried

A room identifier, a nomination credential, the claimant's own presentation, and a reason.
No content, on any tier.

### Correlation

The nomination discloses that this party was named a successor. On every tier the room's
owner is already visible — it is the party the host addresses about quota, abuse and
lifecycle — so a claim moves a visible role rather than revealing a hidden one.

### Retention

The recorded owner persists. The `reason` is member-authored free text and a host **MUST**
treat it as untrusted for rendering and for any agent that reads it back.
