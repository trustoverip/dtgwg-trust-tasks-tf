---
slug: did-management/replica/domain/unassign
version: "0.1"
title: DID Management — Replica Domain Unassign
summary: A control plane tells a hosting server it drives to stop serving a hosting domain; the server schedules the domain's content for deletion after its grace period.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, replica, edge, domain, assignment, grace-period, control-plane]
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
  rationale: "Unassignment stops a server resolving a domain's DIDs and starts the clock on deleting them. A replica acts only for its configured control plane, and only a proof bound to that DID shows the directive came from it on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed unassignment would take a domain off a server it has since been re-assigned to, and start deleting content the operator chose to keep."
sideEffects:
  level: mutating
  rationale: "Removes the assignment, stops the replica accepting new content for the domain, and schedules deletion. The deletion itself is deferred and cancellable by re-assignment."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The request carries a domain name; the response echoes it with the scheduled purge time."
retention:
  class: transient
  rationale: "The replica keeps the schedule the directive creates, not the directive."
errorCodes:
  - code: did-management/replica/domain/unassign:notAuthorized
    meaning: "The document's proven issuer is not this replica's configured control plane."
    retryable: false
related:
  - did-management/domain/unassign
  - did-management/replica/domain/assign
  - did-management/replica/domain/purge
  - did-management/registry/purge-domain
---

## Abstract

When an administrator unassigns a domain from a server ([`did-management/domain/unassign`](../../../../domain/unassign/0.1/spec.md)), the control plane tells the server with **Replica Domain Unassign**. The server stops accepting content for the domain and schedules the deletion of what it holds after its configured grace period. Until then, a re-assignment cancels the deletion and nothing needs resyncing; after it, the content is gone. An administrator who does not want to wait orders [`did-management/registry/purge-domain`](../../../../registry/purge-domain/0.1/spec.md).

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all**.
2. Refuses with `notAuthorized` a document whose proven issuer is not its configured control plane.
3. Otherwise removes its assignment of the domain, stops accepting new content for it, schedules the deletion of its content at its own clock plus its configured grace, and answers `scheduled` with that `purgeAt`. An unassignment that arrives while one is already scheduled replaces the schedule, so the answer always states the schedule now in force. Unassigning a domain it does not serve schedules nothing new and is answered with the existing schedule, or with `purgeAt` equal to the time of processing when it holds nothing.
4. Answers a transient failure with a retryable `trust-task-error`, so the control plane keeps the directive queued.

The control plane delivers from a durable outbox, signs at send time, and settles the entry on the signed `#response` or a signed non-retryable refusal.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being this replica's configured control plane**, established by comparing the proof-verified `issuer` with the one control-plane DID the replica was configured with. The transport sender only has to agree with it.

## Definitions

**Unassignment grace** — the period a replica waits, after a domain is unassigned from it, before deleting that domain's content on its own.

## Request

```json
{
  "id": "urn:uuid:49f51276-0624-4578-8b7c-5e6f7a8b9c01",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/unassign/0.1",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid4:node2.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "domain": "old.example.com" }
}
```

## Response

```json
{
  "id": "urn:uuid:49f51276-0624-4578-8b7c-5e6f7a8b9c02",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/unassign/0.1#response",
  "threadId": "urn:uuid:49f51276-0624-4578-8b7c-5e6f7a8b9c01",
  "issuer": "did:webvh:QmNodeScid4:node2.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "domain": "old.example.com", "status": "scheduled", "purgeAt": "2026-09-27T11:00:01Z" }
}
```

## Security & Privacy

### Data carried

A domain name, and on the response the time its content will be deleted.

### Correlation

As for [`did-management/replica/domain/assign`](../../assign/0.1/spec.md): the payload shows which domain this server stops serving for this control plane, and confidential transport bindings keep it from intermediaries. Both parties declare `identifierScope: public`, and must: the replica trusts exactly one configured control-plane DID, and the control plane addresses the replica by the DID it registered with.

### Retention

The replica keeps the schedule until it runs or is cancelled; the directive is not worth keeping once applied.

### Consent/purpose

The grace period exists to make an unassignment recoverable. A replica **MUST NOT** delete the content before `purgeAt` except on a `did-management/replica/domain/purge` directive.
