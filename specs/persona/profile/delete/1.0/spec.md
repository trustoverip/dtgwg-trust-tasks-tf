---
slug: persona/profile/delete
version: "1.0"
title: Persona Profile — Delete
summary: A holder removes a profile; the removal is refused while a persona is bound to it, unless the holder explicitly unbinds, so no persona silently stops presenting anything.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords: [persona, profile, referential-integrity, tombstone]
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
  rationale: A delete destroys a composition and may leave bound personas presenting nothing. Attribution must survive the transport so an audit record names the key that removed it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A delete and a subsequent re-create at the same identifier are distinguishable only by order.
sideEffects:
  level: destructive
  rationale: "Removes a profile, and with `unbind` clears every binding to it. No attribute is touched — the pool is unaffected, because a profile references rather than owns."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/profile/delete:bound
    meaning: A persona is still bound to this profile and `unbind` was not set. The details name the bound persona DIDs so the holder can decide between rebinding them and unbinding.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["personaDids"]
      properties:
        personaDids:
          type: array
          maxItems: 256
          items:
            type: string
  - code: persona/profile/delete:versionConflict
    meaning: The expectedVersion precondition failed.
    retryable: false
---

## Abstract

**Persona Profile — Delete** removes a composition.

It refuses while a persona is bound to it, and the refusal names the personas.
The alternative — deleting anyway — leaves a persona bound to nothing, which is a
legal state but not one a holder should reach without choosing it: a persona that
has silently stopped presenting anything is a failure the holder discovers from
the other side of a disclosure that did not happen.

`unbind` performs that deliberately and returns which personas were left
presenting nothing.

**The pool is untouched.** A profile references attributes rather than owning
them, so deleting a composition destroys no facts. This is the asymmetry with
[`persona/attribute/delete`](/specs/persona/attribute/delete/1.0/spec.md), where
removing a fact does change what compositions present.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/profile/delete/1.0`, populate
`payload.profileId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).
It **SHOULD** show the holder the `personaDids` from a `bound` error before
re-sending with `unbind`.

A conforming **maintainer** **MUST** reject the document unless the caller is
**holder-authorized and unscoped**; **MUST** refuse a bound profile with
`persona/profile/delete:bound` naming every bound persona unless `unbind` is
true; **MUST NOT** remove or alter any attribute; and **MUST** return
`existed: false` without assigning a new version for a profile that is already
absent.

## Authorization

**Holder-authorized and unscoped.** A context-scoped caller **MUST** be refused
whatever its role. A caller able to delete a profile could stop a persona
presenting, which is a denial the holder would attribute to the counterparty.

## Request

`profileId` names the profile. `expectedVersion` **SHOULD** be supplied when
acting on something read, so a delete cannot race an edit not yet seen.

## Response

`existed` distinguishes removal from no-op; `unboundPersonas` reports what a
single action changed.

## Security & Privacy

### Data carried

The request carries an identifier and two flags — no personal data. The response
carries the holder's own persona DIDs when bindings were cleared, and no values.

A maintainer **MUST NOT** echo the deleted composition in the response or in an
error: returning what was just destroyed puts it into logs and transcripts that
outlive the record.

### Correlation

Nothing here is presented to a third party. The `personaDids` in a `bound` error
associate the holder's profiles with their personas — information about the
holder's own arrangement, returned to the holder, and exactly the association the
authorization rule keeps inside their tooling.

### Retention

The composition is destroyed; a tombstone is retained for a maintainer-stated
period so an incrementally-syncing consumer learns of the removal rather than
resurrecting it on the next rebuild. Attributes referenced by the deleted profile
are unaffected and remain.

### Consent/purpose

The purpose is retirement: a holder stops maintaining a way of presenting
themselves. The refusal-by-default exists so that retiring a composition and
leaving a persona mute stay separate decisions. What confirmation a producer
requires before unbinding is the consumer's policy and is deliberately not
specified here.
