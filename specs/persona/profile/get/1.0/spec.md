---
slug: persona/profile/get
version: "1.0"
title: Persona Profile — Get
summary: A holder reads one profile, either as composed or resolved against the pool into the claims it would actually present.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, profile, projection, resolution]
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
  rationale: A resolved read returns the holder's personal values in bulk, so the agent must attribute the request to a key rather than to the session it arrived on.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A read has no durable effect and no ordering hazard; a replay returns the profile as it stands.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/profile/get:notFound
    meaning: No profile exists at the given identifier.
    retryable: false
---

## Abstract

**Persona Profile — Get** reads one profile.

It answers two different questions and the caller chooses which. *As composed*
returns the entries the holder wrote — references unresolved, the shape of the
decision. *Resolved* returns the claims the profile would actually present, with
overrides applied and pins honoured, which is what a holder means when they ask
what a profile says.

Resolution is opt-in because it is the expensive and the disclosing answer: it
decrypts values and re-derives credential-backed ones. The default returns the
composition, which is enough to edit it.

A resolved claim whose credential backing could not be re-derived comes back
carrying `stale`. A holder inspecting a profile before presenting it needs to see
that it has quietly stopped being fully presentable.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/get/1.0`, populate
`payload.profileId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Return `resolved` only when asked, and in entry order.
3. Re-derive credential-backed values during resolution, marking those it cannot as `stale` rather than returning a cached value whose backing has gone.
4. Emit `persona/profile/get:notFound` rather than an empty success for an unknown identifier — a caller that cannot tell "absent" from "empty" will treat a typo as a profile that discloses nothing.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role. A resolved read is a bulk disclosure of the holder's values,
which is precisely what a context must not be able to obtain.

## Request

`profileId` names the profile; `resolve` chooses which question is being asked.

## Response

The profile as stored, and — when resolution was requested — the claims it would
present.

## Security & Privacy

### Data carried

The request carries an identifier and a flag. The response carries the
composition always, and **personal values when `resolve` is true**. That is the
whole reason resolution is opt-in: a producer editing a profile does not need the
values, and one previewing a disclosure does.

A producer **SHOULD** request resolution only when it is about to render the
values, and **MUST NOT** persist a resolved response beyond that rendering.

### Correlation

A resolved response is the material a persona presents, so a party holding one
can recognise that persona wherever those values appear. Nothing in the document
identifies a verifier or a counterparty — a read is private, and an observer
learns that a profile was inspected, not what will be done with it.

### Retention

A response is a point-in-time view with no evidentiary value and **SHOULD NOT**
be retained. A resolved response held on disk is the same disclosure as the
profile, without the encryption the maintainer applied.

### Consent/purpose

The purpose is inspection: a holder looks at their own composition in order to
edit it or to see what it would present. The data is returned to the party it
belongs to, and is not collected here at all. What gate a producer places in
front of a resolved read is the consumer's policy and is deliberately not
specified here.
