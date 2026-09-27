---
slug: webvh/witness/sign
version: "0.1"
title: WebVH — Witness Sign
summary: A requester asks a did:webvh witness service to witness one log entry; the witness verifies the log up to that entry and that it is named as a witness there, and only then returns its signed proof over the entry's versionId.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, witness, proof, attestation, log]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Requester
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Witness service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A witness proof is an attestation resolvers rely on to accept a log entry, so exercising a witness key is privileged. The witness service authorises on the document's proven issuer, and the proof is what makes the requester the caller on every transport rather than whoever holds a session."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Each request makes the witness sign; a captured one replayed later must be placeable in time so the witness's replay protection can refuse it."
sideEffects:
  level: mutating
  rationale: "The witness signs with its key and counts the signature. The signature is an attestation that outlives the exchange and cannot be withdrawn."
subjectPath: /versionId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a public DID log; the response discloses a signature over one of its versionIds — public data once published in the DID's did-witness.json."
retention:
  class: durable
  rationale: "The response's proof is what the DID's did-witness.json carries for as long as the entry is served; a witness SHOULD retain an audit record of what it signed and for whom."
errorCodes:
  - code: webvh/witness/sign:notFound
    meaning: "The witness service holds no witness identity with the submitted `witnessId`."
    retryable: false
  - code: webvh/witness/sign:invalidLog
    meaning: "`logContent` does not verify as a did:webvh log — a broken hash chain, a bad controller proof, or a pre-rotation violation."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
  - code: webvh/witness/sign:versionNotLast
    meaning: "`versionId` is not the versionId of the last entry of `logContent`."
    retryable: false
  - code: webvh/witness/sign:notListed
    meaning: "The entry's active witness parameter does not name this witness identity's DID, so its proof would witness nothing."
    retryable: false
  - code: webvh/witness/sign:deactivated
    meaning: "The log deactivates the DID before or at the entry; a deactivated DID is not witnessed."
    retryable: false
related:
  - webvh/witness/key/create
  - webvh/witness/key/list
  - webvh/witness/key/delete
  - webvh/witness/publish
---

## Abstract

did:webvh lets a DID controller require that each log entry be co-signed by witnesses before resolvers accept it. **Witness Sign** is how a requester obtains one witness identity's proof for one entry from a witness service. It replaces the witness service's REST `POST /api/proof/{witnessId}`.

That REST call took a bare `versionId` and signed it. This task does not: **a witness MUST NOT sign a versionId it has not verified.** A witness that signs whatever it is handed attests nothing — it is a signing oracle for its own key. So the request carries the log up to and including the entry, and the witness verifies the chain, that the entry is the last one, that the entry's active witness parameter names this witness, and that the DID is not deactivated, before it signs. The proof it returns is then carried into the DID's `did-witness.json` (see [`webvh/witness/publish`](../../publish/0.1/spec.md)).

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming witness service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a requester it does not authorise to exercise the named witness identity ([Authorization](#authorization)) with `permissionDenied`, before looking the identity up.
3. Answers `webvh/witness/sign:notFound` when it holds no identity `witnessId`.
4. Verifies `logContent` as a did:webvh log, else `invalidLog`; requires `versionId` to be its last entry, else `versionNotLast`; requires the witness parameter in force for that entry to name the identity's DID, else `notListed`; and refuses a deactivated DID with `deactivated`.
5. Only then signs a W3C Data Integrity proof over `{"versionId": …}` with the identity's key (`eddsa-jcs-2022` for an Ed25519 key), increments the identity's `proofsSigned`, and returns the proof.

A witness **MUST NOT** sign on any path that skips step 4.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **the witness service's authorisation of the requester to exercise that witness identity**, read from the witness service's own access-control records at execution time — in current deployments, administrator standing on the witness service. The verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Controlling the DID being witnessed is not standing, and neither is being named as a controller in the log: the witness decides whom it serves. Step 4 is not authorization either; it is what makes the signature true.

## Definitions

**Witness identity** — one key, and the DID it controls, that a witness service signs witness proofs with. A did:webvh log names witnesses by that DID.

## Request

```json
{
  "id": "urn:uuid:1f3e5a70-2b4c-4d6e-8f0a-9b8c7d6e5f01",
  "type": "https://trusttasks.org/spec/webvh/witness/sign/0.1",
  "issuer": "did:webvh:QmNodeScid3:node1.example.com",
  "recipient": "did:webvh:QmWitnessScid6:witness.example.com",
  "issuedAt": "2026-09-27T09:30:00Z",
  "payload": {
    "witnessId": "w-01J8Z6Q4M2",
    "versionId": "2-QmB",
    "logContent": "{\"versionId\":\"1-QmA\",\"versionTime\":\"2026-09-01T10:00:00Z\"}\n{\"versionId\":\"2-QmB\",\"versionTime\":\"2026-09-27T08:59:00Z\"}"
  }
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:1f3e5a70-2b4c-4d6e-8f0a-9b8c7d6e5f02",
  "type": "https://trusttasks.org/spec/webvh/witness/sign/0.1#response",
  "threadId": "urn:uuid:1f3e5a70-2b4c-4d6e-8f0a-9b8c7d6e5f01",
  "issuer": "did:webvh:QmWitnessScid6:witness.example.com",
  "recipient": "did:webvh:QmNodeScid3:node1.example.com",
  "issuedAt": "2026-09-27T09:30:01Z",
  "payload": {
    "witnessId": "w-01J8Z6Q4M2",
    "versionId": "2-QmB",
    "proof": {
      "type": "DataIntegrityProof",
      "cryptosuite": "eddsa-jcs-2022",
      "verificationMethod": "did:key:z6MkWitness#z6MkWitness",
      "proofPurpose": "assertionMethod",
      "created": "2026-09-27T09:30:01Z",
      "proofValue": "z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEcqdoDwLWxaqA3Q1geV6"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a witness identifier, a versionId and a public did:webvh log. The response carries a signature over the versionId. Nothing is secret; the witness key never leaves the service.

### Correlation

The request tells the witness service which DID the requester is asking it to witness and when — a relationship the witness would learn anyway by being named in the log. Both parties declare a public identifier scope because they must: the witness service authorises the requester by matching the proven issuer against its access-control records, and the requester addresses the witness service by the DID it was configured with. A pairwise identifier would match no entry.

### Retention

The proof is durable by nature: it is published in the DID's `did-witness.json` for as long as the entry is served. A witness service **SHOULD** retain an audit record of each signature — requester, witness identity, DID and versionId — because a signature is an attestation it cannot withdraw.

### Consent/purpose

The log is submitted so the witness can verify what it attests. The witness **MUST NOT** use it for any other purpose, and **MUST NOT** sign an entry it has not verified.
