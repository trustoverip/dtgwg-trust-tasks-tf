---
slug: consent-receipt
version: "0.9"
title: Consent Receipt
summary: A subject grants scoped, time-bound consent for a processor to use specific data, with a portable, revocable receipt.
status: candidate
targetFrameworkVersion: "0.1"
category: data-exchange
keywords:
  - consent
  - gdpr
  - privacy
  - purpose
  - receipt
  - data sharing
authors:
  - DTGWG Data Exchange TF
parties:
  - role: Subject
    requirement: REQUIRED
    vidSchemes: [did:web, did:key]
  - role: Data processor
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
proofRequirement:
  requirement: REQUIRED
  rationale: A consent receipt is intended to be retained as evidence of a granted permission and to be presented to third parties (auditors, regulators); the receipt must therefore carry a transport-independent, non-repudiable signature by the subject.
errorCodes:
  - code: consent-receipt:scope_unsupported
    meaning: One or more scope entries are not recognized by the processor.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        unsupportedScopes:
          type: array
          items: { type: string }
related:
  - agent-authorization
---

## Abstract

The **Consent Receipt** Trust Task lets a *subject* grant, in a portable and verifiable form, scoped permission for a *data processor* to use a defined set of the subject's data for a defined purpose, optionally bounded in time. The receipt is **the** record of the consent: the processor relies on it; the subject (or a delegated agent) can present it as evidence; and either party can verify it after the transaction.

A Consent Receipt does **not** itself effect data transfer — it authorizes one. The data exchange that follows happens under the receipt's scope, purpose, and time bounds.

## Status of this Document

This is a **candidate** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels). At this level the schema is frozen except for editorial clarifications; the specification will progress to `standard` once two independent, interoperable implementations exist and a 90-day stability window has passed without breaking changes.

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/consent-receipt/0.9`.
2. Identify the subject as `issuer`; identify the data processor as `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the subject's key material.

A conforming **consumer** (the data processor) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the subject's declared verification material before relying on the consent.
3. Limit any subsequent data processing to the declared `scope` and `purpose`, and **MUST** cease processing if the consent is revoked via the `revocation` endpoint.

## Definitions

* **Subject.** The natural or legal person granting consent. Identified by `issuer`.
* **Data processor.** The entity to which consent is granted. Identified by `recipient`.
* **Scope.** The set of data categories the consent covers. Each scope entry is an opaque string whose meaning is defined by the processor's terms of service or by an ecosystem governance framework.
* **Purpose.** A human-readable description of why the data is being processed; **SHOULD** be specific enough that a subject can recognize whether their use matches the declared purpose.
* **Revocation endpoint.** A URI the subject can retain to revoke the consent. The framework does not constrain the revocation protocol; common choices include an HTTPS POST or a follow-up *Trust Task document* of a future revocation type.

## Security & Privacy

A Consent Receipt is, by construction, a record of a subject's permission. Producers and consumers **MUST** treat it as evidence and apply storage controls appropriate to the underlying privacy regime (e.g. GDPR, CCPA). The `proof` requirement (**REQUIRED**) ensures the consent is non-repudiable by the subject and tamper-evident in transit.

Producers **SHOULD NOT** embed the data being consented to within the receipt; the receipt authorizes a transfer rather than carrying one. Where the data itself is conveyed, a separate Trust Task (e.g. a credential issuance or a custom data-share specification) **SHOULD** be used and **SHOULD** carry the receipt's `id` in its `threadId` for audit correlation.
