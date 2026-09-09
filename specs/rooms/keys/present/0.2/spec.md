---
slug: rooms/keys/present
version: "0.2"
title: Rooms Keys — Present
summary: "An agent asks the party holding its principal's room credentials to produce a presentation for one room operation, scoped to that action and granted to the agent that asked."
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
  rationale: "The oracle acts on its principal's key material; a request whose integrity depended on the transport would let a compromised channel choose what gets opened. The proof is also what establishes the producer identity the minted presentation is granted to — see Presenter binding."
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

**A presentation is scoped to the action it was asked for, and granted to the caller that
asked.** An implementation that mints one covering every action has handed the caller its
principal's whole standing in the room — which is precisely the outcome attenuation exists
to prevent, and the reason `action` is a required member rather than an optional hint.

An oracle **MUST NOT** produce a presentation conferring an action the principal's own
credentials do not confer. A caller cannot acquire authority by asking for it, and
`actionNotHeld` says so plainly rather than returning something that will fail at a
verifier for reasons the caller cannot diagnose.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

### What changed from 0.1

`0.1` carried two payload members that no verifier could act on, both filed as
[#414](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues/414). Neither survives.

- **`audience` is removed.** `0.1` described it as *"the party the presentation is for…
  a host's identifier, normally"*, and an implementation that filled it that way minted a
  presentation every host refused. The property it fed — an authority credential's
  `audience` — was defined by the DTG Core Credentials specification as *the presenter*,
  and it has since been removed from that specification outright, because a presenter is
  established by [Presenter binding](#presenter-binding) rather than named in a field. The
  destination binding `0.1` was reaching for is real and is provided by the framework: see
  [Where a presentation may be sent](#where-a-presentation-may-be-sent).
- **`nonce` is removed.** `0.1` said it was *"echoed into the presentation"*. A presentation
  has no member to echo it into — `AuthorityPresentation` is closed, and adding one would
  not help, because nothing in a presentation is signed by the party presenting it. See
  [Freshness](#freshness) for what actually bounds replay.

`0.2` is a breaking change to a `draft` specification and is therefore a `MINOR` increment
per [SPEC.md §5.2](/SPEC.md#52-compatibility-rules). A caller upgrading drops both members;
nothing replaces them, because the framework already carries what they were reaching for.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Presenter binding

**The presentation is granted to the caller, and only the caller can use it.** The
authority chain the oracle returns has a leaf whose subject is the requesting agent, and a
host **MUST** refuse a chain whose leaf grants to anyone but the party it authenticated for
that request. A verified chain is evidence that authority was conferred on somebody; it is
**not** evidence that the party presenting it is that somebody, and a host that conflates
the two authorizes every captured presentation.

This is why the request has no member naming who may present. An oracle **MUST** grant the
leaf to the producer of the request — the party identified by `issuer`, or by the transport
where [SPEC.md §4.8.1](/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)
permits it — never to a party named in the payload. A caller that could name the subject
could ask for a presentation usable by somebody else, which is a credential-release call
wearing a different name.

Producers and consumers of the room tasks that *carry* the resulting presentation should
read this alongside `AuthorityPresentation` in
[`rooms/_shared`](../../../_shared/0.1/room.schema.json): binding presenter to leaf is the
host's obligation, and it is the one an implementation is most likely to omit, because a
chain that verifies looks like an answer.

## Where a presentation may be sent

A presentation is bound to a **room**, by the `scope` of every credential in its chain, and
to a **presenter**, by its leaf. It is deliberately not bound to a host.

That is not a gap. A chain rooted in a room confers nothing anywhere else, and any host
serving that room would honour the same chain — a room **MAY** have more than one host, and
moving between them without reissuing credentials is the property the rooms design exists
to keep. There is no cross-host replay to prevent: the only party who can present the chain
is the party it was granted to, and the only place it means anything is the room it names.

Binding a *request* to its destination is a separate question, and the framework answers it.
Every room task that carries a presentation requires `proof`, and
[SPEC.md §4.8.2](/SPEC.md#482-audience-binding) then requires an in-band `recipient` that
the proof covers. That is the audience binding — at the layer that can enforce it, over
bytes a signature protects.

## Freshness

Nothing in the presentation carries a challenge, and nothing needs to.

A presentation is a bundle of credentials, each signed by its *issuer* — the room, or the
principal. None is signed by the party presenting it, so a value placed inside the bundle
is unauthenticated: an attacker replaying a captured presentation copies the challenge along
with everything else. A nonce is only worth anything when the party answering it signs it.

The party presenting *does* sign, one layer up: the room task document carrying the
presentation. That document carries `id` ([§4.3](/SPEC.md#43-the-id-member)), `issuedAt`
(**REQUIRED** of every consequential room task per
[§7.3 item 17](/SPEC.md#73-specification-requirements)), `recipient`, and a `proof` over all
of it. Replay is closed there, by
[§7.2 item 11](/SPEC.md#72-consumer-requirements) — duplicate-execution protection keyed on
the document `id`, bounded by the acceptance window over `issuedAt`. Freshness is a property
of the request, not of the credentials it carries.

Presentations are additionally short-lived: `expiresAt` bounds the window in which one is
accepted at all, and an oracle **SHOULD** mint leaves that live hours rather than days. That
is what makes withdrawing a caller's access at the oracle actually withdraw it.

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

**Scope every presentation.** A presentation minted for one action cannot be reused for
another, and one minted for one caller cannot be used by a second. An implementation that
mints one covering every action has handed the caller its principal's whole standing, which
is the outcome attenuation exists to prevent.

### Data carried

`open` carries one sealed record and its location; the response carries its plaintext.
`present` carries a room and an action; the response carries a presentation. Neither carries
key material in either direction.

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
deliberately separate from `sign`: an agent that may ask for a scoped presentation granted
to itself is not thereby an agent that may sign anything at all with its principal's key,
and gating this on the generic signing oracle would grant strictly more than the task needs.
It is separate from `roomOpen` for the same reason in the other direction — producing a
presentation and decrypting a record are different powers, and an agent that indexes a room
should not thereby be able to read it.
