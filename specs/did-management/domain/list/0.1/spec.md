---
slug: did-management/domain/list
version: "0.1"
title: DID Management — Domain List
summary: An administrator enumerates every hosting domain a DID hosting service knows — active and disabled, with each one's purge schedule — and the system default.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, domain, multi-domain, admin, list]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The full catalogue — disabled domains and their purge schedules included — spans every tenant on the service and is an administrator's view. The service authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the domain catalogue; nothing is written."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns every domain entry: names, labels, states and lifecycle instants."
retention:
  class: transient
  rationale: "The request carries at most a state filter."
errorCodes: []
related:
  - did-management/me/domains
  - did-management/domain/create
  - did-management/domain/update
  - did-management/domain/set-state
  - did-management/domain/set-default
  - did-management/domain/purge
---

## Abstract

A multi-domain hosting service serves DIDs on several hosting domains, each active or disabled. **Domain List** is the administrator's catalogue: every domain, its state, when a disabled one becomes eligible for purge, and which is the system default — the table an operator console's domains page shows.

A caller who is not an administrator reads the domains *they* may act on with [`did-management/me/domains`](../../../me/domains/0.1/spec.md); this task is the whole catalogue.

The family ships no read-one sibling, and needs none: the collection is small, a domain's identifier is its public name, and every task that acts on a domain by name answers `did-management:unknownDomain` itself, so whether a domain exists is never answered by an empty page.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Returns every domain in the requested state (every state when `status` is absent), ordered by name, and `default` when a system default is set. Changes nothing.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this hosting service**, read from its own access-control records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A caller scoped to some domains reads those with `me/domains`, not a subset of this list.

## Definitions

**Hosting domain**, **system default** — as in the shared `DomainEntry` component and the [category conventions](../../../_shared/0.1/CONVENTIONS.md#1-domain-resolution).

## Request

```json
{
  "id": "urn:uuid:16c2ef43-d3f1-4245-9e4f-2b3c4d5e6f01",
  "type": "https://trusttasks.org/spec/did-management/domain/list/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:16c2ef43-d3f1-4245-9e4f-2b3c4d5e6f02",
  "type": "https://trusttasks.org/spec/did-management/domain/list/0.1#response",
  "threadId": "urn:uuid:16c2ef43-d3f1-4245-9e4f-2b3c4d5e6f01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "domains": [
      { "name": "did.example.com", "label": "Primary", "status": "active", "defaultDomain": true, "createdAt": "2026-09-01T10:00:00Z" },
      { "name": "old.example.com", "status": "disabled", "createdAt": "2026-06-01T10:00:00Z", "disabledAt": "2026-09-20T10:00:00Z", "purgeAt": "2026-09-27T10:00:00Z" }
    ],
    "default": "did.example.com"
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a state. The response carries every domain entry. Domain names are public by nature — each is in the DIDs hosted on it — but the catalogue as a whole lists every tenant on one service, and a disabled domain's `purgeAt` says when its content disappears; both are an operator's view.

### Correlation

The catalogue ties together domains that belong to different tenants by the fact that one service hosts them all, which is not otherwise published. It is disclosed to administrators only. Both parties declare `identifierScope: public`, and must: the service's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the access-control entry is keyed on.

### Retention

Nothing of the request needs keeping beyond the exchange.

### Consent/purpose

The catalogue is disclosed so an administrator can manage the service's domains. It **MUST NOT** be disclosed to tenants, who read their own scope with `me/domains`.
