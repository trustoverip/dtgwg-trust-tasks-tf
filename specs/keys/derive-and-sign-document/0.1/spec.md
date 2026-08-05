---
slug: keys/derive-and-sign-document
version: "0.1"
title: Keys — Derive and Sign Document
summary: A producer hands a custodian a JSON document; the custodian derives a key at a path, canonicalizes the document and returns it with a Data Integrity proof attached.
status: draft
targetFrameworkVersion: "0.1"
category: key-management
keywords:
  - keys
  - signing
  - data-integrity
  - derivation
  - json
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The custodian produces a proof that will be verified by third parties as the derived identity's assertion, so it must be able to attribute the request that asked for it. Reliance by parties beyond the original consumer is exactly the §4.7.1 condition under which a proof is a MUST, and forgery of the request would yield an assertion in the subject's name that no one can trace back to a caller.
sideEffects:
  level: none
  rationale: "No key record is created and no stored state changes. The proof is a durable artefact."
exposure:
  discloses: none
  actsAsSubject: true
  rationale: The custodian exercises the derived key to produce a Data Integrity proof that third-party verifiers will read as the derived identity's own assertion over the document's content.
errorCodes: []
related:
  - keys/derive-and-sign
  - keys/sign
  - vault/sign-trust-task
---

## Abstract

The **Keys — Derive and Sign Document** Trust Task signs a JSON document as a derived identity, returning the document with a [Data Integrity](https://www.w3.org/TR/vc-data-integrity/) `proof` attached and the `did:key` it was signed as.

It differs from [`keys/derive-and-sign`](../../derive-and-sign/0.1/spec.md) in **who canonicalizes**. There, the producer decides what bytes mean and the custodian signs them blind. Here the producer hands over a document and the custodian performs the canonicalization and proof construction — which is the only way the resulting proof can be one a third-party Data Integrity verifier will accept, because the canonicalization must match what that verifier will recompute.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/derive-and-sign-document/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. Populate `keyType`, `derivationPath` and `document`.
3. **Not** rely on any `proof` it includes in `document` surviving — see the consumer rule below.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements).
2. Decide whether this producer may derive and sign at this path, refusing with `permission_denied` ([SPEC.md §8.3](../../../../SPEC.md#83-standard-error-codes)) otherwise.
3. **Strip any existing `proof` member before canonicalizing.** Signing over a document that still carries a previous proof produces a proof over a signature rather than over content, which no verifier will reproduce and which quietly makes the second signature meaningless.
4. Canonicalize and construct the proof, recording `proofPurpose` (defaulting to `assertionMethod`) and a verification method that resolves under the derived `did:key`.
5. Return `signerDid` and the proofed `document`, and **not** the derived private key.

## Definitions

* **Producer.** The party supplying the document; identified by `issuer`.
* **Key custodian.** The party deriving and signing; identified by `recipient`.
* **Signer DID.** The `did:key` of the derived key — the identity a verifier resolves the proof against.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/derive-and-sign-document/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

```json
{
  "id": "e8f90112-2334-4459-5667-788990011223",
  "type": "https://trusttasks.org/spec/keys/derive-and-sign-document/0.1",
  "issuer": "did:web:app.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-07-31T09:50:00Z",
  "payload": {
    "keyType": "ed25519",
    "derivationPath": "m/26'/9'/0'",
    "document": {
      "id": "urn:uuid:6f1b2c3d-4e5f-4061-8293-a4b5c6d7e8f9",
      "type": "https://trusttasks.org/spec/auth/authenticate/0.1",
      "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "payload": { "sessionId": "s-1", "challenge": "c-1" }
    },
    "proofPurpose": "assertionMethod"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/keys/derive-and-sign-document/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

```json
{
  "id": "f9011223-3445-4560-6778-899001122334",
  "type": "https://trusttasks.org/spec/keys/derive-and-sign-document/0.1#response",
  "threadId": "e8f90112-2334-4459-5667-788990011223",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:app.example",
  "issuedAt": "2026-07-31T09:50:01Z",
  "payload": {
    "signerDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "document": {
      "id": "urn:uuid:6f1b2c3d-4e5f-4061-8293-a4b5c6d7e8f9",
      "type": "https://trusttasks.org/spec/auth/authenticate/0.1",
      "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "payload": { "sessionId": "s-1", "challenge": "c-1" },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-07-31T09:50:01Z",
        "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "proofPurpose": "assertionMethod",
        "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSTjXwT2c1h2G5nP8aQ"
      }
    }
  }
}
```

Failures (`permission_denied`, `invalid_argument`) use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

## Security & Privacy

Unlike the opaque-bytes tasks, the custodian here **can** see what it is signing — the document is structured JSON. That is a meaningful difference: a custodian **MAY** apply policy to the document's content, and one signing high-value assertions **SHOULD**. It is the only task in this family where refusing on the basis of *what* is being signed is even possible.

The stripped-`proof` rule is the subtle one. A producer that resubmits an already-proofed document expecting counter-signature will instead receive a document whose earlier proof has been discarded and replaced. Producers needing multiple proofs over one document **MUST** collect them in a structure that holds several, rather than round-tripping through this task twice.

The path-authority discussion in [`keys/derive-and-sign`](../../derive-and-sign/0.1/spec.md) applies unchanged.
