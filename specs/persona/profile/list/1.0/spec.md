---
slug: persona/profile/list
version: "1.0"
title: Persona Profile — List
summary: A holder enumerates their profiles as composed, paginated, with no option to resolve — because resolving every profile at once would decrypt the whole pool to answer a question about names.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, profile, pagination, data-minimisation]
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
  rationale: The response enumerates how the holder has chosen to present themselves, which is identity data even without values, so the agent must attribute the request to a key.
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
  - code: persona/profile/list:cursorInvalid
    meaning: The supplied cursor is unrecognised or expired. A producer restarts the enumeration rather than repairing the token.
    retryable: false
---

## Abstract

**Persona Profile — List** enumerates the holder's profiles.

There is deliberately **no resolve option**. Resolution is available one profile
at a time from [`persona/profile/get`](/specs/persona/profile/get/1.0/spec.md);
offering it here would mean decrypting and re-deriving the holder's entire pool in
order to answer what is usually a question about names. The absence is a design
decision rather than an omission, and a maintainer **MUST NOT** add it as an
extension.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/list/1.0` with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof), **MUST NOT** construct or parse a cursor, and
**MUST NOT** infer exhaustion from a short page.

A conforming **maintainer** **MUST** reject the document unless the caller is
**holder-authorized and unscoped**, **MUST** return profiles in a stable order
across the pages of one enumeration, and **MUST NOT** resolve entries.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role.

## Request

All members optional; an empty payload enumerates from the start.

## Response

Profiles as composed, and a `nextCursor` when more remain.

## Security & Privacy

### Data carried

The request carries pagination state only. The response carries profile names,
entry structure and identifiers — **no values**, because entries are returned as
composed. `inline` entries are the exception and do carry values, since by
definition those live nowhere else; a producer that needs only names **SHOULD**
be aware that a profile heavy in inline entries makes this a heavier response
than its purpose suggests.

### Correlation

Even without values the response is revealing: it shows how many identities the
holder maintains and how they are structured. A party holding it learns the shape
of the holder's compartmentalisation, which is information the compartments exist
to withhold. That is why the task is holder-authorized rather than merely
authenticated.

Nothing in the document names a verifier or counterparty.

### Retention

A point-in-time view with no evidentiary value; **SHOULD NOT** be retained beyond
the rendering it serves. Cursors are retained maintainer-side only for the life
of the enumeration.

### Consent/purpose

The purpose is navigation: a holder finds a profile in order to inspect, edit or
bind it. The data is returned to the party it belongs to. What gate a producer
places in front of it is the consumer's policy and is deliberately not specified
here.
