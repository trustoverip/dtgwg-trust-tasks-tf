---
slug: did-management/replica/domain/upsert
version: "0.1"
title: DID Management — Replica Domain Upsert
summary: A control plane replicates its record of one hosting domain — its label, state and purge schedule — to a hosting server it drives, which makes its own copy match.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, replica, edge, domain, replication, control-plane]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Control plane
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Hosting server (replica)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The directive changes what the replica serves and when it deletes content. A replica acts only for its configured control plane, and only a proof bound to that control plane's DID can show a directive came from it on every transport — a transport sender is a routing hint."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed upsert would roll the replica's copy back to a state the control plane has since moved on from — re-enabling a disabled domain, or re-scheduling a cancelled purge."
sideEffects:
  level: mutating
  rationale: "Creates or replaces the replica's copy of the domain record and starts or cancels its purge schedule."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a domain record — a name, a label, a state and instants. The response echoes the name."
retention:
  class: transient
  rationale: "The replica keeps the record the directive carries, not the directive."
errorCodes:
  - code: did-management/replica/domain/upsert:notAuthorized
    meaning: "The document's proven issuer is not this replica's configured control plane."
    retryable: false
  - code: did-management/replica/domain/upsert:nonCanonicalName
    meaning: "`entry.name` is not in canonical form (lowercase, IDNA-folded). The replica does not canonicalise on the sender's behalf: two spellings of one domain must never become two records."
    retryable: false
related:
  - did-management/replica/domain/assign
  - did-management/replica/domain/unassign
  - did-management/replica/domain/purge
  - did-management/domain/create
  - did-management/domain/update
  - did-management/domain/set-state
---

## Abstract

A control plane is the source of record for its hosting domains; each hosting server it drives (an *edge*, or *replica*) keeps a copy so it can serve, refuse or schedule deletion without asking. **Replica Domain Upsert** carries one domain record from the control plane to a replica whenever an administrator creates, updates, disables or re-enables the domain, and again on every re-registration so a replica that missed one converges.

The replica's copy becomes the entry, whole. The directive is idempotent, so the control plane can re-send it without tracking whether the last one landed.

This task is the control-plane-to-replica hop only. What an *administrator* sends the control plane is [`did-management/domain/create`](../../../../domain/create/0.1/spec.md), [`update`](../../../../domain/update/0.1/spec.md) and [`set-state`](../../../../domain/set-state/0.1/spec.md).

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all** — a signed refusal would settle a directive the control plane never sent.
2. Refuses with `notAuthorized` a document whose proven issuer is not its configured control plane.
3. Refuses with `nonCanonicalName` an entry whose name is not canonical.
4. Otherwise creates or replaces its copy of the record with `entry`, and then: for `status: active`, cancels any purge it has scheduled for the domain; for `status: disabled`, schedules the domain's purge at `entry.purgeAt` (replacing any earlier schedule). It answers `applied`.
5. Answers a transient failure — the control plane's DID cannot be resolved just now, a storage error — with a retryable `trust-task-error`, so the control plane keeps the directive queued.

Re-applying the same entry is a no-op in effect, and **MUST** be answered `applied`.

A conforming control plane delivers the directive from a durable outbox, signs it at send time so a retry is fresh, and settles the outbox entry on the replica's signed `#response` or signed, non-retryable refusal.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being this replica's configured control plane**: the one DID the replica was set up to take directives from. It is established by comparing the proof-verified `issuer` with that configured DID; the transport sender only has to agree with it and is never the basis for acting. Nothing about the domain record — not even that it names a domain the replica serves — widens who may send it.

## Definitions

**Replica** — a hosting server whose domain records and DID content are derived from a control plane and can be rebuilt from it; it holds nothing authoritative of its own.

## Request

```json
{
  "id": "urn:uuid:27d3f054-e402-4356-8f5a-3c4d5e6f7a01",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/upsert/0.1",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid3:node1.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {
    "entry": { "name": "old.example.com", "status": "disabled", "createdAt": "2026-06-01T10:00:00Z", "disabledAt": "2026-09-27T09:00:00Z", "purgeAt": "2026-10-04T09:00:00Z" }
  }
}
```

## Response

```json
{
  "id": "urn:uuid:27d3f054-e402-4356-8f5a-3c4d5e6f7a02",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/upsert/0.1#response",
  "threadId": "urn:uuid:27d3f054-e402-4356-8f5a-3c4d5e6f7a01",
  "issuer": "did:webvh:QmNodeScid3:node1.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "name": "old.example.com", "status": "applied" }
}
```

## Security & Privacy

### Data carried

A domain record: name, optional label, state and lifecycle instants. No hosted DID and no person is named. `label` is operator free text and **MUST NOT** carry anything the operator would not publish.

### Correlation

The directive shows an observer of the replica's traffic that this replica takes directives from this control plane — the relationship a hosted DID never advertises. A confidential transport binding (TSP or DIDComm, as the replica's DID document advertises) keeps the payload from intermediaries; the association of the two DIDs is visible to the mediator either way. Both parties declare `identifierScope: public`, and must: the replica trusts exactly one configured control-plane DID, and the control plane addresses the replica by the DID it registered with — neither relationship can work over a pairwise identifier.

### Retention

The replica keeps the record, which the next upsert replaces; the directive itself is not worth keeping once applied. The control plane removes its outbox entry on acknowledgement.

### Consent/purpose

The record is replicated so the replica can serve, disable and purge the domain as the control plane has decided. The replica **MUST NOT** treat its copy as authoritative, or act on it against a later directive.
