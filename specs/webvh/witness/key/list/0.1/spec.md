---
slug: webvh/witness/key/list
version: "0.1"
title: WebVH — Witness Key List
summary: An administrator lists the witness identities a did:webvh witness service holds — each one's DID, label, creation time and how many proofs it has signed. Never key material.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, witness, key, list, admin]
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
  rationale: "The list reveals every DID this service witnesses as, and is disclosed only to its administrators. The service authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the stored identities. Nothing is signed, counted or touched."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns each witness identity's identifier, DID, label, creation time and signature count. No key material."
retention:
  class: transient
  rationale: "The request carries nothing but its envelope and is needed only to answer it."
errorCodes: []
related:
  - webvh/witness/key/create
  - webvh/witness/key/delete
  - webvh/witness/sign
---

## Abstract

**Witness Key List** returns every witness identity a witness service holds. It replaces the witness service's REST `GET /api/witnesses` and `GET /api/witnesses/{witnessId}`.

**Why there is no get.** The registry's default for a collection is a list and a get ([CONTRIBUTING-SPECS.md, read-one and read-many](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)). A get earns its place when a caller needs a definite `notFound` from a read. Here it never does: the collection is a handful of operator-created identities returned whole, and every task that acts on one — [`webvh/witness/sign`](../../../sign/0.1/spec.md) and [`webvh/witness/key/delete`](../../delete/0.1/spec.md) — returns its own `notFound`. Existence is never load-bearing on a read.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming witness service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Returns every identity it holds, ordered by `createdAt`, changing nothing. A service holding none returns an empty array.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on the witness service**, read from its own access-control records at execution time. The verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Being authorised to request proofs from an identity is not standing to enumerate them.

## Definitions

**Witness identity** — as in [`webvh/witness/sign`](../../../sign/0.1/spec.md#definitions).

## Request

```json
{
  "id": "urn:uuid:9a1b2c3d-4e5f-4a6b-8c7d-0e1f2a3b4c01",
  "type": "https://trusttasks.org/spec/webvh/witness/key/list/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmWitnessScid6:witness.example.com",
  "issuedAt": "2026-09-27T09:45:00Z",
  "payload": {}
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:9a1b2c3d-4e5f-4a6b-8c7d-0e1f2a3b4c02",
  "type": "https://trusttasks.org/spec/webvh/witness/key/list/0.1#response",
  "threadId": "urn:uuid:9a1b2c3d-4e5f-4a6b-8c7d-0e1f2a3b4c01",
  "issuer": "did:webvh:QmWitnessScid6:witness.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:45:01Z",
  "payload": {
    "keys": [
      {
        "witnessId": "w-01J8Z6Q4M2",
        "did": "did:key:z6MkWitness",
        "label": "EU witness 1",
        "createdAt": "2026-09-27T09:40:01Z",
        "proofsSigned": 42
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries, per identity, an identifier, DID, optional label, creation time and signature count. No private or public key material beyond what the DID itself encodes. `label` is operator free text.

### Correlation

The list joins every witness DID the service holds to the service — the complete set of logs an observer could attribute to one operator by their witnesses. That is why it is administrator-only. Both parties declare a public identifier scope because they must: the service authorises the administrator by matching the proven issuer against its access-control records, and the administrator addresses the service by its configured DID.

### Retention

Nothing of the request needs keeping. A service that audits administrative reads **MAY** record the administrator.

### Consent/purpose

The list exists so an administrator can operate the service — see which identities exist, how busy each is, which to retire. It **MUST NOT** be used to publish the association between the identities.
