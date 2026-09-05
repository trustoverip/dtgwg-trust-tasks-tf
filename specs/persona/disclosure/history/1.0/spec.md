---
slug: persona/disclosure/history
version: "1.0"
title: Persona Disclosure — History
summary: What the holder has shared, with whom and when — and which contexts a given attribute has reached, which is the account the agent-scoped pool owes them.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, privacy, correlation]
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
  rationale: The response is the complete record of the holder's disclosures across every context, which is the single most revealing document the agent holds about them.
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
  - code: persona/disclosure/history:cursorInvalid
    meaning: The supplied cursor is unrecognised or expired. A producer restarts the enumeration.
    retryable: false
---

## Abstract

**Persona Disclosure — History** answers *what have I shared, and with whom*.

It also answers a question the design owes the holder. Putting the attribute pool
**above** the context boundary bought a correlation check that can see across
contexts; the debt it incurred is that a holder can no longer tell, by looking at
one context, where a fact has gone. Filtering by `attributeType` settles that —
*where has my home address reached* — and it is the reason this task exists rather
than being folded into an audit log.

Records name claim **types** and never values. The history says what kind of thing
went where; re-storing the values would double the exposure it exists to describe.

`rungs` is positionally aligned with `claimTypes` because the same claim type at
two proof strengths is two very different disclosures, and a holder reviewing what
they have done needs to see which they did.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/disclosure/history/1.0` with a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof), and **MUST NOT** construct or parse a cursor.

A conforming **maintainer** **MUST** reject the document unless the caller is
**holder-authorized and unscoped**; **MUST** return records in a stable order;
**MUST NOT** include claim values; and **MUST NOT** omit or alter a record once
written — the history is append-only, and a disclosure a holder cannot find is a
disclosure they cannot act on.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role.

Omitting `contextId` queries across every context, which only the holder can do
and which is the whole reason the task sits on this side of the boundary. An
application that could read this would learn every other context the holder
operates in.

## Request

See the payload schema; every member carries its own rationale there.

## Response

See the payload schema.

## Security & Privacy

### Data carried

The request carries filters. The response carries **the map of the holder's
disclosures**: which personas presented what kinds of claim to which verifiers,
when, and at what proof strength. No values.

This is the most revealing document the agent will produce about the holder —
more so than any single disclosure, because it is the pattern rather than an
instance.

### Correlation

The response correlates the holder's personas to each other by construction: it
is a list of their identities side by side. That is exactly what makes it useful
to the holder and unacceptable to anyone else, and it is why the authorization is
on the scope axis rather than the role axis.

A producer **MUST NOT** transmit a history response anywhere, and **MUST NOT**
retain one.

### Retention

Disclosure records are **permanent and append-only**. A holder asked six months
later to account for what a verifier holds can only answer if the record survived,
and a record that could be edited would be no answer at all.

A maintainer **MUST** retain them for at least as long as any durable credential
minted by a disclosure remains live, since that is the disclosure still capable of
being re-verified.

### Consent/purpose

The purpose is accountability, in the direction that is usually missing: not the
verifier's record of what it received, but the holder's record of what they gave.

The data is the holder's own history returned to them. What gate a producer places
in front of it is the consumer's policy and is deliberately not specified here.
