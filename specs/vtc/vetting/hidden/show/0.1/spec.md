---
slug: vtc/vetting/hidden/show
version: "0.1"
title: "VTC Vetting — Hidden — Show"
summary: An administrator reads one criterion's stored hidden-vetting configuration, what it publishes, and how many vetters and event volunteers it has — counts only, never who.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - blind-signature
  - pcs
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The stored configuration names who approved each event — a member DID that the published manifest deliberately omits — so the read is for administrator standing only, and the signer must be attributable on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: >-
    Reads the criterion's stored hidden-vetting parameters and aggregate counts. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses community configuration, including the approver DID recorded on each event, and aggregate counts of enrolled vetters and event volunteers. No enrolled vetter or event volunteer is named.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/vetting/hidden/show:noSuchCriterion
    meaning: "`criterionId` does not name an Accepts criterion this community has stored."
    retryable: false
related:
  - vtc/vetting/hidden/publish
  - vtc/vetting/hidden/withdraw
  - vtc/vetting/vetters/pcs-root
  - vtc/vetting/vetters/event-mode
---

## Abstract

[`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) replaces a criterion's `events` wholesale, and what it publishes in the join manifest omits each event's `approvedBy` and `graceDays`. An administrator — or a console acting for one — that re-publishes from the manifest therefore drops every approval it could not see, and un-approves events that were live. **VTC Vetting — Hidden — Show** is the companion read publish/0.1 anticipates: it returns the configuration exactly as stored, beside what it publishes, so that a re-publish can start from the truth.

It also answers the two questions an administrator running hidden vetting has to keep asking — have enough vetters enrolled under the live labels, and has each event drawn enough volunteers to open — as counts, never as names.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **administrator** (`issuer`) names the criterion in `criterionId`.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline and **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** refuse with `vtc/vetting/hidden/show:noSuchCriterion` when `criterionId` names no stored criterion.
3. When the criterion carries no hidden-vetting parameters, **MUST** answer with success, `enabled: false`, the criterion's current `requirementsDigest`, and none of `stored`, `published`, `enrolledVetters` or `eventStatus`. A criterion without hidden vetting is an answer, not a refusal.
4. When it carries them, **MUST** answer `enabled: true` with:
   - `stored` — the parameters exactly as stored, in the shape [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) returns as `stored`, every event's `approvedBy` and `graceDays` included;
   - `published` — the parameters exactly as the join manifest now carries them under `vetting.ext`, in the shape publish returns as `published`;
   - in both, `tickLength` — the drip's tick length; a configuration stored before the member existed is answered as `P3D`, as publish/0.1 reads it;
   - `enrolledVetters` — for each live class label (`vetter/<period>`), the number of members holding an enrolment under it ([`vtc/vetting/vetters/pcs-root`](../../../vetters/pcs-root/0.1/spec.md));
   - `eventStatus` — one entry per stored event, in stored order: its `groupFloor`, its `groupSize` (members who have asked to vet at it through [`vtc/vetting/vetters/event-mode`](../../../vetters/event-mode/0.1/spec.md)), whether it is `approved`, and whether it is `live` — approved, `groupSize` at least `groupFloor`, and today no later than `endDate` plus `graceDays`.
5. **MUST NOT** disclose which members enrolled under a label or asked to vet at an event — not in this response, its `ext`, or any member a future minor version adds. The counts are the whole of what it discloses about them. `groupSize` is the count event-mode already returns to a vetter waiting on that event; this task extends the same count, and nothing more, to the administrator.

## Authorization

The entitlement is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. It is the standing [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) requires, because this read exists to inform a publish, and because the stored configuration names each event's approver.

## Definitions

**Stored configuration**, **published configuration** — as [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) defines them: what the community mints and verifies with, and what applicants and vetters read in the join manifest.

**Enrolled vetter** — a member holding an enrolment under a live class label, granted through [`vtc/vetting/vetters/pcs-root`](../../../vetters/pcs-root/0.1/spec.md). Counted per label, because enrolment is per label.

**Group size** — the number of members who have asked to vet at an event, as [`vtc/vetting/vetters/event-mode`](../../../vetters/event-mode/0.1/spec.md) counts it.

## Request

The administrator sends the request to the community; the payload is the top-level object in [`payload.schema.json`](payload.schema.json).

### Read one criterion

```json
{
  "id": "urn:uuid:5e6f7081-92a3-4a3c-8d4e-5f6a7b8c9d21",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/show/0.1",
  "issuer": "did:web:admin.kernel-vtc.example",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-10-05T09:20:00Z",
  "payload": {
    "criterionId": "vetted-member"
  }
}
```

## Response

The community answers in the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Hidden vetting on, one event open

```json
{
  "id": "urn:uuid:5e6f7081-92a3-4a3c-8d4e-5f6a7b8c9d22",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/show/0.1#response",
  "threadId": "urn:uuid:5e6f7081-92a3-4a3c-8d4e-5f6a7b8c9d21",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-10-05T09:20:01Z",
  "payload": {
    "criterionId": "vetted-member",
    "enabled": true,
    "stored": {
      "suite": "ps-ddh-bls12381",
      "hvk": "z6MkhVerificationKey1abcXYZ",
      "tvk": "z6MktVerificationKey1defXYZ",
      "livePeriods": ["2026-10"],
      "liveTokenLabels": ["token/2026-10"],
      "dripPerTick": 3,
      "tickLength": "P3D",
      "events": [
        {
          "eventId": "summit-2026",
          "startDate": "2026-10-01",
          "endDate": "2026-10-03",
          "graceDays": 14,
          "groupFloor": 5,
          "tiers": [{ "name": "desk", "dripPerTick": 5 }],
          "approvedBy": "did:web:approver.kernel-vtc.example"
        }
      ]
    },
    "published": {
      "suite": "ps-ddh-bls12381",
      "helperKey": "z6MkhVerificationKey1abcXYZ",
      "tokenKey": "z6MktVerificationKey1defXYZ",
      "vetterLabels": ["vetter/2026-10"],
      "tokenLabels": ["token/2026-10"],
      "dripPerTick": 3,
      "tickLength": "P3D",
      "events": [
        {
          "eventId": "summit-2026",
          "startDate": "2026-10-01",
          "endDate": "2026-10-03",
          "groupFloor": 5,
          "tiers": [{ "name": "desk", "dripPerTick": 5 }]
        }
      ]
    },
    "enrolledVetters": { "vetter/2026-10": 7 },
    "eventStatus": [
      {
        "eventId": "summit-2026",
        "groupFloor": 5,
        "groupSize": 6,
        "approved": true,
        "live": true
      }
    ],
    "requirementsDigest": "zQmRequirementsDigestCurrent"
  }
}
```

### Hidden vetting off

```json
{
  "id": "urn:uuid:5e6f7081-92a3-4a3c-8d4e-5f6a7b8c9d24",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/show/0.1#response",
  "threadId": "urn:uuid:5e6f7081-92a3-4a3c-8d4e-5f6a7b8c9d23",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-10-05T09:21:01Z",
  "payload": {
    "criterionId": "open-registration",
    "enabled": false,
    "requirementsDigest": "zQmRequirementsDigestOpen"
  }
}
```

## Security & Privacy

### Data carried

The request carries a criterion identifier. The response carries the community's own configuration — two public verification keys, labels, rates, event windows — plus, per event, the DID of the member who approved it, and aggregate counts. The approver DID is the reason this read is for administrators only: it is the community's record of its own decision, and the published manifest omits it on purpose. A community **MUST NOT** put an enrolled vetter's or event volunteer's identifier into `ext`.

### Correlation

The counts are the privacy-relevant part. A count of one is still a count; an administrator who watches `enrolledVetters` or `groupSize` move from one read to the next can infer that *someone* enrolled or volunteered in between, and with outside knowledge of who was online, perhaps who. That is the same inference event-mode's `groupSize` already offers every vetter waiting on the event, which is why this task discloses that count and nothing finer. A community **SHOULD NOT** add per-member detail, timestamps of enrolment, or any ordering a reader could align with other records.

The administrator declares `identifierScope: any` and the community `identifierScope: public`, for the reasons [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) gives.

### Retention

Transient. The community keeps nothing it did not already hold.

### Consent/purpose

The purpose is to let an administrator see what is in force before changing it, and whether hidden vetting is actually usable — enough vetters enrolled, events able to open. A console that re-publishes **SHOULD** read this first and send the stored events back with their `approvedBy`, which is what keeps a re-publish from un-approving them.

### Threats

*Re-publish from the manifest.* The failure this task exists to prevent: an administrator who builds a publish from `published` drops every `approvedBy` and silently closes approved events. Reading `stored` avoids it.

*Counting as enumeration.* Repeated reads timed against a member's actions could reveal that the member enrolled or volunteered. The counts are coarse by design, and a community **MAY** rate-limit this read as it does other administrative reads.
