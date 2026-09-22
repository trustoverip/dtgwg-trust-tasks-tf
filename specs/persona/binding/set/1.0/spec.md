---
slug: persona/binding/set
version: "1.0"
title: Persona Binding — Set
summary: A holder assigns a profile to a persona DID in a context, or clears it; this is the push that carries a composition across the context boundary, and it is the family's critical authorization gate.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords: [persona, binding, context, authorization, correlation]
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
  rationale: Binding determines what a persona presents and moves a composition across a trust boundary. Attribution must survive the transport, because this is the operation whose misuse is directly exploitable rather than merely a leak.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A bind and a clear applied out of order leave the wrong one in effect, and the wrong one may be the bind — a persona presenting a composition the holder had already withdrawn.
sideEffects:
  level: mutating
  rationale: "Sets or clears one binding and materialises the profile into the context. Recoverable — clearing restores the unbound state — but a materialised projection already read by an application cannot be un-read."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/binding/set:profileNotFound
    meaning: The named profile does not exist. The binding is not written; a binding to a missing profile would present nothing while appearing configured.
    retryable: false
  - code: persona/binding/set:contextNotFound
    meaning: OPTIONAL diagnostic for a maintainer whose authorization model can distinguish "no such context" from "not permitted to reach it". Where it cannot, the framework's standard permissionDenied is the conforming answer to both.
    retryable: false
  - code: persona/binding/set:versionConflict
    meaning: The expectedVersion precondition failed.
    retryable: false
  - code: persona/binding/set:profileRetired
    meaning: The named face is retired. It is not worn until persona/profile/reinstate — a retired face is one the holder has stopped being, and binding it back by accident would undo that.
    retryable: false
  - code: persona/binding/set:untilNotFuture
    meaning: "`until` is not in the future, or accompanies a null `profileId`. Nothing is written."
    retryable: false
  - code: persona/binding/set:noPersonaHere
    meaning: "`personaDid` was omitted and the holder has no persona in `contextId` — no DID has a binding there. Nothing is written. A persona is minted first, through the DID-template path; wearing a face never mints one."
    retryable: false
  - code: persona/binding/set:personaAmbiguous
    meaning: "`personaDid` was omitted and the holder has several personas in `contextId`. The details name them. Nothing is written — choosing one would decide which of the holder's identities this context sees."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["personaDids"]
      properties:
        personaDids:
          type: array
          maxItems: 256
          items:
            type: string
  - code: persona/binding/set:outsideReach
    meaning: The face's `reach` does not include this context. Nothing is written. The holder widens the reach with persona/profile/put, or wears another face here.
    retryable: false
---

## Abstract

**Persona Binding — Set** assigns a profile to a persona DID within a context.

This is the task where the family's boundary rule is actually enforced. The
attribute pool and profiles are **agent-scoped**, above every context; bindings
are **context-scoped**. Setting a binding is the moment a composition crosses
from one to the other, and the crossing has a direction: **the holder pushes a
materialised projection down, and a context never pulls.**

What lands in the context is the resolved claim set, flat, with no back-reference
into the pool. That is what makes the boundary hold under compromise: an attacker
with administrative access to the context sees exactly what was pushed, and
nothing there leads anywhere else. The rest of the pool is not merely forbidden
to them — it is absent.

**`profileId: null` is a first-class value.** A persona with no profile is a
legitimate and common state, and the schema says so rather than leaving a
consumer to infer it from an absent member.

**Correlation is scored here as well as at composition**, because composing is
hypothetical and binding is when a value actually crosses into a context.
Binding one profile to a second persona is reported at `high` unconditionally:
that act makes the two personas the same person by construction, and no
subsequent narrowing undoes it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/binding/set/1.0`, populate
`payload.contextId` and `payload.personaDid`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming producer **SHOULD** surface a `correlation.severity` of `high` to
the holder, and **MUST NOT** populate `publicEntries` other than at the holder's
explicit instruction.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped** — see [Authorization](#authorization).
2. Refuse with `persona/binding/set:profileNotFound` when the profile does not exist, rather than writing a binding that presents nothing while appearing configured.
3. Materialise the resolved claim set into the context, **without** any reference back into the pool.
4. Re-materialise the projection when an attribute it depends on changes, so that "edit once, everywhere" holds — a write initiated above the boundary, never a read from below.
5. Publish nothing beyond `publicEntries`, and treat an empty or absent `publicEntries` as publishing nothing.
6. Record an audit event naming the context, the persona and the claims that crossed.

`label` is what the context may call the face this persona wears there. A
maintainer **MUST** return it, and not the face's own name, from
`persona/binding/get` and `persona/binding/list` to a caller that is not
holder-authorized. A face's name is the holder's filing — "Job hunting", "the
divorce" — and binding a face to a persona is not consent to tell a context what
the holder calls it. An absent `label` gives the context no name at all, which
is a legitimate choice rather than a gap.

When `personaDid` is omitted, a conforming maintainer **MUST** use the persona
the holder already uses in `contextId` — the one DID with a binding record
there, current or cleared — and name it in the response; **MUST** refuse with
`noPersonaHere` when there is none, and with `personaAmbiguous`, naming them,
when there are several. It **MUST NOT** mint a persona here: a DID has a
lifecycle of its own — keys, hosting, services — and a write that could half
create one would leave a published identity nobody holds.

A conforming maintainer **MUST** refuse to bind a retired face, with
`persona/binding/set:profileRetired`, and a face whose `reach` does not include
`contextId`, with `persona/binding/set:outsideReach`.

A conforming maintainer **MUST** refuse, with `persona/binding/set:untilNotFuture`, an
`until` that is not in the future or that accompanies a null `profileId`. At
`until` it **MUST** clear the binding exactly as a null `profileId` would, and
then, when the face is worn nowhere, retire it as persona/profile/retire would —
never delete it. A face still worn elsewhere stays active: the expiry was about
this context, and retiring would take the face off contexts the holder said
nothing about. A maintainer **MAY** act on an expiry late, but **MUST NOT**
disclose a face through a binding whose `until` has passed.

## Authorization

**Holder-authorized and unscoped**, and this is the task where that matters most.

An application able to call this could **bind any profile to a persona it
controls** and then read the result back through a disclosure it requests of
itself. Every other read in the family leaks; this one is directly exploitable.

The check is on the **scope** axis and not the role axis. A guard written as "is
this caller an administrator" **passes for an administrator scoped to a single
context**, who could then bind the holder's compositions to personas in their own
context. An administrator in one context **MUST** be as powerless here as an
application in that context.

## Request

`contextId` and `personaDid` address the binding; `profileId` names the profile
or `null` clears it.

`publicEntries` is empty by default and **MUST** stay empty unless the holder
asks. A published attribute is one document every relying party sees identically
— a permanent correlation point, and the thing a per-verifier projection exists
to avoid. Offering it at all is a concession to holders who want a thin public
card; making it opt-in per attribute is what keeps that concession small.

## Response

Identifiers, the new version, a count of claims materialised, and the advisory
correlation result.

`materialisedClaimCount` is a count so the holder can see that a push happened
and how large it was, without the response restating the values that just
crossed.

## Security & Privacy

### Data carried

The request carries identifiers only — no values. The **effect** is what moves
personal data: the maintainer materialises the profile's resolved claims into the
context, where an application in that context can obtain them through a
disclosure.

The response carries counts and no values. A maintainer **MUST NOT** return the
materialised claims here; a producer that needs them reads the profile.

### Correlation

Binding is the moment correlation becomes real rather than hypothetical, which is
why it is scored here. Two figures matter and only one is returned: the severity,
and a **count** of other personas bound to the same profile. The identifiers of
those personas are not returned, because that association is precisely what an
attacker wants and a count is enough to warn.

`publicEntries` is the one path in this family that creates a *permanent* and
*universal* correlation point rather than a per-verifier one. A maintainer
**SHOULD** make that asymmetry visible to the holder.

### Retention

A binding is durable holder data retained until cleared, and belongs in the
backed-up partition. The materialised projection lives in the context and is
retained for as long as the binding stands; clearing the binding **MUST** remove
it, though a maintainer **MUST NOT** represent that as undoing a disclosure — an
application that already read the projection has it, and only future reads stop.
Minimisation at push time is the control; clearing is cleanup.

### Consent/purpose

The purpose is assignment: the holder decides that this persona, in this context,
presents this composition. The data crosses a trust boundary because the holder
pushed it, which is the only way it crosses at all.

What gate a producer places in front of binding — and it is the operation most
deserving of one — is the consumer's policy and is deliberately not specified
here.
