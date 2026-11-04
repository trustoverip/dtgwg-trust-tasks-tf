---
slug: rooms/keys/welcome
version: "0.1"
title: Rooms Keys — Welcome
summary: "A room's owner delivers an MLS Welcome to the agent that will hold a new member's room keys, authorized by the invitation the room already issued."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords:
  - room
  - mls
  - welcome
  - membership
  - consent
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The message carries a group's secrets. A recipient that accepted one whose origin depended on the transport would let a compromised channel choose whose keys it holds."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed Welcome would re-add key material for a group the recipient may since have left."
sideEffects:
  level: mutating
  rationale: "The recipient joins an MLS group and retains its key material."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: "The response carries an epoch and nothing else. The request carries a group's secrets, sealed to the recipient's KeyPackage — material the recipient must then protect on every other member's behalf, which is what makes this `secret` rather than merely personal. Separately, and captured by neither axis: routing a Welcome discloses to whoever carries it that this key-holder is joining this room, so on a `private` room it MUST NOT go through the host. See Security & Privacy."
retention:
  class: durable
  rationale: "Group key material persists for as long as the membership does; that is the point."
errorCodes:
  - code: rooms/keys/welcome:notInvited
    meaning: "The recipient holds no valid, unconsumed invitation from this room for this party."
    retryable: false
  - code: rooms/keys/welcome:alreadyJoined
    meaning: "The recipient already holds current group state for this room for the member the invitation names — state that no commit it has processed since removed the member from. State from which a processed commit removed the member is a prior membership, not a join, and state it holds for another member it serves is not a join by this one."
    retryable: false
  - code: rooms/keys/welcome:welcomeInvalid
    meaning: "The Welcome did not process — wrong KeyPackage, malformed, or missing the ratchet tree."
    retryable: false
related:
  - rooms/keys/key-package
  - rooms/keys/commit
  - rooms/keys/open
---

## Abstract

The **Rooms Keys — Welcome** Trust Task delivers an MLS Welcome to the agent that will hold a
member's room keys, so it can join the room's group.

It exists because [`rooms/keys/open`](../../open/0.2/spec.md) assumes something nothing
specified: that a key-holding agent *has* the room's group. An oracle that opens records
cannot open anything until a group arrives, and a group arrives exactly once, in a Welcome.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Behaviour

### The invitation is what makes a Welcome acceptable

A Welcome carries a group's secrets. Anyone who can reach a key-holding agent could otherwise
push group state into it — filling its storage at best, and at worst making it hold keys for
a room nobody consented to join.

So a recipient **MUST** refuse a Welcome for which it holds no valid, unconsumed invitation
credential naming the joining party and issued by `roomId`, and **MUST** answer `notInvited`.
This is not a new gate: joining a room is already a two-party act, the invitation is already
the consent artefact, and this task is where that consent becomes load-bearing rather than
ceremonial. A recipient that accepts an uninvited Welcome has made the invitation decorative.

An invitation **MUST** be consumed on a successful join. It names its subject and is
single-use; a second Welcome under the same invitation is `notInvited`, not a second join.

### Joining twice is refused, not merged

A recipient already holding **current** group state for `roomId` for the member the
invitation names **MUST** answer `alreadyJoined`. Group state is current from the Welcome that
formed it until the recipient processes a commit that removes the member from the room's
group. A key holder serving several members — several principals, or several personas of
one — keeps one current group state per member per room; state it holds for another member is
not a join by this one and **MUST NOT** cause `alreadyJoined`. Two current group states for
one member in one room is a condition nothing downstream can resolve: both cover the same
epochs, an [`open`](../../open/0.2/spec.md) call has nothing to choose between them by, and
choosing wrong returns "did not open" for a record the member can plainly see.

### Rejoining after a removal

Leaving and rejoining is a removal followed by a fresh invitation, not a second Welcome under
the first one ([`rooms/owner/group/remove`](../../../owner/group/remove/0.4/spec.md),
*Re-admitting someone removed*; [`rooms/owner/revoke`](../../../owner/revoke/0.1/spec.md),
*Re-admitting a party*). The fresh invitation is answered by a Welcome like any other, and
this task is where it arrives.

A removal does not take the member's group state away. Once the recipient processes a commit
that removes the member, the state it held becomes a **prior membership**: the member's state
for the epochs at or after the one it joined at and below the epoch of the removal. A prior
membership is not a join and **MUST NOT** cause `alreadyJoined`. A recipient holding only
prior memberships for `roomId` for the member the invitation names **MUST** process a Welcome
that arrives under a valid, unconsumed invitation exactly as it would a first join, and the
group state that Welcome forms is the member's current state. It **MUST** keep each prior
membership beside it, unmerged, for records sealed under the epochs that membership covers
(*Retention*).

A member may have several prior memberships, one per removal, each covering the epochs at or
after its own join epoch and below its own removal epoch. They never overlap one another or
the current state: a member is removed before it is re-admitted, and the commit admitting it
again comes after the one removing it, so every join epoch is at or after the removal epoch
before it. For any epoch, therefore, at most one of a member's states covers it, and
[`open`](../../open/0.2/spec.md) chooses by the record's epoch rather than between states. An
epoch at or after a removal epoch and below the next join epoch is covered by none: the
member was not in the group when it was sealed.

### The response says which epoch

The joiner reports the epoch it joined at, because the sender needs it. A member added at
epoch 7 has none of the commits that produced epochs 1–7 and does not need them; but the
sender must know not to start replaying from 1, and must know where to begin the
[commit](../../commit/0.1/spec.md) stream if the group has moved on since.

## Security & Privacy

**Routing a Welcome discloses membership.** Whoever carries it learns that this key-holder is
joining this room. On an `open` or `attributed` room that is already visible and the room's
host **MAY** carry it. On a `private` room it **MUST NOT** be routed through the host: the
host was built never to learn the membership, and handing it every join is handing it the
membership one message at a time. Deliver it directly — the same rule, and the same reason,
as the invitation itself.

**The recipient is a key-holder, not a host.** It is the member's own agent infrastructure,
already in their trusted computing base. A host is not, which is why this task is addressed
to one and never the other.

**A Welcome is not authorization.** Holding a group's keys lets an agent *decrypt*; it confers
no standing to act in the room. That comes from the room's authority credentials, verified
separately. An implementation that treated group membership as permission would have replaced
a credential check with a key check.

### Data carried

A room identifier, an MLS Welcome, optionally a ratchet tree, and an invitation credential.
The Welcome's secrets are sealed to the recipient's KeyPackage and readable by nobody else,
including whoever routes it.

### Correlation

Anyone routing this learns a joining event: that identifier, that room, that time. This is
the strongest correlation signal in the room family, which is why the private tier removes
the host from the path entirely rather than relying on it not to look.

### Retention

The recipient retains group key material for as long as the membership lasts, and after a
removal keeps it as a prior membership (*Rejoining after a removal*). A removal is
forward-only: what it takes from the member is every epoch from the removal on, which no state
the member holds can reach, because no later epoch derives from one the member had. It does
not take back what was sealed while the member was in the group — the member could open that
then, and a recipient that discarded the state would not unread it, only turn a record the
member had already seen into `didNotOpen`. The recipient **MUST** keep a prior membership's
state for records sealed under the epochs it covers, subject to the room's epoch floor: the
room erases its history below an epoch with [`rooms/epoch/prune`](../../../epoch/prune/0.2/spec.md),
and a key holder discards what it holds below the floor when it learns of it, prior
memberships included.

### Consent/purpose

The invitation is the basis, and it is what makes this the completion of a two-party act
rather than an imposition: the room offered, the party answered by minting a KeyPackage, and
the Welcome is the owner acting on that answer. A Welcome carrying no invitation the room
actually issued is not a join, which is what the authorization check settles.

Consent extends to the recipient and no further. The addressee is the member's own
key-holding agent, inside their trusted computing base, which is why this task is never
addressed to a host and why on a `private` room it **MUST NOT** be routed through one. A
member who accepted an invitation consented to their own infrastructure holding the group's
keys. They did not thereby consent to the room's host learning that they are in it, and
routing a join through the host discloses precisely that, one member at a time.

Holding the keys is not permission to act. The purpose here is decryption capability,
standing comes from the room's authority credentials, and an implementation that conflated
them would have granted by delivery what it meant to grant by credential.
