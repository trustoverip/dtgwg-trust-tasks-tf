---
slug: webvh/witness/key/create
version: "0.1"
title: WebVH — Witness Key Create
summary: An administrator has a did:webvh witness service generate a new witness identity — a key and the DID it controls — that DID logs can then name as a witness.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, witness, key, create, admin]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Witness service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A new witness identity is a new signing authority resolvers may come to rely on. The witness service authorises on the document's proven issuer, and the proof is what makes the administrator the caller on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Each request creates a key; a replayed one would mint identities the administrator did not ask for, so it must be placeable in time for replay protection."
sideEffects:
  level: mutating
  rationale: "Generates and stores a new witness key and identity."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns the new identity's DID and metadata. The private key never leaves the witness service."
retention:
  class: exchange
  rationale: "The witness service keeps the identity it created, not the request; the request is needed only to answer it."
errorCodes: []
related:
  - webvh/witness/key/list
  - webvh/witness/key/delete
  - webvh/witness/sign
---

## Abstract

A witness service signs did:webvh witness proofs with one or more **witness identities**. **Witness Key Create** has the service generate a new one and returns its DID, which a DID controller then names in a log's witness parameter. It replaces the witness service's REST `POST /api/witnesses`.

The service generates the key itself. The request carries no key material and the response returns none; there is no way to import a key through this task.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming witness service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Generates a new key from a cryptographically secure source, derives the identity's DID from it, stores both, and returns the identity with `proofsSigned: 0`.
4. **MUST NOT** return, log or export the private key.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on the witness service**, read from its own access-control records at execution time. The verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Witness identity** — as in [`webvh/witness/sign`](../../../sign/0.1/spec.md#definitions).

## Request

```json
{
  "id": "urn:uuid:6d8f0a12-3b5c-4e7f-9a1b-2c3d4e5f6a01",
  "type": "https://trusttasks.org/spec/webvh/witness/key/create/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmWitnessScid6:witness.example.com",
  "issuedAt": "2026-09-27T09:40:00Z",
  "payload": { "label": "EU witness 1" }
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:6d8f0a12-3b5c-4e7f-9a1b-2c3d4e5f6a02",
  "type": "https://trusttasks.org/spec/webvh/witness/key/create/0.1#response",
  "threadId": "urn:uuid:6d8f0a12-3b5c-4e7f-9a1b-2c3d4e5f6a01",
  "issuer": "did:webvh:QmWitnessScid6:witness.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:40:01Z",
  "payload": {
    "key": {
      "witnessId": "w-01J8Z6Q4M2",
      "did": "did:key:z6MkWitness",
      "label": "EU witness 1",
      "createdAt": "2026-09-27T09:40:01Z",
      "proofsSigned": 0
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries an optional label; the response the new identity's identifier, DID, label and creation time. `label` is operator free text and **MUST NOT** be presented as an attested property. No key material crosses the wire.

### Correlation

The response links the witness service to a DID that resolvers will see in logs — which is the point: the identity is meant to be public once named. Both parties declare a public identifier scope because they must: the service authorises the administrator by matching the proven issuer against its access-control records, and the administrator addresses the service by its configured DID.

### Retention

The witness service keeps the identity and its key for as long as it witnesses with it, and nothing of the request. It **SHOULD** audit the creation.

### Consent/purpose

The identity is created so the service can witness DID logs that name it, and for nothing else.
