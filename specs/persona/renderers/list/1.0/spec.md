---
slug: persona/renderers/list
version: "1.0"
title: Persona Renderers — List
summary: Enumerate the output formats an agent can present in, and what each discards, so a client negotiates format rather than discovering lossiness afterwards.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, privacy, correlation]
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
  rationale: The response describes the agent's capabilities rather than the holder's data, but it precedes a disclosure and the audit trail should be able to name the application that negotiated the format.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A capability read has no durable effect and no ordering hazard.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
errorCodes: []
---

## Abstract

**Persona Renderers — List** enumerates the output formats this agent can present
in.

It exists because **a client cannot negotiate a format it cannot discover**, and
because the interesting property of a renderer is not what it produces but what it
**discards**. Most general-purpose contact formats have nowhere to put provenance,
so a disclosure rendered through one arrives at the verifier with the values intact
and no way to tell an assertion from an attestation.

That loss is legitimate — it is what interoperating with an older format costs —
but it **MUST** be declared rather than discovered. Enumerating it here is what
lets a preview tell the holder *this verifier will see your work number but not
that your employer attested it* before they decide.

`canCarryPredicates` is the sharpest case. A claim proven rather than shown has no
value to render, and most formats have no field for it. A disclosure containing a
predicate through such a renderer fails at negotiation rather than silently
dropping the claim — a verifier receiving fewer claims than were approved cannot
tell that from a holder who approved fewer.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/renderers/list/1.0` with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** mark exactly one renderer `canonical`;
**MUST** declare `drops` honestly, including an empty array for a lossless
renderer; and **MUST NOT** offer a renderer here that a disclosure would refuse.

A conforming maintainer **MUST NOT** add capability to a renderer at disclosure
time that it did not declare here. A renderer maps; it never decides what is
disclosed. A renderer that could add, infer or expand a field would let a
formatting change silently widen a disclosure — a defect that lives in a mapping
table where nobody looks for one.

## Authorization

**Context-scoped**, confined to the caller's own context. The response describes
the agent's capabilities and carries nothing about the holder.

## Request

See the payload schema; every member carries its own rationale there.

## Response

See the payload schema.

## Security & Privacy

### Data carried

Neither document carries personal data. The request is empty but for extensions;
the response describes formats.

### Correlation

Nothing here identifies the holder. A maintainer's renderer set is a
fingerprinting surface in the weak sense that an unusual configuration
distinguishes one agent from another, which is a property of the deployment rather
than of the person.

### Retention

A capability listing is stable and **MAY** be cached by a producer for the life of
a session. A maintainer that changes its renderer set **SHOULD** expect stale
clients to request a renderer that no longer exists, which the disclosure path
refuses cleanly.

### Consent/purpose

The purpose is format negotiation before disclosure. The data is about the agent,
not the holder, and reusing it beyond choosing a format has no meaning.
