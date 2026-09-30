---
slug: vta/metrics/show
version: "0.1"
title: "VTA Metrics — Show"
summary: "An administrator reads a Verifiable Trust Agent's operational metrics snapshot: named counters, gauges and histograms, for a metrics collector to scrape over a Trust Task instead of an unauthenticated Prometheus endpoint."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vta, metrics, prometheus, observability, admin, read]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: VTA
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Operational metrics reveal load, error rates and internal topology to administrators only. The VTA authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport, and there is no unauthenticated equivalent — see Consent/purpose.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no effect to replay, but an issue time dates the request for a VTA that audits administrative reads.
sideEffects:
  level: none
  rationale: "Reads the VTA's own instrumentation; nothing is written and no counter is reset by being read."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Returns named counters, gauges and histograms describing the VTA's own operation. None of it is about a hosted identity, a context or a person; a counter labelled by task name or transport is operational metadata, not subject data.
retention:
  class: transient
  rationale: "The request carries nothing but its envelope; the response is a point-in-time reading a collector timestamps and stores on its own terms."
errorCodes: []
related:
  - did-management/server/metrics
  - vta/health/details
  - vta/management/reload-services
---

## Abstract

**VTA Metrics — Show** is how a Verifiable Trust Agent exposes the counters, gauges and histograms an operator's metrics stack needs — request counts, error rates, queue depths, DIDComm and TSP dispatch latency — as a Trust Task instead of an unauthenticated Prometheus `/metrics` endpoint. Every remote surface the VTA serves moves onto Trust Task dispatch; an operational metrics scrape is a remote surface like any other, and it needs administrator standing and a transport binding like every other administrative read.

The shared shape, [`did-management/_shared/0.2/metrics-snapshot`](../../../../did-management/_shared/0.2/metrics-snapshot.schema.json), is deliberately generic — it was defined for a DID hosting service and is reused here unchanged, because the VTA's own metrics are the same kind of thing: named counters, gauges and histograms, read whole, at an instant.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A conforming VTA:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Otherwise answers with a `snapshot`: the VTA's own counters, gauges and histograms at the instant it was read (`takenAt`).
4. **MUST NOT** serve an unauthenticated equivalent of this task's response — no `/metrics` route, and no member of any public VTA read carries a counter, a gauge or a histogram.
5. **SHOULD** name metrics and labels consistently with whatever the VTA already used on its (now-deleted) Prometheus endpoint, so existing dashboards and alert rules can be re-pointed at a scrape adapter with a name mapping only, not a redesign.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this VTA**, read from its own access-control records at execution time. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). This task deliberately requires ordinary administrator standing rather than super-admin: metrics are read-only operational data, not the sensitive control-plane actions ([`vta/attestation/mnemonic-status`](../../../attestation/mnemonic-status/0.1/spec.md), `vta/restart`) that require the stronger entitlement.

## Definitions

**Snapshot** — the VTA's counters, gauges and histograms as read at one instant (`takenAt`); not a stream, and not a diff against a previous read.

**Counter**, **gauge**, **histogram** — as in [`did-management/_shared/0.2/metrics-snapshot`](../../../../did-management/_shared/0.2/metrics-snapshot.schema.json): a counter only rises; a gauge may rise or fall; a histogram is a distribution reported as cumulative buckets. `counters`, `gauges` and `histograms` are each always present in a response, empty when the VTA exposes none of that kind — so a collector can tell "none of this kind" from "the field was dropped".

## Request

An administrator sends the request to the VTA naming no selector; the whole snapshot is always returned. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### The whole snapshot, always

```json
{
  "id": "urn:uuid:1a2b3c4d-5e6f-4a1b-8c2d-3e4f5a6b7c01",
  "type": "https://trusttasks.org/spec/vta/metrics/show/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-30T09:00:00Z",
  "payload": {}
}
```

## Response

The VTA answers with `snapshot`, the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` rather than this shape.

### A snapshot with one counter, one gauge and one histogram

```json
{
  "id": "urn:uuid:1a2b3c4d-5e6f-4a1b-8c2d-3e4f5a6b7c02",
  "type": "https://trusttasks.org/spec/vta/metrics/show/0.1#response",
  "threadId": "urn:uuid:1a2b3c4d-5e6f-4a1b-8c2d-3e4f5a6b7c01",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-30T09:00:01Z",
  "payload": {
    "snapshot": {
      "takenAt": "2026-09-30T09:00:01Z",
      "counters": [
        {
          "name": "vta_requests_total",
          "value": 184213.0,
          "labels": { "task": "vta/app-state/get" }
        }
      ],
      "gauges": [
        { "name": "vta_sessions_active", "value": 4.0 }
      ],
      "histograms": [
        {
          "name": "vta_dispatch_seconds",
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

The request carries nothing but its envelope. The response carries named counters, gauges and histograms describing the VTA's own operation — request volumes, queue depths, latency distributions — and their labels. A producer **MUST NOT** put a subject identifier, a context DID, or any other personal or secret value into a metric's `name` or a label's value; a label distinguishes an operational dimension (a task name, a transport, a context), not a person or a specific hosted identity.

### Correlation

An administrator reading metrics is identified to a VTA it already administers; both parties declare `identifierScope: public` for the reason [`did-management/server/metrics`](../../../../did-management/server/metrics/0.1/spec.md) does — the administrator's DID is what the VTA's access-control entry is keyed on, and the VTA's DID is the recipient every administrator addresses. A label that names a transport or a task is an operational dimension shared by many requests and many subjects, not a correlation handle for any one of them.

### Retention

Nothing about the request needs keeping. The response is a point-in-time reading; a metrics collector timestamps and stores it on its own retention policy, which this specification does not constrain.

### Consent/purpose

The task exists so an administrator's metrics stack can read the VTA's operational state without an unauthenticated network endpoint. It **MUST NOT** be used as a substitute for a plain, unauthenticated liveness probe — that probe exists for a different audience, an orchestrator that needs "is it up", not "how is it performing" — and stays outside the Trust Task surface for that reason: it carries no detail to protect, so requiring a proof would only cost every orchestrator a key it has no other use for.
