---
slug: git-ns/account/link-status
version: "0.1"
title: "Git Namespaces — Forge Account Link Status"
summary: "A member asks where a forge account link they began stands: pending, linked, expired or failed."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - account
  - oauth
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: "A poll. It changes nothing and answers only the member who began the link, whom the VTC identifies from the transport or the proof."
sideEffects:
  level: none
  rationale: "Reads the state of a link attempt and changes nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns the state and, once linked, the member's own forge account id and login."
retention:
  class: exchange
  rationale: "The request is needed only to answer it."
errorCodes:
  - code: git-ns/account/link-status:unknownLink
    meaning: "No link with this identifier was begun by the caller, or the VTC no longer remembers it."
    retryable: false
related:
  - git-ns/account/link
---

## Abstract

A member asks where a forge account link begun with [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md) stands. Clients poll it while the member authorises on the forge.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The entitlement is **having begun the link**: a VTC answers only the member who sent the [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md) request that returned `linkId`. Anyone else is refused with `git-ns/account/link-status:unknownLink`, exactly as if the identifier did not exist, so that a guessed identifier reveals nothing.

## Definitions

**`state`** — `pending`, `linked`, `expired` or `failed`, as in the schema. `linked`, `expired` and `failed` are final.

**`account`** — the linked account, when `linked`.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A client **SHOULD** poll no faster than every five seconds, and stop at a final state or at the link's `expiresAt`. A VTC **MAY** forget a link some time after it reaches a final state, and then answers `unknownLink`.

### Bob polls

```json
{
  "id": "urn:uuid:93e860d1-c1e5-4c80-a687-5bb01367eb01",
  "type": "https://trusttasks.org/spec/git-ns/account/link-status/0.1",
  "threadId": "urn:uuid:93e860d1-c1e5-4c80-a687-5bb01367eb01",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:02:10Z",
  "payload": {
    "linkId": "lnk_4Tq9Xw2P"
  }
}
```

## Response

The VTC, now responding, returns the state, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Linked

```json
{
  "id": "urn:uuid:93e860d1-c1e5-4c80-a687-5bb01367eb02",
  "type": "https://trusttasks.org/spec/git-ns/account/link-status/0.1#response",
  "threadId": "urn:uuid:93e860d1-c1e5-4c80-a687-5bb01367eb01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-23T10:02:10Z",
  "payload": {
    "state": "linked",
    "account": {
      "forge": "github.com",
      "id": "9120045",
      "login": "bob-builds"
    }
  }
}
```

## Security & Privacy

### Data carried

A link identifier in; a state and, once linked, the member's own forge account out.

### Correlation

The VTC declares `identifierScope: public`; the member `pairwise`. The answer discloses the forge account only to the member who linked it.

### Retention

The request is needed only to answer it. The link record behind it is kept until it is final and then for as long as the VTC chooses to answer for it.

### Consent/purpose

The purpose is to tell the member whether their link completed. Nothing in the exchange may be used for anything else.
