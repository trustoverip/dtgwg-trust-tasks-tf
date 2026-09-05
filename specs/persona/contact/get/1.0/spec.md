---
slug: persona/contact/get
version: "1.0"
title: Persona Contact — Get
summary: Read one contact at its current or a named earlier revision, with an optional cheap timeline of what changed when.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, contact, revision, history]
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
  rationale: The response returns a third party's personal data, so the agent must attribute the request to a key and an audit record must be able to name which application read it.
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
  - code: persona/contact/get:notFound
    meaning: No contact exists at that identifier in this context.
    retryable: false
  - code: persona/contact/get:revisionReaped
    meaning: The named revision existed and has been reaped under the retention policy. Distinct from notFound on purpose — a caller comparing against history must be able to tell "never existed" from "no longer kept", because only the second means their comparison is unsound rather than mistaken.
    retryable: false
---

## Abstract

**Persona Contact — Get** reads what a peer disclosed.

Two choices are worth noting. `includeHistory` returns revision **metadata
without documents**, so a producer can render a timeline cheaply and fetch only
the revision the holder actually opens — a contact with forty revisions should
not cost forty documents to display as a list.

And a reaped revision is reported as `revisionReaped`, distinct from `notFound`.
A caller comparing a current value against history needs to tell *never existed*
from *no longer kept*: the first means their premise was wrong, the second means
their comparison is unsound. Collapsing the two would let a producer silently
conclude "nothing changed" from an absence that means the opposite.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/get/1.0`, populate `contextId` and
`contactId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context;
**MUST** return the current revision when `rev` is omitted; **MUST** emit
`persona/contact/get:revisionReaped` rather than falling back to the nearest
surviving revision; and **MUST NOT** include documents in `history`.

## Authorization

**Context-scoped**, confined to the caller's own context.

## Request

The contact, optionally a revision, optionally a timeline.

## Response

The contact at the requested revision, and when asked a timeline of revision
metadata.

## Security & Privacy

### Data carried

The response carries **a third party's personal data** and, when present, the
holder's private `notes` about them. Notes are the most sensitive member: their
subject cannot see them and did not consent to them.

A maintainer **MAY** omit `notes` for a caller it judges should not see them, and
a producer **MUST NOT** treat their absence as an error.

### Correlation

Reading history reveals when a counterparty changed what they present, which over
time is a behavioural profile of a person who is not the holder. A producer
**SHOULD** request history only when the holder is inspecting a change, not as a
matter of course.

Nothing in the response reveals the holder's other personas or contexts; the
record is confined to the relationship it belongs to.

### Retention

A response is a point-in-time view with no evidentiary value of its own — the
stored contact is the evidence. It **SHOULD NOT** be retained by the caller
beyond the rendering it serves; a cached copy is the same disclosure of a third
party's data, outside the store that protects it.

### Consent/purpose

The purpose is recall and verification: the holder checks what a counterparty
told them, and whether it has changed. The data was disclosed to the holder
deliberately by its subject.

Reading it under a persona other than the one it was disclosed to would present
the counterparty with an identity they never disclosed to; the record's
`knownByPersona` exists so that boundary is checkable. What gate a producer
applies is the consumer's policy and is deliberately not specified here.
