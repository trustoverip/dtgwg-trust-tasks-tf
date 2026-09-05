---
slug: persona/local/profile/get
version: "1.0"
title: Persona Local Profile — Get
summary: Read one context-local profile.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, local, context, throwaway]
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
  rationale: The response returns values the application composed within this context; the agent must attribute the request to a key.
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

**Persona Local Profile — Get** reads a throwaway composition. It resolves
nothing against the pool, because a local profile references nothing there.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/local/profile/get/1.0` with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context and
**MUST NOT** resolve any identifier against the holder's pool.

## Authorization

**Context-scoped**, confined to the caller's own context. Safe to expose because
every object it touches lives below the boundary: a local profile holds only
values supplied to it, and a local binding names only local objects. Nothing here
can reach the holder's pool.

## Request

See the payload schema; every member carries its own rationale there.

## Response

See the payload schema.

## Security & Privacy

### Data carried

The request carries values the application itself supplied — personal data that
by definition is not in the holder's pool. The response carries identifiers and,
on a write, a correlation assessment.

A producer **MUST NOT** place secret material in an inline value or in `ext`.

### Correlation

**Local profiles are correlation-indexed like any other**, and the naive
implementation that skips them — *they are local, they do not matter* — loses the
guard exactly where a human most needs it. A throwaway identity is precisely
where somebody reuses a real value; "I will just use my normal address for this
one thing" defeats the entire purpose of the throwaway.

Indexing them is not itself a leak: the index sits above the boundary, is keyed
by a hash rather than plaintext, and only the holder can query it.

A local persona **SHOULD** default to a freshly minted pairwise identifier with
no persona credential asserted — maximally uncorrelated by construction, which is
what "throwaway" ought to mean.

### Retention

Local profiles are context data, retained until deleted or until the context is.
They are **not** promoted into the holder's pool by any automatic process:
lifting a value up is a holder action, because an application that could write to
the pool could inject an attribute that later appears in the holder's builder and
gets pushed somewhere else. Pollution is a quieter attack than exfiltration and
needs the same answer.

### Consent/purpose

The purpose is a disposable identity: an application composes something for its
own context that the holder need not curate. It is the counterweight to the
boundary — authoring below it is safe, reading across it is not.

Disclosing a local profile still runs through the disclosure path, because the
gate is on what leaves rather than on what exists. Whether a producer prompts for
a disclosure composed entirely of values it supplied itself is the consumer's
policy and is deliberately not specified here.
