---
slug: webvh/witness/key/delete
version: "0.1"
title: WebVH — Witness Key Delete
summary: An administrator has a did:webvh witness service destroy one witness identity's key; every DID log that names it can no longer obtain its proofs.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, witness, key, delete, admin]
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
  rationale: "Destroying a witness key can leave DIDs unable to update. The witness service authorises on the document's proven issuer, and the proof is what makes the administrator the caller on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A destructive act must be placeable in time so a captured request cannot be replayed."
sideEffects:
  level: destructive
  rationale: "The key is destroyed irrecoverably. Proofs it signed stay valid, but no further proof can be obtained from it; a DID whose witness threshold needs it can no longer be updated until its controller names another witness."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The request names one identity; the response confirms its deletion."
retention:
  class: exchange
  rationale: "The service keeps an audit record of the deletion, not the request."
errorCodes:
  - code: webvh/witness/key/delete:notFound
    meaning: "The witness service holds no witness identity with the submitted `witnessId`."
    retryable: false
related:
  - webvh/witness/key/create
  - webvh/witness/key/list
  - webvh/witness/sign
---

## Abstract

**Witness Key Delete** destroys one witness identity's key. It replaces the witness service's REST `DELETE /api/witnesses/{witnessId}`.

The effect reaches beyond the service. Proofs the identity already signed stay valid — they verify against its DID, not against the service — but it can sign no more. **Every DID whose log names this witness can no longer obtain its proofs, and where the log's witness threshold needs this witness, can no longer be updated** until its controller publishes an entry naming a different witness — which itself needs the current witnesses' proofs. An administrator should confirm no active DID depends on the identity first.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming witness service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`, before looking the identity up.
3. Answers `webvh/witness/key/delete:notFound` when it holds no identity `witnessId`.
4. Otherwise destroys the key and the identity record, so no later request can sign with it, and returns the deletion time.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on the witness service**, read from its own access-control records at execution time. The verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Controlling a DID that names the witness gives no standing either to delete it or to prevent its deletion.

## Definitions

**Witness identity** — as in [`webvh/witness/sign`](../../../sign/0.1/spec.md#definitions).

## Request

```json
{
  "id": "urn:uuid:2b4d6f81-5c7e-4a9b-8d0f-1e2a3b4c5d01",
  "type": "https://trusttasks.org/spec/webvh/witness/key/delete/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmWitnessScid6:witness.example.com",
  "issuedAt": "2026-09-27T09:50:00Z",
  "payload": { "witnessId": "w-01J8Z6Q4M2" }
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:2b4d6f81-5c7e-4a9b-8d0f-1e2a3b4c5d02",
  "type": "https://trusttasks.org/spec/webvh/witness/key/delete/0.1#response",
  "threadId": "urn:uuid:2b4d6f81-5c7e-4a9b-8d0f-1e2a3b4c5d01",
  "issuer": "did:webvh:QmWitnessScid6:witness.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:50:01Z",
  "payload": { "witnessId": "w-01J8Z6Q4M2", "deletedAt": "2026-09-27T09:50:01Z" }
}
```

## Security & Privacy

### Data carried

The request carries one identity identifier; the response echoes it with the deletion time.

### Correlation

The request tells nothing beyond which identity the administrator retired. Both parties declare a public identifier scope because they must: the service authorises the administrator by matching the proven issuer against its access-control records, and the administrator addresses the service by its configured DID.

### Retention

The service **SHOULD** keep an audit record of the deletion — administrator, identity and its DID, time — because it can strand DIDs that depended on the identity and the record is how that is later explained. Nothing of the key is kept.

### Consent/purpose

Deletion exists so an operator can retire a witness identity — at end of life, or because its key is suspected compromised, in which case destroying it is the point despite the DIDs it strands. The service **MUST NOT** keep a copy of the key after answering.
