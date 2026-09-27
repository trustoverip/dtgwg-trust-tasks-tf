---
slug: did-management/stats/get
version: "0.1"
title: DID Management — Stats Get
summary: A DID owner reads the resolve and update counters of a slot they own, or an administrator reads those of any slot or the hosting service's server-wide aggregate.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, stats, counters, observability, read]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Owner or administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "How often a DID is resolved is activity metadata about its holder, disclosed only to the slot's owner or an administrator. The hosting service authorises on the document's proven issuer, so the proof is what ties the read to the owner on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits reads of activity metadata."
sideEffects:
  level: none
  rationale: "Reads counters the hosting service already keeps; reading them is not counted."
subjectPath: /mnemonic
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns resolve and update counts and the instants of the last of each — for one slot, or summed over the service."
retention:
  class: transient
  rationale: "The request names at most one slot and is needed only to answer it."
errorCodes:
  - code: did-management/stats/get:notFound
    meaning: "No slot the caller may read has this `mnemonic`. Returned alike for a slot that does not exist and for one the caller does not own, so the code cannot be used to learn which slots exist."
    retryable: false
  - code: did-management:unknownDomain
    meaning: "The submitted `domain` is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error)."
    retryable: false
related:
  - did-management/stats/timeseries
  - did-management/did/info
  - did-management/registry/list
  - did-management/server/stats-sync
---

## Abstract

A hosting service counts how often each DID it hosts is resolved and updated — the edges report their counts to the control plane with [`did-management/server/stats-sync`](../../../server/stats-sync/0.1/spec.md). **Stats Get** reads the totals: one slot's, for its owner's dashboard, or the whole service's, for an administrator's.

For the same counters over time, bucketed, see [`did-management/stats/timeseries`](../../timeseries/0.1/spec.md).

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the [category conventions](../../../_shared/0.1/CONVENTIONS.md). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. With `mnemonic` absent: refuses a caller without administrator standing with `permissionDenied`; otherwise returns the server-wide aggregate, with `totalDids`, and without `mnemonic`.
3. With `mnemonic` present: resolves the slot per the conventions (`domain` disambiguates). If no such slot exists, or the caller is neither its owner nor an administrator, answers `did-management/stats/get:notFound` — the two are indistinguishable. Otherwise returns the slot's counters with `mnemonic` echoed and without `totalDids`.
4. Changes nothing, and does not count this read as a resolve.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **ownership of the slot** (the slot's recorded owner is the proven issuer) or **administrator standing on the hosting service**, both read from the service's own records at execution time. The server-wide aggregate is an operator metric and needs administrator standing; owning slots on the service is not enough. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Resolve** — one public retrieval of the slot's DID resolution artifacts from any edge. **Update** — one accepted publication of a new log version.

## Request

```json
{
  "id": "urn:uuid:9e4a6dbc-5b7f-4acd-9c6d-4f5a6b7c8d01",
  "type": "https://trusttasks.org/spec/did-management/stats/get/0.1",
  "issuer": "did:webvh:QmVtaScid5:vta.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "mnemonic": "alice" }
}
```

## Response

```json
{
  "id": "urn:uuid:9e4a6dbc-5b7f-4acd-9c6d-4f5a6b7c8d02",
  "type": "https://trusttasks.org/spec/did-management/stats/get/0.1#response",
  "threadId": "urn:uuid:9e4a6dbc-5b7f-4acd-9c6d-4f5a6b7c8d01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "mnemonic": "alice",
    "totalResolves": 1287,
    "totalUpdates": 4,
    "lastResolvedAt": "2026-09-27T08:58:12Z",
    "lastUpdatedAt": "2026-09-20T14:02:00Z"
  }
}
```

The server-wide aggregate an administrator reads with an empty payload:

```json
{
  "id": "urn:uuid:9e4a6dbc-5b7f-4acd-9c6d-4f5a6b7c8d04",
  "type": "https://trusttasks.org/spec/did-management/stats/get/0.1#response",
  "threadId": "urn:uuid:9e4a6dbc-5b7f-4acd-9c6d-4f5a6b7c8d03",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "totalDids": 412, "totalResolves": 90211, "totalUpdates": 1733, "lastResolvedAt": "2026-09-27T09:00:00Z", "lastUpdatedAt": "2026-09-27T08:41:07Z" }
}
```

## Security & Privacy

### Data carried

The request names at most one slot. The response carries counts and two instants. A slot's resolve count and last-resolved time say how much, and how recently, the DID's holder is being looked up — activity metadata about a person or organisation, which is why the read is owner-or-administrator only.

### Correlation

The counters are aggregates; no resolver identity, address or request is kept or disclosed. `lastResolvedAt` at fine granularity could be matched against a known interaction ("they resolved me at 08:58"), so a consumer **MAY** coarsen it. Both parties declare `identifierScope: public`, and must: the owner's DID is the one the slot's record names as owner, and the hosting service's DID is the recipient the owner registered with — the task cannot be answered for a pairwise identifier the record does not hold.

### Retention

Nothing of the request needs keeping beyond the exchange. The counters themselves are the service's operational records and outlive it.

### Consent/purpose

The counters are disclosed so an owner can see their DID is being served and so an operator can size and monitor the service. They **MUST NOT** be disclosed to other owners, or used to profile a holder's contacts.
