---
slug: vtc/endorsement-types/delete
version: "0.1"
title: VTC Endorsement-Types — Delete
summary: Remove an endorsement type from a community's registry; refused while anything still references it.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords: [vtc, endorsements, endorsement-types, delete]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Removing a recognised endorsement type changes what the community accepts; it MUST be attributable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Deleting an endorsement type invalidates the endorsements that reference it. Replayed after the type was re-registered, it deletes the new registration and the endorsements hanging off it.
sideEffects:
  level: destructive
  rationale: "Removes an endorsement type from the registry; recoverable only by re-registering it."
subjectPath: /typeUri
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsement-types/delete:notFound
    meaning: No endorsement type with the supplied typeUri is registered.
    retryable: false
  - code: vtc/endorsement-types/delete:inUse
    meaning: >-
      Something still references this type — a live endorsement of it, an
      admission criterion requiring statements of it, or both. `details`
      says which, and a consumer that can determine both SHOULD report both
      rather than making the caller clear one and discover the other.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        liveEndorsements:
          description: >-
            Count of unrevoked endorsements of this type. Absent when the
            consumer keeps no endorsement store of its own.
          type: integer
          minimum: 1
        criteria:
          description: >-
            Identifiers of the admission criteria requiring statements of
            this type. Absent when the consumer has no criteria concept.
          type: array
          minItems: 1
          items: { type: string }
---

## Abstract

The **VTC Endorsement-Types — Delete** Trust Task removes an endorsement type `typeUri` from the registry. It is refused while anything still references the type — the no-orphans precondition.

Two kinds of reference exist, and a consumer refuses on either. A **live endorsement** of the type would be left pointing at a registration that no longer exists. An **admission criterion** requiring statements of the type would be left asking applicants for evidence the community no longer recognises — and, where registering such a criterion is itself refused, stranded in a state it could not be created in.

## Conformance

Producer: supply `typeUri`. Carry a proof.

Consumer: verify the community-admin capability. Resolve the type (`notFound` if absent). Gather **every** reference it can determine — live endorsements of the type, admission criteria requiring statements of it — and if any exist, return `inUse` with those references in `details` and delete nothing. Otherwise remove the type and return `{ typeUri }`. Audit the deletion.

A consumer that can determine both kinds of reference **SHOULD** gather both before refusing, rather than returning on the first found. Refusing on one at a time turns a single determination into as many round trips as there are kinds of reference, each ending in the same code.

A consumer **MAY** hold neither concept — an endorsement store and an admission-criteria registry are both optional. Absent members in `details` mean "not applicable here", never "none found": a caller reading `details.criteria` as absent must not conclude the type is unreferenced by criteria, only that this consumer does not track them.

## Security & Privacy

**No-orphans deletion.** The `inUse` refusal guarantees no live endorsement and no admission criterion is left pointing at a deleted type. The removal is proof-REQUIRED and audited.

**What the refusal discloses.** `details` names admission criteria by identifier and counts live endorsements. Both are community-governance facts an administrator authorised to delete the type can already enumerate, so the refusal tells an authorised caller nothing new. It is reachable only after the community-admin capability has been verified — a consumer that returned these details before that check would be disclosing its criteria set to an unauthorised caller.
