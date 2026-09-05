---
slug: persona/local/binding/set
version: "1.0"
title: Persona Local Binding — Set
summary: Bind a context-local profile to a persona in the same context — safely context-callable, because both objects live below the boundary.
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
  rationale: Binding determines what a persona presents in this context; the agent must attribute it to a key.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A bind and a clear applied out of order leave the wrong one in effect.
sideEffects:
  level: mutating
  rationale: "Sets or clears one local binding. Touches nothing above the boundary."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/local/binding/set:notLocalProfile
    meaning: The identifier names a profile in the holder's pool rather than a context-local one. Honouring it would let a context-scoped caller bind the holder's composition, which is the one escalation the boundary exists to prevent.
    retryable: false
---

## Abstract

**Persona Local Binding — Set** assigns a throwaway composition to a persona.

This is **context-callable where its pool counterpart is not**, and the reason is
worth being precise about: both objects it names live below the boundary. Nothing
crosses, so nothing needs the holder.

The one obligation that carries the whole distinction is the refusal. A
`profileId` naming a **pool** profile **MUST** be refused. Honouring it would let a
context-scoped caller bind the holder's composition to a persona it controls and
read it back through a disclosure it requests of itself — which is precisely the
escalation `persona/binding/set` is holder-authorized to prevent. A maintainer
that resolved the identifier against both address spaces would reintroduce it
here, in the one task that looks harmless.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/local/binding/set/1.0`, populate `contextId`
and `personaDid`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context, and
**MUST** resolve `profileId` **only** against the context-local address space —
emitting `persona/local/binding/set:notLocalProfile` for an identifier that names
a pool profile, rather than resolving it.

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
