---
slug: vta/health/details
version: "0.1"
title: "VTA Health — Details"
summary: "Ask a Verifiable Trust Agent for its public health flags — messaging, seal and storage-encryption state, trusted-execution status and transport advertisement — with a fixed answer that never identifies anyone."
status: draft
targetFrameworkVersion: "0.6.0"
category: provenance
keywords:
  - health
  - status
  - observability
  - monitoring
parties:
  - role: Observer
    requirement: REQUIRED
    member: issuer
  - role: Verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    The request asks for public facts about the agent's own deployment, and is sent by monitoring and by parties deciding whether to rely on the agent — often before, or without, holding any identity the agent would recognise. Requiring a proof would gate a read the agent answers for anyone on an identity nobody needs to have. The response is REQUIRED because it is the agent's statement about itself: unsigned, an intermediary could report the agent sealed and encrypted when it is neither, and the observer would have no way to tell.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: >-
    Nothing is executed. A replayed request is answered with the agent's current state, the same answer a fresh one gets; there is no window to bound.
sideEffects:
  level: none
  rationale: "Read-only: the agent reports its configuration and posture."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Seal and encryption posture, platform, and the mediator the agent's DID document already advertises. Nothing that identifies a person, a former identity or a key, and not the software version — those are served only to administrators, by vta/restore/status.
retention:
  class: transient
  rationale: >-
    The answer is a snapshot consumed by a monitor or a relying party to decide what to do next, and is superseded by the next one.
errorCodes: []
related:
  - vta/restore/status
  - vta/attestation/status
  - vta/attestation/report
---

## Abstract

An operator watching a Verifiable Trust Agent, and a party deciding whether to rely on one, need a few facts about its deployment that are safe to tell anyone: whether it is serving, which mediator its messaging routes through, whether it is sealed against host-side changes, whether its key store is encrypted at rest, which trusted execution environment it detected, and whether it advertises TSP.

This task returns those facts and nothing else, to any asker, over TSP, DIDComm and HTTPS alike.

The answer is the agent's claim about itself. Its signature attributes it to the agent's DID; it proves nothing about hardware or code. A relying party **MUST NOT** treat `teeStatus` as attestation — [`vta/attestation/report`](../../../attestation/report/0.1/spec.md) is the evidence — and **MUST NOT** treat `sealed` or `storageEncrypted` as more than the agent's report.

## What this task never carries

Two facts an operator also wants are **not** part of this task, for any asker, and the response schema (`additionalProperties: false`) does not admit them:

- **The software version.** A version string is a fingerprint: it tells anyone who asks exactly which known vulnerabilities to try against this agent.
- **The restore record** (VTI-VTA-051). Whether the agent's state derives from a backup restore is worth knowing, but the record that says so identifies the super-administrator who committed the restore, the agent's former DID, the DIDs it hosts and the identifiers of keys it lost.

Both are served by [`vta/restore/status`](../../../restore/status/0.1/spec.md), which requires a signed request from an administrator of the agent.

They are a separate task, rather than members this task would add for an authenticated caller, on purpose. A response whose content depends on who asked invites the richer answer to leak — through a cache keyed without the caller, a log line, or one code path that forgets the check — and leaves the schema unable to say what the task discloses. Two tasks give each a fixed schema and a fixed disclosure policy: this one tells everyone the same non-identifying flags, and the other tells administrators the rest.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the agent) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements), accepting it without a `proof`.
2. Answer with its current state, computed when the request is answered — never a value latched at boot for a member that can change while it runs (`sealed`, and the messaging members after a runtime reconfiguration).
3. Give every asker the same answer. It **MUST NOT** add members, or vary the value of any member, according to whether or how the asker authenticated, and **MUST NOT** carry its software version or restore record in this response — including in `ext`.
4. Report `teeStatus` exactly as it would answer [`vta/attestation/status`](../../../attestation/status/0.1/spec.md), and omit it when it would answer that task with `vta/attestation/status:notAttested`.
5. Sign the response with a key of its own DID's `authentication` relationship ([SPEC §4.7](/SPEC.md#47-proof)).

A conforming **producer** that relies on the answer — rather than merely displaying it — **MUST** verify the response's `proof` against the agent's DID first.

## Authorization

None. The request asks for public facts about the agent's own deployment and the agent answers anyone who asks, including a party it cannot identify. A consumer **MUST NOT** refuse the request for want of a proof, an access-control entry or a session. The answer is safe to give to anyone precisely because it never contains anything that would need an authorization decision.

A consumer **MAY** rate-limit the request per transport peer, as it would any unauthenticated read, and answer beyond that limit with `unavailable` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).

## Definitions

* **Observer.** The party asking; identified by `issuer` when it identifies itself at all.
* **Verifiable trust agent.** The agent reporting on itself; identified by `recipient`.
* **Sealed.** An agent is sealed when host-side commands that would change its access control, keys or configuration, or export secrets, are refused, so it can be managed only through authenticated tasks. Unsealing requires proof of an administrator's key.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/health/details/0.1` and an empty payload that validates against the top-level schema in [`payload.schema.json`](payload.schema.json). The request is the question.

### An anonymous observer asks

```json
{
  "id": "urn:uuid:1e2d3c4b-5a69-4788-97a6-b5c4d3e2f101",
  "type": "https://trusttasks.org/spec/vta/health/details/0.1",
  "recipient": "did:webvh:QmExampleScid:vta.example.com",
  "payload": {}
}
```

## Response

The agent answers with a document of type `https://trusttasks.org/spec/vta/health/details/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json), signed with its own DID's `authentication` key.

- `status` — `ok`; an agent able to answer is serving.
- `mediatorUrl`, `mediatorDid` — the mediator its messaging routes through, absent when it has none. Both are already public: the DID document advertises the mediator, and the mediator's own DID document its endpoint.
- `teeStatus` — the trusted execution environment detected at boot, absent when there is no attestation provider.
- `sealed`, `storageEncrypted` — its seal and at-rest encryption state.
- `tspEnabled` — whether it advertises TSP. The DID document remains authoritative for which transports an agent speaks; this member reports configuration, and a party choosing a transport reads the DID document.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### An agent running in an enclave

```json
{
  "id": "urn:uuid:2f3e4d5c-6b7a-4899-a8b7-c6d5e4f3a212",
  "threadId": "urn:uuid:1e2d3c4b-5a69-4788-97a6-b5c4d3e2f101",
  "type": "https://trusttasks.org/spec/vta/health/details/0.1#response",
  "issuer": "did:webvh:QmExampleScid:vta.example.com",
  "issuedAt": "2026-09-27T12:00:00Z",
  "payload": {
    "status": "ok",
    "mediatorUrl": "https://mediator.example.com",
    "mediatorDid": "did:web:mediator.example.com",
    "teeStatus": {
      "teeType": "nitro",
      "detected": true,
      "platformVersion": "aws-nitro-enclaves"
    },
    "sealed": true,
    "storageEncrypted": true,
    "tspEnabled": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmExampleScid:vta.example.com#key-1",
    "created": "2026-09-27T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries four booleans, a platform name and version string, and a mediator's endpoint and DID that the agent's DID document already advertises. None of it is secret and none of it identifies a person, a former identity or a key. The software version and the restore record are withheld from this task entirely — see [What this task never carries](#what-this-task-never-carries). A consumer **MUST NOT** place in `ext` anything it would not answer to an anonymous asker.

### Correlation

The agent's DID is public — it is the recipient — and the answer is the same for every asker, so it links nothing to the observer. An anonymous request carries no identifier of the observer, so nothing in it links one request to another beyond transport metadata.

### Retention

Neither side needs to keep anything. The answer is a snapshot, superseded by the next one, and an observer asks again when it next needs to know. Because it identifies no one, a monitor that keeps a history of it takes on no obligation about personal data by doing so.

### Consent/purpose

The purpose is operational observability and the reliance decisions it informs: whether an agent is up and how its state is protected. The agent speaks for itself and names no other party.

### Threats

*The answer is a claim.* An attacker who controls the agent's host but not an enclave can make the agent report anything here. The signature attributes the answer to the agent's DID; only [`vta/attestation/report`](../../../attestation/report/0.1/spec.md), verified against the platform vendor's root, says what code is running.

*Disclosure by variation.* The richer operator view lives in another task, so there is no code path in this one that could hand it to the wrong asker, and no cached copy of this response that could contain it.
