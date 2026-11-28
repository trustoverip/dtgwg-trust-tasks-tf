---
slug: vta/hosting-account/get
version: "0.1"
title: "VTA Hosting Account — Get"
summary: "A VTA's principal reads the state of the account the VTA holds on the DID hosting service it self-registered with: whether it is active, warned or disabled, and what the host last said."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [vta, did-hosting, account, expiry, self-registration]
parties:
  - role: VTA principal
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: VTA
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer says whether the VTA's own DID is about to stop resolving, which is administrative state: the VTA authorises on the document's proven issuer, so the proof is what makes the principal the caller on every transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read whose answer is time-sensitive; an issue time lets the VTA refuse a stale replay rather than answer it.
sideEffects:
  level: none
  rationale: >-
    Reads what the VTA recorded. With `refresh`, the VTA also reads its account from the host with did-management/account/status, which by that specification counts as no activity and changes nothing at the host.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Returns the VTA's own hosting account — the host's DID, the domain, the account's lifecycle state and instants, and the last error reaching the host — to the VTA's principal.
retention:
  class: transient
  rationale: >-
    The request carries nothing but its envelope and a flag.
errorCodes:
  - code: vta/hosting-account/get:hostUnreachable
    meaning: "`refresh` was asked and the VTA could not read its account from the host. The details say why; the response the VTA would otherwise give is unchanged, and a request without `refresh` still answers from what it recorded."
    retryable: true
related:
  - did-management/account/register
  - did-management/account/renew
  - did-management/account/status
  - did-management/account/expiry-notice
---

## Abstract

A VTA that self-registered its DID with a third-party DID hosting service ([`did-management/account/register`](../../../../did-management/account/register/0.1/)) holds that DID under the domain's self-registration terms, which can include an inactivity lifecycle: an account with no activity is warned, then disabled — its DIDs stop resolving — and after a grace period purged. The VTA keeps the account alive itself, by renewing ([`did-management/account/renew`](../../../../did-management/account/renew/0.1/)), and it hears from the host by renewing and by expiry notices ([`did-management/account/expiry-notice`](../../../../did-management/account/expiry-notice/0.1/)).

**VTA Hosting Account — Get** is how the VTA's principal — the person or application operating it — finds out where that account stands, so a client can warn its user before the VTA's identity stops resolving, and say what to do when it has. The host speaks only to the VTA; this task carries what the VTA knows to the party that can act on it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A conforming VTA:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Answers `registered: false`, with no `hosting`, when it holds no self-registered hosting account.
4. Otherwise answers `registered: true` and a `hosting` record. Without `refresh` it answers from what it recorded and **MUST NOT** contact the host.
5. With `refresh`, reads its account from the host with `did-management/account/status` before answering, records the result, and answers with it. It **MUST NOT** renew in order to answer: a read by the principal is not activity of the VTA's, and renewing on every client refresh would make the host's liveness signal mean nothing. When the host cannot be read, it refuses with `hostUnreachable` and records the failure in `lastError`.
6. Reports `account` verbatim as the host last sent it, and accepts an expiry notice into `lastNotice` only from `serverDid`.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this VTA**, read from its own access-control records at execution time. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Hosting account** — the account a DID hosting service opened for this VTA when it self-registered. A VTA holds at most one; a DID hosted any other way (by the VTA itself, or on a host where an operator created the account) is not one.

**`refresh`** — `true` asks the VTA to read its account from the host now; absent or `false`, it answers from its record.

**`registered`** — whether the VTA holds a hosting account.

**`hosting`** — what the VTA knows about it:

- `serverDid` and `domain` name the host and the domain the account is on.
- `registeredAt` is when the host opened the account.
- `account` is the host's own record of it ([`did-management/_shared/0.4` HostingAccount](../../../../did-management/_shared/0.4/hosting-account.schema.json)) as the host last reported it: its `state` (`active`, `warned`, `disabled`), `disableAt` while the lifecycle applies, `purgeAt` once disabled, `exempt` when an operator lifted the terms. A client reads the lifecycle from here.
- `checkedAt` is when `account` was last heard from the host; `renewedAt` when the host last accepted a renewal.
- `lastNotice` is the latest expiry notice accepted from the host. A notice can be lost while the VTA is offline, and the host does not resend it, so a client **MUST** read the lifecycle from `account`, not from `lastNotice`.
- `lastError` is why the VTA's last attempt to reach the host failed, cleared by the next success.

## Request

The VTA's principal sends the request to the VTA, naming nothing but, optionally, `refresh`. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Ask the host first

```json
{
  "id": "urn:uuid:5d0f8e2a-7b3c-4c1d-9e4f-0a1b2c3d4e01",
  "type": "https://trusttasks.org/spec/vta/hosting-account/get/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:scid:vh:1:QmVta?src=did.example.com/agents/my-vta",
  "issuedAt": "2026-11-28T09:00:00Z",
  "payload": { "refresh": true }
}
```

## Response

The VTA answers with whether it holds a hosting account and, if it does, its record — the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error`.

### An account the host has warned about

```json
{
  "id": "urn:uuid:5d0f8e2a-7b3c-4c1d-9e4f-0a1b2c3d4e02",
  "type": "https://trusttasks.org/spec/vta/hosting-account/get/0.1#response",
  "issuer": "did:scid:vh:1:QmVta?src=did.example.com/agents/my-vta",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-28T09:00:01Z",
  "threadId": "urn:uuid:5d0f8e2a-7b3c-4c1d-9e4f-0a1b2c3d4e01",
  "payload": {
    "registered": true,
    "hosting": {
      "serverDid": "did:webvh:QmHost:did.example.com",
      "domain": "did.example.com",
      "registeredAt": "2026-08-01T10:00:00Z",
      "account": {
        "subject": "did:scid:vh:1:QmVta",
        "role": "owner",
        "domain": "did.example.com",
        "state": "warned",
        "admission": "invite",
        "registeredAt": "2026-08-01T10:00:00Z",
        "disableAt": "2026-12-01T10:00:00Z"
      },
      "checkedAt": "2026-11-28T09:00:01Z",
      "renewedAt": "2026-08-31T10:00:00Z",
      "lastNotice": { "stage": "disablePending", "receivedAt": "2026-11-17T10:00:00Z" }
    }
  }
}
```

### A VTA that never self-registered

```json
{
  "id": "urn:uuid:5d0f8e2a-7b3c-4c1d-9e4f-0a1b2c3d4e03",
  "type": "https://trusttasks.org/spec/vta/hosting-account/get/0.1#response",
  "issuer": "did:webvh:QmVta2:vta.example.com",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-28T09:00:01Z",
  "threadId": "urn:uuid:5d0f8e2a-7b3c-4c1d-9e4f-0a1b2c3d4e04",
  "payload": { "registered": false }
}
```

## Security & Privacy

### Data carried

The request carries a flag. The response carries the VTA's own hosting arrangement: the host's DID, the domain, the account's lifecycle state and dates, and an operator-readable error. Nothing in it is about a person or a credential. `lastError` **MUST NOT** carry a secret — not the invite token or claim code the VTA registered with, and not a bearer token from the transport.

### Correlation

Both parties declare a public identifier scope on purpose: the VTA is the agent its principal administers, addressed by the DID its own access-control records and its host know it by, and the principal is the identity those records grant administrator standing to. A pairwise identifier for either would name a party the VTA has no standing record for, so the request could not be authorised. The answer goes only to the VTA's own administrators. It links the VTA to its host and domain, which its DID already does publicly. A `refresh` makes the VTA contact the host at a time the principal chooses, which tells the host the VTA is online then; a client refreshing on every screen draw would tell the host when its user is looking, so a client **SHOULD** refresh on demand or rarely, and otherwise read the record.

### Retention

The VTA keeps one record per hosting account and replaces it as it hears from the host; nothing about the request is kept beyond its audit entry.

### Consent/purpose

Collected so the principal can keep the VTA's DID resolving — renew in time, or register again after a purge — and for nothing else.
