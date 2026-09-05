---
slug: persona/contact/list
version: "1.0"
title: Persona Contact — List
summary: Enumerate a context's contacts as summaries, narrowed by persona or by what changed since a given instant, with no claim values in the listing.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, contact, privacy]
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
  rationale: The response enumerates the holder's counterparties, which is relationship data about third parties, so the agent must attribute the request to a key.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A list has no durable effect and no ordering hazard.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/contact/list:cursorInvalid
    meaning: The supplied cursor is unrecognised or expired. A producer restarts the enumeration.
    retryable: false
---

## Abstract

**Persona Contact — List** finds a contact. It returns summaries and no claim
values, because finding one contact does not require disclosing the details of
every contact.

Two members do the useful work. `changedSince` answers *what changed while I was
away* cheaply, which is the question the revision history exists to make
answerable at all. And `hasUnreviewedChange` is what lets a producer badge the
contact whose payment address moved — a revision history nobody is shown is an
archive, not a defence.

Omitting `knownByPersona` lists a context's contacts across every persona in it.
That view puts the holder's personas side by side, so a producer **SHOULD** offer
it deliberately rather than making it the default.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/list/1.0`, populate
`payload.contextId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).
It **MUST NOT** construct or parse a cursor, and **MUST NOT** infer exhaustion
from a short page.

A conforming **maintainer** **MUST** confine the caller to its own context,
**MUST** return summaries in a stable order across the pages of one enumeration,
and **MUST NOT** include claim values in a summary.

A conforming maintainer **MUST NOT** substitute a DID for an absent
`displayName`. A producer that renders an identifier where a name would go, with
no signal that the contact disclosed none, teaches the holder to read identifiers
as names — which is the habit a display-name spoof relies on.

## Authorization

**Context-scoped**, confined to the caller's own context.

## Request

See the payload schema; every member carries its own rationale there.

## Response

See the payload schema.

## Security & Privacy

### Data carried

The request carries a context, an optional persona filter and pagination state.
The response carries **identifiers and display names of third parties** — no
claim values.

`displayName` is the one member drawn from a contact's disclosed data, present so
a list is legible. Everything else is metadata about the relationship rather than
about the person.

### Correlation

The unfiltered listing is the sensitive view: it shows which of the holder's
personas know which counterparties, side by side, in one response. That is a map
of the holder's compartmentalisation, and it is why the task is confined to a
single context — a caller can see the arrangement inside its own context and
learns nothing about any other.

A maintainer **MUST NOT** offer a cross-context listing on this task. Assembling
that view is the holder's prerogative from their own tooling, not an
application's.

### Retention

A point-in-time view with no evidentiary value; **SHOULD NOT** be retained beyond
the rendering it serves. Cursors live only as long as their enumeration.

### Consent/purpose

The purpose is navigation and triage: the holder finds a counterparty, or sees
which relationships have changed since they last looked. The data was disclosed
to the holder by its subjects.

What gate a producer applies before showing the unfiltered, cross-persona view is
the consumer's policy and is deliberately not specified here.
