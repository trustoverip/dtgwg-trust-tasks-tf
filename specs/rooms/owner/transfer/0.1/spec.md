---
slug: rooms/owner/transfer
version: "0.1"
title: Rooms Owner — Transfer
summary: "A room's owner hands ownership to another member, deliberately and while still present."
status: draft
targetFrameworkVersion: "0.5"
category: access-control
keywords:
  - room
  - ownership
  - transfer
  - handover
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
  rationale: "A transfer hands a shared space to someone else. One whose origin depended on the transport would let a compromised channel give a room away."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed transfer would hand back a room that has since changed hands again."
sideEffects:
  level: mutating
  rationale: "Changes the room's recorded owner."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Names a room and its incoming owner. On every tier the owner is already visible — it is the party the host addresses about quota, abuse and lifecycle — so this moves a visible role rather than revealing a hidden one."
retention:
  class: durable
  rationale: "The recorded owner persists; it is who the host addresses about lifecycle."
errorCodes:
  - code: rooms/owner/transfer:notAuthorized
    meaning: "The presentation does not confer `admin` at this room's scope, or its chain does not reach the room."
    retryable: false
  - code: rooms/owner/transfer:notAMember
    meaning: "The host could independently establish that the incoming owner is not a member, and so could not renew what they are being given. A host with no basis to judge does not raise this."
    retryable: false
related:
  - rooms/owner/claim
  - rooms/epoch/mint
---

## Abstract

The **Rooms Owner — Transfer** Trust Task hands a room to another member while its current
owner is still present to do so.

The deliberate counterpart to [`rooms/owner/claim`](../../claim/0.1/spec.md), which exists
for when they are not. Both end with a different recorded owner; everything else about them
differs, which is why they are two tasks rather than one with a flag.

| | `transfer` | `claim` |
|---|---|---|
| initiated by | the outgoing owner | the incoming one |
| authorized by | the owner's own `admin` chain | a nomination the owner issued earlier |
| requires the room to be | anything | dormant |
| the outgoing owner is | present | not |

Collapsing them would mean either a transfer that waits for the room to lapse, or a claim
that works while the owner is still renewing. Both are wrong.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Behaviour

### The incoming owner must be a member — and the host cannot check it

A transferring owner **MUST NOT** name a `newOwnerDid` that is not a member of the room's
group. A room's owner is its sole committer, so an owner who cannot commit cannot renew the
room; handing someone a room they cannot renew looks like a successful transfer and produces
a room that lapses on schedule with nobody able to save it — a year later, and with no event
to point at.

**The obligation is the owner's, not the host's, because only the owner can discharge it.**
A host holds no roster and no group state: the incoming owner is not the party making this
request, presents nothing, and may be someone the host has never seen. A host that *can*
independently establish membership — because it holds a membership credential for that party
from some other exchange — **MAY** refuse with `notAMember`; one that cannot **MUST NOT**
invent a check it has no basis for, and **MUST NOT** treat its own ignorance as evidence.

This is the same boundary every other room task draws, arriving from an unfamiliar
direction: the host verifies what is presented to it, and a claim about a third party is not
that. The outgoing owner *can* see the group, which is why the requirement sits with them.

### Every credential stays valid

A transfer changes who is *accountable* for a room. It does not reissue anything.

The room's identifier does not change, so every credential the room has issued — memberships,
authority chains, invitations — remains valid and verifies exactly as before. This is the
whole reason a room has an identifier of its own rather than borrowing its owner's: an
identity that belonged to a person would have to be rebuilt every time a person moved on.

What the incoming owner does need is **control of that identifier**, so they can issue
credentials going forward. That is a DID operation — witnessed pre-rotation — and it is not
this task. A transfer recorded at a host by someone who never took control of the room's DID
gives them the accountable role and no ability to issue, which is a state their next
credential issuance will make obvious.

### Authorized by `admin`, the same grant that mints epochs

Transferring is the most consequential thing an owner does, and it is deliberately not given
a grant of its own. `admin` already means "can renew this room and decide its membership";
somebody holding that can already do everything a new owner could. A separate
`transferOwnership` action would suggest a distinction the room's authority model does not
actually make.

## Security & Privacy

**A transfer is irreversible from the host's side.** The host records what an authorized party
told it. If a transfer was a mistake, the fix is another transfer, performed by the new
owner — the host cannot undo one and **MUST NOT** offer to.

**The host is not an arbiter.** If the parties disagree about who owns a room, ownership of
its identifier is settled by whoever controls that identifier — a matter for the DID's own
controller and its witnesses, not for a service that stores ciphertext.

**Transferring does not remove the outgoing owner.** They remain a member with whatever
authority their credentials confer, which is usually still `admin`. An owner handing over and
wanting to leave has two more things to do: narrow their own authority, and be removed from
the group. Neither happens here, and a surface that implied otherwise would leave someone
with standing they believed they had given up.

### Data carried

A room identifier, the incoming owner's identifier, and a reason. No content, on any tier.

### Correlation

A host learns that a room changed hands, and to whom. The owner is already the one visible
party on every tier, so this discloses a change to something already disclosed.

### Retention

The recorded owner persists. The `reason` is member-authored free text and a host **MUST**
treat it as untrusted for rendering and for any agent that reads it back.
