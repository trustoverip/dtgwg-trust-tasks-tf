---
slug: credential-issuance
version: "1.1"
title: Credential Issuance
summary: An issuer commits to producing a verifiable credential for a holder against a published schema, with status notification on completion.
status: standard
targetFrameworkVersion: "0.1"
category: credentials
keywords:
  - credential
  - vc
  - issuance
  - issuer
  - holder
  - schema
authors:
  - DTGWG Credentials TF
parties:
  - role: Issuer
    requirement: REQUIRED
    vidSchemes: [did:web, did:key]
  - role: Holder
    requirement: REQUIRED
    vidSchemes: [did:web, did:key]
proofRequirement:
  requirement: REQUIRED
  rationale: The commitment to issue a credential is a binding undertaking that the holder may later present to a third-party verifier or audit process; transport-independent integrity is required.
errorCodes:
  - code: credential-issuance:schema_unsupported
    meaning: The issuer does not support the requested credentialSchema.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        requestedSchema: { type: string, format: uri }
        supportedSchemas:
          type: array
          items: { type: string, format: uri }
related:
  - kyc-handoff
  - trust-registry-query
---

## Abstract

The **Credential Issuance** Trust Task lets an *issuer* commit to producing a verifiable credential for a *holder*, against a named credential schema and in a named credential format (e.g. `vc-jwt`, `sd-jwt-vc`, `mdoc`). It is a forward-looking commitment, not the credential itself: actual issuance happens at the named delivery endpoint, optionally accompanied by status notifications that this task **MAY** be re-used (with a new `id` and the same `threadId`) to convey.

The task makes the issuer's commitments — schema, format, and delivery — auditable and portable, so a holder can present the commitment to a verifier or coordinator that needs to know what to expect.

## Status of this Document

This is a **standard** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels).

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/credential-issuance/1.1`.
2. Identify the issuer as `issuer`; identify the holder as `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.
3. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
4. Include a `proof` member signed by the issuer's key material.

A conforming **consumer** (the holder, or a coordinator acting on the holder's behalf) **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the issuer's declared verification material.
3. Treat the `credentialSchema` and `format` values as authoritative for the credential about to be issued.

## Definitions

* **Issuer.** The party committing to produce the credential. Identified by `issuer`.
* **Holder.** The party for whom the credential will be issued. Identified by `recipient`.
* **Credential schema.** A URI pointing to the schema definition the issued credential will conform to.
* **Format.** A short label for the credential serialization format (`vc-jwt`, `vc-jose-cose`, `sd-jwt-vc`, `mdoc`).
* **Delivery endpoint.** Optional URI where the issued credential will be delivered. Where absent, delivery happens over the same transport that carried this commitment.

## Security & Privacy

The Credential Issuance commitment carries no credential subject claims — claims are negotiated and delivered separately. Producers **MAY** include high-level claim hints in `payload.claims` for coordination purposes; these hints **SHOULD NOT** include sensitive personal data, since the commitment is delivered before consent-bound claim release.

The `proof` requirement (**REQUIRED**) ensures the issuer's commitment is bound to its own keys end-to-end, so that the holder can present the commitment to third parties after the original transport has closed.
