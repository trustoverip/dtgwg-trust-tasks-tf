---
slug: rooms/keys/open
version: "0.1"
title: Rooms Keys — Open
summary: "An agent asks the party holding its principal's room keys to open one sealed record; the plaintext comes back and the key never does."
status: draft
targetFrameworkVersion: "0.5"
category: ai-agents
keywords:
  - room
  - oracle
  - decryption
  - agent
  - key-custody
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Agent
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Oracle
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The oracle acts on its principal's key material; a request whose integrity depended on the transport would let a compromised channel choose what gets opened."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed request would re-run an operation the principal authorized once, at a time they did not choose."
sideEffects:
  level: none
  rationale: "Returns plaintext for one sealed record; stores nothing."
subjectPath: /roomId
exposure:
  discloses: secret
  ingests: none
  actsAsSubject: false
  rationale: "Returns the plaintext of one record — material the room's members can read and nobody else should. The oracle is the principal's own infrastructure, which is why it may see it."
retention:
  class: transient
  rationale: "The task returns a result and stores nothing; an oracle that cached plaintext would create a second copy of the room outside the room."
errorCodes:
  - code: rooms/keys/open:notAuthorized
    meaning: "The caller is not authorized to open records for this room."
    retryable: false
  - code: rooms/keys/open:unknownEpoch
    meaning: "The oracle holds no key for the epoch this record was sealed under."
    retryable: false
  - code: rooms/keys/open:didNotOpen
    meaning: "The record failed authentication — it was sealed under a different key, or relocated."
    retryable: false
related:
  - rooms/keys/present
  - rooms/records/get
---

## Abstract

The **Rooms Keys — Open** Trust Task opens one sealed room record on behalf of a caller that
does not hold the room's keys.

**The key never crosses.** This is a decryption oracle, not a key-release call, and the
distinction is the whole point. The caller is typically an **AI agent**: it needs to *read*
what a room holds, and it should not therefore hold — on a general-purpose machine, for as
long as the software lives — key material belonging to every other member of that room.

Two properties follow, and both are things a key-release design cannot offer:

- **Revoking the caller actually revokes it.** Withdrawing an agent's authorization at the
  oracle withdraws its access, because there is nothing else for it to keep.
- **The blast radius of a compromised caller is bounded by time**, not by the lifetime of a
  key it was given.

A failure to open is not an error to paper over: a record that does not authenticate was
sealed under a different key, or has been moved. Either way the honest answer is a refusal,
not a decryption attempted with a different key until one works.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Security & Privacy

**The key never crosses.** That is the whole design. An oracle that returned key material
would be a key-release call wearing a different name, and the caller — typically an AI agent
running on a general-purpose machine — would then hold, indefinitely, material belonging to
every other member of the room.

**Revocation must actually revoke.** Because the caller holds no key, withdrawing its access
is withdrawing its authorization at the oracle. If a caller retains anything that keeps
working afterwards, the oracle has leaked.

**The oracle sees plaintext, and that is the trade.** It is the principal's own agent
infrastructure, which is already in their trusted computing base; a host is not, which is
why the host sees only ciphertext.

**Scope every presentation.** A presentation minted for one action and one audience cannot
be replayed for another. An implementation that mints one covering every action has handed
the caller its principal's whole standing, which is the outcome attenuation exists to
prevent.

### Data carried

`open` carries one sealed record and its location; the response carries its plaintext.
`present` carries a room, an action, an audience and a nonce; the response carries a
presentation. Neither carries key material in either direction.

### Correlation

An oracle learns which records its principal's agents read and when — a complete picture of
that principal's own activity. It learns nothing about other members.

### Retention

Neither task stores anything of its own. An oracle that caches opened plaintext has created
a second copy of the room outside the room.

### Consent/purpose

The caller acts for the principal whose keys the oracle holds, within whatever authorization
the principal granted it. That grant is what an implementation checks; this specification
defines the shape of the request, not the policy.

Where that grant is expressed as a device capability, the registered value is **`roomOpen`**
(`room-open` in the `0.1` casing) — see `Capability` in
[`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is separate
from `roomPresent` on purpose: producing a presentation and decrypting a record are different
powers, and an agent that indexes a room should not thereby be able to read it. Both are
separate from `sign`, which would grant strictly more than either task needs.
