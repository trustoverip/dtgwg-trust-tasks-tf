---
slug: persona/local/profile/put
version: "1.0"
title: Persona Local Profile — Put
summary: An application composes a throwaway profile inside its own context from values it supplies, with pool references refused — which is what keeps the local surface pool-free.
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
  rationale: A local profile determines what a persona in this context presents, so the agent must attribute the composition to a key.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A put replaces a composition wholesale, so two copies applied out of order leave the older one in effect.
sideEffects:
  level: mutating
  rationale: "Creates or replaces one context-local profile. Touches nothing above the boundary."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: The request carries personal values the application supplies, since by definition they are not in the holder's pool. The response returns identifiers and a correlation assessment, and no values.
errorCodes:
  - code: persona/local/profile/put:referenceNotPermitted
    meaning: An entry attempted to reference a pool attribute. Local profiles are inline-only, and honouring a reference would let a context-authored object acquire pool reach.
    retryable: false
---

## Abstract

**Persona Local Profile — Put** composes a throwaway identity inside a context.

Authoring **below** the boundary is safe, and forbidding it would be over-applying
a rule that exists to stop reading **across** it. A local profile holds only
values the application supplied; it reads nothing from the holder's pool, which is
the opposite direction from the one the boundary guards.

The isolation is **structural rather than a check**. Local profiles live in their
own address space, so a context-scoped enumeration scans somewhere that cannot
contain a pool profile — and the schema admits only inline entries, so a reference
is not a rejected input but an unrepresentable one.

A local entry also carries no `provenance`, and that absence is a rule rather
than an oversight. The inline object here is `{type, valueType, value, label?}`
— narrower than a pool profile's inline entry, which requires `provenance` —
because a `credentialBacked` provenance names a `credentialId` and a
`claimPath`, and a value authored inside a context has nowhere to put either. A
context-local value is therefore **self-asserted by construction**, and that is
what a maintainer presents it as.

This is the same boundary the missing `ref`, pinned and override forms enforce,
one member along. Those stop a context-authored object acquiring pool *reach*;
this stops it acquiring an issuer's *authority* — asserting that a value is
attested when no credential was ever checked, over a value the issuer never saw.
A holder who needs a context to present an attested claim binds a pool profile,
which is holder-authorized, rather than authoring one here. Adding a
`provenance` member to this object would be a privilege escalation dressed as a
convenience.

The one thing that must not be inferred from "local" is "unimportant". These
profiles are correlation-indexed like any other, because a throwaway identity is
exactly where somebody reuses a real value.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/local/profile/put/1.0`, populate
`contextId`, `name` and `entries`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context;
**MUST** store local profiles in an address space distinct from the holder's
pool; **MUST** refuse any entry that references a pool attribute; **MUST**
include local values in the holder's correlation index; and **MUST** treat every
local value as `selfAsserted`, presenting it as such wherever a provenance is
required.

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
