---
slug: rooms/keys/present
version: "0.1"
title: Rooms Keys — Present
summary: "An agent asks the party holding its principal's room credentials to produce a presentation for one room operation, scoped to that action and audience."
status: draft
targetFrameworkVersion: "0.5"
category: ai-agents
keywords:
  - room
  - oracle
  - presentation
  - agent
  - attenuation
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
  rationale: "Produces a presentation; stores nothing."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns a presentation naming a room and an action. On a room that withholds the subject the presentation discloses no member identifier; on one that does not, it discloses the principal's."
retention:
  class: transient
  rationale: "The task returns a result and stores nothing; an oracle that cached plaintext would create a second copy of the room outside the room."
errorCodes:
  - code: rooms/keys/present:notAuthorized
    meaning: "The caller is not authorized to present for this room, or not for this action."
    retryable: false
  - code: rooms/keys/present:actionNotHeld
    meaning: "The principal's own credentials do not confer the requested action."
    retryable: false
related:
  - rooms/keys/open
  - rooms/records/put
---

## Abstract

The **Rooms Keys — Present** Trust Task produces a presentation for one room operation,
on behalf of a caller that does not hold the credentials behind it.

The counterpart to [`rooms/keys/open`](../../open/0.1/spec.md): that one keeps key material
from crossing to an agent, this one keeps *credentials* from crossing.

**A presentation is scoped to the action and audience it was asked for.** An implementation
that mints one covering every action, for any audience, has handed the caller its
principal's whole standing in the room — which is precisely the outcome attenuation exists
to prevent, and the reason `action` is a required member rather than an optional hint.

An oracle **MUST NOT** produce a presentation conferring an action the principal's own
credentials do not confer. A caller cannot acquire authority by asking for it, and
`actionNotHeld` says so plainly rather than returning something that will fail at a
verifier for reasons the caller cannot diagnose.

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

Where that grant is expressed as a device capability, the registered value is
**`roomPresent`** (`room-present` in the `0.1` casing) — see `Capability` in
[`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is
deliberately separate from `sign`: an agent that may ask for a scoped, audience-bound
presentation is not thereby an agent that may sign anything at all with its principal's key,
and gating this on the generic signing oracle would grant strictly more than the task needs.
It is separate from `roomOpen` for the same reason in the other direction — producing a
presentation and decrypting a record are different powers, and an agent that indexes a room
should not thereby be able to read it.
