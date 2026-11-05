---
slug: did-management/stats/set-resolution
version: "0.1"
title: DID Management — Stats Set Resolution
summary: A DID owner chooses less statistical detail for a slot they own, down to totals only, or an administrator chooses any level for any slot, including always keeping the finest series.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, stats, time-series, resolution, retention, observability]
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
  rationale: "The hosting service authorises on the document's proven issuer: the owner may only lower a slot's detail, and only an administrator may raise it to `fine`. The proof is what ties the request to one or the other on every transport, and it leaves the service a signed record of who changed what it keeps about a holder."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The resolution is set rather than advanced, so a stale copy replayed later moves the slot back to a level its owner or an administrator has since left: an owner's `off` undone by an old `auto`, or an old `fine` reinstated after an administrator withdrew it."
sideEffects:
  level: mutating
  rationale: "Changes what the hosting service records about the slot from now on. Reversible with another request, except that `off` and `coarse` may let the service discard detail it already held, which no later request brings back."
subjectPath: /mnemonic
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response says which level of detail the service keeps for the slot and why. `tierReason: traffic` with `tier: fine` tells the caller the slot is busy by the service's measure, which is activity metadata about its holder, disclosed only to its owner or an administrator."
retention:
  class: durable
  rationale: "The resolution persists until changed, and the service keeps who set it, so an owner who turned statistics off can show they did, and an operator can account for a slot that keeps the finest series outside the service's automatic policy."
errorCodes:
  - code: did-management/stats/set-resolution:notFound
    meaning: "No slot the caller may manage has this `mnemonic`. Returned alike for a slot that does not exist and for one the caller does not own, so the code cannot be used to learn which slots exist."
    retryable: false
  - code: did-management:unknownDomain
    meaning: "The submitted `domain` is not a known hosting domain on this consumer. See [category conventions](../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error)."
    retryable: false
related:
  - did-management/stats/get
  - did-management/stats/timeseries
  - did-management/did/info
---

## Abstract

A hosting service counts how often each DID it hosts is resolved and updated, and keeps those counts over time as a series that [`did-management/stats/timeseries`](../../timeseries/0.2/spec.md) reads. Keeping a fine series for every slot costs the service storage and writes in proportion to the slots it hosts, most of which are rarely resolved. So a service typically keeps a coarse series for every slot and a fine one only for its busy slots, choosing them by traffic.

**Stats Set Resolution** overrides that choice for one slot:

- An **owner** can keep less about their own DID: `coarse` (never the fine series) or `off` (totals only, no series), or return it to `auto`, the service's own choice.
- An **administrator** can do the same for any slot, and can also set `fine`, keeping the fine series regardless of traffic. Only an administrator can, because it spends the operator's capacity outside the operator's own policy.

The bucket widths are the service's to choose; this task names levels, not widths. A caller learns the width a level gives from the `bucketSeconds` of a `stats/timeseries` response.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the [category conventions](../../../_shared/0.1/CONVENTIONS.md). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Resolves the slot per the conventions (`domain` disambiguates). If no such slot exists, or the caller is neither its owner nor an administrator, answers `did-management/stats/set-resolution:notFound`; the two are indistinguishable.
3. Answers `permissionDenied` when `resolution` is `fine` and the caller is not an administrator, and changes nothing.
4. Otherwise records the resolution for the slot, replacing any earlier one; `auto` clears it. A request for the resolution already set succeeds with `changed: false`.
5. From then on keeps, for the slot:
   - under `fine`, its finest series, whatever the slot's traffic, and outside any cap the service puts on how many slots keep it;
   - under `coarse`, no finest series;
   - under `off`, no series at all. Its totals, which [`did-management/stats/get`](../../get/0.1/spec.md) reads, are still kept;
   - under `auto`, whatever the service's own policy chooses.
6. **MAY** discard series the new resolution no longer keeps. A `stats/timeseries` read of a slot under `off` returns the window's buckets with no activity counted, because nothing is recorded; `tier` in this task's response is how a caller tells "not recorded" from "not resolved".
7. **SHOULD** apply the resolution to everything it records for the slot from the response onward. A service of several replicas **MAY** take longer to apply it everywhere, and **SHOULD** say how long in its own documentation.
8. Deletes the resolution with the slot when the slot is deleted. A later slot with the same `mnemonic` starts at `auto`.

A service that keeps only one level of series for every slot conforms by recording the resolution, keeping no series under `off`, and answering `tier` and `tierReason` truthfully: `fine` then never applies beyond what its one level is.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Consequential: `sideEffects.level` is `mutating`.

The entitlement is **ownership of the slot** (the slot's recorded owner is the proven issuer) for `auto`, `coarse` and `off`, and **administrator standing on the hosting service** for every value, `fine` included. Both are read from the service's own records at execution time.

The asymmetry is deliberate. Lowering a slot's detail only reduces what the service records about the owner's own DID, so the owner needs no one else's leave. Raising it to `fine` spends the operator's storage and write capacity outside the operator's automatic policy and its cap on how many slots keep the finest series, so it is the operator's decision. An owner who wants finer detail for their slot asks the operator.

The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). An administrator's `fine` on a slot does not stop its owner from later setting `off` or `coarse`. The latest request from an entitled caller holds.

## Definitions

**Series.** The slot's resolve and update counts grouped into buckets of a fixed width over a window ending now, as `stats/timeseries` returns them. A service keeps one or more series per slot, at different bucket widths and over different windows.

**Finest series.** The series with the narrowest buckets the service keeps for any slot. Which width that is, and how long it is kept, is the service's choice.

**Resolution.** The level of detail requested for a slot: `auto`, `fine`, `coarse` or `off`. **Tier.** The level the service actually keeps: `fine`, `coarse` or `off`. Under `auto` the tier is the service's choice; under any other resolution it equals the resolution.

| Member | Meaning |
|---|---|
| `mnemonic` | The slot. Required: the server-wide series is not settable by this task. |
| `domain` | Disambiguates `mnemonic`, per the category conventions. |
| `resolution` | The level requested. |

## Request

The owner or an administrator sends the request to the hosting service; the top-level schema in [`payload.schema.json`](payload.schema.json) describes the payload.

### An owner turns off statistics over time for their DID

```json
{
  "id": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c01",
  "type": "https://trusttasks.org/spec/did-management/stats/set-resolution/0.1",
  "issuer": "did:webvh:QmVtaScid5:vta.example.com",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-11-05T09:00:00Z",
  "payload": { "mnemonic": "alice", "resolution": "off" }
}
```

### An administrator keeps the finest series for a slot

```json
{
  "id": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c11",
  "type": "https://trusttasks.org/spec/did-management/stats/set-resolution/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:ops",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-11-05T09:05:00Z",
  "payload": { "mnemonic": "issuer", "domain": "did.example.com", "resolution": "fine" }
}
```

## Response

The hosting service answers with the payload under `$defs.Response` (`$anchor: "response"`) in [`payload.schema.json`](payload.schema.json):

- `mnemonic`: the slot.
- `resolution` and `previousResolution`: the level now set and the one it replaced, `auto` when none was.
- `changed`: whether they differ.
- `tier`: the level the service keeps for the slot as of the response.
- `tierReason`: why that tier applies:
  - `override`: this slot's resolution;
  - `policy`: a service-wide setting that applies to every slot alike;
  - `traffic`: the service's automatic choice from the slot's activity.

Failures use `trust-task-error`, not a `#response` document.

### The owner's `off` is recorded

```json
{
  "id": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c02",
  "type": "https://trusttasks.org/spec/did-management/stats/set-resolution/0.1#response",
  "threadId": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-11-05T09:00:01Z",
  "payload": {
    "mnemonic": "alice",
    "resolution": "off",
    "previousResolution": "auto",
    "changed": true,
    "tier": "off",
    "tierReason": "override"
  }
}
```

### The owner returns the slot to `auto`, and traffic decides

```json
{
  "id": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c04",
  "type": "https://trusttasks.org/spec/did-management/stats/set-resolution/0.1#response",
  "threadId": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c03",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-11-12T10:00:01Z",
  "payload": {
    "mnemonic": "alice",
    "resolution": "auto",
    "previousResolution": "off",
    "changed": true,
    "tier": "coarse",
    "tierReason": "traffic"
  }
}
```

### An owner asks for `fine`

The owner may not raise their slot's detail, so the service changes nothing:

```json
{
  "id": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c06",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:4b1e9a3c-2d7f-4e0a-9c1b-6f2e8d3a5c05",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmVtaScid5:vta.example.com",
  "issuedAt": "2026-11-05T09:10:01Z",
  "payload": {
    "code": "permissionDenied",
    "message": "Only an administrator can set the fine resolution.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request names one slot and a level. The response names the level set, the one replaced, and the tier the service keeps with its reason. No counts are carried. The tier and its reason are not neutral, though: `fine` because of `traffic` says the slot is among the service's busiest, which is activity metadata about its holder. It is disclosed only to the slot's owner or an administrator, the same parties who may read the counts themselves with `stats/get`.

### Correlation

Nothing here identifies a resolver. Both parties declare `identifierScope: public`, and must: the owner's DID is the one the slot's record names as owner, and the hosting service's DID is the recipient the owner registered with. The task cannot be answered for a pairwise identifier the record does not hold. A change of tier from `coarse` to `fine` under `auto` tracks a change in the slot's traffic, so an owner who reads the tier repeatedly learns roughly when their DID became busy. That is information about their own DID, which they can already read in finer form with `stats/timeseries`.

### Retention

The resolution is kept until it is changed or the slot is deleted. Who set it **SHOULD** be kept with it, or in the service's audit log, so an owner's `off` and an administrator's `fine` can each be accounted for. The request document need not be kept beyond that record. A service that discards series under `off` or `coarse` reduces what it holds about the holder, which is the purpose an owner has for asking.

### Consent/purpose

The setting exists so an owner can limit what the service records over time about their own DID, and so an operator can spend capacity on the slots it chooses. Neither party may use the setting, or the tier it reports, for anything else: a service **MUST NOT** treat an owner's `off` as a signal about the holder, and **MUST NOT** disclose a slot's tier to anyone other than its owner or an administrator.
