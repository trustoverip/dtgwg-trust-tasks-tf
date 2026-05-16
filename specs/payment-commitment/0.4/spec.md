---
slug: payment-commitment
version: "0.4"
title: Payment Commitment
summary: Two parties commit to a payment with conditions of release, settlement rail, and tolerance window — settlement happens off-protocol.
status: draft
targetFrameworkVersion: "0.1"
category: payments
keywords:
  - payment
  - escrow
  - settlement
  - commitment
  - iso20022
  - rail
authors:
  - DTGWG Payments TF
parties:
  - role: Payer
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
  - role: Payee
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
  - role: Settlement agent
    requirement: OPTIONAL
    vidSchemes: [did:web, x509]
proofRequirement:
  requirement: REQUIRED
  rationale: A payment commitment is a binding financial undertaking that may be presented to a settlement agent, an auditor, or a court; non-repudiable, transport-independent integrity is required.
errorCodes:
  - code: payment-commitment:rail_unavailable
    meaning: The requested settlement rail is not available between the parties at the time of evaluation.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        rail: { type: string }
        retryAfter: { type: string, format: date-time }
  - code: payment-commitment:tolerance_exceeded
    meaning: The actual settlement amount or timing fell outside the declared tolerance window.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        observedAmount: { type: string }
        observedAt: { type: string, format: date-time }
related:
  - credential-issuance
---

## Abstract

The **Payment Commitment** Trust Task lets a *payer* and *payee* record a binding intention to settle a specified amount over a specified rail under a specified set of release conditions, with a declared tolerance window. The commitment is **not** the settlement itself — actual fund movement happens on the chosen rail (SEPA Instant, FedNow, SWIFT, on-chain, or an internal book entry) and is reported back by transport-appropriate means.

Where a *settlement agent* is involved, the agent is named as an optional third party so that it can present the commitment as evidence to either side and to its own controls.

## Status of this Document

This is a **draft** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels). At this level the schema **MAY** change without notice.

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/payment-commitment/0.4`.
2. Identify the payer as `issuer`; identify the payee as `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the payer's key material.

A conforming **consumer** **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the payer's declared verification material before relying on the commitment.
3. Treat `amount`, `currency`, `rail`, and `conditions` as authoritative; **MUST NOT** initiate settlement that exceeds the declared `tolerance`.

## Definitions

* **Payer.** The party undertaking to settle the amount. Identified by `issuer`.
* **Payee.** The party to be paid. Identified by `recipient`.
* **Settlement agent.** An optional third party that mediates or witnesses settlement; identified, where present, by `payload.settlementAgent`.
* **Rail.** The settlement system over which value will move (e.g. `sepa-instant`, `fednow`, `swift`, `onchain`, `internal`).
* **Tolerance.** An [ISO 8601 duration](https://en.wikipedia.org/wiki/ISO_8601#Durations) within which the payer commits to settle. A consumer that observes settlement outside this window **SHOULD** emit a `payment-commitment:tolerance_exceeded` error response.

## Security & Privacy

Producers and consumers **MUST** treat a Payment Commitment as a binding financial undertaking. The `proof` requirement (**REQUIRED**) ensures that the payer cannot repudiate the commitment after delivery and that intermediaries cannot tamper with the amount, currency, rail, or conditions.

The framework's `expiresAt` member **SHOULD** be populated to bound the period during which the commitment is actionable. Implementations **SHOULD NOT** retain expired commitments beyond their internal audit requirements.

A commitment is not a payment instruction; conforming implementations **MUST NOT** treat receipt of a commitment as authorization to debit any account without separately verifying that the payer's underlying account permissions are in place.
