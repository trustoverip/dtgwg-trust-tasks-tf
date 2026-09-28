---
slug: vtc/vetting/auto-grant/update
version: "0.1"
title: "VTC Vetting — Auto-Grant — Update"
summary: An administrator turns the automatic vetter-grant sweep on or off and sets its interval and the validity of the grants it issues.
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
  requirement: REQUIRED
  rationale: >-
    Turning the sweep on lets the community issue, and revoke, vetter role credentials without a human deciding each one, under the community's name and on the signer's administrator standing — so the signer must be attributable on every transport, and the act attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the sweep configuration. The write itself grants nothing, but an enabled sweep then issues and revokes vetter role credentials on its own schedule, so this changes what the community will do unattended. Recoverable by another update; grants already issued keep their validity.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Configuration in, the effective configuration and last-sweep counts out.
retention:
  class: durable
  rationale: >-
    The configuration lives until the next update, and the change is audited against the administrator.
errorCodes: []
related:
  - vtc/vetting/auto-grant/show
  - vtc/vetting/vetters/grant
---

## Abstract

The **VTC Vetting — Auto-Grant — Update** Trust Task replaces the configuration of the community's automatic vetter-grant sweep (see [`vtc/vetting/auto-grant/show`](../../show/0.1/spec.md)). An absent `sweepMinutes` or `validitySeconds` takes its default — hourly, one year — rather than keeping the previous value: the request states the whole configuration.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** store the configuration with defaults filled in (`sweepMinutes: 60`, `validitySeconds` one year) and apply it from the next sweep.
3. **MUST** give a grant the sweep issues the stored `validitySeconds`, and **MUST NOT** change the validity of a grant already issued.
4. **MUST** audit the change — the administrator and the stored values — and **MUST** refuse with `unavailable` rather than make an unaudited change.
5. **MUST** answer with the effective configuration and the last sweep, the same shape vtc/vetting/auto-grant/show returns.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`enabled`** — whether the sweep runs.
- **`sweepMinutes`** — minutes between sweeps, 5–1440; 60 when absent.
- **`validitySeconds`** — validity of a grant the sweep issues, one day to two years; one year when absent.

## Request

A community administrator (`issuer`) sends the configuration to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Turn it on, hourly, one-year grants

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/vetting/auto-grant/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "enabled": true
  }
}
```

### Turn it off

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vtc/vetting/auto-grant/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "enabled": false
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the effective configuration and the last sweep. A refusal is a `trust-task-error`.

### Stored, defaults filled in

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/vtc/vetting/auto-grant/update/0.1#response",
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

The configuration lives until the next update; the audit record names the administrator and the values stored.

### Consent/purpose

The purpose is to let a community name vetters automatically, on its own vetter-eligibility policy, instead of one at a time. Whether a member is eligible is decided by that policy on each sweep, not by this configuration.
