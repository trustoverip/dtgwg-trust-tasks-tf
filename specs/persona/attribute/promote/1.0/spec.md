---
slug: persona/attribute/promote
version: "1.0"
title: Persona Attribute — Promote
summary: A holder makes values typed into a context-local face reusable across their faces. The face moves above contexts, keeping its id and every place it is worn; the step is one-way.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - attribute
  - profile
  - scope
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
  rationale: Promotion moves a value out of one context into the holder's pool, where every other face can reach it. Attribution must survive the transport so an audit record names the key that widened it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Promotion addresses entries by position in a face that may be edited concurrently; an issue time lets an agent order the two.
sideEffects:
  level: mutating
  rationale: "Creates pool attributes (or references existing ones), replaces a context-local face with a pool face of the same id and the same presented values, and rebinds the personas that wore it. Not reversible — see Abstract."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The request carries identifiers and positions; the values moved are already held. The response returns identifiers only.
errorCodes:
  - code: persona/attribute/promote:entryOutOfRange
    meaning: A position is beyond the end of the face's entries. The details carry the face's entry count. Nothing is written.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["entryCount"]
      properties:
        entryCount:
          type: integer
          minimum: 0
  - code: persona/attribute/promote:versionConflict
    meaning: The face's version is not `expectedVersion`. Details carry the current version. Nothing is written.
    retryable: false
---

## Abstract

**Persona Attribute — Promote** makes a value the holder typed for one context
reusable across their faces.

persona/profile/compose keeps a typed value local by default, so that nothing
typed for one context reaches another by accident. This task is the deliberate
act that lets it: "use this address in my other faces too".

**The face moves with the value.** A face that references the pool cannot live
inside one context — that is the boundary, not an implementation detail — so
promoting any entry moves the whole face above contexts. The maintainer:

1. creates a pool attribute for each promoted value (or references a
   self-asserted one that already holds exactly that type and value);
2. writes a pool face with the same id and the same entries, the promoted ones
   now references, the rest still carried inline;
3. rebinds every persona in the context that wore the local face to the pool
   face — what each presents is unchanged;
4. removes the local face.

**It is one-way.** Demotion cannot be defined: by the time a holder wants it,
the pool attribute may be presented by other faces, and removing it would
silently change what they present. A producer **MUST** tell the holder that
the value becomes reusable across their faces, and **MUST NOT** offer an undo.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/attribute/promote/1.0`, populate
`payload.contextId`, `payload.profileId`, `payload.entries` and
`payload.expectedVersion`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Refuse with `versionConflict` when the local face's version is not `expectedVersion`, and with `entryOutOfRange` when a position is beyond its entries, writing nothing.
3. Treat a `profileId` that names no context-local face in `contextId` as not found. A pool face is already reusable and is not promoted.
4. Record each created attribute as self-asserted, whatever the entry's provenance said — a context-local value is self-asserted by construction.
5. Keep the face's id, name, order, slots and labels, and every entry not promoted exactly as it was.
6. Leave every persona that wore the local face wearing the pool face, presenting the same values.
7. Order the steps so that a failure part-way leaves the local face in place and worn — a retry then completes the promotion, reusing any attribute already created.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role. Promotion carries a value out of a context into the pool;
a context that could trigger it could widen what it was told into everything
the holder composes later.

## Request

Positions, not values: the values are already held, and restating them would
put personal data on the wire for no purpose. `expectedVersion` is required
because a position is only meaningful against the face as the producer read it.

## Response

`promoted` names each attribute and whether it was created. `created: false`
means the face now shares a fact with whatever else presents it, so an edit to
it changes both — the holder is owed that. `reboundPersonaDids` names the
personas whose binding now reads the pool face.

## Security & Privacy

### Data carried

Identifiers and positions in; identifiers out.

### Correlation

Promotion does not disclose anything to anyone. It changes what the holder's
own later compositions can reach. The linkage risk arrives when a second face
references the promoted attribute and both are worn — persona/profile/put and
persona/binding/set report it then.

### Retention

The local face is removed; its values live on in the pool face and, for
promoted entries, in pool attributes retained until deleted.

### Consent/purpose

Widening is the purpose. It is a separate task from composing precisely so that
the widening is its own decision rather than a side effect of saving a face.
