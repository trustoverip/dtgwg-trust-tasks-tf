---
slug: did-management/server/metrics
version: "0.1"
title: "DID Management — Server Metrics"
summary: "An administrator reads a DID hosting service's operational metrics snapshot: named counters, gauges and histograms, for a metrics collector to scrape over a Trust Task instead of a Prometheus endpoint."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, metrics, prometheus, observability, admin, read]
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
  rationale: "Operational metrics reveal load, error rates and internal topology to administrators only. The service authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport, and there is no unauthenticated equivalent — see Consent/purpose."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the service's own instrumentation; nothing is written and no counter is reset by being read."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns named counters, gauges and histograms describing the service's own operation. None of it is about a hosted DID or a person; a counter labelled by DID method or domain is operational metadata, not subject data."
retention:
  class: transient
  rationale: "The request carries nothing but its envelope; the response is a point-in-time reading a collector timestamps and stores on its own terms."
errorCodes: []
related:
  - did-management/server/config
  - did-management/server/info
  - did-management/stats/get
  - did-management/registry/list
---

## Abstract

**Server Metrics** is how a DID hosting service exposes the counters, gauges and histograms an operator's metrics stack needs — request counts, error rates, queue depths, resolve latency — as a Trust Task instead of an unauthenticated Prometheus `/metrics` endpoint. Every remote surface the service serves moves onto Trust Task dispatch; an operational metrics scrape is a remote surface like any other; it needs administrator standing and a transport binding like every other administrative read in this category.

The shared shape, [`_shared/0.2/metrics-snapshot`](../../../_shared/0.2/metrics-snapshot.schema.json), is deliberately generic — any service in this ecosystem can answer its own `*/server/metrics`-shaped task with the same `MetricsSnapshot`, not only a DID hosting service.

This is not the DID-specific resolve/update counters — those are [`did-management/stats/get`](../../../stats/get/0.1/spec.md) and [`did-management/stats/timeseries`](../../../stats/timeseries/0.1/spec.md), scoped to one slot or the service's DID-serving totals. **Server Metrics** is the operator's process-level view: how the service itself is running, independent of any one DID.

A generic, unauthenticated liveness probe (no metrics, no detail) is a separate, deliberately different concern and stays outside the Trust Task surface — see [`did-management/server/info`](../../info/0.1/spec.md) for the public read this category does define, and Consent/purpose below for why a liveness probe is not this task.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Otherwise answers with a `snapshot`: the service's own counters, gauges and histograms at the instant it was read (`takenAt`).
4. **MUST NOT** serve an unauthenticated equivalent of this task's response — no `/metrics` route, and no member of [`did-management/server/info`](../../info/0.1/spec.md) or any other public task carries a counter, a gauge or a histogram.
5. **SHOULD** name metrics and labels consistently with whatever the service already used on its (now-deleted) Prometheus endpoint, so existing dashboards and alert rules can be re-pointed at a scrape adapter with a name mapping only, not a redesign.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this hosting service**, read from its own access-control records at execution time. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Snapshot** — the service's counters, gauges and histograms as read at one instant (`takenAt`); not a stream, and not a diff against a previous read.

**Counter**, **gauge**, **histogram** — as in [`_shared/0.2/metrics-snapshot`](../../../_shared/0.2/metrics-snapshot.schema.json): a counter only rises; a gauge may rise or fall; a histogram is a distribution reported as cumulative buckets. `counters`, `gauges` and `histograms` are each always present in a response, empty when the service exposes none of that kind — so a collector can tell "none of this kind" from "the field was dropped", the same convention [`did-management/registry/list`](../../../registry/list/0.1/spec.md) uses for `servedDomains`.

## Request

An administrator sends the request to the hosting service naming no selector; the whole snapshot is always returned. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### The whole snapshot, always

```json
{
  "id": "urn:uuid:d38f4a1e-9c6b-4d2f-8a1e-2b3c4d5e6f01",
  "type": "https://trusttasks.org/spec/did-management/server/metrics/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {}
}
```

## Response

The hosting service answers with `snapshot`, the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` rather than this shape.

### A snapshot with one counter, one gauge and one histogram

```json
{
  "id": "urn:uuid:d38f4a1e-9c6b-4d2f-8a1e-2b3c4d5e6f02",
  "type": "https://trusttasks.org/spec/did-management/server/metrics/0.1#response",
  "threadId": "urn:uuid:d38f4a1e-9c6b-4d2f-8a1e-2b3c4d5e6f01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "snapshot": {
      "takenAt": "2026-09-27T09:00:01Z",
      "counters": [
        {
          "name": "did_hosting_requests_total",
          "value": 184213.0,
          "labels": { "task": "did-management/did/log" }
        }
      ],
      "gauges": [
        { "name": "did_hosting_registry_instances", "value": 4.0 }
      ],
      "histograms": [
        {
          "name": "did_hosting_resolve_seconds",
          "count": 9210,
          "sum": 184.4,
          "buckets": [
            { "le": 0.01, "count": 8000 },
            { "le": 0.1, "count": 9100 },
            { "le": 1.0, "count": 9210 }
          ]
        }
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing but its envelope. The response carries named counters, gauges and histograms describing the service's own operation — request volumes, queue depths, latency distributions — and their labels. A producer **MUST NOT** put a subject identifier, a DID it hosts, or any other personal or secret value into a metric's `name` or a label's value; a label distinguishes an operational dimension (a task name, a transport, a domain), not a person or a specific hosted DID.

### Correlation

An administrator reading metrics is identified to a service it already administers; both parties declare `identifierScope: public` for the same reason [`did-management/server/config`](../../config/0.1/spec.md) does — the administrator's DID is what the service's access-control entry is keyed on, and the service's DID is the recipient every administrator addresses. A label that names a domain or a DID method is an operational dimension shared by many requests and many subjects, not a correlation handle for any one of them.

### Retention

Nothing about the request needs keeping. The response is a point-in-time reading; a metrics collector timestamps and stores it on its own retention policy, which this specification does not constrain.

### Consent/purpose

The task exists so an administrator's metrics stack can read the service's operational state without an unauthenticated network endpoint. It **MUST NOT** be used as a substitute for the plain, unauthenticated liveness probes (`/api/health`, a daemon's `/health`) those probes exist for a different audience — a load balancer or orchestrator that needs "is it up", not "how is it performing" — and stay outside the Trust Task surface for that reason: they carry no detail to protect, so requiring a proof would only cost every orchestrator a key it has no other use for.
