---
slug: webvh/witness/publish
version: "0.1"
title: WebVH — Witness Publish
summary: A did:webvh owner publishes a witness-signed proof over a log entry so the hosting service can append it to the DID's permanent witness file.
status: draft
targetFrameworkVersion: "0.1"
category: did-management
keywords: [webvh, witness, oracle, did, proof]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID owner
    requirement: REQUIRED
  - role: Hosting service
    requirement: REQUIRED
proofRequirement:
  requirement: RECOMMENDED
  rationale: The witness proof object inside `payload.witness` carries its own cryptographic signature from the witness oracle; an outer Trust Task `proof` becomes valuable only when the request is replayed for audit and is not strictly required when the producer is bound by an authenticated transport.
errorCodes:
  - code: webvh/witness/publish:not_owner
    meaning: The caller is not the owner of the named slot.
    retryable: false
  - code: webvh/witness/publish:invalid_witness
    meaning: The supplied `witness` object failed structural validation (empty object, missing signature, or signature did not verify against the expected witness DID).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
  - code: webvh/witness/publish:slot_not_found
    meaning: The named `mnemonic` does not exist on this hosting service.
    retryable: false
related:
  - did-management/did/publish
  - webvh/sync/update
---

## Abstract

The **Witness Publish** Trust Task is part of did:webvh's witness-oracle pattern. A DID owner who has obtained a witness signature over the current log-entry hash submits that signed witness proof to the hosting service, which appends it to the DID's `did-witness.json` block so future resolvers can verify the published-at-time-T claim independently of the host.

This task is webvh-protocol-specific — other DID methods do not use this oracle pattern, which is why the spec lives under the `webvh/` family rather than `did-management/`. The witness proof itself (the contents of `payload.witness`) follows the did:webvh specification's witness-proof shape; this Trust Task spec governs only the envelope and the delivery semantics.

The hosting service stores the latest witness proof per DID; uploading a new witness with a higher version-id supersedes the previous one. After a successful publish, the hosting service fans the updated witness content out to any registered hosting servers via [`webvh/sync/update`](../../sync/update/0.1/spec.md).

## Status of this Document

Draft.

## Conformance

Producer (DID owner) MUST emit `type: https://trusttasks.org/spec/webvh/witness/publish/0.1` with `payload.mnemonic` (the slot identifier) and `payload.witness` (the witness proof as a JSON object). Consumer (hosting service) MUST:

1. Verify the caller is the owner of `mnemonic`, else reject with `webvh/witness/publish:not_owner`.
2. Validate that `witness` is a non-empty object conforming to the did:webvh witness-proof shape and that its signature verifies against the configured witness DID(s) for the slot's domain, else reject with `webvh/witness/publish:invalid_witness`.
3. Persist the witness proof as the slot's canonical `did-witness.json` and respond with the URL at which it is served.
4. Trigger fan-out to registered servers via `webvh/sync/update`.

## Request

```json
{ "id": "wp-1", "type": "https://trusttasks.org/spec/webvh/witness/publish/0.1",
  "issuer": "did:key:z6MkAlice", "recipient": "did:web:did.example.com",
  "issuedAt": "2026-06-25T09:00:00Z",
  "payload": {
    "mnemonic": "alice",
    "witness": {
      "versionId": "5-abcdef...",
      "witness": "did:webvh:WIT1:witness.example.com",
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-06-25T08:59:55Z",
        "verificationMethod": "did:webvh:WIT1:witness.example.com#key-1",
        "proofPurpose": "assertionMethod",
        "proofValue": "z..."
      }
    }
  } }
```

## Response

```json
{ "id": "wp-1-r", "type": "https://trusttasks.org/spec/webvh/witness/publish/0.1#response",
  "threadId": "wp-1", "issuer": "did:web:did.example.com", "recipient": "did:key:z6MkAlice",
  "issuedAt": "2026-06-25T09:00:01Z",
  "payload": {
    "mnemonic": "alice",
    "witnessUrl": "https://did.example.com/alice/did-witness.json"
  } }
```

## Security & Privacy

The witness proof inside `payload.witness` carries its own signature from the witness oracle; that signature is the load-bearing authenticator. The outer Trust Task envelope binds the witness publication to the caller's DID, which the hosting service uses to authorize the slot update — without that ownership binding an attacker who intercepted a published witness could replay it under a different owner.

Witness oracles MUST be allow-listed per hosting domain; consumers MUST refuse witness proofs signed by DIDs not on the domain's witness list, even if the caller is the legitimate slot owner.
