---
slug: persona/binding/get
version: "1.0"
title: Persona Binding — Get
summary: An application in a context learns whether a persona has a profile bound and what the holder calls it, and never what it contains.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, binding, context, least-disclosure]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Application
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The response names the holder's composition, so the agent must attribute the request to a key rather than to the session it arrived on, and an audit record must be able to say which application asked.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A read has no durable effect and no ordering hazard.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes: []
---

## Abstract

**Persona Binding — Get** is the first of the two things an application inside a
context is permitted to do with the holder's identity, and its shape is the whole
point.

An application legitimately needs to know **which identity is in use** — to show
"signed in as Work", to decide whether to prompt. It does not need the contents to
do that. So the response carries whether a profile is bound, the holder's label
for it, an identifier the caller cannot resolve, and a **count** of available
claims. It carries no values.

The other permitted verb is to request a disclosure, where the holder sees a
preview and decides. Between them those two exhaust what a context-scoped caller
can obtain, and the consequence is worth stating plainly: **being inside a context
confers no privilege over identity data. An application in a context is a
verifier, and takes the same path as a stranger's web page.** There is no trusted
insider tier, which is fortunate, because that tier is where this class of defect
lives in every system that has one.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/binding/get/1.0`, populate
`payload.contextId` and `payload.personaDid`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Confine the caller to its own context: a caller scoped to context A **MUST NOT** learn about a binding in context B.
2. Return `bound: false` as an ordinary success for a persona with no profile — it is a normal state, not an error.
3. **MUST NOT** return any claim value, any attribute identifier, or any indication of what the profile contains beyond `claimCount`.
4. **MUST NOT** allow `profileId` to be used by a context-scoped caller to read the profile: profile reads are holder-authorized, and the identifier is returned for correlation with a later disclosure, not for dereferencing.

## Authorization

**Context-scoped**, and confined to the caller's own context. This is one of the
two context-callable tasks in the family.

It is safe to expose precisely because of what it withholds. The identifier it
returns is inert to the caller — `persona/profile/get` refuses a context-scoped
session — so the response is a statement about *arrangement*, not about content.

## Request

The context and the persona.

## Response

Whether bound; the label, identifier, claim count and time when it is.

## Security & Privacy

### Data carried

The request carries identifiers. The response carries the holder's **label** for
a composition — their own words, which may themselves be revealing ("Job hunting")
— and a count. It carries no values.

The smallest response that answers the task is `bound`. `profileName` and
`claimCount` are affordances for rendering, and a maintainer that judged a label
too revealing to hand an application **MAY** omit it; a producer **MUST NOT**
treat its absence as an error.

### Correlation

The response tells an application which of the holder's identities is in use in
its context. Across two contexts, two cooperating applications comparing
`profileId` would learn that the same composition backs both personas — which is
a real correlation channel and the reason `profileId` is an opaque identifier
rather than anything derivable, and the reason the holder is warned at bind time
when a profile is bound twice.

Nothing here reveals the holder's other contexts, other personas, or the pool.

### Retention

A point-in-time view with no evidentiary value. An application **MAY** cache
`bound` and `profileName` for a session's UI and **SHOULD** discard them after,
since a binding can be cleared at any time and a stale cache would show an
identity that is no longer in use.

### Consent/purpose

The purpose is orientation: an application shows the holder which identity it is
operating as. It is deliberately not a data-access path — the disclosure tasks
are that, and they run through a preview the holder sees.

What gate a producer places in front of the disclosure this read precedes is the
consumer's policy and is deliberately not specified here.
