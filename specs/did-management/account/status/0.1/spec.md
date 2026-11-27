---
slug: did-management/account/status
version: "0.1"
title: "DID Management — Account Status"
summary: "An account holder reads its own account on a DID hosting service — its state, quotas, and when it would be disabled or purged — without changing anything."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, status, expiry, inactivity]
parties:
  - role: Account holder
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The response describes one party's account, including when it lapses; only the holder's signature, checked against
    its DID document, shows the request comes from that party rather than from someone probing whether a DID's hosting
    is about to lapse.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no effect, but its answer is time-sensitive; placing the request in time lets a consumer refuse a stale
    replay rather than answer it.
sideEffects:
  level: none
  rationale: >-
    Reads the issuer's account. It does not count as a liveness signal: a service MUST NOT restart an account's
    inactivity period because its holder asked when it ends.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns the holder's own account — role, state, quotas and lifecycle instants — to the holder.
retention:
  class: transient
  rationale: >-
    Nothing is written, and the request carries nothing to keep.
errorCodes:
  - code: did-management/account/status:noAccount
    meaning: The issuer holds no account on this service, or held one that has been purged.
    retryable: false
related:
  - did-management/account/renew
  - did-management/account/register
  - did-management/account/expiry-notice
---

## Abstract

An agent that was offline may have missed its hosting service's expiry notices (did-management/account/expiry-notice). **Account Status** lets it ask, when it comes back, where its account stands: `active`, `warned`, or `disabled` — with `disableAt` or `purgeAt` saying how long it has. It answers for any account holder, not only self-registered ones; an account that is not subject to expiry has no `disableAt`.

Status is a read. It does not keep the account alive: the holder renews with did-management/account/renew. Keeping the two apart lets an agent check without committing to anything, and lets the service's liveness signal mean what it says.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** answer with the account whose subject is the identity `issuer` names (a `did:scid` without its `src`), and `noAccount` when there is none.
2. **MUST NOT** treat the request as a liveness signal.
3. **MUST** answer for a `disabled` account: its holder still authenticates (its DID document is the one the service holds), and needs `purgeAt` most of all.

## Authorization

The entitlement is **holding the account**. The consumer establishes it by verifying the document's proof against the issuer's DID document, for `authentication`, and comparing the issuer's identity with the account's `subject`. No party may read another's account through this task.

## Definitions

* **Account holder.** The DID an account was registered to (`subject`).

## Request

The holder sends the request to its hosting service; the payload, which validates against the top-level schema in [`payload.schema.json`](payload.schema.json), is empty.

### An agent checks its account at startup

```json
{
  "id": "urn:uuid:6c5b4a3f-2e1d-4c0b-9a8f-7e6d5c4b3a2f",
  "type": "https://trusttasks.org/spec/did-management/account/status/0.1",
  "issuer": "did:scid:vh:1:QmScidExample",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2027-03-20T07:30:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-20T07:30:00Z",
    "verificationMethod": "did:scid:vh:1:QmScidExample#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z8LqPoX3xdRbW"
  }
}
```

## Response

The service answers with the issuer's account, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The account was disabled while the agent was offline

```json
{
  "id": "urn:uuid:7d6c5b4a-3f2e-4d1c-0b9a-8f7e6d5c4b3a",
  "type": "https://trusttasks.org/spec/did-management/account/status/0.1#response",
  "threadId": "urn:uuid:6c5b4a3f-2e1d-4c0b-9a8f-7e6d5c4b3a2f",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:scid:vh:1:QmScidExample",
  "issuedAt": "2027-03-20T07:30:01Z",
  "payload": {
    "account": {
      "subject": "did:scid:vh:1:QmScidExample",
      "role": "guest",
      "domain": "did.example.com",
      "state": "disabled",
      "admission": "invite",
      "registeredAt": "2026-12-01T09:00:01Z",
      "lastActiveAt": "2026-12-15T18:00:00Z",
      "purgeAt": "2027-04-14T18:00:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-20T07:30:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z9MqPoX3xdRbW"
  }
}
```

The agent renews (did-management/account/renew) to restore it.

## Security & Privacy

### Data carried

The request carries nothing beyond the envelope; the response carries the holder's own account.

### Correlation

`issuer` is the holder's public DID (`identifierScope: public`), the identity the account belongs to; a pairwise identifier would name no account. Beyond that the task adds nothing to what the service already holds.

### Retention

Nothing is written. The service need not keep the document.

### Consent/purpose

The response exists so the holder can act before its account lapses. It is not shown to anyone else.
