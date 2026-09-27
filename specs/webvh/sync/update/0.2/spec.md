---
slug: webvh/sync/update
version: "0.2"
title: WebVH — Sync Update
summary: A source — a did:webvh control plane — replicates one slot's complete current log, witness proofs and disabled state to a replica it drives, a hosting server or a watcher, which verifies the history before serving it.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, sync, replication, control-plane, hosting-server, watcher]
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
  rationale: "A sync overwrites what the replica serves publicly as a DID's current state. It must be attributable to the replica's configured source on every transport — a transport sender is a routing hint, not authority — and a replica never acts on, or answers, a document that did not verify as its source's."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A sync update overwrites what the replica serves, so a stale copy re-delivered from a queue must be placeable in time; the replica's replay protection needs a bounded window."
sideEffects:
  level: mutating
  rationale: "Replaces the slot's log, witness proofs and disabled state on the replica, and with them what resolvers read from it."
subjectPath: /mnemonic
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a public DID log and its public witness proofs; the response discloses only whether the state was written."
retention:
  class: exchange
  rationale: "The replica keeps the content it serves, not the document; the signed request is needed only until the update is applied and acknowledged."
errorCodes:
  - code: webvh/sync/update:notAuthorized
    meaning: "The document's proven issuer is not a configured source of this replica."
    retryable: false
  - code: webvh/sync/update:invalidLog
    meaning: "`logContent` does not verify as a did:webvh log establishing `didId` — a broken hash chain, a bad proof, a pre-rotation violation, or an entry count that is not `versionCount`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
  - code: webvh/sync/update:historyRewrite
    meaning: "`logContent` does not strictly extend the log the replica holds for the slot, or falls below the replica's high-water mark for the DID: it rewrites or truncates history the replica has already served."
    retryable: false
  - code: webvh/sync/update:deactivated
    meaning: "The replica holds the DID as deactivated; a deactivated DID takes no further entries."
    retryable: false
related:
  - webvh/sync/delete
  - webvh/sync/batch
  - did-management/did/register
  - webvh/witness/publish
---

## Abstract

The **Sync Update** Trust Task replicates the current state of one did:webvh slot from its **source** — the control plane that is the source of record — to a **replica** the source drives: a hosting server (edge) that resolves the DID publicly, or a watcher that mirrors it. The source sends it whenever the slot changes, and again on resync; the payload is always the complete state, never a delta, so a replica that missed updates converges on the next one it receives.

The replica does not take the source's word for the history. It verifies the log cryptographically and refuses any log that does not strictly extend what it has already served, so a replica structurally cannot be made to serve a rewritten history — even by its own source.

0.2 differs from [0.1](../0.1/spec.md) in four ways. The proof is **REQUIRED**. The recipient is any replica, watchers included — a watcher is a replica whose configured sources are the control planes it mirrors, and it verifies exactly as an edge does. The slot's **disabled** state travels with its content, so disabling a DID reaches every replica through the same channel. And 0.1's wire drift is not carried forward: implementations of 0.1 sent snake_case members (`did_id`, `log_content`, …) the schema never allowed; 0.2's members are camelCase exactly as schema'd, and a replica **MUST** reject anything else.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all** — a signed refusal would settle an update the source never sent.
2. Refuses with `webvh/sync/update:notAuthorized` a document whose proven issuer is not one of its configured sources.
3. Verifies `logContent` as a did:webvh log establishing exactly `didId`, with exactly `versionCount` entries, else `webvh/sync/update:invalidLog`.
4. Refuses with `webvh/sync/update:deactivated` any change to a DID it holds as deactivated.
5. Refuses with `webvh/sync/update:historyRewrite` a log that is not the held log extended by zero or more entries, or whose version falls below the replica's per-DID high-water mark.
6. Answers `unchanged`, writing nothing, when it already holds identical content (the same log, witness proofs and disabled state).
7. Otherwise applies the log, the witness proofs (dropping any it holds when `witnessContent` is absent) and the disabled state as **one atomic change**, advances its high-water mark, and answers `applied`.
8. Answers a transient failure — the source's DID cannot be resolved just now, a storage error — with a retryable `trust-task-error`, so the source keeps the update queued. It **MUST NOT** answer a transient failure with a non-retryable code.

A conforming source retries an unacknowledged update until it receives a signed `#response` or a signed, non-retryable refusal, and treats delivery as at-least-once.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being a configured source of this replica** — for an edge, the control plane it was registered with; for a watcher, the control planes it was configured to mirror — read from the replica's own configuration. The verified `proof` establishes who sent the update, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Nothing else is standing: not an access-control role, not ownership of the DID, not transport authentication. Even a configured source cannot make a replica serve a rewritten history; that is enforced by steps 3–5, not by authorization.

## Definitions

**Source** — the party that holds the authoritative state of the slot and pushes it; a did:webvh control plane.

**Replica** — a party that serves or mirrors what its sources push: a hosting server or a watcher. A replica is a cache: everything it holds is reconstructible from its sources.

**High-water mark** — the highest version of a DID the replica has ever served, kept even across a delete, so a later update cannot roll the DID back below it.

## Request

```json
{
  "id": "urn:uuid:3c1a9e70-6f2b-4d8e-a1c3-5b7d9f0e2a11",
  "type": "https://trusttasks.org/spec/webvh/sync/update/0.2",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid3:node1.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {
    "mnemonic": "alice",
    "didId": "did:webvh:QmAliceScid4:did.example.com:alice",
    "logContent": "{\"versionId\":\"1-QmA\",\"versionTime\":\"2026-09-01T10:00:00Z\"}\n{\"versionId\":\"2-QmB\",\"versionTime\":\"2026-09-27T08:59:00Z\"}",
    "witnessContent": "[{\"versionId\":\"2-QmB\",\"proof\":[]}]",
    "versionCount": 2,
    "disabled": false
  }
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

```json
{
  "id": "urn:uuid:3c1a9e70-6f2b-4d8e-a1c3-5b7d9f0e2a12",
  "type": "https://trusttasks.org/spec/webvh/sync/update/0.2#response",
  "threadId": "urn:uuid:3c1a9e70-6f2b-4d8e-a1c3-5b7d9f0e2a11",
  "issuer": "did:webvh:QmNodeScid3:node1.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "mnemonic": "alice", "status": "applied" }
}
```

## Security & Privacy

### Data carried

The request carries a slot path, the DID, its complete public log and witness proofs, and a disabled flag — all of which the replica will serve publicly (the disabled flag as the absence of an answer). The response carries only the outcome. A producer **MUST NOT** put anything in `ext` that the replica is not meant to serve or keep.

### Correlation

The update ties a hosted DID to the edge or watcher serving it and to the source driving it. The first is public by nature — resolvers reach the DID at that host — but the second is the association this deployment keeps out of every public document. It travels only between the source and its replica, and consumers **SHOULD** use a confidential binding.

Both parties declare a public identifier scope because they must: the replica authorises the source by matching the proven issuer against the source DID in its configuration, and the source addresses the replica by the DID it registered with. A pairwise identifier would match neither.

### Retention

The replica keeps the served content and its high-water mark, not the request document; the signed request needs keeping only until it is applied and acknowledged. The high-water mark outlives the content — it is what stops a delete followed by an older update from rolling a DID back.

### Consent/purpose

The content is replicated so the replica can serve or mirror the DID. It **MUST NOT** be used for anything else; in particular a replica **MUST NOT** publish which source drives it.
