---
slug: vtc/endorsement-types/register
version: "0.1"
title: VTC Endorsement-Types — Register
summary: Register a statement predicate a community will accept — a DTG VSC registry IRI or a community-namespace IRI — optionally with a schema for the statement's object value.
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords: [vtc, endorsements, endorsement-types, register, predicate, vsc]
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
  rationale: Registering a predicate changes which statements the community will accept; it MUST be attributable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Registering a predicate adds vocabulary every later statement is written against. A replay re-adds a predicate the community has retired, and statements under it then count again.
sideEffects:
  level: mutating
  rationale: "Adds a predicate to the community's accept-list."
subjectPath: /typeUri
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/endorsement-types/register:reserved
    meaning: The typeUri is the reserved value `role:vetter`, which names a record kind vtc/endorsements/* keeps for itself, and cannot be registered.
    retryable: false
  - code: vtc/endorsement-types/register:exists
    meaning: This typeUri is already registered.
    retryable: false
  - code: vtc/endorsement-types/register:invalidUri
    meaning: The typeUri is empty, exceeds 512 bytes, or is not an absolute predicate IRI.
    retryable: false
---

## Abstract

The **VTC Endorsement-Types — Register** Trust Task adds a statement predicate the community will accept, keyed by `typeUri`, with an optional `description` and `claimSchema`. It returns the stored [`EndorsementType`](../../../_shared/0.1/endorsement-type.schema.json).

A DTG statement — a Verifiable Statement Credential, `type` `StatementCredential` — carries its meaning in `credentialSubject.predicate`, an absolute IRI. `typeUri` is that IRI: a predicate from the DTG VSC predicate registry, such as `https://registry.trustoverip.org/dtg/vsc/vetted/1` for peer identity vetting, or one in a namespace the community controls, defined in the registry's predicate definition format. The registered set is the community's accept-list. A verifier applying the community's policy **MUST** reject a statement whose predicate is not registered: it fails closed, and never treats an unlisted predicate as a generic statement or accepts one because it resembles a listed one. `claimSchema`, when given, is the schema of the statement's `credentialSubject.object.value`; for a registry predicate it is the object schema the profile publishes.

**The core predicates are seeded.** A community seeds its registry with the four DTG core predicates when it starts — `https://registry.trustoverip.org/dtg/vsc/endorses/1`, `.../witnessed/1`, `.../vetted/1` and `.../presented/1`, before any administrator registers anything, so the statements the registry defines count without a registration step. Registering one of them again is `exists`.

**Only predicate IRIs are registrable.** A `typeUri` that is not an absolute IRI is refused with `invalidUri`. One value is **reserved** and refused with `reserved`, because it names the one record kind [`vtc/endorsements/*`](../../../endorsements/issue/0.1/spec.md) keeps beside statements: `role:vetter`, the record of a vetter role credential issued by [`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md). It is not a predicate IRI, so it cannot collide with a registered predicate. A community's own identity checks need no reserved value: they are statements under `https://registry.trustoverip.org/dtg/vsc/vetted/1`, a seeded core predicate.

Roles are not endorsements. A role such as `vetter` is a decision by the community and is conferred by a Verifiable Authority Credential ([`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md)), never registered here. The family keeps its name, and `typeUri` its member name, from when statements were endorsement types.

## Conformance

Producer: supply `typeUri`; optionally `description` and `claimSchema`. Carry a proof.

Consumer: verify the community-admin capability. Refuse the reserved value `role:vetter` (`reserved`); an empty, oversized, or non-IRI `typeUri` (`invalidUri`); and one already registered, the four seeded core predicates included (`exists`). Otherwise store the predicate and return the full `EndorsementType`.

## Security & Privacy

**Widens what the community accepts.** Registering a predicate expands the statements the community will honour, so the change is proof-REQUIRED and audited. A predicate the community has not registered never counts, whatever its signature.
