---
slug: rooms/keys/commit
version: "0.1"
title: Rooms Keys — Commit
summary: "A room's owner delivers an MLS Commit to a member's key-holding agent, advancing it one epoch; every member must apply every commit, in order."
status: draft
targetFrameworkVersion: "0.5"
category: ai-agents
keywords:
  - room
  - mls
  - commit
  - epoch
  - membership
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
  rationale: "A commit advances the key schedule. One whose origin depended on the transport would let a compromised channel fork a member off the group."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Ordering is the whole contract; an undated commit cannot be reasoned about when two arrive out of order."
sideEffects:
  level: mutating
  rationale: "The recipient advances its MLS epoch and derives new keys."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Discloses to whoever routes it that this room's membership changed and when — not what changed, and not to whom."
retention:
  class: durable
  rationale: "The advanced group state persists; the commit message itself need not be kept once applied."
errorCodes:
  - code: rooms/keys/commit:epochGap
    meaning: "The commit is more than one epoch ahead. The response carries the recipient's current epoch so the sender can resume from there."
    retryable: false
  - code: rooms/keys/commit:notAMember
    meaning: "The recipient holds no group state for this room."
    retryable: false
  - code: rooms/keys/commit:commitInvalid
    meaning: "The commit did not process — malformed, or not signed by a member of the group the recipient holds."
    retryable: false
related:
  - rooms/keys/welcome
  - rooms/keys/open
  - rooms/epoch/mint
---

## Abstract

The **Rooms Keys — Commit** Trust Task delivers an MLS Commit to a member's key-holding agent.

It is the half of group delivery that is easy to forget and impossible to omit. A
[Welcome](../../welcome/0.1/spec.md) gets an agent into the group once; a commit is what keeps
it there. An agent that misses one is stuck at its last epoch and can open **nothing** sealed
after it — and the failure surfaces as "this record does not open", which reads like
corruption rather than a missed message.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Behaviour

### Strictly in order, and the response says where you are

MLS epochs advance one at a time and out of order is not a thing a recipient can repair on
its own. So:

- `epoch` equal to the recipient's current epoch is a **replay**: the recipient **MUST**
  answer success with its unchanged epoch and apply nothing. Delivery retries, and a retry
  that failed would make every unreliable transport a liveness problem.
- `epoch` exactly one ahead is applied.
- `epoch` more than one ahead is a **gap**: the recipient **MUST** answer `epochGap` and
  **MUST NOT** apply it. The response carries where it actually is, so the sender resumes
  from there rather than guessing.

The epoch is stated in the payload rather than parsed out of the commit so a recipient can
tell replay from gap **before** doing cryptographic work on a message it may not want.

### Who may commit is decided inside the group

A recipient **MUST** verify the commit against the group state it already holds — MLS
authenticates the committer as a member of that group — and **MUST NOT** decide from an
access-control list of its own. This is the same rule the rest of the room family follows:
authority comes from the room, not from whoever is hosting or holding.

The room's own policy restricts committing to the owner. That restriction is expressed and
enforced where the room's credentials are checked; it is not something a key-holder can
independently confirm, and a specification that asked it to would be asking it to duplicate a
judgement it lacks the inputs for.

### Fan-out is O(n), and saying so is the honest thing

Every member needs every commit. MLS's logarithmic property is the *size* of a commit, not
the number of recipients — a group of n members means n deliveries per change, from whoever
is doing the delivering.

On an `open` or `attributed` room the host **MAY** carry them, which is what a Delivery
Service is for. On a `private` room it **MUST NOT**, and the sender fans out directly. That
is a real cost of the private tier and it belongs stated in the specification rather than
discovered in an implementation: on that tier, membership changes need the owner online.

## Security & Privacy

**A missed commit is a silent capability loss, not an error.** The member keeps working
against everything sealed before their last epoch and fails on everything after. An
implementation **SHOULD** surface a recipient's epoch to its operator, because "we are three
epochs behind" is diagnosable and "some records will not open" is not.

**Applying a commit is not authorization to act.** It advances key material. What a member
may *do* in the room comes from the room's authority credentials, checked separately.

### Data carried

A room identifier, an MLS Commit, and the epoch it produces. The commit's contents are
readable only by group members; a router learns that the group changed, not how.

### Correlation

A router sees the timing and frequency of membership changes in a room. On a `private` room
that is one more reason the host is not on the path.

### Retention

Once applied, the commit message itself need not be retained — the advanced group state is
what matters. A recipient **MUST** discard its group state entirely on removal from the
group.
