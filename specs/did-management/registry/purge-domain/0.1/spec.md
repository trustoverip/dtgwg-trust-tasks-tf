---
slug: did-management/registry/purge-domain
version: "0.1"
title: DID Management — Registry Purge Domain
summary: An administrator tells the control plane to have one hosting server delete, now, every DID it holds on a domain that has been unassigned from it — bypassing the server's unassignment grace period.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, registry, admin, domain, purge, grace-period]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Control plane
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The task deletes content from a server irreversibly. The control plane authorises it on the document's proven issuer on every transport, and retains the signed request as the record of who ordered the deletion."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A purge is irreversible. Replayed after the domain has been re-assigned to the server, it would wipe content the operator chose to keep; the issue time is what lets the control plane, and the server downstream, place the order before or after that re-assignment."
sideEffects:
  level: destructive
  rationale: "Queues a directive that makes the server delete every DID it holds on the domain immediately. The server's copy cannot be restored except by re-assigning the domain and resyncing from the control plane."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The response echoes the request and says the directive was queued."
retention:
  class: durable
  rationale: "The signed request is the record of an irreversible administrative deletion and is worth keeping as long as the control plane keeps its audit trail."
errorCodes:
  - code: did-management/registry/purge-domain:notFound
    meaning: "No registry entry has the submitted `instanceId`."
    retryable: false
  - code: did-management/registry/purge-domain:notAServer
    meaning: "The instance is not a hosting server (`serviceType` is not `server`), so it holds no domain content."
    retryable: false
  - code: did-management/registry/purge-domain:stillAssigned
    meaning: "The domain is still assigned to this server in the control plane's registry. Unassign it first (did-management/domain/unassign); a purge of a domain the server is meant to serve would leave it unable to resolve DIDs it is responsible for."
    retryable: false
  - code: did-management:unknownDomain
    meaning: "The submitted `domain` is not a known hosting domain on this control plane. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error)."
    retryable: false
related:
  - did-management/domain/unassign
  - did-management/domain/purge
  - did-management/replica/domain/purge
  - did-management/registry/get
---

## Abstract

When a domain is unassigned from a hosting server ([`did-management/domain/unassign`](../../../domain/unassign/0.1/spec.md)), the server stops accepting content for it and schedules the deletion of what it holds after a grace period, so an operator who unassigned by mistake can re-assign without a resync. **Registry Purge Domain** is the administrator's "purge now" for one server: the control plane queues a [`did-management/replica/domain/purge`](../../../replica/domain/purge/0.1/spec.md) directive to that server, which deletes the domain's DIDs immediately.

It is per-server and applies only to a domain the server no longer serves. Removing a disabled domain from the whole fleet is [`did-management/domain/purge`](../../../domain/purge/0.1/spec.md).

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming control plane:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`), and one with no `issuedAt` with `malformedRequest`.
2. Refuses a caller without administrator standing ([Authorization](#authorization)) with `permissionDenied`.
3. Answers `notFound` for an unknown `instanceId`, `notAServer` for an instance that is not a hosting server, and `did-management:unknownDomain` for a domain it does not know.
4. Answers `stillAssigned` when its registry records the domain as assigned to the instance.
5. Otherwise enqueues a `did-management/replica/domain/purge` directive for the instance in its durable outbox, signed at send time, and answers `status: "queued"`. The directive's own `issuedAt` is what the server's freshness rule compares against a later re-assignment, so the control plane **MUST NOT** re-use a directive signed before the domain was last assigned.

The server's acknowledgement arrives on the replica task's own thread and settles the outbox entry; the administrator reads the effect from the instance's `servedDomains` and the server's reply, not from this response.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on this control plane**, read from its own access-control records at execution time; the verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The `stillAssigned` precondition is a safety interlock, not authorization: it stops an administrator deleting content the server still serves, and says nothing about whether the caller may order a deletion at all.

## Definitions

**Unassignment grace** — the period a hosting server waits, after a domain is unassigned from it, before deleting that domain's content on its own.

## Request

```json
{
  "id": "urn:uuid:8d3f5cab-4a6e-4fbd-8b5c-3e4f5a6b7c01",
  "type": "https://trusttasks.org/spec/did-management/registry/purge-domain/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "instanceId": "did_webvh_node2_example_com", "domain": "old.example.com" }
}
```

## Response

```json
{
  "id": "urn:uuid:8d3f5cab-4a6e-4fbd-8b5c-3e4f5a6b7c02",
  "type": "https://trusttasks.org/spec/did-management/registry/purge-domain/0.1#response",
  "threadId": "urn:uuid:8d3f5cab-4a6e-4fbd-8b5c-3e4f5a6b7c01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "instanceId": "did_webvh_node2_example_com", "domain": "old.example.com", "status": "queued" }
}
```

## Security & Privacy

### Data carried

The request carries an instance identifier and a domain name; the response echoes both. No hosted DID is named.

### Correlation

The request joins an administrator to a server and a domain the server used to serve. It is addressed only to the control plane, which already holds that association. Both parties declare `identifierScope: public`, and must: the control plane's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the access-control entry is keyed on.

### Retention

The signed request is the record of an irreversible deletion ordered by a named administrator; a control plane **SHOULD** retain it with its audit trail. The outbox entry it creates is exchange-scoped and removed when the server acknowledges.

### Consent/purpose

The task exists to reclaim a server's storage, or to remove content from a server that must no longer hold it, without waiting out the grace period. It deletes only the server's copy; the control plane's own records and every other server are untouched.
