---
slug: vtc/community/branding/show
version: "0.1"
title: "VTC Community — Branding — Show"
summary: Read how the community asks to be shown to a prospective applicant — a display name, an accent colour and a logo URL.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: member or administrator
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
    The community answers only a caller it can identify, and an authenticated transport identifies one. A proof is recommended so the read is attributable on every transport, relayed ones included. What it returns is configuration the community publishes to applicants anyway, so nothing here needs a stronger binding.
sideEffects:
  level: none
  rationale: >-
    Reads one configuration row of the community. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses community configuration — how the community asks to be shown to a prospective applicant — a display name, an accent colour and a logo URL. No personal data about any member or applicant.
retention:
  class: transient
  rationale: >-
    A read. The consumer keeps nothing, and the caller's copy is stale at the next update.
errorCodes: []
related:
  - vtc/community/branding/update
  - vtc/join-requests/manifest
---

## Abstract

The **VTC Community — Branding — Show** Trust Task returns the community's branding: the display name, accent colour and logo URL it asks a prospective applicant's client to show. It is exactly what [`vtc/join-requests/manifest/0.2`](../../../../join-requests/manifest/0.2/spec.md) publishes as `branding`, stored by [`vtc/community/branding/update`](../../update/0.1/spec.md).

Branding is presentation only: self-asserted, unverified, and never an identifier. `communityDid` identifies the community; `branding` only asks to be shown a certain way.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST NOT** answer a caller it cannot identify — one with neither a verified `proof` nor a transport-authenticated sender — and refuses it with `permissionDenied`. Any identified caller may read: a member and an administrator see what an applicant sees.
2. **MUST** return the stored branding, and **MUST** return `branding: {}` — not an error — when none has been set. An empty branding is the state of every community that never chose one, and it publishes no `branding` in the manifest.
3. **MUST** return what the manifest would publish: the same members, with `accentColor` in lower case.

## Definitions

- **`branding`** — a `CommunityBranding` from [`vtc/_shared/0.1/community-presentation.schema.json`](../../../../_shared/0.1/community-presentation.schema.json): optional `displayName`, `accentColor` (`#rrggbb`) and `logoUrl` (https).

## Request

A member or administrator (`issuer`) sends an empty payload to the community (`recipient`).

### A console loads the branding page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/community/branding/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The stored branding

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/community/branding/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "branding": {
      "displayName": "Riverside Makers",
      "accentColor": "#0a7f5c",
      "logoUrl": "https://riverside.example/logo.svg"
    }
  }
}
```

### A community that never set any

An empty object, not an error: the manifest publishes no `branding`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/community/branding/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:member",
  "issuedAt": "2026-09-28T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "branding": {}
  }
}
```

## Security & Privacy

### Data carried

Community configuration only. `displayName` is free text the community's administrators wrote, bounded at 128 characters and shown to applicants; `logoUrl` points at an image an applicant's client fetches, so a client **MUST** treat it as an untrusted image from wherever it points.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. A read reveals to the community that an identified caller looked at its configuration, which it learns from any authenticated request.

### Retention

Nothing is kept by the community. A caller's copy is stale at the next update and **SHOULD NOT** be cached as authoritative: vtc/join-requests/manifest is what an applicant relies on.

### Consent/purpose

The purpose is to let an administration surface display the setting it edits, and a member see what the community publishes. It discloses nothing that the join manifest does not already publish to applicants.
