---
slug: did-management/replica/domain/purge
version: "0.1"
title: DID Management — Replica Domain Purge
summary: A control plane tells a hosting server it drives to delete, now, every DID it holds on a hosting domain — refused if the server has been re-assigned the domain since the directive was issued.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, replica, edge, domain, purge, control-plane]
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
  rationale: "The directive deletes content irreversibly. A replica acts only for its configured control plane, and only a proof bound to that DID shows the directive came from it on every transport; the proof also covers `issuedAt`, which the freshness rule depends on."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The freshness rule compares the signed issue time with the replica's current assignment of the domain. Without it a purge queued before a re-assignment — delayed in a mediator, say — would wipe content the operator chose to keep."
sideEffects:
  level: destructive
  rationale: "Deletes every slot the replica holds on the domain, with its log, witness proofs and agent-name index. The replica's copy can only be rebuilt by re-assigning the domain and resyncing from the control plane."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The request carries a domain name; the response, how many slots were removed."
retention:
  class: transient
  rationale: "The deletion is recorded by the administrator-facing task that ordered it; the directive is not worth keeping once applied."
errorCodes:
  - code: did-management/replica/domain/purge:notAuthorized
    meaning: "The document's proven issuer is not this replica's configured control plane."
    retryable: false
  - code: did-management/replica/domain/purge:stalePurge
    meaning: "The replica holds a current assignment of the domain made after the directive's `issuedAt`. The purge predates the re-assignment and is refused; nothing was deleted."
    retryable: false
related:
  - did-management/registry/purge-domain
  - did-management/domain/purge
  - did-management/replica/domain/unassign
  - did-management/replica/domain/assign
---

## Abstract

**Replica Domain Purge** makes a server delete a domain's content immediately rather than at the end of its unassignment grace period. The control plane sends it when an administrator orders [`did-management/registry/purge-domain`](../../../../registry/purge-domain/0.1/spec.md) for one server, or purges a disabled domain across the fleet with [`did-management/domain/purge`](../../../../domain/purge/0.1/spec.md).

Directives travel through a durable outbox and may be delayed. The one hazard that creates for a deletion is a purge that arrives after the operator has changed their mind and re-assigned the domain: this task's freshness rule makes the replica refuse it.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming replica:

1. Verifies the `proof` and binds it to the in-band `issuer`. A document that does not verify gets **no reply at all**.
2. Refuses with `notAuthorized` a document whose proven issuer is not its configured control plane.
3. **Freshness.** If it holds a current assignment of the domain whose assignment time is later than the document's `issuedAt`, refuses with `stalePurge` and deletes nothing. A failure to read its assignment **MUST NOT** skip this check: it is a transient failure (rule 5).
4. Otherwise deletes every slot it holds on the domain — content, witness proofs, derived agent-name index — cancels any purge it had scheduled for the domain, and answers `purged` with the number removed. A domain on which it holds nothing is answered `purged` with `removed: 0`.
5. Answers a transient failure with a retryable `trust-task-error`, so the control plane keeps the directive queued.

A conforming control plane signs each delivery attempt at send time, so a directive re-sent after a re-assignment carries an `issuedAt` later than that assignment and is **not** protected by rule 3. It therefore **MUST** drop a queued purge for a server when it assigns that server the domain again.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being this replica's configured control plane**, established by comparing the proof-verified `issuer` with the one control-plane DID the replica was configured with. The freshness rule is a safety interlock, not authorization: a stale purge from the right control plane is refused, and a fresh one from any other party is refused for lack of standing.

## Definitions

**Assignment time** — the replica's own clock at the moment it applied the [`did-management/replica/domain/assign`](../../assign/0.1/spec.md) directive now in force.

## Request

```json
{
  "id": "urn:uuid:5a062387-1735-4689-9c8d-6f7a8b9c0d01",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/purge/0.1",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmNodeScid4:node2.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "domain": "old.example.com" }
}
```

## Response

```json
{
  "id": "urn:uuid:5a062387-1735-4689-9c8d-6f7a8b9c0d02",
  "type": "https://trusttasks.org/spec/did-management/replica/domain/purge/0.1#response",
  "threadId": "urn:uuid:5a062387-1735-4689-9c8d-6f7a8b9c0d01",
  "issuer": "did:webvh:QmNodeScid4:node2.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:02Z",
  "payload": { "domain": "old.example.com", "status": "purged", "removed": 37 }
}
```

## Security & Privacy

### Data carried

A domain name, and on the response a count. The deletion removes public DID content from one server; the control plane still holds it.

### Correlation

As for [`did-management/replica/domain/assign`](../../assign/0.1/spec.md): the payload shows which domain this server held for this control plane, and confidential transport bindings keep it from intermediaries. The count says how many DIDs the server held on the domain — fleet metadata, disclosed only to the control plane that synced them. Both parties declare `identifierScope: public`, and must: the replica trusts exactly one configured control-plane DID, and the control plane addresses the replica by the DID it registered with.

### Retention

The replica keeps nothing of the purged content. The administrator-facing task that ordered the purge is the audit record; the directive need not be kept once applied.

### Consent/purpose

The directive exists to remove a domain's content from a server that must no longer hold it. It deletes only the replica's copy.
