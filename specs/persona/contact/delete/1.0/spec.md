---
slug: persona/contact/delete
version: "1.0"
title: Persona Contact — Delete
summary: Remove a contact and its revisions, reporting honestly how many survived because a disclosure record still references them.
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
  rationale: A delete destroys a third party's disclosed data and the record of what they asserted. Attribution must survive the transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A delete and a subsequent re-filing at the same address are distinguishable only by order.
sideEffects:
  level: destructive
  rationale: "Removes a contact and every revision not held by a disclosure reference. Not recoverable."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
errorCodes: []
---

## Abstract

**Persona Contact — Delete** removes a contact and its history.

The member worth reading is `retainedForDisclosure`. A revision referenced by a
disclosure record is evidence of something the holder did — what they were shown
before they presented — and a maintainer may legitimately keep it. What it
**MUST NOT** do is stay quiet about that: an incomplete erasure the holder
believes is complete is worse than one they know about, because they will make
the next decision on a false premise.

A repeated delete converges: `existed: false`, and no new counter value, so
nothing watching the store observes a change that did not happen.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/delete/1.0`, populate `contextId`
and `contactId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context;
**MUST** remove every revision not referenced by a disclosure record; **MUST**
report the count of any it retained; and **MUST** return `existed: false` without
assigning a new version for a contact that is already absent.

A conforming maintainer **MUST NOT** report a deletion as total while retaining
revisions.

## Authorization

**Context-scoped**, confined to the caller's own context.

## Request

See the payload schema; every member carries its own rationale there.

## Response

See the payload schema.

## Security & Privacy

### Data carried

The request carries identifiers. The response carries counts. Neither carries the
data being destroyed, and a maintainer **MUST NOT** echo it — returning what was
just deleted would put a third party's details into logs that outlive the record.

### Correlation

Nothing is presented to a third party. The counts describe the holder's own store.

### Retention

This task IS the retention control for contact data, and its honesty is the
whole feature. Revisions referenced by a disclosure record survive; everything
else goes. A maintainer **MUST** state its policy so a holder can predict which
of the two a given revision falls under before they act.

The subject of the deleted data has no visibility of the deletion, which is
correct — they disclosed to the holder and the holder's retention is the
holder's.

### Consent/purpose

The purpose is erasure of a relationship record the holder no longer wishes to
keep. It is the counterpart to the revisioning that makes contacts useful:
history accumulates, and this is how it stops.

What confirmation a producer requires before destroying a history is the
consumer's policy and is deliberately not specified here.
