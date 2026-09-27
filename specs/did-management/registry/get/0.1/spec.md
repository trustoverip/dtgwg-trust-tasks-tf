---
slug: did-management/registry/get
version: "0.1"
title: DID Management — Registry Get
summary: An administrator reads one service instance's record from the control-plane registry — its DID, type, health verdict, served domains and the transports it was last seen on.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, registry, admin, service-instance, read]
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
  rationale: "The registry is the control plane's map of its own fleet — which DIDs it trusts as edges, where they are and how they are reached. It is disclosed only to administrators, and the control plane authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport rather than whoever holds a session."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a control plane that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the stored registry record. Nothing is probed, re-resolved or updated — that is did-management/registry/check."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns one instance's DID, URL, served domains, health verdict and transport observations: operational topology of the fleet, never key material."
retention:
  class: transient
  rationale: "The request names one instance and is needed only to answer it."
errorCodes:
  - code: did-management/registry/get:notFound
    meaning: "No registry entry has the submitted `instanceId`."
    retryable: false
related:
  - did-management/registry/list
  - did-management/registry/check
  - did-management/registry/admin-register
  - did-management/registry/deregister
  - did-management/server/register
---

## Abstract

A control plane keeps a registry of the service instances it drives — hosting servers it syncs DID logs to, witnesses and watchers. **Registry Get** reads one entry: what an operator console shows when an administrator opens an instance, and what a script reads before assigning a domain to it.

It is the read-one half of the registry reads; [`did-management/registry/list`](../../list/0.1/spec.md) enumerates. The two are split because they fail differently: an unknown `instanceId` here is a definite `notFound`, where a list filtered to nothing is an empty, successful page.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming control plane:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing ([Authorization](#authorization)) with `permissionDenied`, before looking the instance up — so a non-administrator learns nothing about which instances exist.
3. Answers `did-management/registry/get:notFound` when no entry has `instanceId`.
4. Otherwise returns the entry as it is stored, changing nothing.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this control plane**, read from the control plane's own access-control records at execution time. The verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Owning DIDs hosted by the control plane is not standing: an owner has no business with the fleet that serves them. A registered instance is not standing either — an instance reads nothing about its peers.

## Definitions

**Instance** — one service the control plane addresses by DID: a hosting server, witness or watcher, entered in the registry by self-registration ([`did-management/server/register`](../../../server/register/0.1/spec.md)) or by an administrator ([`did-management/registry/admin-register`](../../admin-register/0.1/spec.md)).

## Request

```json
{
  "id": "urn:uuid:5a0c2f7e-1d3b-4c8a-9e2f-0b1c2d3e4f01",
  "type": "https://trusttasks.org/spec/did-management/registry/get/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "instanceId": "did_webvh_node1_example_com" }
}
```

## Response

The entry, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:5a0c2f7e-1d3b-4c8a-9e2f-0b1c2d3e4f02",
  "type": "https://trusttasks.org/spec/did-management/registry/get/0.1#response",
  "threadId": "urn:uuid:5a0c2f7e-1d3b-4c8a-9e2f-0b1c2d3e4f01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "instance": {
      "instanceId": "did_webvh_node1_example_com",
      "did": "did:webvh:QmNodeScid3:node1.example.com",
      "serviceType": "server",
      "label": "EU edge 1",
      "publicUrl": "https://node1.example.com",
      "status": "active",
      "servedDomains": ["did.example.com"],
      "enabledMethods": ["webvh", "web"],
      "advertisedServices": ["TSPTransport", "DIDCommMessaging"],
      "servicesCheckedAt": "2026-09-27T08:55:00Z",
      "lastInbound": { "transport": "tsp", "at": "2026-09-27T08:59:30Z" },
      "lastOutbound": { "transport": "tsp", "at": "2026-09-27T08:59:29Z" },
      "lastHealthCheck": "2026-09-27T08:59:30Z",
      "registeredAt": "2026-09-01T10:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries one instance identifier. The response carries that instance's registry record: its DID, public URL, served domains, DID methods, health verdict and when and how the control plane last exchanged documents with it. None of it is key material and none of it names a hosted DID. `label` is operator-written free text and **MUST NOT** be presented as an attested property.

### Correlation

The record joins an edge's DID to the domains it serves and to the control plane that drives it — the fleet topology the control plane otherwise never publishes (a hosted DID deliberately does not advertise its control plane). That is why the task is administrator-only and answers `permissionDenied` before `notFound`: a non-administrator cannot use it to learn which edges belong to this control plane. Consumers **SHOULD** require a confidential transport.

Both parties declare `identifierScope: public`, and must. The control plane's DID is the one recipient every administrator addresses and the key its replies are verified against; the administrator's DID is what the control plane's access-control entry is keyed on, so a pairwise identifier would match no entry and be refused. Neither identifier is new to the other party — each already holds the other's from the administrator's enrolment.

### Retention

Nothing of the request needs keeping beyond the exchange. A control plane that audits administrative reads **MAY** record the administrator and `instanceId`.

### Consent/purpose

The record is disclosed so an administrator can operate the fleet — inspect an instance, decide where to assign a domain, diagnose a stalled sync. It **MUST NOT** be used to publish or advertise the control plane's association with the domains it serves.
