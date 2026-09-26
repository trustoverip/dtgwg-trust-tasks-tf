---
slug: vta/attestation/mnemonic-export
version: "1.0"
title: "VTA Attestation — Mnemonic Export"
summary: "Release a TEE agent's seed mnemonic once, sealed to the requester, over an end-to-end channel only."
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

The mnemonic is the root every derived key of the agent comes from. So the task is shaped to keep it off every party except the one that asked: the answer is sealed to a key the producer generated for this one export, the recipient proves by attestation that the answer came from the enclave, and the task travels only over a channel that is confidential end to end.

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

**`clientDid`** — the producer's ephemeral Ed25519 `did:key`. The answer is sealed to its X25519 counterpart. It is a `did:key` so that no resolution is involved: the key the root mnemonic is sealed to is in the request itself, and no party that controls a resolver can substitute another.

**`nonce`** — sixteen random bytes, base64url without padding. The sealed bundle is identified by it, and the recipient's attestation quote binds it.

**Sealed bundle** — the answer: the mnemonic sealed with HPKE to `clientDid`'s X25519 key, carrying a producer assertion whose attestation quote binds `SHA-256(client Ed25519 key ‖ nonce ‖ recipient sealing key)`. `digest` is the SHA-256 of the sealed bytes.

## Channel requirement

This task **MUST** be carried over a channel confidential end to end between the producer and the recipient: one on which the producer encrypts to the recipient itself, such as the DIDComm binding with authenticated encryption, or the TSP binding. A channel confidential only hop by hop does not qualify, the HTTPS binding included: TLS terminates wherever the recipient's operator terminates it, which for an agent in a TEE is outside the enclave by definition.

The seal does not make the channel irrelevant. The seal keeps the words from anyone without `clientDid`'s private key. The channel keeps an intermediary from reading the request, learning which key the root is sealed to and when the one release happened, or holding the sealed answer against a later compromise of the producer's machine. For the root of every key the agent holds, both are required.

A recipient **MUST** refuse this task with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)) when it arrives over any other channel. It refuses after establishing entitlement (see [Authorization](#authorization)) and before it reads the entropy, so a refused request does not spend the release. The refusal **SHOULD** name the bindings the recipient accepts.

## Request

The producer is the backup operator, on the machine that will hold the backup; the recipient is the agent. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Asking for the mnemonic

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

1. Establish entitlement ([Authorization](#authorization)), then the channel ([Channel requirement](#channel-requirement)).
2. Refuse, with a failure naming the reason, when it holds no entropy (not a first boot, the window has closed, or the release already happened), or when another release is in progress.
3. Seal the mnemonic to `clientDid` and produce the attestation-bound assertion.
4. Durably record the release — who asked, the recipient key, the transport, and `digest`; never the words — and refuse if it cannot. An unrecorded release of the root is not permitted.
5. Wipe the entropy, then answer.

A failure in steps 2 to 4 **MUST** leave the release available inside the window: the entropy exists nowhere else, and consuming it before the answer is built would turn a failed seal into a lost root.

## Security & Privacy

### Data carried

The response carries the agent's **root secret**, sealed. Whoever opens it can re-derive every key the agent derives. The producer **MUST** open it only on the machine that will hold the backup, **MUST** confirm `digest` out of band before opening, and **SHOULD** discard `clientDid`'s private key once the words are written down.

The request carries only a public key and a nonce, and the recipient **MUST NOT** treat `label` as anything but text to show an operator.

The recipient **MUST NOT** answer with the mnemonic in any form other than the sealed bundle, and **MUST NOT** log, audit or cache the words or the bundle.

### Correlation

The recipient learns when the operator took the backup and which ephemeral key it used. Both are recorded, deliberately: the release of the root is the one fact about the agent a later investigation has to be able to find. `clientDid` is ephemeral and **SHOULD NOT** be reused anywhere else.

### Retention

The recipient retains no copy: the entropy is wiped with the release. It **SHOULD** retain its record of the release for as long as the agent's keys are in use.

### Consent/purpose

The purpose is an offline backup of the agent's root, for recovery. A producer **MUST NOT** use the mnemonic to run a second copy of the agent: two agents deriving the same keys defeats the custody every other authority in the system rests on.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraph above states a purpose limitation, which is a different thing.
