---
slug: vta/restore/status
version: "0.1"
title: "VTA Restore — Status"
summary: "An administrator asks a Verifiable Trust Agent whether its state derives from a backup restore — when, by whom, from which agent, and what did not come back — and which software version it runs."
status: draft
targetFrameworkVersion: "0.6.0"
category: provenance
keywords:
  - restore
  - backup
  - provenance
  - version
  - disaster-recovery
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is the evidence an administrator asked: the answer names another administrator, a former identity, hosted DIDs and key identifiers, so the agent must be able to attribute and authorize the asker from the document itself, over any transport. The response is REQUIRED because it is the agent's account of its own provenance: unsigned, an intermediary could strip the restore record and present a restored agent as one that has run continuously.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer is released only to an authorized administrator, so a captured request replayed later must be placeable in time and refusable; an issuedAt is what bounds that window.
sideEffects:
  level: none
  rationale: "Read-only: the agent reports the restore record it wrote when the restore was applied, and its version."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The restore record identifies the administrator who committed the restore (stagedBy), the agent's former DID, the DIDs it hosts and the identifiers of keys it lost; the version fingerprints the build. None is secret, which is why this is metadata, but all of it is withheld from anyone who is not an administrator of the agent.
retention:
  class: transient
  rationale: >-
    The answer is consumed to decide recovery work — re-registering detached DIDs, replacing lost keys — or to judge the agent's continuity. The durable record of the restore is the agent's own audit trail, not this response.
errorCodes: []
related:
  - vta/health/details
  - vta/backup/finalize-import
  - vta/attestation/status
---

## Abstract

A Verifiable Trust Agent's state can be replaced by a backup restore — possibly taken from a different agent, on a different kind of deployment. A restored agent is a different thing from one that has run continuously: keys it generated internally and never exports did not come back, hosted DIDs may need registering again, and whoever relies on it may want to know that its history has a seam. The agent therefore keeps a restore record, and this task is how an administrator reads it, together with the agent's software version.

Both facts are deliberately withheld from the public [`vta/health/details`](../../../health/details/0.1/spec.md): the record identifies a super-administrator, a former DID, hosted DIDs and key identifiers, and the version tells an attacker which known vulnerabilities to try. Serving them from a separate, administrator-only task — instead of adding them to the public answer when the caller is authenticated — gives each task a fixed schema and a fixed disclosure policy. A response whose content varies with authentication invites the richer answer to leak through a cache, a log line or one code path that forgets the check, and leaves the schema unable to say what the task discloses.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/restore/status/0.1`, with itself as `issuer` and the agent as `recipient`, carrying a `proof` and an `issuedAt`.
2. Verify the response's `proof` against the agent's DID before relying on it.

A conforming **consumer** (the agent) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify its `proof`.
2. Establish that the proof's signer is an administrator of the agent (see Authorization) before reading anything, and refuse anyone else with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)). The refusal **MUST NOT** reveal whether the agent was restored.
3. Answer `restored: true` together with the `restore` record whenever its state derives from a restore, for as long as that state persists, and `restored: false` with no `restore` member otherwise. The two members **MUST** agree — `restore` is present exactly when `restored` is `true` — and a producer **MUST** treat a response in which they disagree as malformed. An agent **MUST NOT** answer `false` to present a restored state as continuous: reporting that its state derives from a restore, and from when, is the purpose of the task.
4. Answer `version` with the software version its build reports.
5. Sign the response with a key of its own DID's `authentication` relationship ([SPEC §4.7](/SPEC.md#47-proof)).

## Authorization

The authority this task assumes is **administration of the agent**: the proof's signer holds an administrator role in the agent's access-control list. Any administrator qualifies — the answer concerns the whole agent and every administrator is accountable for acting on it — and a party holding any other role, or none, is refused with `permissionDenied`.

Verifying the `proof` establishes who asked, never that they may know ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the access-control check is the authorization. The response's `proof` establishes that the answer is the agent's; it asserts nothing about the asker.

## Definitions

* **Administrator.** The party asking; identified by `issuer`, and authorized by its administrator role at the agent.
* **Verifiable trust agent.** The agent reporting on itself; identified by `recipient`.
* **Restore.** Replacement of an agent's state by the contents of an encrypted backup, applied at a boot.
* **Deployment environment.** How an agent's state is protected at rest, as a restore classifies it: `plain`, `hardened` (encrypted under a key derived from the agent's seed) or `tee` (sealed to an attested trusted execution environment).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/restore/status/0.1` and an empty payload that validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator asks

```json
{
  "id": "urn:uuid:4b5a6f7e-8d9c-4abb-8ad9-e8f7a6b5c434",
  "type": "https://trusttasks.org/spec/vta/restore/status/0.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmExampleScid:vta.example.com",
  "issuedAt": "2026-09-27T12:00:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-09-27T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The agent answers with a document of type `https://trusttasks.org/spec/vta/restore/status/0.1#response`, whose payload validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json), signed with its own DID's `authentication` key.

- `version` — the agent's software version.
- `restored` — always present: whether the agent's state derives from a restore.
- `restore` — present exactly when `restored` is `true`: `appliedAt`, `stagedAt`, `stagedBy`, `sourceDid`, `sourceEnvironment`, `targetEnvironment`, `internalKeysLost`, `hostedDidsDetached`.

*This paragraph is non-normative.* The agreement of `restored` and `restore` is stated in prose rather than in the schema because the conditional form JSON Schema offers (`if`/`then`) is not supported by the code generators this registry publishes bindings through. A single nullable member would have been enforceable, but degrades to untyped JSON in two of the four generated libraries.

`permissionDenied` and other failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### An agent that has run continuously

```json
{
  "id": "urn:uuid:5c6b7a8f-9e0d-4bcc-9bea-f9a8b7c6d545",
  "threadId": "urn:uuid:4b5a6f7e-8d9c-4abb-8ad9-e8f7a6b5c434",
  "type": "https://trusttasks.org/spec/vta/restore/status/0.1#response",
  "issuer": "did:webvh:QmExampleScid:vta.example.com",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-09-27T12:00:01Z",
  "payload": {
    "version": "0.40.0",
    "restored": false
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmExampleScid:vta.example.com#key-1",
    "created": "2026-09-27T12:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### An agent restored from another agent's backup

```json
{
  "id": "urn:uuid:6d7c8b9a-0f1e-4cdd-8cfb-0a9b8c7d6e56",
  "threadId": "urn:uuid:4b5a6f7e-8d9c-4abb-8ad9-e8f7a6b5c434",
  "type": "https://trusttasks.org/spec/vta/restore/status/0.1#response",
  "issuer": "did:webvh:QmExampleScid:vta.example.com",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-09-27T12:00:01Z",
  "payload": {
    "version": "0.40.0",
    "restored": true,
    "restore": {
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
    "created": "2026-09-27T12:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries the software version and the restore record. Nothing is secret, but the record identifies parties: `stagedBy` is the DID of the administrator who committed the restore, `sourceDid` an agent's former identity, `hostedDidsDetached` the DIDs of identities the agent hosts, and `internalKeysLost` key identifiers. That is why the task is administrator-only and why none of it appears in the public `vta/health/details`. A consumer **MUST NOT** place anything further in `ext`.

### Correlation

The record links an administrator (`stagedBy`) to this agent, and a former identity (`sourceDid`) to the current one — links the restore itself created, disclosed here only to the agent's other administrators. The request links the asking administrator to the agent in the agent's own logs, which it already is by holding a role there.

### Retention

The agent keeps the restore record for as long as its state derives from the restore; the record, and the audit row written when the restore was applied, are the durable account. An administrator's copy of a response is a working note for recovery and **SHOULD** be discarded once the recovery work it prompted is done.

### Consent/purpose

The purpose is recovery and continuity: knowing that the agent's state has a seam, which keys and hosted DIDs need attention, and which build is running so it can be patched. `stagedBy` is disclosed to the agent's administrators because accountability for a restore is part of what a restore record is for; using these members for anything else — profiling an administrator, enumerating hosted identities for another purpose — is outside it.

### Threats

*Stripping the record.* An intermediary that rewrote an unsigned answer to `restored: false` would present a restored agent as continuous. The response proof, verified by the producer, prevents it.

*Probing.* A refused asker learns nothing about whether the agent was restored, because the refusal happens before the record is read and is the same either way.
