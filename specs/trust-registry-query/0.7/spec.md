---
slug: trust-registry-query
version: "0.7"
title: Trust Registry Query
summary: A relying party asks a trust registry whether a given entity is authorized to perform a given role under a governance framework.
status: candidate
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - registry
  - trqp
  - authorization
  - ecosystem
  - governance
  - egf
authors:
  - DTGWG Governance TF
parties:
  - role: Relying party
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
  - role: Trust registry operator
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
proofRequirement:
  requirement: RECOMMENDED
  rationale: The query and response are typically short-lived and consumed over an authenticated transport, but a proof binds the registry's answer to its keys for downstream retention or replay scenarios.
errorCodes:
  - code: trust-registry-query:framework_unknown
    meaning: The named governance framework is not recognized by the registry.
    retryable: false
  - code: trust-registry-query:role_unknown
    meaning: The named role is not defined under the named framework.
    retryable: false
related:
  - credential-issuance
---

## Abstract

The **Trust Registry Query** Trust Task lets a *relying party* ask a *trust registry operator* a precise, machine-checkable question: is a given entity currently authorized to perform a given role under a named governance framework, as of a given moment in time? The registry's answer is what binds the relying party's downstream behavior — a credential it accepts, a transaction it processes, a service it allows.

The task is deliberately a **query**, not an authorization-grant: it does not issue, suspend, or alter status. It conveys the registry's current attestation, scoped by the four inputs (`registry`, `entity`, `role`, `framework`), with an optional `asOf` for point-in-time queries.

## Status of this Document

This is a **candidate** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels). At this level the schema is frozen except for editorial clarifications.

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the relying party initiating the query) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-registry-query/0.7`.
2. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
3. Identify itself as `issuer`; identify the registry as `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.

A conforming **consumer** (the registry operator) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Resolve the query against its current state (or, where `asOf` is provided, against the state at that time) and respond with either a `trust-task-ok` *Trust Task document* carrying the registry's answer (when that response type is published — see [SPEC.md §8.6](../../../SPEC.md#86-reserved-response-type-slugs)) or a `trust-task-error` *Trust Task document* per [SPEC.md §8](../../../SPEC.md#8-error-responses).
3. Honor the query's `expiresAt` and any `proof` per the framework.

## Definitions

* **Relying party.** The querying entity that will act on the registry's answer. Identified by `issuer`.
* **Trust registry operator.** The operator of the registry being queried. Identified by `recipient`.
* **Governance framework.** A URI that identifies the rule set under which the role's authorization is defined; the registry's answer is meaningful only relative to this framework.
* **Role.** A short string naming the authorization being checked (e.g. `verifier`, `issuer-of:passport`, `processor-of:health-data`).
* **`asOf`.** Optional point-in-time the query should be evaluated at; absent means "now".

## Security & Privacy

Trust Registry Query responses inform downstream authorization decisions; tampered answers can directly enable fraud. While the proof requirement is **RECOMMENDED** rather than **REQUIRED** — recognizing that most queries are short-lived and consumed over an authenticated transport — implementations that retain or replay registry answers **SHOULD** require an in-band `proof` so the registry's attestation is non-repudiable.

A registry **SHOULD NOT** include in its response any data not necessary to answer the query. In particular, supplementary attributes about the queried entity that are not part of the role determination **SHOULD** be omitted from the response payload to avoid incidental disclosure.
