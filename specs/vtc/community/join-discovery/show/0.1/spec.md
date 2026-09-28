---
slug: vtc/community/join-discovery/show
version: "0.1"
title: "VTC Community — Join Discovery — Show"
summary: Read whether the community answers its join manifest to a caller it cannot identify.
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
    Discloses community configuration — whether the community answers its join manifest to a caller it cannot identify. No personal data about any member or applicant.
retention:
  class: transient
  rationale: >-
    A read. The consumer keeps nothing, and the caller's copy is stale at the next update.
errorCodes: []
related:
  - vtc/community/join-discovery/update
  - vtc/join-requests/manifest
---

## Abstract

The **VTC Community — Join Discovery — Show** Trust Task returns whether the community answers [`vtc/join-requests/manifest`](../../../../join-requests/manifest/0.2/spec.md) — *what do you require of people who join?* — to a caller it cannot identify.

`public: true` is the default and what every community did before the setting existed: the manifest is a public read by design, because an applicant has to know what is asked of them before they disclose anything. `public: false` does not make the manifest secret — an identified caller is answered either way — it makes every answer attributable, which is what a closed or invite-only community wants rather than a crawler enumerating its criteria.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST NOT** answer a caller it cannot identify, and refuses it with `permissionDenied`.
2. **MUST** return the stored setting, and **MUST** return `public: true` when none has been stored.

## Definitions

- **`joinDiscovery.public`** — whether the join manifest is answered to an unidentified caller.

## Request

A member or administrator (`issuer`) sends an empty payload to the community (`recipient`).

### A console loads the setting

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/community/join-discovery/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### A closed community

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/community/join-discovery/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "joinDiscovery": {
      "public": false
    }
  }
}
```

## Security & Privacy

### Data carried

Community configuration only. One boolean.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. A read reveals to the community that an identified caller looked at its configuration, which it learns from any authenticated request.

### Retention

Nothing is kept by the community. A caller's copy is stale at the next update and **SHOULD NOT** be cached as authoritative: vtc/join-requests/manifest is what an applicant relies on.

### Consent/purpose

The purpose is to let an administration surface display the setting it edits, and a member see what the community publishes. It discloses nothing that the join manifest does not already publish to applicants.
