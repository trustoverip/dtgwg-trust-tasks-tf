---
slug: did-management/registry/check
version: "0.1"
title: DID Management — Registry Check
summary: An administrator asks the control plane to re-evaluate one instance now — recompute its health verdict and re-resolve the services its DID document advertises — and returns the refreshed entry.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, registry, admin, health, service-instance]
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
  rationale: "The task makes the control plane resolve an instance's DID over the network and rewrite its registry record, and answers with the fleet topology. Both are an administrator's to trigger, and the control plane authorises on the document's proven issuer on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The task changes the stored record. A replayed check is harmless in outcome but is still a resolution and a write the administrator did not ask for at that time, and an issue time bounds how long a captured one can be spent."
sideEffects:
  level: mutating
  rationale: "Rewrites the instance's health verdict and its advertised-services observation in the registry."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns the refreshed registry record: operational topology, never key material."
retention:
  class: transient
  rationale: "The request names one instance and is needed only to answer it."
errorCodes:
  - code: did-management/registry/check:notFound
    meaning: "No registry entry has the submitted `instanceId`."
    retryable: false
related:
  - did-management/registry/get
  - did-management/registry/list
  - did-management/server/health
---

## Abstract

A control plane probes its instances on a schedule with [`did-management/server/health`](../../../server/health/0.1/spec.md) and keeps a verdict for each. **Registry Check** is the administrator's "check now": the control plane recomputes the instance's verdict from the most recent probe it has answered, re-resolves the instance's DID document to refresh the services it advertises, stores both, and returns the entry.

It does not send a probe and wait for it. A probe answered asynchronously would make the task's latency the instance's worst case, and an unreachable instance would hold the administrator's request open until it timed out; the scheduled probes already give the control plane the evidence the verdict is computed from.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming control plane:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`), and one with no `issuedAt` with `malformedRequest`.
2. Refuses a caller without administrator standing ([Authorization](#authorization)) with `permissionDenied`, before looking the instance up.
3. Answers `did-management/registry/check:notFound` when no entry has `instanceId`.
4. Recomputes the verdict from the time of the instance's last answered probe against its own configured probe interval, and stores it.
5. Re-resolves the instance's DID and stores the `service[].type` values of the resolved document as `advertisedServices`, with `servicesCheckedAt`. A resolution that fails leaves the previous observation in place and **MUST NOT** fail the task: the verdict is still refreshed and returned.
6. Returns the entry as stored after steps 4 and 5.

The verdict and the advertised services are observations for display. A control plane **MUST NOT** route on `advertisedServices` — it chooses a transport from the DID document it resolves at send time.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on this control plane**, read from its own access-control records at execution time; the verified `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A registered instance cannot check itself or a peer with this task.

## Definitions

**Health verdict** — `active`, `degraded` or `unreachable`, as defined by the shared `ServiceInstance` component.

## Request

```json
{
  "id": "urn:uuid:7c2e4b9a-3f5d-4eac-9a4b-2d3e4f5a6b01",
  "type": "https://trusttasks.org/spec/did-management/registry/check/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "instanceId": "did_webvh_node2_example_com" }
}
```

## Response

```json
{
  "id": "urn:uuid:7c2e4b9a-3f5d-4eac-9a4b-2d3e4f5a6b02",
  "type": "https://trusttasks.org/spec/did-management/registry/check/0.1#response",
  "threadId": "urn:uuid:7c2e4b9a-3f5d-4eac-9a4b-2d3e4f5a6b01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:02Z",
  "payload": {
    "instance": {
      "instanceId": "did_webvh_node2_example_com",
      "did": "did:webvh:QmNodeScid4:node2.example.com",
      "serviceType": "server",
      "status": "degraded",
      "servedDomains": ["did.example.com"],
      "enabledMethods": ["webvh"],
      "advertisedServices": ["TSPTransport"],
      "servicesCheckedAt": "2026-09-27T09:00:02Z",
      "lastHealthCheck": "2026-09-27T08:54:10Z",
      "registeredAt": "2026-09-20T10:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries one instance identifier; the response, that instance's refreshed registry record. No key material and no hosted DID appears.

### Correlation

The task makes the control plane resolve the instance's DID, which an observer of the instance's hosting (or of the resolver the control plane uses) can see as a fetch from the control plane at that moment. That fetch is the same one the scheduled refresh makes, so the task reveals only that an administrator acted, not anything new about the fleet. The response carries the same topology as [`did-management/registry/get`](../../get/0.1/spec.md) and is administrator-only for the same reason.

Both parties declare `identifierScope: public`, and must: the control plane's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the access-control entry is keyed on.

### Retention

Nothing of the request needs keeping beyond the exchange. The refreshed observations replace the previous ones in the registry; a control plane **MAY** audit the administrator and `instanceId`.

### Consent/purpose

The task exists so an administrator diagnosing an instance does not have to wait for the next scheduled refresh. Its result **MUST NOT** be used to route documents.
