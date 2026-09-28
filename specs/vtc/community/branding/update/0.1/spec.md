---
slug: vtc/community/branding/update
version: "0.1"
title: "VTC Community — Branding — Update"
summary: An administrator replaces how the community asks to be shown to a prospective applicant — a display name, an accent colour and a logo URL.
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
    The write changes what the community tells every prospective applicant, under the community's name, and is authorized only by the signer's administrator standing — so the signer has to be attributable on every transport, and the change has to be attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would silently put back a configuration an administrator has since replaced, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Replaces one configuration row. Recoverable by another update; nothing is issued, revoked or deleted beyond the row itself.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries community configuration an administrator wrote; the response echoes what was stored. No personal data.
retention:
  class: durable
  rationale: >-
    The stored value is the community's configuration until the next update, and the change is recorded in the audit trail against the administrator who made it.
errorCodes: []
related:
  - vtc/community/branding/show
  - vtc/join-requests/manifest
---

## Abstract

The **VTC Community — Branding — Update** Trust Task replaces the community's branding — the display name, accent colour and logo URL that [`vtc/join-requests/manifest/0.2`](../../../../join-requests/manifest/0.2/spec.md) publishes as `branding`. What an administrator stores here is exactly what the manifest publishes.

The update replaces the whole value: a member absent from the request is cleared. An empty branding removes it, and the manifest then publishes none.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller that does not hold administrator standing with `permissionDenied`, after resolving the proof's signer — or, for a signer that is a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the identity it acts for — to its access-control entry at execution time.
2. **MUST** refuse, with `malformedRequest`, a branding the manifest schema refuses — the check is the same one a manifest consumer applies, so nothing is stored that the manifest could not publish.
3. **MUST** store `accentColor` in lower case, since the colour is compared case-insensitively, and **MUST** treat an empty branding as removing it.
4. **MUST** record the change in its audit trail, naming the administrator and the members that changed, and **MUST** refuse with `unavailable` rather than make an unaudited change when it cannot write the audit record. An update that changes nothing is not audited.
5. **MUST** answer with what it stored.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`branding`** — the whole new `CommunityBranding`. Every member is optional; an absent member is cleared.

## Request

A community administrator (`issuer`) sends the whole new branding to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Set a name, colour and logo

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vtc/community/branding/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "branding": {
      "displayName": "Riverside Makers",
      "accentColor": "#0A7F5C",
      "logoUrl": "https://riverside.example/logo.svg"
    }
  }
}
```

### Clear the branding

An empty object removes it; the manifest publishes no `branding` afterwards.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000005",
  "type": "https://trusttasks.org/spec/vtc/community/branding/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "branding": {}
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: what it stored. A refusal is a `trust-task-error`.

### Stored, colour normalised

The accent colour was sent in upper case and is stored, and returned, in lower case.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000006",
  "type": "https://trusttasks.org/spec/vtc/community/branding/update/0.1#response",
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

## Security & Privacy

### Data carried

Community configuration an administrator wrote. `displayName` is free text, bounded at 128 characters, written by an administrator and shown to every prospective applicant as the community's own words; a client **MUST** attribute it to the community. `logoUrl` is fetched by applicants' clients — an administrator **SHOULD** host it where fetching it discloses nothing about the applicant beyond what loading the community's own site would.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The audit trail ties each change to the administrator who made it; that attribution is the point of requiring a proof.

### Retention

The stored value lives until the next update. The audit record of the change is durable and names the administrator and what changed, not the whole value.

### Consent/purpose

The purpose is to set how the community presents itself to, or what it asks of, prospective applicants. Using the value for anything else — for instance deciding an application on an attribute the community did not publish as requested — is outside it.
