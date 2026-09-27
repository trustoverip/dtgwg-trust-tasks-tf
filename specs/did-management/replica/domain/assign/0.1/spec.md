---
slug: did-management/replica/domain/assign
version: "0.1"
title: DID Management — Replica Domain Assign
summary: A control plane tells a hosting server it drives to start serving a hosting domain, cancelling any purge the server had scheduled for it.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, replica, edge, domain, assignment, control-plane]
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
  rationale: "Assignment decides which DIDs a public resolver gets from this server. A replica acts only for its configured control plane, and only a proof bound to that DID shows a directive came from it on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The replica records the assignment's time and compares later purge directives against it; a replayed assignment would re-bind a domain the control plane has since taken away."
sideEffects:
  level: mutating
  rationale: "Records the assignment, makes the replica serve the domain, and cancels a pending unassignment purge."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The request carries a domain name; the response echoes it."
retention:
  class: transient
  rationale: "The replica keeps the assignment the directive creates, not the directive."
errorCodes:
  - code: did-management/replica/domain/assign:notAuthorized
    meaning: "The document's proven issuer is not this replica's configured control plane."
    retryable: false
related:
  - did-management/domain/assign
  - did-management/replica/domain/unassign
  - did-management/replica/domain/purge
  - did-management/replica/domain/upsert
---

## Abstract

When an administrator assigns a hosting domain to a server ([`did-management/domain/assign`](../../../../domain/assign/0.1/spec.md), administrator to control plane), the control plane tells the server with **Replica Domain Assign**. The server records the assignment and serves the domain's DIDs as the control plane syncs them. If the domain had been unassigned from it and its content was waiting out the grace period, the purge is cancelled — the content is kept.

The control plane re-sends every unacknowledged assignment on each registration of the server, so the directive is idempotent.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all**.
2. Refuses with `notAuthorized` a document whose proven issuer is not its configured control plane.
3. Otherwise records the domain as assigned to itself, with **its own clock** as the assignment time (the time [`did-management/replica/domain/purge`](../../purge/0.1/spec.md)'s freshness rule compares against), cancels any purge it has scheduled for the domain, and answers `applied`. Re-assigning a domain it already serves refreshes the assignment time and is answered `applied`.
4. Answers a transient failure with a retryable `trust-task-error`, so the control plane keeps the directive queued.

The control plane delivers the directive from a durable outbox, signs it at send time, and settles the entry on the signed `#response` or a signed non-retryable refusal.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being this replica's configured control plane**, established by comparing the proof-verified `issuer` with the one control-plane DID the replica was configured with. The transport sender only has to agree with it. The administrator's decision is authorised at the control plane, on the administrator-facing task; the replica neither sees nor re-checks it.

## Definitions

**Replica** — as in [`did-management/replica/domain/upsert`](../../upsert/0.1/spec.md).

## Request

```json
{
  "id": "urn:uuid:38e40165-f513-4467-9a6b-4d5e6f7a8b01",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/assign/0.1",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid3:node1.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "domain": "did.example.com" }
}
```

## Response

```json
{
  "id": "urn:uuid:38e40165-f513-4467-9a6b-4d5e6f7a8b02",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/assign/0.1#response",
  "threadId": "urn:uuid:38e40165-f513-4467-9a6b-4d5e6f7a8b01",
  "issuer": "did:webvh:QmNodeScid3:node1.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "domain": "did.example.com", "status": "applied" }
}
```

## Security & Privacy

### Data carried

A domain name, in both directions.

### Correlation

The directive reveals, to anyone who can see its payload, that this server serves this domain for this control plane. Confidential transport bindings keep the payload from intermediaries. Both parties declare `identifierScope: public`, and must: the replica trusts exactly one configured control-plane DID, and the control plane addresses the replica by the DID it registered with.

### Retention

The replica keeps the assignment and its time for as long as the domain is assigned; the directive is not worth keeping once applied.

### Consent/purpose

The directive exists so a server serves the domains its control plane has placed on it. The replica **MUST NOT** begin serving a domain on any other signal.
