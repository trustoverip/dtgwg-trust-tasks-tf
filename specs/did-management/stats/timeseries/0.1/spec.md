---
slug: did-management/stats/timeseries
version: "0.1"
title: DID Management — Stats Timeseries
summary: A DID owner, a domain-scoped caller or an administrator reads resolve and update counts over a recent window, bucketed — for one slot, one domain or the whole hosting service.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, stats, timeseries, observability, read]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner, domain-scoped caller or administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A slot's activity over time is metadata about its holder, and a domain's is metadata about every holder on it. The hosting service authorises on the document's proven issuer, so the proof is what ties the read to a caller entitled to that scope on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits reads of activity metadata."
sideEffects:
  level: none
  rationale: "Reads buckets the hosting service already keeps."
subjectPath: /mnemonic
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns resolve and update counts per time bucket for the requested scope."
retention:
  class: transient
  rationale: "The request names a scope and a window and is needed only to answer it."
errorCodes:
  - code: did-management/stats/timeseries:notFound
    meaning: "No slot the caller may read has this `mnemonic`. Returned alike for a slot that does not exist and one the caller does not own."
    retryable: false
  - code: did-management:unknownDomain
    meaning: "The submitted `domain` is not a known hosting domain on this consumer, or — for a domain series — not one the caller is scoped to. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error)."
    retryable: false
related:
  - did-management/stats/get
  - did-management/me/domains
---

## Abstract

**Stats Timeseries** reads the same counters as [`did-management/stats/get`](../../get/0.1/spec.md), bucketed over a recent window: the chart on an owner's DID page, a tenant's domain page, or an operator's dashboard.

The scope is chosen by which members are present: `mnemonic` for one slot, `domain` alone for the sum over a domain's slots, neither for the whole service.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the [category conventions](../../../_shared/0.1/CONVENTIONS.md). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. **Slot series** (`mnemonic` present): resolves the slot per the conventions; answers `did-management/stats/timeseries:notFound` when it does not exist or the caller is neither its owner nor an administrator, alike.
3. **Domain series** (`domain` present, `mnemonic` absent): answers `did-management:unknownDomain` unless the domain is known and the caller is an administrator or the domain is within the caller's recorded domain scope (the set [`did-management/me/domains`](../../../me/domains/0.1/spec.md) returns) — a domain outside the caller's scope is answered exactly as an unknown one.
4. **Server-wide series** (both absent): refuses a caller without administrator standing with `permissionDenied`.
5. Returns one point per bucket covering the window that ends at the time of the request, oldest first, including empty buckets, with the bucket width it chose as `bucketSeconds`. The consumer chooses the width; it **SHOULD** keep a window to no more than a few hundred points.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement depends on the scope: **ownership of the slot** for a slot series, **a recorded domain scope that includes the domain** for a domain series, and **administrator standing** for the server-wide series; administrator standing suffices for all three. Each is read from the hosting service's own records at execution time. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Bucket** — a fixed-width interval, aligned to a multiple of its width, over which resolves and updates are summed.

## Request

```json
{
  "id": "urn:uuid:af5b7ecd-6c8a-4bde-8d7e-5a6b7c8d9e01",
  "type": "https://trusttasks.org/spec/did-management/stats/timeseries/0.1",
  "issuer": "did:webvh:QmVtaScid5:vta.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "range": "lastHour", "mnemonic": "alice" }
}
```

## Response

Abbreviated to three of the window's twelve buckets.

```json
{
  "id": "urn:uuid:af5b7ecd-6c8a-4bde-8d7e-5a6b7c8d9e02",
  "type": "https://trusttasks.org/spec/did-management/stats/timeseries/0.1#response",
  "threadId": "urn:uuid:af5b7ecd-6c8a-4bde-8d7e-5a6b7c8d9e01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "range": "lastHour",
    "bucketSeconds": 300,
    "points": [
      { "at": "2026-09-27T08:00:00Z", "resolves": 3, "updates": 0 },
      { "at": "2026-09-27T08:05:00Z", "resolves": 0, "updates": 0 },
      { "at": "2026-09-27T08:10:00Z", "resolves": 7, "updates": 1 }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request names a window and at most one slot or domain. The response carries counts per bucket. A slot's series shows when its holder is being looked up — finer-grained activity metadata than [`did-management/stats/get`](../../get/0.1/spec.md)'s totals — and a domain's series shows the same for every holder on it in aggregate.

### Correlation

A slot series at five-minute resolution could be matched against a known interaction. A consumer **MAY** widen the buckets it uses for slot series. The domain series sums over every slot, so it does not single out one holder, but it is disclosed only to callers scoped to the domain because a domain with one or two slots would. Both parties declare `identifierScope: public`, and must: the caller's DID is the one the slot record or the access-control entry names, and the hosting service's DID is the recipient the caller enrolled with.

### Retention

Nothing of the request needs keeping beyond the exchange. The buckets are the service's operational records; a consumer **SHOULD** keep no more history than its longest window.

### Consent/purpose

The series are disclosed so owners, tenants and operators can see how a DID or domain is being used. They **MUST NOT** be disclosed beyond the scopes above, or used to profile a holder's contacts.
