---
slug: keys/import-wrapping-key
version: "0.1"
title: "Keys — Import Wrapping Key"
summary: "A producer about to import a private key asks the custodian for a fresh, short-lived, single-use public key to seal it to, so the key stays confidential over a transport that is not end to end."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - keys
  - import
  - wrapping-key
  - hpke
  - sealed-transfer
  - custody
parties:
  - role: Producer (the party about to import a key)
    requirement: REQUIRED
    member: issuer
  - role: Key custodian
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: RECOMMENDED
  response: REQUIRED
  rationale: >-
    The response is the whole point of the task and the only part worth attacking. This task exists for transports whose confidentiality ends at an intermediary — TLS terminated at a load balancer or a host outside an enclave — and that same intermediary can replace the returned key with one of its own and read every private key sealed to it. Only a proof by the custodian's own DID, verified by the producer, detects the substitution, so the response proof is REQUIRED. The request asks for a public key and changes nothing durable; a proof on it is the family's read default, RECOMMENDED, where the transport does not already authenticate the producer.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: >-
    Nothing durable is executed. A replayed request yields another independent, single-use key that expires on its own within minutes, so there is no duplicate to absorb and no window to bound.
sideEffects:
  level: none
  rationale: >-
    Generates an ephemeral key pair that lives only in the custodian's memory, expires within minutes and is discarded on first use. No key record is created, nothing is persisted, and no existing state changes; the import that follows is its own consequential task.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The response is a public key, an opaque handle and an expiry. The private half never leaves the custodian. The request carries nothing.
retention:
  class: transient
  rationale: >-
    The custodian holds the private half until it is used or expires — minutes — and MUST NOT persist it. The producer discards the public key once it has sealed to it.
errorCodes: []
related:
  - keys/import
  - keys/create
  - vta/attestation/mnemonic-export
---

## Abstract

[`keys/import`](../../import/0.1/spec.md) carries a private key to a custodian in one of three carriers. Two of them — `privateKeySealed` and `privateKeyJwe` — encrypt the key *to the custodian*, and they are the only carriers a custodian accepts over a transport that is not end-to-end confidential, such as HTTPS whose TLS terminates at a load balancer, a gateway, or a host outside an enclave. A producer cannot encrypt to the custodian without a public key to encrypt to. This task supplies one.

The custodian generates a fresh key pair for each request, returns its public half, and keeps the private half in memory for a few minutes or until one import has used it, whichever comes first. The key belongs to no DID document and signs nothing; it exists so that one private key can cross one hop-by-hop transport without any intermediary reading it.

A producer on a transport that is end-to-end confidential between it and the custodian — an authenticated and encrypted TSP or DIDComm envelope — does not need this task, and **MAY** use `keys/import`'s `privateKeyMultibase` carrier instead.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/keys/import-wrapping-key/0.1`, with itself as `issuer` and the custodian as `recipient`.
2. **Verify the response's `proof` against the custodian's DID before sealing anything to `wrappingKey`**, and refuse to proceed when the proof is absent, fails to verify, or was made by any party other than the custodian it addressed. This is the step the task depends on: an unverified wrapping key is a key the transport's intermediary may have chosen.
3. Seal the private key to the X25519 counterpart of `wrappingKey` and send it in `keys/import`'s `privateKeySealed` carrier (or, for a producer that speaks JOSE, a `privateKeyJwe` whose `kid` is `keyId`) before `expiresAt`.
4. Use each wrapping key for at most one import, and discard it afterwards whether or not the import succeeded.

A conforming **consumer** (the key custodian) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements).
2. Generate a fresh key pair from a cryptographically secure source for every accepted request, and **MUST NOT** return the same `wrappingKey` twice.
3. Sign the response with a key of its own DID's `authentication` relationship ([SPEC §4.7](/SPEC.md#47-proof)).
4. Hold the private half only in memory, never in persistent storage, a backup, a log or an error, and zeroise it when it is used or expires. A custodian restart therefore invalidates every outstanding wrapping key, which is intended.
5. Open a `keys/import` carrier with a wrapping key **only** while the key is unexpired and unused, and discard the key as soon as it has opened one. A second import sealed to the same key, or any import arriving after `expiresAt`, cannot be opened and is refused as `keys/import` refuses any carrier it cannot open.
6. Set `expiresAt` to the instant it stops accepting the key. The lifetime **SHOULD** be no longer than a few minutes; long enough for a producer to seal and send one key, and no longer.

A consumer **MAY** bound the number of unexpired wrapping keys it holds, per producer or in total, and answer a request beyond that bound with `unavailable` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) and a `retryAfter`.

## Authorization

The authority this task assumes is **the authority to import a key into this custodian** — the same authority [`keys/import`](../../import/0.1/spec.md) requires, established the same way. A consumer **SHOULD** refuse the request with `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) from a producer that could not then import.

That gate does not protect the key, which is public by construction. It protects the custodian: every wrapping key costs it memory until it expires, and an ungated request lets any party that can reach it make it hold as many as it likes. The import that follows is the consequential step and carries its own authorization; nothing about holding a wrapping key entitles a producer to import.

A wrapping key is not bound to the producer that asked for it. A consumer **MAY** bind it — refusing an import sealed to a key a different producer obtained — and a producer **MUST NOT** rely on it being bound.

Verifying the producer's `proof` or transport identity establishes who asked, never that they may import ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The response's `proof` establishes that the key came from the custodian, which is what the producer needs; it asserts nothing about the producer.

## Definitions

* **Producer.** The party about to import a key; identified by `issuer`.
* **Key custodian.** The party that will receive and store the imported key; identified by `recipient`.
* **Wrapping key.** An ephemeral key pair generated by the custodian for one import. Its public half is returned as an Ed25519 `did:key`; producers seal to that key's X25519 counterpart, derived by the birational map of [RFC 7748 §4.1](https://www.rfc-editor.org/rfc/rfc7748#section-4.1), which is how every sealed-transfer recipient in this registry is addressed (compare `clientDid` in [`vta/attestation/mnemonic-export`](../../../vta/attestation/mnemonic-export/1.0/spec.md)).
* **Sealed-transfer bundle.** The armored HPKE envelope `keys/import`'s `privateKeySealed` carries: HPKE base mode, X25519-HKDF-SHA256 KEM, HKDF-SHA256 KDF, ChaCha20-Poly1305 AEAD, info string `vta-sealed-transfer/v1`, in OpenPGP-style ASCII armor.

## Request

A *request* document carries `type: https://trusttasks.org/spec/keys/import-wrapping-key/0.1` and an empty payload that validates against the top-level schema in [`payload.schema.json`](payload.schema.json). The request is the question.

### A producer asks for a wrapping key over HTTPS

```json
{
  "id": "urn:uuid:3b9f4a1e-7c2d-4e8f-9a6b-1c0d2e3f4a5b",
  "type": "https://trusttasks.org/spec/keys/import-wrapping-key/0.1",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:custodian.example",
  "issuedAt": "2026-09-27T09:10:00Z",
  "payload": {}
}
```

## Response

The custodian answers with a document of type `https://trusttasks.org/spec/keys/import-wrapping-key/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json), signed with its own DID's `authentication` key.

- `wrappingKey` — the public half, as an Ed25519 `did:key`. Seal to its X25519 counterpart.
- `keyId` — the custodian's opaque handle for the key; a `privateKeyJwe` names it as its `kid`.
- `expiresAt` — when the custodian stops accepting it.

Failures — `permissionDenied`, `unavailable` — use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### A wrapping key valid for one minute

```json
{
  "id": "urn:uuid:8d7c6b5a-4e3f-4a2b-9c1d-0e9f8a7b6c5d",
  "type": "https://trusttasks.org/spec/keys/import-wrapping-key/0.1#response",
  "threadId": "urn:uuid:3b9f4a1e-7c2d-4e8f-9a6b-1c0d2e3f4a5b",
  "issuer": "did:web:custodian.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-09-27T09:10:00Z",
  "payload": {
    "wrappingKey": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "keyId": "5f2c0a9e-1b7d-4c3e-8f6a-2d9b0e4c7a18",
    "expiresAt": "2026-09-27T09:11:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:custodian.example#key-1",
    "created": "2026-09-27T09:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

The producer then verifies that proof, seals the private key to the X25519 counterpart of `wrappingKey`, and sends `keys/import` with the armored bundle in `privateKeySealed` before 09:11:00.

## Security & Privacy

### Data carried

The request carries nothing. The response carries a public key, an opaque handle and an expiry — none of it secret and none of it personal. The private half of the wrapping key is the one sensitive thing this task creates, and it never leaves the custodian: it is held in memory, used at most once, and zeroised on use or expiry. A custodian **MUST NOT** write it to persistent storage, a backup or a log, because a wrapping key recovered later opens whatever private key was sealed to it.

### Correlation

A wrapping key is used once and names no one, so it links nothing beyond the one import it serves. That link is the point: a request for a wrapping key followed shortly by an import from the same producer tells an observer of the transport that a key is being imported, and when. It does not tell them which key, and — provided the producer verifies the response — the observer cannot learn the key itself. Request timing is unavoidable; a producer that cares can fetch a wrapping key well ahead of the import, within `expiresAt`.

### Retention

Transient on both sides. The custodian holds the private half for the key's lifetime and no longer; a restart discards every outstanding key, and a producer whose import then fails asks for another. The producer discards the public key once it has sealed to it. Neither the request nor the response has evidentiary value on its own — the import that follows is what the custodian records.

### Consent/purpose

The wrapping key's sole purpose is to receive one private key through one `keys/import`. A custodian **MUST NOT** open anything else with it — a carrier addressed to another task, or a second import — and a producer **MUST NOT** seal anything else to it.

### Threats

*Key substitution* is the threat this task exists under. The transports that need it are the ones with an intermediary that sees plaintext, and an intermediary that sees the response can replace `wrappingKey` with its own, open the bundle, and re-seal the private key to the real key so the import still succeeds and nobody notices. The response's `proof` defeats this only if the producer verifies it against the custodian's DID before sealing; a producer that skips the check has the confidentiality of the transport and nothing more, which is exactly what the cleartext carrier would have given it. Trusting the transport's own server certificate is not a substitute, because the substituting intermediary is the one presenting it.

*Resource exhaustion.* Every accepted request costs the custodian a key pair held in memory until it expires. The authorization gate and the per-producer bound above keep that cost with parties already entitled to import.

*Reuse.* A wrapping key opened twice would let a captured bundle be replayed into a second import. Single use is enforced by the custodian discarding the key on first successful open, not by the producer's good behaviour.
