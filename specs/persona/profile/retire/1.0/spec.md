---
slug: persona/profile/retire
version: "1.0"
title: Persona Profile — Retire
summary: A holder stops being a face everywhere without destroying it — taken off every persona, hidden from pickers, unwearable until reinstated, with its values and disclosure history kept.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - profile
  - lifecycle
  - retire
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
  rationale: Retiring takes a face off every persona wearing it, which changes what each of those contexts is shown. Attribution must survive the transport so an audit record names the key that did it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Retire races persona/binding/set and persona/profile/reinstate; an issue time lets an agent order them.
sideEffects:
  level: mutating
  rationale: "Clears every binding to the face and marks it retired. Reversible through persona/profile/reinstate; no value and no history is removed."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: Identifiers in, identifiers out.
errorCodes:
  - code: persona/profile/retire:versionConflict
    meaning: The `expectedVersion` precondition failed. Details carry the current version.
    retryable: false
---

## Abstract

**Persona Profile — Retire** is how a holder stops being a face without
destroying it: the relationship is over, the job ended, the event passed.

Removing a face means one of three things to the person doing it, and the
family now has a task for each:

| the holder means | task |
|---|---|
| stop wearing it *here* | persona/binding/set with `profileId: null` |
| stop wearing it *anywhere*, keep it | **persona/profile/retire** (reversible) |
| destroy it | persona/profile/delete |

A retired face is taken off every persona wearing it, left out of pickers and
of persona/profile/list unless `includeRetired` is asked for, and refused by
persona/binding/set until persona/profile/reinstate. Everything it carried and
every disclosure it made is kept: retiring is "stop being this", not "forget
this", and the second is a separate, deliberate act.

A binding with an `until` (persona/binding/set) retires its face at expiry when
it is then worn nowhere else, which is what makes a face for one weekend end on
its own.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/retire/1.0`, populate
`payload.profileId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Treat a `profileId` that names no face — in the pool, or in `contextId` when given — as not found.
3. Clear every binding to the face, in every context, and report each in `unbound`.
4. Mark the face `retired` with a `retiredAt`, keeping its entries, values and every disclosure record.
5. Refuse to bind a retired face (persona/binding/set `profileRetired`) until it is reinstated.
6. Leave a retired face out of persona/profile/list unless `includeRetired` is true.
7. Treat retiring a retired face as a success that changes nothing and takes no new version.

## Authorization

**Holder-authorized and unscoped**, as elsewhere in the family — including for
a context-local face, since retiring one is the holder's decision about their
own identity, not the context's.

## Request

`contextId` names the context of a context-local face; a pool face is addressed
by its id alone.

## Response

`unbound` names where the face was worn and no longer is. A holder is owed the
list of what their one action changed.

## Security & Privacy

### Data carried

Identifiers only.

### Correlation

Retiring changes nothing any counterparty holds. Contexts that were shown the
face keep what they were shown; persona/disclosure/history continues to say so.

### Retention

Nothing is removed. A retired face is retained until deleted, in the
maintainer's backed-up partition like any other.

### Consent/purpose

Ending a relationship without erasing the record of it is the purpose. Erasure
is persona/profile/delete, a separate act with its own warning.
