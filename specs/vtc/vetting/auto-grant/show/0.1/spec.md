---
slug: vtc/vetting/auto-grant/show
version: "0.1"
title: "VTC Vetting — Auto-Grant — Show"
summary: An administrator reads whether the community names vetters automatically, how often the sweep runs, how long its grants last, and what the last sweep did.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
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
  requirement: RECOMMENDED
  rationale: >-
    Authority is the signer's administrator standing, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: >-
    Reads the sweep configuration and its last outcome. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses community configuration and aggregate counts; no member is named.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes: []
related:
  - vtc/vetting/auto-grant/update
  - vtc/vetting/vetters/grants/list
---

## Abstract

A community can name vetters automatically. A periodic **sweep** evaluates every member against the community's vetter-eligibility policy, grants the vetter role to members it allows who do not hold it, and revokes the automatic grants of members it no longer allows — never an administrator's manual grant.

The **VTC Vetting — Auto-Grant — Show** Trust Task returns the sweep's effective configuration — whether it runs, its interval, the validity of the grants it issues, with defaults filled in — and what the last sweep did. [`vtc/vetting/auto-grant/update`](../../update/0.1/spec.md) changes the configuration.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** report the effective configuration: `enabled: false`, `sweepMinutes: 60` and one year of validity when none has been stored — the sweep is off unless an administrator turns it on.
3. **MUST** include `lastSweep` once a sweep has run, and omit it before.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **Sweep** — one run of the automatic grant.
- **`autoGrant`** — an `AutoGrantStatus` from [`vtc/_shared/0.1/vetting-auto-grant.schema.json`](../../../../_shared/0.1/vetting-auto-grant.schema.json).

## Request

A community administrator (`issuer`) sends an empty payload to the community (`recipient`).

### Read the configuration

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/auto-grant/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Enabled, hourly, and the last sweep granted two

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/auto-grant/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "autoGrant": {
      "enabled": true,
      "sweepMinutes": 60,
      "validitySeconds": 31536000,
      "lastSweep": {
        "ranAt": "2026-09-28T09:00:00Z",
        "granted": 2,
        "revoked": 0,
        "errors": 0
      }
    }
  }
}
```

## Security & Privacy

### Data carried

Configuration and three counts. No member is named: the sweep's individual grants are listed by vtc/vetting/vetters/grants/list.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered.

### Retention

A read. The caller's copy is stale at the next sweep.

### Consent/purpose

The purpose is to let a community name vetters automatically, on its own vetter-eligibility policy, instead of one at a time. Whether a member is eligible is decided by that policy on each sweep, not by this configuration.
