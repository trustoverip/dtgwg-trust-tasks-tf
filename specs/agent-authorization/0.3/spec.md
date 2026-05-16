---
slug: agent-authorization
version: "0.3"
title: Agent Authorization
summary: A principal delegates a scoped authority to an AI agent for a finite window, with revocation and audit-log commitments.
status: draft
targetFrameworkVersion: "0.1"
category: ai-agents
keywords:
  - agent
  - delegation
  - authority
  - scope
  - ai
  - principal
authors:
  - DTGWG AI Agents TF
parties:
  - role: Principal
    requirement: REQUIRED
    vidSchemes: [did:web, did:key]
  - role: Agent
    requirement: REQUIRED
    vidSchemes: [did:web, did:key]
proofRequirement:
  requirement: REQUIRED
  rationale: An authorization carried by an autonomous agent is presented to many third-party services without further interaction with the principal; non-repudiable, transport-independent integrity is mandatory.
errorCodes:
  - code: agent-authorization:scope_exceeded
    meaning: The agent attempted an action outside the granted scope.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        attemptedAction: { type: string }
        grantedScope:
          type: array
          items: { type: string }
related:
  - consent-receipt
---

## Abstract

The **Agent Authorization** Trust Task lets a *principal* — a person, a service, or another agent — delegate a precisely-scoped authority to an *agent* (typically an AI agent acting autonomously) for a bounded period. The agent presents the authorization to downstream services as evidence of what it is permitted to do on the principal's behalf.

Authorizations are intentionally **finite**: every authorization is scoped (what), bounded in time via the framework's `expiresAt`, and revocable via a principal-controlled revocation endpoint. Downstream services rely on these three constraints to bound their own exposure when an agent goes off-policy.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels). At this level the schema **MAY** change without notice as the AI-agent ecosystem's needs become clearer.

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** (the principal) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/agent-authorization/0.3`.
2. Identify itself as `issuer`; identify the agent as `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the principal's key material.
5. Populate the framework's `expiresAt` member; an Agent Authorization without an expiry is not conformant.

A conforming **consumer** (a downstream service receiving the agent's actions) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the principal's declared verification material.
3. Treat `scope` as exhaustive: any action outside the listed scopes is unauthorized, and the consumer **SHOULD** emit an `agent-authorization:scope_exceeded` error response.
4. Consult the `revocation` endpoint when retaining or replaying the authorization over time; treat a revoked authorization as if it were expired.

## Definitions

* **Principal.** The party delegating authority. Identified by `issuer`.
* **Agent.** The party receiving authority, typically an AI agent. Identified by `recipient`.
* **Scope.** An array of opaque scope strings whose interpretation is shared between the principal, the agent, and the downstream services that will receive the agent's actions.
* **Audit log.** Optional URI where the agent's actions taken under this authorization are recorded. Where present, downstream services **MAY** require it to enable post-hoc review.
* **Revocation endpoint.** URI the principal can use to revoke the authorization before its `expiresAt`. Revocation is asynchronous; downstream services **SHOULD** check it for long-lived sessions.

## Security & Privacy

Autonomous agents act at machine speed and at machine scale. An Agent Authorization carries the principal's commitment to bear the consequences of the agent's in-scope actions; consequently:

* The `proof` requirement (**REQUIRED**) and the mandatory `expiresAt` are both non-negotiable: implementations that omit either are not conformant.
* Principals **SHOULD** prefer narrow, action-specific scopes over broad ones, and **SHOULD** prefer short expiries with renewal over long-lived authorizations.
* Downstream services **SHOULD** log the authorization's `id` against every action they accept from the agent, so revocations and post-hoc audits can trace consequences accurately.
* Scopes carrying personal data references **SHOULD** be paired with a separate [Consent Receipt](../../consent-receipt/0.9/spec.md) governing the data the agent will touch.
