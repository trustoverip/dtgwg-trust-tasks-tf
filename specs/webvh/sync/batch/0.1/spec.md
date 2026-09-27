---
slug: webvh/sync/batch
version: "0.1"
title: WebVH — Sync Batch
summary: A source — a did:webvh control plane — replicates many slots' complete state to a replica in one document, as a resync after the replica (re-)registers, with a per-entry result.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, sync, replication, batch, resync, hosting-server, watcher]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Source
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Replica
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Each entry overwrites what the replica serves publicly as a DID's current state. The batch must be attributable to the replica's configured source on every transport, and a replica never acts on, or answers, a document that did not verify as its source's."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A batch overwrites what the replica serves; a stale one re-delivered from a queue must be placeable in time for the replica's replay protection."
sideEffects:
  level: mutating
  rationale: "Each entry replaces one slot's log, witness proofs and disabled state on the replica."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries public DID logs and witness proofs; the response discloses only a per-entry outcome."
retention:
  class: exchange
  rationale: "The replica keeps the content it serves, not the document; the signed request is needed only until acknowledged."
errorCodes:
  - code: webvh/sync/batch:notAuthorized
    meaning: "The document's proven issuer is not a configured source of this replica. The whole batch is refused."
    retryable: false
related:
  - webvh/sync/update
  - webvh/sync/delete
---

## Abstract

When a replica registers or re-registers, its source resyncs every slot the replica should hold. Sent one [`webvh/sync/update/0.2`](../../update/0.2/spec.md) per slot, a resync of a large domain is thousands of frames. **Sync Batch** carries up to fifty complete slot states in one document and answers with a result per entry.

Each entry means exactly what the same state would mean in a single update — the same verification, the same refusals, the same atomic apply — so a batch is only a framing. It is on the wire today, unspecified, as `https://trusttasks.org/spec/webvh/sync/batch/0.1`; this document specifies it.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming source sends each mnemonic at most once per batch and never splits one slot across batches. A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all**.
2. Refuses the whole batch with `webvh/sync/batch:notAuthorized` when the proven issuer is not one of its configured sources.
3. Applies each entry independently and exactly as [`webvh/sync/update/0.2`](../../update/0.2/spec.md#conformance) steps 3–7 specify. One entry's permanent refusal does not stop the others.
4. If **any** entry fails transiently, answers the whole batch with a retryable `trust-task-error`. Re-applying the entries that did land is a no-op (`unchanged`), and acknowledging the batch would settle it with that entry missing.
5. Otherwise answers with one result per entry, in request order: `applied`, `unchanged`, or `refused` with the `webvh/sync/update` error code that entry would have received on its own.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being a configured source of this replica**, exactly as for [`webvh/sync/update/0.2`](../../update/0.2/spec.md#authorization). The verified `proof` establishes who sent the batch, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Being authorised to send the batch does not let any entry bypass the history checks.

## Definitions

**Source**, **Replica** and **high-water mark** are as in [`webvh/sync/update/0.2`](../../update/0.2/spec.md#definitions). **Entry** — one `SyncUpdate` in `updates`.

## Request

```json
{
  "id": "urn:uuid:c4b2a190-7e3d-4a1f-8c2b-6d5e4f3a2b01",
  "type": "https://trusttasks.org/spec/webvh/sync/batch/0.1",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid3:node1.example.com",
  "issuedAt": "2026-09-27T09:20:00Z",
  "payload": {
    "updates": [
      {
        "mnemonic": "alice",
        "didId": "did:webvh:QmAliceScid4:did.example.com:alice",
        "logContent": "{\"versionId\":\"1-QmA\",\"versionTime\":\"2026-09-01T10:00:00Z\"}",
        "versionCount": 1,
        "disabled": false
      },
      {
        "mnemonic": "bob",
        "didId": "did:webvh:QmBobScid5:did.example.com:bob",
        "logContent": "{\"versionId\":\"1-QmC\",\"versionTime\":\"2026-09-02T10:00:00Z\"}",
        "versionCount": 1,
        "disabled": true
      }
    ]
  }
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:c4b2a190-7e3d-4a1f-8c2b-6d5e4f3a2b02",
  "type": "https://trusttasks.org/spec/webvh/sync/batch/0.1#response",
  "threadId": "urn:uuid:c4b2a190-7e3d-4a1f-8c2b-6d5e4f3a2b01",
  "issuer": "did:webvh:QmNodeScid3:node1.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:20:02Z",
  "payload": {
    "results": [
      { "mnemonic": "alice", "status": "unchanged" },
      { "mnemonic": "bob", "status": "refused", "code": "webvh/sync/update:historyRewrite" }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries up to fifty slots' complete public logs, witness proofs and disabled flags; the response a per-entry outcome and, for a refusal, an error code. A producer **MUST NOT** put anything in `ext` that the replica is not meant to serve or keep.

### Correlation

A batch shows, in one document, a large part of what one replica serves and which source drives it — more of the fleet topology than any single update. It travels only between the two and consumers **SHOULD** use a confidential binding. Both parties declare a public identifier scope because they must: the replica authorises the source by matching the proven issuer against its configured source DID, and the source addresses the replica by its registered DID. A pairwise identifier would match neither.

### Retention

As for [`webvh/sync/update/0.2`](../../update/0.2/spec.md#retention): the replica keeps the served content and high-water marks, not the request.

### Consent/purpose

The content is replicated so the replica can serve or mirror the DIDs. It **MUST NOT** be used for anything else.
