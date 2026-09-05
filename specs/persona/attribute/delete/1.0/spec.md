---
slug: persona/attribute/delete
version: "1.0"
title: Persona Attribute — Delete
summary: A holder removes one fact from their attribute pool; the removal is refused while any profile still references it, unless the holder explicitly cascades, so that deleting one fact never silently changes what a profile discloses.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - persona
  - attribute
  - referential-integrity
  - tombstone
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
  rationale: A delete destroys holder data and may, when cascading, change what several profiles disclose. Attribution must survive the transport so an audit record names the key that removed it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A delete and a subsequent re-create at the same address are distinguishable only by order; without an issue time an agent cannot refuse a delete that arrives after the re-create it was meant to precede.
sideEffects:
  level: destructive
  rationale: "Removes an attribute, and with `cascade` removes every profile entry referring to it. A maintainer retains a tombstone so that peers syncing incrementally learn of the removal, but the value itself is gone."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
errorCodes:
  - code: persona/attribute/delete:referenced
    meaning: The attribute is referenced by at least one profile and `cascade` was not set. The details name the referring profiles so the holder can decide between editing those profiles and cascading.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["profileIds"]
      properties:
        profileIds:
          type: array
          maxItems: 256
          items:
            type: string
  - code: persona/attribute/delete:versionConflict
    meaning: The `expectedVersion` precondition failed — the attribute changed after the read the caller is acting on.
    retryable: false
---

## Abstract

**Persona Attribute — Delete** removes one fact from the holder's pool.

The whole of the design is in what it refuses. An attribute may be referenced by
several profiles, and those profiles reference rather than copy — which is the
property that lets a holder change a phone number once. The same property means
deleting a fact would change what every referring profile discloses. So a
referenced attribute is **refused by default**, and the refusal names the
profiles.

`cascade` performs the destructive version deliberately, removing the attribute
and every entry referring to it, and the response returns which profiles changed.
Cascade is never the default: deleting one fact and silently altering what three
profiles present is precisely the surprise this family exists to prevent, and a
holder who has been shown the list is in a position to choose.

A repeated delete converges. The second attempt finds a tombstone, returns
`existed: false`, and deliberately does **not** take a new version — had it done
so, every consumer watching the store would observe a change that did not happen,
and the task could not be safely retried.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/attribute/delete/1.0`, populate
`payload.attributeId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming producer **SHOULD** present the `profileIds` from a `referenced`
error to the holder before re-sending with `cascade`, because the list is the
information that makes the second attempt an informed one.

A conforming **maintainer** **MUST**:

1. Reject the document unless the caller is **holder-authorized and unscoped**.
2. Refuse with `persona/attribute/delete:referenced`, naming every referring profile, when the attribute is referenced and `cascade` is not true.
3. When `cascade` is true, remove the attribute and every profile entry referring to it, and return the affected `profileIds`.
4. Retain a tombstone for a stated retention period, so that a consumer syncing incrementally learns of the removal rather than resurrecting the record on its next rebuild.
5. Return `existed: false` for an attribute that is already absent, and **MUST NOT** assign a new version in that case.

## Authorization

**Holder-authorized and unscoped**, as elsewhere in the family. A context-scoped
caller **MUST** be refused whatever its role.

The destructive nature sharpens the rule rather than changing it: an
authorization defect on a read discloses the pool, and one here destroys it.

## Request

`attributeId` names the attribute. `expectedVersion` **SHOULD** be supplied when
the holder is acting on something they read, so a delete cannot race an edit they
have not seen.

## Response

`existed` distinguishes a removal from a no-op. `removedFromProfiles` is present
only when a cascade actually changed something — a holder is owed the list of what
their single action altered.

## Security & Privacy

### Data carried

The request carries an identifier and two flags — no personal data. The response
carries identifiers of the holder's own profiles when a cascade occurred, and no
values.

A maintainer **MUST NOT** echo the deleted value in the response or in an error.
Returning what was just destroyed would put plaintext into logs and transcripts
that outlive the record, which is the opposite of what a delete is for.

### Correlation

Nothing in either document is presented to a third party, and neither carries a
verifier or counterparty. The `profileIds` in a `referenced` error tell the
producer which of the holder's profiles use a fact — information about the
holder's own composition, returned to the holder.

A tombstone is retained after the value is gone. It carries the identifier and
the fact of removal, not the value, so an attacker reading the store after the
event learns that a fact existed and when it went, and not what it was.

### Retention

The value is destroyed. The **tombstone** is retained for a period the maintainer
**MUST** state, because a consumer synchronising incrementally schedules against
that number: a peer that has been away longer than the retention window cannot be
brought up to date incrementally and must rebuild, and it can only know that if
the window is knowable.

A maintainer that keeps prior versions to serve `pinVersion` **MUST** remove
those for a deleted attribute too. A delete that left recoverable copies behind
would be a delete in name only.

### Consent/purpose

The purpose is erasure: a holder removes a fact they no longer wish their agent
to hold or to be able to present. It is the mechanism by which the pool stays the
holder's rather than accumulating.

The cascade behaviour exists so that erasure and disclosure stay consistent with
one another — a fact that is gone cannot still be presented by a profile that
referred to it. What confirmation a producer requires before cascading is the
consumer's policy and is deliberately not specified here.
