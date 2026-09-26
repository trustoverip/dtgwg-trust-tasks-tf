---
slug: vta/attestation/mnemonic-export
version: "1.0"
title: "VTA Attestation — Mnemonic Export"
summary: "Release a TEE agent's seed mnemonic once, sealed to the requester, over an end-to-end channel or as a request the requester signed for its own key."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - mnemonic
  - seed
  - backup
  - tee
  - sealed-transfer
parties:
  - role: backup operator
    requirement: REQUIRED
    member: issuer
  - role: verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is the only evidence of who asked for the root of every key the agent holds, and the recipient spends a one-time release on it. The response is REQUIRED because it is the agent's own statement of what it released and to which key: an unsigned answer is one an intermediary could replace with a bundle sealed to a key of its own choosing.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The release is one-time, so a replay cannot obtain a second copy; it can still spend the release on a stale request whose key the operator no longer holds, and the operator's backup is lost. Bounding replay keeps the one release for a request someone is waiting on.
sideEffects:
  level: mutating
  rationale: >-
    Releasing the mnemonic consumes it: the agent wipes the entropy once the sealed answer is built and recorded, and no later request can obtain it. The agent's keys are unchanged, but the one chance to take an offline copy of their root is spent.
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response is the agent's root derivation material, sealed. Whoever opens it can re-derive every key the agent derives, for as long as those keys are in use. The request carries only a public key and a nonce.
retention:
  class: durable
  rationale: >-
    The purpose is an offline paper backup, held indefinitely. The obligation that comes with that is the producer's: the words are held to the standard of the agent's own key custody, offline and nowhere else.
errorCodes: []
related:
  - vta/backup/initiate-export
  - keys/export-secret
---

## Abstract

An agent running inside a trusted execution environment (TEE) generates its seed on first boot, inside the enclave, and never displays it. If the operator wants an offline backup of that seed, there is one short window after first boot in which the agent will release its BIP-39 mnemonic, once. This task is that release.

The mnemonic is the root every derived key of the agent comes from. So the task is shaped to keep it off every party except the one that asked: the answer is sealed to a key only the producer holds, the recipient proves by attestation that the answer came from the enclave, and the request is either carried over a channel that is confidential end to end or signed by the producer so that no intermediary can change whom the answer is sealed to (see [Channel](#channel)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **custody of the agent itself**, and in addition the entitlement to **export key material**. The mnemonic reproduces every derived key the agent holds, so releasing it is the widest export there is. A recipient **MUST** refuse a producer that is not an owner or operator of the agent at the level that could equally destroy it, and **MUST** refuse one whose authority has been narrowed so that it may not export keys, even if it could otherwise administer the agent.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes *who asked*. None of it establishes entitlement.

This specification does not, and **MUST NOT**, declare that a human approval or a step-up is required ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). That is consumer policy.

## Definitions

**Export window** — the period after the agent's first boot during which it holds the entropy its seed was generated from. Outside it, or once the mnemonic has been released, the agent holds no entropy and cannot perform this task. The window's length is deployment configuration; an agent **MAY** have none.

**`clientDid`** — the Ed25519 `did:key` the answer is sealed to, by its X25519 counterpart. On the [end-to-end path](#end-to-end-path) it is a key the producer generated for this one export; on the [signed first-boot path](#signed-first-boot-path) it is the producer's own DID, the one that signed the request. It is a `did:key` so that no resolution is involved: the key the root mnemonic is sealed to is in the request itself, and no party that controls a resolver can substitute another.

**`nonce`** — sixteen random bytes, base64url without padding. The sealed bundle is identified by it, and the recipient's attestation quote binds it.

**Sealed bundle** — the answer: the mnemonic sealed with HPKE to `clientDid`'s X25519 key, carrying a producer assertion whose attestation quote binds `SHA-256(client Ed25519 key ‖ nonce ‖ recipient sealing key)`. `digest` is the SHA-256 of the sealed bytes.

## Channel

A recipient **MUST** accept this task over exactly two paths, the [end-to-end path](#end-to-end-path) and the [signed first-boot path](#signed-first-boot-path), and **MUST** refuse it over any other with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)). Whichever path carries it, the recipient establishes entitlement first (see [Authorization](#authorization)) and applies the path's rules before it reads the entropy, so a refused request does not spend the release. A refusal **SHOULD** say which path the request failed and what that path requires.

On both paths the request carries a `proof` ([SPEC §4.7](/SPEC.md#47-proof)) with `proofPurpose` `authentication`, made by a verification method listed under the `issuer`'s `authentication` relationship. The `issuer` **MUST** be the identity the transport or the session authenticated, and `recipient` **MUST** be present and name the recipient. A recipient **MUST** refuse a request whose proof is absent or fails verification, or verifies as a DID other than `issuer`.

### End-to-end path

The task travels over a channel confidential end to end between the producer and the recipient: one on which the producer encrypts to the recipient itself, such as the DIDComm binding with authenticated encryption, or the TSP binding. `clientDid` **SHOULD** be a key the producer generated for this one export, on the machine that will hold the backup.

The seal does not make the channel irrelevant. The seal keeps the words from anyone without `clientDid`'s private key. The channel keeps an intermediary from reading the request, learning which key the root is sealed to and when the one release happened, or holding the sealed answer against a later compromise of that key.

### Signed first-boot path

An agent that has just booted may not yet be reachable over an end-to-end channel: its DID is not yet published with a DIDComm or TSP endpoint, or it has no mediator. The export window is short, so waiting for one can mean losing the only chance to take the backup. For that case the recipient **MAY** accept the task over a channel confidential only hop by hop, the HTTPS binding included, when **all** of the following hold. It **MUST** refuse it otherwise.

1. The producer is entitled ([Authorization](#authorization)).
2. The request carries a `proof` that verifies as `issuer`, with `proofPurpose` `authentication`, and `issuer` is the DID the session authenticated. A bearer credential alone never qualifies: whoever terminates TLS holds it too.
3. `clientDid` is **exactly** `issuer`. The producer signs with the key the answer is sealed to.
4. `recipient` is present and is the recipient's own DID.
5. `issuedAt` is present and inside the recipient's freshness bound, and the document's `id` has not been seen before: the recipient records it in its duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) before it acts, and a duplicate **MUST NOT** be answered with the bundle.

The requirements are chosen against an active intermediary that terminates TLS and holds the producer's bearer credential. It cannot sign as the producer, so it cannot make a request of its own that passes item 2. It cannot change `clientDid` in the producer's request, because the proof covers the payload, and item 3 leaves it nothing to change it to. It cannot redirect the request to another agent (item 4) or replay it (item 5). What it can still do is see that the export happened, which key the answer is sealed to, and the sealed answer itself, and keep that answer. Whoever later obtains the producer's private key can open it. So on this path the producer **SHOULD** sign with a `did:key` credential kept for this purpose on the machine that will hold the backup, and **SHOULD** retire it (revoke its access to the agent and destroy its private key) once the words are written down.

A request over a hop-by-hop channel whose `clientDid` differs from `issuer`, or which lacks a proof, **MUST** be refused, even when every other requirement holds.

## Request

The producer is the backup operator, on the machine that will hold the backup; the recipient is the agent. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Asking for the mnemonic over an end-to-end channel

The answer is sealed to an ephemeral key; the operator signs as itself.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/attestation/mnemonic-export/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "payload": {
    "clientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "nonce": "BwcHBwcHBwcHBwcHBwcHBw"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-01-01T00:00:00Z",
    "verificationMethod": "did:example:operator#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTNQ2p4D5eQzGnakbDwhXSbpYKMFwHUdktgYZZT"
  }
}
```

### Asking for the mnemonic on the signed first-boot path

The operator is `did:key:z6MkhaXg…`, signs with that key, and names the same DID as `clientDid`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vta/attestation/mnemonic-export/1.0#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "payload": {
    "clientDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "nonce": "CAgICAgICAgICAgICAgICA"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-01-01T00:00:00Z",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "proofPurpose": "authentication",
    "proofValue": "z4oey5q2M3XKaxup3tmzN4DRFTLVqpLMweBrSxMY2xHX5XTYVQeVbY8nQAVHMrXFkXJpmEAXm5Yjv8fbvq7FxpLpJ"
  }
}
```

## Response

The response payload is `#/$defs/Response` in [`payload.schema.json`](payload.schema.json).

### The sealed mnemonic

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/attestation/mnemonic-export/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "payload": {
    "bundle": "-----BEGIN VTA SEALED BUNDLE-----\n…\n-----END VTA SEALED BUNDLE-----",
    "digest": "9f2c3a7e5d1b4c8a6e0f2d4b6a8c0e2f4a6c8e0b2d4f6a8c0e2b4d6f8a0c2e4b",
    "windowRemainingSecs": 412
  }
}
```

## Processing

A recipient performs the release in this order, and **MUST NOT** reorder the last three steps:

1. Establish entitlement ([Authorization](#authorization)), then the path and its rules ([Channel](#channel)).
2. Refuse, with a failure naming the reason, when it holds no entropy (not a first boot, the window has closed, or the release already happened), or when another release is in progress.
3. Seal the mnemonic to `clientDid` and produce the attestation-bound assertion.
4. Durably record the release — who asked, the recipient key, the transport, and `digest`; never the words — and refuse if it cannot. An unrecorded release of the root is not permitted.
5. Wipe the entropy, then answer.

A failure in steps 2 to 4 **MUST** leave the release available inside the window: the entropy exists nowhere else, and consuming it before the answer is built would turn a failed seal into a lost root.

## Security & Privacy

### Data carried

The response carries the agent's **root secret**, sealed. Whoever opens it can re-derive every key the agent derives. The producer **MUST** open it only on the machine that will hold the backup, **MUST** confirm `digest` out of band before opening, and **SHOULD** discard `clientDid`'s private key once the words are written down. On the [signed first-boot path](#signed-first-boot-path) that key is the producer's own credential, and discarding it means retiring the credential.

The request carries only a public key and a nonce, and the recipient **MUST NOT** treat `label` as anything but text to show an operator.

The recipient **MUST NOT** answer with the mnemonic in any form other than the sealed bundle, and **MUST NOT** log, audit or cache the words or the bundle. Its duplicate-execution record keeps the document's `id` and digest, never the response.

### Correlation

The recipient learns when the operator took the backup and which key it used. Both are recorded, deliberately: the release of the root is the one fact about the agent a later investigation has to be able to find. On the end-to-end path `clientDid` is ephemeral and **SHOULD NOT** be reused anywhere else. On the signed first-boot path an intermediary terminating TLS learns the same two facts; that is the price of the path, and the reason the credential is retired afterwards.

### Retention

The recipient retains no copy: the entropy is wiped with the release. It **SHOULD** retain its record of the release for as long as the agent's keys are in use.

### Consent/purpose

The purpose is an offline backup of the agent's root, for recovery. A producer **MUST NOT** use the mnemonic to run a second copy of the agent: two agents deriving the same keys defeats the custody every other authority in the system rests on.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraph above states a purpose limitation, which is a different thing.
