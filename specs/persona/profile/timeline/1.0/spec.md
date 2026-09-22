---
slug: persona/profile/timeline
version: "1.0"
title: Persona Profile — Timeline
summary: A holder reads what one face has done, in order — composed, worn and taken off, what it told whom, when a value it shows changed, retired — with no value and no private label anywhere in it.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - profile
  - history
  - audit
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
  requirement: OPTIONAL
  rationale: A read of the holder's own history, returned to the holder.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: Read-only; replay changes nothing.
sideEffects:
  level: none
  rationale: Reads the face's history. Writes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Returns contexts, personas, parties, claim types and versions for the holder's own face. Never a value.
errorCodes: []
---

## Abstract

**Persona Profile — Timeline** is one face's history, joined.

Everything that happens to a face is recorded somewhere — its bindings, its
disclosures, the versions of the values it shows — and nothing joined them. A
holder asking "what has my Conference face done?" had to read four places and
line them up. This task lines them up: composed, worn here, told this party
these types, the name it shows changed, taken off, retired.

**Never a value, never a private label.** The timeline says a `name.display`
changed at version 7, not what it changed from or to; it names a context and a
persona, not the name the holder files the face under. A history is itself a
privacy cost, and one that carried values would be a second copy of every value
the face ever showed, with a lifetime the holder's edits and purges do not
reach.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/timeline/1.0` and populate
`payload.profileId`, and **MUST NOT** construct or parse a `cursor`.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Treat a `profileId` that names no face — in the pool, or in `contextId` when given — as not found.
3. Return events oldest first, and in a stable order across the pages of one enumeration.
4. Include no value, no attribute label, no face name and no binding label in any event.
5. Record, from the time it implements this task, each event whose evidence it would not otherwise keep — a binding taken off, in particular, which leaves no trace in the binding it replaces.

A maintainer **MAY** omit events that happened before it implemented this task
and left no record; it **SHOULD** still report the face's composition, from
the face's own creation time.

## Authorization

**Holder-authorized and unscoped.** A face's history spans every context it was
worn in.

## Request

`since` narrows by time; `cursor` and `limit` page.

## Response

`events`, oldest first. `nextCursor` while more follow.

## Security & Privacy

### Data carried

Contexts, persona DIDs, verifier DIDs, claim types, versions and times.

### Correlation

The timeline is the holder's correlation map over time — which personas wore
one face, and whom it told. Holder-only for that reason.

### Retention

The event record a maintainer keeps for this task holds identifiers, types and
times, never values or labels, so it acquires no lifetime a value does not
have. It is retained with the face and removed when the face is deleted; the
disclosure records it draws on keep their own retention.

### Consent/purpose

The holder reading their own history.
