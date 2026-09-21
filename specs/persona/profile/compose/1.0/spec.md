---
slug: persona/profile/compose
version: "1.0"
title: Persona Profile — Compose
summary: A holder composes a face for one context at the moment it is asked for, from values typed there and attributes already held, local by default, and may wear it in the same act.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - persona
  - profile
  - composition
  - context
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
  rationale: A composed face determines what a persona discloses, and may be worn by the same act. Attribution must survive the transport so an audit record names the key that composed it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Compose can bind, and a binding replaced out of order leaves the older face worn — which may disclose more than the newer one was composed to.
sideEffects:
  level: mutating
  rationale: "Creates one face, may create pool attributes for claims the holder chose to share, and may replace the persona's binding in the context. Nothing existing is edited: a pool attribute that already held the value is referenced, not changed."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: New claims carry personal values in the request. The response returns identifiers, versions and an advisory count; no value, and no other face.
errorCodes:
  - code: persona/profile/compose:unresolvedReference
    meaning: A held claim names an attribute the pool does not hold. The details name the offending `attributeId`s. Nothing is written.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["attributeIds"]
      properties:
        attributeIds:
          type: array
          maxItems: 64
          items:
            type: string
  - code: persona/profile/compose:duplicateSlot
    meaning: Two claims carry the same `slot`. The details name the slot. Nothing is written.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["slot"]
      properties:
        slot:
          type: string
  - code: persona/profile/compose:labelWithoutPersona
    meaning: A `label` was given without a `personaDid`. A label names the face to the context it is worn in, and a face composed without being worn has no context to name it to. Nothing is written.
    retryable: false
---

## Abstract

**Persona Profile — Compose** makes a face for one context, at the moment the
context asks for one.

That moment matters. An invitation, a join manifest's requested attributes, a
form: arrival in a context is the only time the holder's agent knows what is
actually being asked. Everywhere else a holder composing faces is guessing at
contexts they may one day need. This task is the step a producer offers there —
"here is what they ask; answer with a face you have, or make one now".

**Local by default.** A value typed at this step stays in this face and this
context. It enters no pool, so no other face can come to present it, and no
link between contexts is created by an act the holder took for one context.
Making a value reusable is a choice the holder makes per claim (`share: pool`),
or later, for a value already here, through persona/attribute/promote.

**Where the face lives follows from what it carries.** Every claim local: the
face is context-local, exactly as persona/local/profile/put would have made it.
Any claim pooled or drawn from the pool: the face lives above contexts, because
a face that references the pool cannot live in context-addressable space — and
it is bound into this context when `personaDid` is given. There is no scope
parameter, because a scope a producer can set independently of the claims is a
scope a producer can set wrongly.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/compose/1.0`, populate
`payload.contextId`, `payload.name` and `payload.claims`, and include a `proof`
per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming producer **SHOULD** run persona/correlation/analyze with each new
value as `candidate` while the holder is still typing, and show what it
reports before sending. That is the warning at the point it can still change the
holder's mind; the `correlation` count on the response arrives after the write.

A conforming producer **SHOULD**, before sending a `share: pool` claim, tell the
holder that the value becomes reusable across their faces.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Refuse the whole compose, writing nothing, when any held claim does not resolve (`unresolvedReference`), two claims carry one `slot` (`duplicateSlot`), or `label` is given without `personaDid` (`labelWithoutPersona`).
3. Record every new claim as self-asserted.
4. For a `local` claim, carry the value in the face itself and create no pool attribute.
5. For a `pool` claim, reference a pool attribute whose type and value equal the claim's, creating a self-asserted one only when none exists, and report which in `pooled`. It **MUST NOT** edit an attribute it reuses.
6. Store the face in `contextId`'s local address space when every claim is `local`, and above contexts otherwise, and say which in `scope`.
7. When `personaDid` is given, bind the face to that persona in `contextId` as persona/binding/set (or persona/local/binding/set, for a local face) would, replacing any face the persona wore there.
8. Leave no pool attribute it created referenced by nothing when a later step of the same compose fails.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role — even for a compose whose every claim is local, which
persona/local/profile/put already offers to a context-scoped caller. This task
can reach the pool; that one cannot, and the difference is its authorization.

## Request

`claims` is ordered, and the order is display order. A held claim presents the
attribute live; pinning and overriding are persona/profile/put's, on a face that
already exists.

## Response

`profileId` and `scope` say what was made and where it lives. `pooled` says,
for each shared value, whether an attribute was created or an existing one
reused — the second means this face now shares a value with whatever else
presents it, and a holder is owed that fact. `binding` is present when the face
was worn.

## Security & Privacy

### Data carried

The request carries personal values in new claims, and identifiers for held
ones. `name` and every `label` are the holder's own words. The response carries
no value.

### Correlation

Local by default is the correlation control. A holder who types the same email
into two contexts' faces is linked by that email whether or not either value
entered the pool — which is why the correlation index covers local values too,
and why a producer checks each value as a candidate before sending. What local
by default prevents is the *silent* link: a value made reusable for one context
turning up in another face the holder composed later.

`share: pool` finding an existing attribute with the same value does not create
the linkage — the holder already typed that value somewhere — but it does make
the two faces draw one fact, so an edit to it changes both. `pooled[].created:
false` is how the holder learns that.

### Retention

As persona/profile/put. A local face is removed with its context's other
persona data; a pool face is retained until deleted.

### Consent/purpose

Wearing a face is what carries its values into a context. Composing and wearing
in one act is offered because the moment of composition is the moment of need;
a producer **MUST** show the holder what the face will present before sending a
compose that carries `personaDid`.
