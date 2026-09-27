---
slug: webvh/sync/delete
version: "0.2"
title: WebVH — Sync Delete
summary: A source — a did:webvh control plane — tells a replica it drives, a hosting server or a watcher, that a slot no longer exists, and the replica stops serving it and removes its content.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [webvh, sync, replication, delete, hosting-server, watcher]
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
  rationale: "A delete takes a DID off a public host. It must be attributable to the replica's configured source on every transport, and a replica never acts on, or answers, a document that did not verify as its source's."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A delete is destructive; a captured one re-delivered after the slot was re-created would remove the new DID, so the replica's replay protection needs a bounded window to place it in."
sideEffects:
  level: destructive
  rationale: "The replica stops serving the slot and removes its log, witness proofs and derived agent-name index. It keeps only the DID's high-water mark."
subjectPath: /mnemonic
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names one slot path; the response discloses only whether the replica held it."
retention:
  class: exchange
  rationale: "The signed request is needed only until the delete is applied and acknowledged."
errorCodes:
  - code: webvh/sync/delete:notAuthorized
    meaning: "The document's proven issuer is not a configured source of this replica."
    retryable: false
related:
  - webvh/sync/update
  - webvh/sync/batch
  - did-management/did/delete
---

## Abstract

The **Sync Delete** Trust Task removes one slot from a replica. The source sends it when the slot is deleted at the source, and on resync for every slot the replica reports holding that the source no longer publishes.

The source is the source of record, so the replica does not second-guess it: a delete is applied even when the replica holds a newer version than the source last sent. It is idempotent — deleting a slot the replica does not hold answers `absent`.

0.2 differs from [0.1](../0.1/spec.md) in making the proof **REQUIRED** and in generalising the recipient to any replica, watchers included.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all**.
2. Refuses with `webvh/sync/delete:notAuthorized` a document whose proven issuer is not one of its configured sources.
3. When it holds the slot: stops serving it, removes its log, witness proofs and derived agent-name index in one atomic change, keeps the DID's high-water mark, and answers `deleted`.
4. When it holds nothing for the slot: answers `absent`.
5. Answers a transient failure with a retryable `trust-task-error`, so the source keeps the delete queued.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being a configured source of this replica**, read from the replica's own configuration, exactly as for [`webvh/sync/update/0.2`](../../update/0.2/spec.md). The verified `proof` establishes who sent the delete, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Owning the DID is not standing: an owner deletes through the source ([`did-management/did/delete`](../../../../did-management/did/delete/0.1/spec.md)), never at a replica.

## Definitions

**Source**, **Replica** and **high-water mark** are as in [`webvh/sync/update/0.2`](../../update/0.2/spec.md#definitions).

## Request

```json
{
  "id": "urn:uuid:8e2d4b61-0a3c-4f5e-9b7d-1c2e3f4a5b01",
  "type": "https://trusttasks.org/spec/webvh/sync/delete/0.2",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid3:node1.example.com",
  "issuedAt": "2026-09-27T09:10:00Z",
  "payload": { "mnemonic": "alice" }
}
```

## Response

Per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:8e2d4b61-0a3c-4f5e-9b7d-1c2e3f4a5b02",
  "type": "https://trusttasks.org/spec/webvh/sync/delete/0.2#response",
  "threadId": "urn:uuid:8e2d4b61-0a3c-4f5e-9b7d-1c2e3f4a5b01",
  "issuer": "did:webvh:QmNodeScid3:node1.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:10:01Z",
  "payload": { "mnemonic": "alice", "status": "deleted" }
}
```

## Security & Privacy

### Data carried

The request carries one slot path. The response says whether the replica held it. A producer **MUST NOT** put anything else in `ext`.

### Correlation

The delete ties a slot on the replica to the source driving it — the association the deployment keeps out of public documents — and travels only between the two; consumers **SHOULD** use a confidential binding. Both parties declare a public identifier scope because they must: the replica authorises the source by matching the proven issuer against the source DID it is configured with, and the source addresses the replica by the DID it registered with. A pairwise identifier would match neither.

### Retention

The replica keeps nothing of the slot but its high-water mark, which it **MUST** keep: without it a re-delivered older update could resurrect the DID at a version below one it already served. The signed request is needed only until acknowledged.

### Consent/purpose

The delete exists so replicas stop serving a DID its source no longer publishes. A replica **MUST NOT** keep the removed content for any other purpose.
