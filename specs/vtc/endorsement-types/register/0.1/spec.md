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
    meaning: The typeUri is one the implementation reserves for its own use and cannot be registered.
    retryable: false
  - code: vtc/endorsement-types/register:exists
    meaning: This typeUri is already registered.
    retryable: false
  - code: vtc/endorsement-types/register:invalidUri
    meaning: The typeUri is empty or exceeds 512 bytes.
    retryable: false
---

## Abstract

The **VTC Endorsement-Types — Register** Trust Task adds a statement predicate the community will accept, keyed by `typeUri`, with an optional `description` and `claimSchema`. It returns the stored [`EndorsementType`](../../../_shared/0.1/endorsement-type.schema.json).

A DTG statement — a Verifiable Statement Credential, `type` `StatementCredential` — carries its meaning in `credentialSubject.predicate`, an absolute IRI. `typeUri` is that IRI: a predicate from the DTG VSC predicate registry, such as `https://registry.trustoverip.org/dtg/vsc/vetted/1` for peer identity vetting, or one in a namespace the community controls, defined in the registry's predicate definition format. The registered set is the community's accept-list. A verifier applying the community's policy **MUST** reject a statement whose predicate is not registered: it fails closed, and never treats an unlisted predicate as a generic statement or accepts one because it resembles a listed one. `claimSchema`, when given, is the schema of the statement's `credentialSubject.object.value`; for a registry predicate it is the object schema the profile publishes.

Roles are not endorsements. A role such as `vetter` is a decision by the community and is conferred by a Verifiable Authority Credential ([`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md)), never registered here. The family keeps its name, and `typeUri` its member name, from when statements were endorsement types.

## Conformance

Producer: supply `typeUri`; optionally `description` and `claimSchema`. Carry a proof.

Consumer: verify the community-admin capability. Refuse URIs the implementation reserves (`reserved`), duplicates (`exists`), and empty/oversized URIs (`invalidUri`). Otherwise store the predicate and return the full `EndorsementType`.

## Security & Privacy

**Widens what the community accepts.** Registering a predicate expands the statements the community will honour, so the change is proof-REQUIRED and audited. A predicate the community has not registered never counts, whatever its signature.
