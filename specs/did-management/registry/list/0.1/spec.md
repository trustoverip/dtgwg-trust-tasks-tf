---
slug: did-management/registry/list
version: "0.1"
title: DID Management — Registry List
summary: An administrator enumerates the service instances registered with a control plane — hosting servers, witnesses and watchers — with each one's health verdict, served domains and transport observations.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, registry, admin, service-instance, list, fleet]
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
  rationale: "The answer is the control plane's whole fleet map, disclosed only to administrators. The control plane authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport rather than whoever holds a session."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a control plane that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the stored registry. Nothing is probed or updated — that is did-management/registry/check."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns every matching instance's DID, URL, served domains, health verdict and transport observations: the fleet's operational topology, never key material."
retention:
  class: transient
  rationale: "The request carries at most two filters and is needed only to answer it."
errorCodes: []
related:
  - did-management/registry/get
  - did-management/registry/check
  - did-management/registry/admin-register
  - did-management/registry/deregister
  - did-management/stats/get
  - did-management/server/config
---

## Abstract

**Registry List** enumerates the instances a control plane drives. It is what an operator console's fleet view reads, and — with [`did-management/stats/get`](../../../stats/get/0.1/spec.md) for the counters and [`did-management/server/config`](../../../server/config/0.1/spec.md) for the control plane's own settings — everything its dashboard needs. There is no separate "overview" task: the aggregate counts a dashboard shows (instances per verdict) are computed from this list by the reader.

The list and [`did-management/registry/get`](../../get/0.1/spec.md) are the family's split read pair. The list offers two filters and no paging: a registry is a small, operator-scoped collection — tens of instances, not thousands — and a caller that needs one instance by identifier uses `get`, whose `notFound` is definite where an empty filtered page is not.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming control plane:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing ([Authorization](#authorization)) with `permissionDenied`.
3. Returns every registry entry matching both filters that are present, ordered by `instanceId`, changing nothing. No match is an empty `instances` array.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this control plane**, read from its own access-control records at execution time; the verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Owning hosted DIDs is not standing, and neither is being a registered instance.

## Definitions

**Instance**, **health verdict** — as in [`did-management/registry/get`](../../get/0.1/spec.md) and the shared `ServiceInstance` component.

## Request

```json
{
  "id": "urn:uuid:6b1d3a8f-2e4c-4d9b-8f3a-1c2d3e4f5a01",
  "type": "https://trusttasks.org/spec/did-management/registry/list/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "serviceType": "server" }
}
```

## Response

```json
{
  "id": "urn:uuid:6b1d3a8f-2e4c-4d9b-8f3a-1c2d3e4f5a02",
  "type": "https://trusttasks.org/spec/did-management/registry/list/0.1#response",
  "threadId": "urn:uuid:6b1d3a8f-2e4c-4d9b-8f3a-1c2d3e4f5a01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "instances": [
      {
        "instanceId": "did_webvh_node1_example_com",
        "did": "did:webvh:QmNodeScid3:node1.example.com",
        "serviceType": "server",
        "publicUrl": "https://node1.example.com",
        "status": "active",
        "servedDomains": ["did.example.com"],
        "enabledMethods": ["webvh"],
        "lastHealthCheck": "2026-09-27T08:59:30Z",
        "registeredAt": "2026-09-01T10:00:00Z"
      },
      {
        "instanceId": "did_webvh_node2_example_com",
        "did": "did:webvh:QmNodeScid4:node2.example.com",
        "serviceType": "server",
        "status": "unreachable",
        "servedDomains": [],
        "enabledMethods": ["webvh"],
        "registeredAt": "2026-09-20T10:00:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a service type and a health verdict. The response carries every matching registry record: DIDs, URLs, served domains, DID methods, verdicts and transport observations. No key material and no hosted DID appears. `label` is operator-written free text and **MUST NOT** be presented as an attested property.

### Correlation

The list is the complete association between this control plane and every edge, witness and watcher it drives, and between edges and the domains they serve — the topology a hosted DID deliberately never advertises. It is disclosed to administrators only, and consumers **SHOULD** require a confidential transport.

Both parties declare `identifierScope: public`, and must: the control plane's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the control plane's access-control entry is keyed on — a pairwise identifier would match no entry. Each party already holds the other's identifier from the administrator's enrolment.

### Retention

Nothing of the request needs keeping beyond the exchange. A control plane that audits administrative reads **MAY** record the administrator and the filters.

### Consent/purpose

The list is disclosed so an administrator can operate the fleet. It **MUST NOT** be used to publish the control plane's association with the domains its edges serve.
