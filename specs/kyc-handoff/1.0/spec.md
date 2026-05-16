---
slug: kyc-handoff
version: "1.0"
title: KYC Handoff
summary: A counterparty proves it has performed Know-Your-Customer verification on a subject and conveys the result to a relying party.
status: standard
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - kyc
  - aml
  - onboarding
  - compliance
  - subject
  - verification
authors:
  - DTGWG Identity Subgroup
parties:
  - role: Verifier (KYC provider)
    requirement: REQUIRED
    vidSchemes: [did:web, did:key, x509]
  - role: Relying party
    requirement: REQUIRED
    vidSchemes: [did:web, x509]
proofRequirement:
  requirement: REQUIRED
  rationale: The recipient retains the verification result for compliance reporting and may rely on it after delivery; a transport-bound integrity guarantee alone is insufficient.
errorCodes:
  - code: kyc-handoff:document_revoked
    meaning: A breeder document used in the verification was revoked by its issuing authority after the verification completed.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [documentRef]
      properties:
        documentRef: { type: string }
        revokedAt:   { type: string, format: date-time }
related:
  - credential-issuance
  - trust-registry-query
---

## Abstract

The **KYC Handoff** Trust Task lets a *verifier* — typically a regulated KYC/AML provider — convey to a *relying party* the outcome of a Know-Your-Customer check on a named subject, in a self-contained, portable, and verifiable form. The relying party can rely on the handoff to satisfy onboarding or transaction-eligibility checks without re-running the underlying verification.

The handoff carries the verified subject identifier, a pass/fail result, an assurance level, and (optionally) references to the evidence the verifier relied upon. It does **not** carry the underlying evidence itself; references are pointers the relying party may dereference under its own trust framework.

## Status of this Document

This is a **standard** *Trust Task specification* of the Trust Tasks framework, published under the maturity model defined in [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels). Changes that affect the document model **MUST** follow the versioning rules in [SPEC.md §5](../../../SPEC.md#5-versioning).

Comments and suggestions are welcome via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The keywords **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** in this document are to be interpreted as described in [[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/kyc-handoff/1.0`.
2. Populate `payload` with an object that validates against the JSON Schema in §JSON Schema.
3. Include a `proof` member per the [framework's proof rules](../../../SPEC.md#47-proof). This specification declares `proof` as **REQUIRED**; see the front matter for the rationale.
4. Identify itself as the *verifier* in the `issuer` member; identify the relying party in `recipient`. Per [SPEC.md §4.8.1](../../../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity), these in-band values are authoritative when present.

A conforming **consumer** **MUST**:

1. Validate the outer document and `payload` per [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements).
2. Verify the `proof` against the *verifier*'s declared verification material.
3. Treat the `result` value as authoritative within the assurance bound declared by `level`. Treat any extension entries in `evidence` per local policy.

Both parties **SHOULD** preserve unrecognized members to allow forward compatibility.

## Definitions

* **Verifier.** The KYC/AML provider that performed the underlying check on the subject. Identified by `issuer`.
* **Relying party.** The entity that consumes the handoff to make an onboarding or transaction-eligibility decision. Identified by `recipient`.
* **Subject.** The natural or legal person the KYC check was performed on. Identified by `payload.subject` as a *VID*; this VID is **not** required to be the same scheme as `issuer` or `recipient`.
* **Assurance level (`level`).** A NIST-style level-of-assurance label that bounds the strength of the underlying verification.

## Security & Privacy

The KYC Handoff payload conveys a single, finite verification outcome, not the underlying evidence. Producers **SHOULD NOT** embed primary identity attributes (date of birth, document numbers, biometric data) in `payload`; where evidence references are needed, they belong in the `evidence` array as opaque URIs the relying party can dereference under its own trust framework.

Because the handoff carries the subject's *VID* and a verification result, a captured document is sensitive personal data. Consumers **MUST** treat the document as personally identifiable information under their applicable privacy regime, **MUST** apply transport confidentiality appropriate to that regime, and **SHOULD** minimize retention to the period required for their own compliance evidence.

The `proof` requirement (**REQUIRED**) ensures that the verifier's attestation is bound to its own keys end-to-end, so that the relying party can rely on the result after the original transport interaction has ended.
