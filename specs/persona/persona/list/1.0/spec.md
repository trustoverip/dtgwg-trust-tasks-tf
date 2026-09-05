---
slug: persona/persona/list
version: "1.0"
title: Persona — List
summary: An application learns which of the holder's personas operate in its own context, and nothing about anywhere else the holder operates.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, context, least-disclosure]
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
  rationale: The response names the holder's identities operating in this context, so the agent must attribute the request to a key and an audit record must be able to say which application asked.
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
errorCodes:
  - code: persona/persona/list:cursorInvalid
    meaning: The supplied cursor is unrecognised or expired. A producer restarts the enumeration.
    retryable: false
---

## Abstract

**Persona — List** is the other half of what an application inside a context is
permitted to know: which identities operate here.

It carries the same thin summary as
[`persona/binding/get`](/specs/persona/binding/get/1.0/spec.md) and for the same
reason — an application needs to know *which identity is in use*, not what it
contains. Whether a profile is bound, the holder's label for it, a claim count,
and nothing more.

`isLocal` distinguishes a persona bound to a context-local composition from one
bound to a profile the holder pushed down. That is worth surfacing because an
application should be able to tell what it authored itself from what the holder
chose to share with it, and treating the two identically is how an application
comes to assume a throwaway carries the same weight as a curated identity.

**This context only.** The enumeration is confined to the caller's context, so an
application learns the arrangement where it operates and nothing about anywhere
else the holder operates. The cross-context view is the holder's, from their own
tooling, through
[`persona/disclosure/history`](/specs/persona/disclosure/history/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/persona/list/1.0`, populate
`payload.contextId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the enumeration to the caller's own
context; **MUST NOT** return any claim value or attribute identifier; and **MUST
NOT** offer a cross-context enumeration on this task.

## Authorization

**Context-scoped**, confined to the caller's own context. Safe to expose because
of what it withholds: it describes arrangement, never content.

## Request

The context, and pagination.

## Response

One thin summary per persona.

## Security & Privacy

### Data carried

The request carries a context identifier and pagination state. The response
carries persona DIDs and the holder's **labels** for the profiles bound to them —
the holder's own words, which can themselves be revealing — plus counts. No
values.

The smallest response that answers the task is the persona DIDs and `bound`.
Everything else is a rendering affordance, and a maintainer that judged a label
too revealing to hand an application **MAY** omit it; a producer **MUST NOT**
treat its absence as an error.

### Correlation

The response shows an application which of the holder's identities operate
alongside it. Within one context that is the arrangement the application is
already part of; across contexts it would be a map of the holder's
compartmentalisation, which is why the enumeration is confined.

Two cooperating applications in different contexts could compare the persona DIDs
they each see. Because personas are per-context by construction, that comparison
yields nothing unless the holder deliberately reused one — which is the case the
correlation guard reports at bind time.

### Retention

A point-in-time view with no evidentiary value. An application **MAY** cache it
for a session's UI and **SHOULD** discard it after; bindings change, and a stale
cache shows an identity no longer in use.

### Consent/purpose

The purpose is orientation: an application shows the holder which identities it
can operate as, so they can choose between them. It is deliberately not a
data-access path — the disclosure tasks are that, and they run through a preview.

What gate a producer places in front of a disclosure this listing precedes is the
consumer's policy and is deliberately not specified here.
