---
slug: vtc/directory/card/present
version: '0.1'
title: VTC Directory — Card — Present
summary: A member presents a credential so the community can fill credential-sourced directory fields with the disclosed claims, badged with their issuer; the community keeps the claims, never the presentation.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: The document binds the presentation to the member who sent it and to this community; without the proof a presentation could be replayed onto another member's card.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. The presentation's freshness is bound to this document, so the document itself must be placeable in time.
sideEffects:
  level: mutating
  rationale: Writes credential-sourced values on the member's own card. Recoverable by clearing them with a later presentation or by the community when the credential lapses.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: true
  rationale: The member presents a credential carrying personal claims. The community keeps only the claims the fields ask for, with their issuer, and returns them to the member.
maxDocumentBytes:
  request: 98304
  rationale: A presentation carrying a credential and its proofs, base64- or JSON-encoded, plus envelope and proof headroom.
retention:
  class: durable
  rationale: The verified values are kept on the card until the credential expires or the member clears them; the presentation itself is discarded once verified.
errorCodes:
- code: vtc/directory/card/present:notMember
  meaning: The signer is not an active member of this community.
  retryable: false
- code: vtc/directory/card/present:fieldUnknown
  meaning: '`fields` names a field the schema does not have.'
  retryable: false
- code: vtc/directory/card/present:fieldNotWritable
  meaning: '`fields` names a field that is not `credential`-sourced.'
  retryable: false
- code: vtc/directory/card/present:presentationInvalid
  meaning: The presentation does not verify, is not bound to this document and this community, or its holder is not the signer.
  retryable: false
- code: vtc/directory/card/present:issuerNotAccepted
  meaning: The credential's issuer is not one the field accepts.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    properties:
      field:
        type: string
        maxLength: 128
- code: vtc/directory/card/present:claimMissing
  meaning: The presentation does not disclose the claim a named field reads.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - field
    properties:
      field:
        type: string
        maxLength: 128
related:
- vtc/directory/card/update
- vtc/directory/fields/define
---

## Abstract

The **VTC Directory — Card — Present** Trust Task fills `credential`-sourced fields — a verified employer, a professional registration, "over 18" — from a credential the member presents. The community verifies the presentation, keeps **only** the disclosed claims each named field reads, records the issuer of each as its verification badge, and discards the presentation.

Only a value written this way is ever shown as verified, and it always names its issuer. A self-asserted value is never badged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is the member. It binds the presentation to this document: a Data Integrity presentation carries `nonce` equal to this document's `id` and `domain` equal to the community's DID as members of the presentation its holder proof signs, and a proof that also states `challenge` or `domain` states the same values; an SD-JWT presentation's key-binding JWT carries `nonce` equal to this document's `id` and `aud` equal to the community's DID. It discloses only the claims the named fields read.

A conforming **consumer** (the community) **MUST**:

1. Resolve the signer to their membership and refuse with `notMember` when they are not an active member.
2. Refuse a field that is not `credential`-sourced with `fieldNotWritable`.
3. Verify the presentation — its proofs, the credential's validity period and status, the binding above, and that its holder is the signer's membership DID or a DID the signer controls under the community's rules — and refuse with `presentationInvalid` otherwise.
4. For each named field, check the credential type and issuer against the field's `ClaimSelector` and refuse with `issuerNotAccepted` or `claimMissing`.
5. Write each field as a `VerifiedValue` naming the issuer, with `validUntil` the credential's expiry, leave the member's visibility choices untouched, increment `revision`, and audit by reference.
6. **MUST NOT** retain the presentation, the credential, or any claim no named field reads, once the request completes.
7. Withdraw a verified value once its `validUntil` passes.

## Authorization

**Authority: being the member.** The values written are the signer's own, on their own card. The presentation is evidence of the claim, not of the authority to write it; the binding in Conformance item 3 is what stops a presentation obtained from another member being written onto this one's card. The proof on the document establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `VerifiedValue` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).
- `ClaimSelector` — see [`field-definition.schema.json`](../../../_shared/0.1/field-definition.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Present a professional registration

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001b",
  "type": "https://trusttasks.org/spec/vtc/directory/card/present/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001a",
  "payload": {
    "fields": [
      "org.example.registration"
    ],
    "presentation": "eyJhbGciOiJFZERTQSJ9.example-sd-jwt~disclosure~kb-jwt"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/card/present/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Present a professional registration — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001c",
  "type": "https://trusttasks.org/spec/vtc/directory/card/present/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001a",
  "payload": {
    "verified": {
      "org.example.registration": {
        "value": "REG-48213",
        "issuer": "did:example:registry",
        "credentialType": "ProfessionalRegistration",
        "verifiedAt": "2026-10-18T09:00:01Z",
        "validUntil": "2027-06-30T00:00:00Z"
      }
    },
    "revision": 6
  }
}
```

## Security & Privacy

### Data carried

A presentation, which may carry more than the fields need; the community keeps only the claims named fields read. A producer **SHOULD** use selective disclosure so that nothing else is sent at all.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. A credential names its issuer and usually its subject; presenting it links the member's membership to that issuer's view of them. The community shows the issuer as a badge, to the audiences the member chooses for the field, so the member decides who learns of the link.

### Retention

The presentation is **transient**: discarded once verified. The verified values are **durable** until the credential expires, the member clears them or the member departs.

### Consent/purpose

To let a member show a verified fact in the directory. The disclosed claims are kept for that and nothing else; the community **MUST NOT** reuse them, or treat a presentation here as satisfying any join criterion or other check, without a separate presentation for that purpose.
