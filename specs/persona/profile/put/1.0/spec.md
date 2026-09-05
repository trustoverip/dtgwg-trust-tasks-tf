---
slug: persona/profile/put
version: "1.0"
title: Persona Profile — Put
summary: A holder composes a named projection over their attribute pool, referencing facts rather than copying them, with four entry forms covering reuse, pinning, per-profile override and profile-local values.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - persona
  - profile
  - projection
  - composition
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A profile determines what a persona will disclose. Attribution must survive the transport so that an audit record names the key that composed it, not the session it arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A put replaces a composition wholesale, so two copies applied out of order leave the older composition in place — and the older composition may disclose more than the newer one was written to.
sideEffects:
  level: mutating
  rationale: "Creates or replaces one profile. Recoverable — the prior composition is overwritten but no attribute is touched, and a conditional write cannot clobber a version it did not see."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: An `inline` or `override` entry carries a personal value directly in the request, since by definition it is not in the pool. The response returns identifiers, a version, and an advisory count of how many entries present a value another profile also presents; it returns no value and names no other profile.
errorCodes:
  - code: persona/profile/put:unresolvedReference
    meaning: An entry references an attribute the pool does not hold. The details name the offending `attributeId`s. The profile is not written — a profile with a dangling reference would silently disclose less than the holder composed.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["attributeIds"]
      properties:
        attributeIds:
          type: array
          maxItems: 256
          items:
            type: string
  - code: persona/profile/put:pinnedVersionUnavailable
    meaning: An entry pins a version of an attribute the maintainer no longer retains. The details name the attribute and the versions available, so the caller can repin rather than guess.
    retryable: false
  - code: persona/profile/put:versionConflict
    meaning: The `expectedVersion` precondition failed. Details carry the maintainer's current version.
    retryable: false
---

## Abstract

**Persona Profile — Put** composes a named projection over the holder's
attribute pool — "Work", "Gaming", "Family".

The design turns on one decision: **a profile references attributes, it does not
copy them.** A holder who changes a phone number changes it once, and every
profile presenting it follows. Copying would mean six profiles to remember and
one of them wrong at the worst moment.

Reference alone is too rigid, so there are four entry forms and each covers a
case the others handle badly. `{ref}` reuses live. `{ref, pinVersion}` freezes a
value, for a profile that must keep presenting what a counterparty already
verified. `{ref, override}` presents the same fact differently here. `{inline}`
supplies a value that never enters the pool and so can never leak into another
profile.

**Omission is exclusion.** There is no removal marker, because a profile is a
whitelist: a blacklist over a growing pool leaks by default the first time an
attribute is added, and the holder who added it would not be the one who noticed.

An `override` may replace a value and **MUST NOT** replace provenance. Allowing
it would let a self-asserted value present as attested, which is the one thing
provenance exists to prevent.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/put/1.0`, populate `payload.name`
and `payload.entries`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Resolve every `ref` against the pool and refuse the whole write with `persona/profile/put:unresolvedReference` when any does not resolve. A partially-applied composition would disclose less than the holder composed and tell them nothing about it.
3. Refuse with `persona/profile/put:pinnedVersionUnavailable` when a `pinVersion` names a version it no longer retains, naming the versions it does hold.
4. Preserve entry order.
5. Treat an `override` as replacing `value` and `label` only, inheriting `type`, `valueType` and `provenance` from the referenced attribute.

A conforming maintainer **MUST NOT** refuse a composition because of its
correlation result, and **MUST NOT** return the identifiers of other profiles in
the `correlation` member — a count is enough to warn, and identifiers would make
every profile write a disclosure of the holder's other compositions to whatever
tool made it.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role.

A profile determines what a persona discloses, so a caller that could write one
could cause a disclosure the holder never composed. That makes this task, with
`persona/binding/set`, one of the two whose authorization defect is
directly exploitable rather than merely a leak.

## Request

`entries` is ordered and the order is display order. An empty array is legal and
means a profile that discloses nothing — a reasonable starting point, and
distinct from having no profile at all.

`credentialRefs` is inventory: what this persona can prove. It is deliberately
separate from the evidence relationship a `credentialBacked` attribute expresses,
because the two answer different questions and conflating them would make an
inventory entry look like a claim.

## Response

Identifiers, the new version, and whether the write created or replaced.

`correlation` reports on the composition as a whole: how many of its entries
present a value another profile also presents. It is advisory, it is a count, and
it is returned on the write so that a builder can say something useful while the
holder is still composing rather than after a round trip they may not make.

## Security & Privacy

### Data carried

The request carries the composition. `inline` and `override` entries carry
**personal values directly**, since by definition those are not in the pool;
`ref` entries carry only identifiers. `name` and any `label` are the holder's own
words and are personal too, though never disclosed to a verifier.

The smallest request that answers the task is `name` plus the entries. A producer
**MUST NOT** place secret material in an `inline` value, a `label` or `ext` — the
vault exists for material that must never be disclosed, and this store is built to
disclose under control.

### Correlation

This is the task at which correlation risk is *created*, which is why the
response reports on it. Reusing one value across two profiles links the personas
presenting them, permanently, to anyone who sees both — and the holder is best
placed to choose otherwise at exactly this moment.

The reported figure is a count and not a list. A list would tell whatever tool
made the write which of the holder's other profiles carry the value, which is a
disclosure of the holder's compositions to that tool. A producer needing to render
remedies calls `persona/correlation/analyze`, which is equally holder-authorized
and where that reasoning belongs.

Note that an `override` does not reduce correlation. Changing a displayed value
does nothing about the credential underneath it, and a producer that scored on the
displayed value would report a false all-clear — worse than reporting nothing.

### Retention

A profile is durable holder data retained until deleted, and belongs in a
maintainer's backed-up partition: a restore without it returns an agent that can
no longer present the identity it was restoring.

A replaced composition is not evidence and need not be kept. A maintainer that
retains prior versions to serve `pinVersion` **SHOULD** say so, since a holder
who overwrites a composition may reasonably believe the previous one is gone.

### Consent/purpose

The purpose is composition: assembling, from facts the holder already holds, the
subset a particular persona will present. A profile is not itself a disclosure —
nothing leaves the agent when one is written — it is the standing decision about
what *may* leave, which a later disclosure narrows further.

What gate a producer places in front of composing, or in front of the disclosure
that follows, is the consumer's policy and is deliberately not specified here.
