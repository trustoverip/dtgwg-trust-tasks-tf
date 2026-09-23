---
slug: git-ns/bridge/result
version: "0.1"
title: "Git Namespaces — Bridge Job Result"
summary: "The bridge reports how a job ended — succeeded, failed or partial, per step — and the repository's forge id, so the VTC can activate it, record its bootstrap status, or retry."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - bridge
  - automation
parties:
  - role: bridge
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A result moves repositories into the active state, which publishes their owners' rights. The VTC must be able to attribute it to the bridge that serves the namespace on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A result replayed out of order would report an old outcome as current."
sideEffects:
  level: mutating
  rationale: "Updates the VTC's record of a repository — state, forge id, bootstrap status — and on success publishes its owners' rights."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a job identifier, outcomes, a repository name and forge id, and free-text details."
retention:
  class: durable
  rationale: "The VTC keeps each result with the job it closes, as the record of what was done on the forge in its name."
errorCodes:
  - code: git-ns/bridge/result:unknownJob
    meaning: "The VTC sent no job with this `jobId` to this bridge."
    retryable: false
related:
  - git-ns/bridge/job
  - git-ns/bridge/event
  - git-ns/repo/create
---

## Abstract

The bridge reports how a job sent with [`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.1/spec.md) ended: the outcome, the repository it touched as the forge now identifies it, and the outcome of each step. The VTC uses it to move a repository from `pendingCreate` to `active`, to record its forge id and bootstrap status, and to decide whether to send the job again.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the bridge that serves the job's namespace**. A VTC records, per namespace, which bridge DID serves it, and **MUST** refuse, with `permissionDenied`, a result from any other issuer. A result can only report on a job the VTC sent; it cannot change any right. What it changes — a repository's state, forge id, bootstrap status — the VTC would otherwise learn by inspection.

## Definitions

**`outcome`** — `succeeded`, `failed` or `partial`, as in the schema.

**`repo`** — the repository the job touched, with the forge's id for it. The VTC keys the repository's rights by that id.

**`steps`** — per-step outcomes, in the order they ran.

**`error`** — why the job failed or stopped. The codes are forge-neutral and open: `nameTaken`, `notFound`, `forbidden` (the bridge's forge credentials lack a permission), `notCapable`, `rateLimited`, `expired` (a `begin*` job's person never completed it), `forgeError`. A VTC treats a code it does not know as `forgeError`.

## Request

The bridge sends the result to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming bridge sends exactly one result per accepted job, and sends it again when the VTC repeats the job. A conforming VTC:

1. Refuses a `jobId` it did not send with `git-ns/bridge/result:unknownJob`.
2. Records the result once. A repeated result for a job already recorded is acknowledged and changes nothing.
3. For `createRepo`, on `succeeded` sets the repository `active`, records `repo.forgeId`, and publishes its owners' rights; otherwise leaves it `pendingCreate` and records the failed step. For `bootstrap` and `inspect`, updates the repository's bootstrap status from the steps. For `archive`, on `succeeded` nothing further. For `beginBind` and `beginAccountLink`, a `failed` result ends the pending namespace or link; success was already reported by the event.
4. **MUST NOT** take `repo.resource` as a rename. Renames arrive as `repoRenamed` events, which the VTC applies by forge id.

### A repository created and bootstrapped

```json
{
  "id": "urn:uuid:fb9517a4-8ef8-41df-aef3-380e7fa51f01",
  "type": "https://trusttasks.org/spec/git-ns/bridge/result/0.1",
  "threadId": "urn:uuid:fb9517a4-8ef8-41df-aef3-380e7fa51f01",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:40Z",
  "payload": {
    "jobId": "job_01J8ZA3K7F",
    "outcome": "succeeded",
    "repo": {
      "resource": "github.com/acme/gadgets",
      "forgeId": "812736990"
    },
    "steps": [
      {
        "step": "create",
        "outcome": "applied"
      },
      {
        "step": "workflow",
        "outcome": "applied"
      },
      {
        "step": "keyring",
        "outcome": "applied"
      },
      {
        "step": "variables",
        "outcome": "applied"
      },
      {
        "step": "requiredCheck",
        "outcome": "applied"
      },
      {
        "step": "roles",
        "outcome": "applied"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:40Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zG3UD8d6xMATBJFTFfaeRkCC5FzhYwYJpJHECtHkfV4fyjiiGesGi9vkzLE8YaZGBvTyyDQq2MMCVDzcPPLHP6F"
  }
}
```

### A Forgejo bootstrap that could not find a runner

Every step but the last applied. The repository is protected, but its required check can never pass until the instance offers a runner — which the VTC shows on the repository rather than hides.

```json
{
  "id": "urn:uuid:fb9517a4-8ef8-41df-aef3-380e7fa51f03",
  "type": "https://trusttasks.org/spec/git-ns/bridge/result/0.1",
  "threadId": "urn:uuid:fb9517a4-8ef8-41df-aef3-380e7fa51f03",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZD1F5S",
    "outcome": "partial",
    "repo": {
      "resource": "codeberg.org/acme/sprockets",
      "forgeId": "334021"
    },
    "steps": [
      {
        "step": "workflow",
        "outcome": "unchanged"
      },
      {
        "step": "variables",
        "outcome": "applied"
      },
      {
        "step": "mergeStyle",
        "outcome": "applied"
      },
      {
        "step": "requiredCheck",
        "outcome": "applied"
      },
      {
        "step": "runner",
        "outcome": "failed",
        "detail": "No runner picked up the verify-trust job within 10 minutes."
      }
    ],
    "error": {
      "code": "forgeError",
      "message": "The instance offers no runner for this repository; the check is required but can never pass."
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zUo2civRis64RDNrGy84MnV3v8kbWSK3n4XD5EMrmtnTBd6hGAYWaVP6FqHENTD2Q96BP7rU51UrCFiu7MFEz1s"
  }
}
```

## Response

The VTC, now responding, acknowledges the result, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Recorded

```json
{
  "id": "urn:uuid:fb9517a4-8ef8-41df-aef3-380e7fa51f02",
  "type": "https://trusttasks.org/spec/git-ns/bridge/result/0.1#response",
  "threadId": "urn:uuid:fb9517a4-8ef8-41df-aef3-380e7fa51f01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:41Z",
  "payload": {
    "jobId": "job_01J8ZA3K7F"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:41Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zsNBhcsMawQ96N6GJfD94KE1ySYqEVPaWB9K9q6vJ1NvvL3wPQYE3uq3CCksdFE4UNXGyr92MH1CiugbXi4EZjE"
  }
}
```

## Security & Privacy

### Data carried

A job identifier, an outcome, a repository name and forge id, step names and free-text details. A bridge **MUST NOT** put forge credentials, tokens or raw API responses in `detail` or `error.message`.

### Correlation

The VTC declares `identifierScope: public`; the bridge `pairwise`. Results carry no member identifiers.

### Retention

Durable. The VTC keeps results with the jobs they close, as the record of what was done on the forge in its name.

### Consent/purpose

The purpose is to keep the VTC's record of the forge accurate. Nothing in a result may be used for anything else.
