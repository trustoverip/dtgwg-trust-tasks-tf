---
slug: vta/health/details
version: "0.1"
title: "VTA Health — Details"
summary: "Ask a Verifiable Trust Agent how its deployment stands — version, messaging, seal and storage-encryption state, trusted-execution status, and whether its state derives from a backup restore."
status: draft
targetFrameworkVersion: "0.6.0"
category: provenance
keywords:
  - health
  - status
  - version
  - restore
  - provenance
  - observability
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
    The request asks for facts about the agent's own deployment, and is sent by monitoring and by parties deciding whether to rely on the agent — often before, or without, holding any identity the agent would recognise. Requiring a proof would gate a read the agent answers for anyone on an identity nobody needs to have. The response is REQUIRED because it is the agent's statement about itself: unsigned, an intermediary could hide that the agent's state was restored from a backup, or report it sealed and encrypted when it is neither, and the observer would have no way to tell.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: >-
    Nothing is executed. A replayed request is answered with the agent's current state, the same answer a fresh one gets; there is no window to bound.
sideEffects:
  level: none
  rationale: "Read-only: the agent reports its configuration and the state it booted into."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The response fingerprints the deployment — software version, mediator, seal and encryption posture, platform — and, after a restore, names the administrator who committed it and the identifiers of keys and hosted DIDs the restore could not bring back. No secret is disclosed; the Security & Privacy section discusses the restore members, which identify parties.
retention:
  class: transient
  rationale: >-
    The answer is a snapshot consumed by a monitor or a relying party to decide what to do next, and is superseded by the next one. A party that keeps a history of it for its own records does so under its own purpose.
errorCodes: []
related:
  - vta/attestation/status
  - vta/attestation/report
  - trust-task-discovery
---

## Abstract

An operator watching a Verifiable Trust Agent, and a party deciding whether to rely on one, need the same few facts about its deployment: which software version it runs, which mediator its messaging routes through, whether it is sealed against host-side changes, whether its key store is encrypted at rest, which trusted execution environment it detected, and — because a restored agent is a different thing from one that has run continuously — whether its current state derives from a backup restore, and from when.

This task returns those facts. It replaces a transport-specific health endpoint with a Trust Task, so an agent answers it identically over TSP, DIDComm and HTTPS.

The answer is the agent's claim about itself. Its signature attributes it to the agent's DID; it proves nothing about hardware or code. A relying party **MUST NOT** treat `teeStatus` as attestation — [`vta/attestation/report`](../../../attestation/report/0.1/spec.md) is the evidence — and **MUST NOT** treat `sealed` or `storageEncrypted` as more than the agent's report.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the agent) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements), accepting it without a `proof`.
2. Answer with its current state, computed when the request is answered — never a value latched at boot for a member that can change while it runs (`sealed`, and the messaging members after a runtime reconfiguration).
3. Include `restored` whenever its state derives from a backup restore, and for as long as that state persists. An agent **MUST NOT** omit `restored` to present a restored state as continuous: reporting that its state derives from a restore, and from when, is the purpose of the member.
4. Report `teeStatus` exactly as it would answer [`vta/attestation/status`](../../../attestation/status/0.1/spec.md), and omit it when it would answer that task with `vta/attestation/status:notAttested`.
5. Sign the response with a key of its own DID's `authentication` relationship ([SPEC §4.7](/SPEC.md#47-proof)).

A conforming **producer** that relies on the answer — rather than merely displaying it — **MUST** verify the response's `proof` against the agent's DID first.

## Authorization

None. The request asks for facts about the agent's own deployment and the agent answers anyone who asks, including a party it cannot identify. A consumer **MUST NOT** refuse the request for want of a proof, an access-control entry or a session, and **MUST NOT** vary its answer by who asked — an answer that differs by asker cannot be compared between observers, which is how a monitor notices an agent telling different parties different things.

A consumer **MAY** rate-limit the request per transport peer, as it would any unauthenticated read, and answer beyond that limit with `unavailable` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).

## Definitions

* **Observer.** The party asking; identified by `issuer` when it identifies itself at all.
* **Verifiable trust agent.** The agent reporting on itself; identified by `recipient`.
* **Sealed.** An agent is sealed when host-side commands that would change its access control, keys or configuration, or export secrets, are refused, so it can be managed only through authenticated tasks. Unsealing requires proof of an administrator's key.
* **Restore.** Replacement of an agent's state by the contents of an encrypted backup — possibly taken from a different agent, and possibly on a different kind of deployment — applied at a boot.
* **Deployment environment.** How an agent's state is protected at rest, as a restore classifies it: `plain`, `hardened` (encrypted under a key derived from the agent's seed) or `tee` (sealed to an attested trusted execution environment).

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
- `version` — the agent's software version.
- `mediatorUrl`, `mediatorDid` — the mediator its messaging routes through, absent when it has none.
- `teeStatus` — the trusted execution environment detected at boot, absent when there is no attestation provider.
- `sealed`, `storageEncrypted` — its seal and at-rest encryption state.
- `tspEnabled` — whether it advertises TSP. The DID document remains authoritative for which transports an agent speaks; this member reports configuration, and a party choosing a transport reads the DID document.
- `restored` — the restore its state derives from, absent when it was never restored.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### An agent running continuously in an enclave

```json
{
  "id": "urn:uuid:2f3e4d5c-6b7a-4899-a8b7-c6d5e4f3a212",
  "threadId": "urn:uuid:1e2d3c4b-5a69-4788-97a6-b5c4d3e2f101",
  "type": "https://trusttasks.org/spec/vta/health/details/0.1#response",
  "issuer": "did:webvh:QmExampleScid:vta.example.com",
  "issuedAt": "2026-09-27T12:00:00Z",
  "payload": {
    "status": "ok",
    "version": "0.40.0",
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

### An agent whose state was restored from another agent's backup

```json
{
  "id": "urn:uuid:3a4f5e6d-7c8b-49aa-b9c8-d7e6f5a4b323",
  "threadId": "urn:uuid:1e2d3c4b-5a69-4788-97a6-b5c4d3e2f101",
  "type": "https://trusttasks.org/spec/vta/health/details/0.1#response",
  "issuer": "did:webvh:QmExampleScid:vta.example.com",
  "issuedAt": "2026-09-27T12:00:00Z",
  "payload": {
    "status": "ok",
    "version": "0.40.0",
    "sealed": true,
    "storageEncrypted": true,
    "tspEnabled": false,
    "restored": {
      "appliedAt": "2026-09-26T08:15:02Z",
      "stagedAt": "2026-09-26T08:14:40Z",
      "stagedBy": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "sourceDid": "did:webvh:QmOldScid:vta-old.example.com",
      "sourceEnvironment": "hardened",
      "targetEnvironment": "tee",
      "internalKeysLost": ["audit-checkpoint-signer"],
      "hostedDidsDetached": ["did:webvh:QmHostedScid:app.example.com"]
    }
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

The request carries nothing. The response carries configuration and state about the agent's deployment, and none of it is secret: no key material, no credential, no token. Most of it is also not personal — a version string, a mediator's endpoint and DID (which the agent's DID document already advertises), four booleans and a platform name.

The exception is `restored`. `stagedBy` is the DID of the administrator who committed the restore, `sourceDid` an agent's former identity, `hostedDidsDetached` the DIDs of identities the agent hosts, and `internalKeysLost` the identifiers of keys. Answering these to anyone is a deliberate choice: an agent **MUST** be able to report that its state derives from a restore and from when, and a relying party that cannot learn that has no reason to treat a restored agent differently from one that has run continuously. A producer **MUST NOT** place anything in `ext` that it would not answer to an anonymous asker.

### Correlation

The agent's DID is public — it is the recipient — and so is the answer, which is the same for every asker. The answer does link an administrator's DID (`stagedBy`) to this agent, and an agent's former identity (`sourceDid`) to its current one; both are links the restore itself created, and neither can be unlinked by the observer choosing a different identity. An anonymous request carries no identifier of the observer, so nothing in it links one request to another beyond transport metadata.

### Retention

Neither side needs to keep anything. The answer is a snapshot, superseded by the next one, and an observer asks again when it next needs to know. A monitor that keeps a history — to notice when `restored` first appears, or when `version` changes — keeps it under its own purpose, and holds `stagedBy` and `sourceDid` in that history for as long as it keeps it.

### Consent/purpose

The purpose is operational observability and the reliance decisions it informs: whether an agent is up, what it runs, how its state is protected, and whether that state is continuous. The agent speaks for itself; the one party named who is not the agent is the administrator in `stagedBy`, disclosed because accountability for a restore is part of what a restore report is for. Any use of these members beyond observing and relying on this agent — profiling the administrator, enumerating hosted identities — is outside that purpose.

### Threats

*The answer is a claim.* An attacker who controls the agent's host but not an enclave can make the agent report anything here. The signature attributes the answer to the agent's DID; only [`vta/attestation/report`](../../../attestation/report/0.1/spec.md), verified against the platform vendor's root, says what code is running.

*Fingerprinting.* `version` tells an attacker which vulnerabilities to try. That is the same trade every public version endpoint makes; an operator who does not want it answered publicly has no way to withhold it under this version of the specification, because the answer does not vary by asker.

*Stripping `restored`.* An intermediary that removed `restored` from an unsigned answer would present a restored agent as continuous. The response proof, verified by the producer, is what prevents it.
