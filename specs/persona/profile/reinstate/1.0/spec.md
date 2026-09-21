---
slug: persona/profile/reinstate
version: "1.0"
title: Persona Profile — Reinstate
summary: A holder makes a retired face wearable again. It comes back worn nowhere — the bindings retiring cleared are not restored.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - profile
  - lifecycle
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
  rationale: Reinstating makes a face wearable again. Attribution must survive the transport so an audit record names the key that did it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Reinstate races persona/profile/retire; an issue time lets an agent order the two.
sideEffects:
  level: mutating
  rationale: "Marks a retired face active. Binds nothing."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: Identifiers in, identifiers out.
errorCodes:
  - code: persona/profile/reinstate:versionConflict
    meaning: The `expectedVersion` precondition failed. Details carry the current version.
    retryable: false
---

## Abstract

**Persona Profile — Reinstate** undoes persona/profile/retire: the face appears
in pickers again and can be worn.

It comes back **worn nowhere**. The bindings retiring cleared are not restored,
because wearing a face in a context is a decision about that context, and the
holder takes it again, in that context, when they mean it. A reinstate that
silently re-presented a face to every context it had left would turn "I might
use this again" into "I am back" everywhere at once.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/reinstate/1.0`, populate
`payload.profileId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Treat a `profileId` that names no face — in the pool, or in `contextId` when given — as not found.
3. Mark the face `active` and remove `retiredAt`.
4. Bind nothing.
5. Treat reinstating an active face as a success that changes nothing and takes no new version.

## Authorization

**Holder-authorized and unscoped.**

## Request

As persona/profile/retire.

## Response

The face's id and version.

## Security & Privacy

### Data carried

Identifiers only.

### Correlation

Nothing leaves. Linkage returns only when the holder wears the face again,
where persona/binding/set reports it.

### Retention

Unchanged.

### Consent/purpose

Undoing a retire the holder chose, without deciding for them where the face is
worn.
