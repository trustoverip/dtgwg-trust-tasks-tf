---
slug: did-management/account/policy/set
version: "0.1"
title: "DID Management — Account Policy Set"
summary: "An administrator sets one hosting domain's self-registration terms — open or closed, the authorising parties whose vouchers it trusts, where self-registered DIDs may sit, and their limits — taking effect at once."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, self-registration, policy, trusted-issuers, vouchers, administration]
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The terms decide who may take space on the service without an administrator's per-party approval, and which
    outside parties' word is enough to admit one. Changing them is an administrator's act, and the proof is what ties
    the change to the administrator accountable for it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A consequential change; a replayed one could reopen a domain an administrator has since closed, or restore trust
    in a party since removed. The duplicate-execution record of SPEC §7.2 item 11 is bounded by this timestamp.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the domain's terms. Existing accounts are not removed by any change: closing a domain stops new
    registrations, and removing a trusted party stops its future vouchers, not the accounts they admitted.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The request and the response carry the domain's terms, including the operator's labels for the parties it trusts.
retention:
  class: durable
  rationale: >-
    The terms are the service's standing admission policy; the record of who last changed them is kept with them.
errorCodes:
  - code: did-management/account/policy/set:invalidPolicy
    meaning: The terms are inconsistent — vouchers admitted with no trusted party, trusted parties with vouchers not admitted, the same party twice, or an open domain admitting no evidence at all. `reason` says which.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason: { type: string }
  - code: did-management:unknownDomain
    meaning: The submitted `domain` is not a known hosting domain on this consumer. See [the category conventions](../../../../_shared/0.1/CONVENTIONS.md#2-unknown-domain-error).
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        domain: { type: string }
        activeDomains:
          type: array
          items: { type: string }
related:
  - did-management/account/policy/list
  - did-management/account/register
  - did-management/account/invite/issue
  - did-management/server/info
---

## Abstract

**Account Policy Set** replaces one hosting domain's terms for admitting parties with no account (did-management/account/register), while the service runs. It is how an administrator opens a domain to self-registration, closes it again, and decides which **authorising parties** — trust communities, or any other party whose judgement the operator accepts — may admit a registrant by signing it a `DidHostingVoucher`.

The request carries the whole of the domain's terms, every member stated: a set never merges with what was there, so what the administrator sent is what applies. `enabled: false` closes the domain without discarding its terms.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** refuse with `permissionDenied` unless the issuer is an administrator of the domain.
2. **MUST** refuse with `invalidPolicy` terms in which `admissions` includes `voucher` and `trustedIssuers` is empty, or `trustedIssuers` is not empty and `admissions` does not include `voucher`, or a DID appears twice in `trustedIssuers` (compared as identities: a `did:scid` without its `src`), or `enabled` is `true` and `admissions` is empty.
3. **MUST** replace the domain's terms with exactly the request's, set `updatedAt` and `updatedBy` itself (ignoring any in the request), and apply them to every registration it admits after answering.
4. **MUST NOT** remove or disable any existing account because of a change of terms. A registration already admitted under the old terms completes under them.
5. **SHOULD** apply a change of limits (`maxDidCount`, `maxTotalSizeBytes`) to accounts registered afterwards; whether it re-applies them to existing accounts is its own policy, which it **SHOULD** document.

Setting is a *setting a state* operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): setting the terms already in force succeeds, with `changed: false`.

## Authorization

The entitlement is **administrative authority over the domain**, as the service's access-control list records it. The proof establishes who is asking; the consumer checks that party's ACL entry.

Trusting an authorising party delegates part of that authority: each voucher it signs admits one registrant without the administrator's further approval. The delegation is bounded by the terms — the domain, its path prefixes and limits, and the registrant's own proof — and is withdrawn by removing the party.

## Definitions

* **Terms.** A [SelfRegistrationPolicy](../../../../_shared/0.4/self-registration-policy.schema.json).
* **Authorising party.** A [TrustedIssuer](../../../../_shared/0.4/self-registration-policy.schema.json).

## Request

An administrator sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Trusting a community's vouchers on a domain

```json
{
  "id": "urn:uuid:c9d0e1f2-a3b4-4c5d-8e6f-7a8b9c0d1e2f",
  "type": "https://trusttasks.org/spec/did-management/account/policy/set/0.1",
  "issuer": "did:webvh:QmAdminScid:admin.example.com",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-04T11:00:00Z",
  "payload": {
    "policy": {
      "domain": "did.example.com",
      "enabled": true,
      "admissions": ["invite", "voucher"],
      "trustedIssuers": [
        { "did": "did:webvh:QmVtcScid:community.example", "label": "Example community" }
      ],
      "pathPrefixes": ["g"],
      "maxRegistrations": 1000,
      "maxDidCount": 3,
      "maxTotalSizeBytes": 262144,
      "inactivityDays": 90,
      "graceDays": 30,
      "maxPendingInvites": 1000
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-04T11:00:00Z",
    "verificationMethod": "did:webvh:QmAdminScid:admin.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z2YqPoX3xdRbW"
  }
}
```

## Response

The service answers with the terms the domain now applies and whether they changed, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The terms are applied

```json
{
  "id": "urn:uuid:d0e1f2a3-b4c5-4d6e-9f7a-8b9c0d1e2f3a",
  "type": "https://trusttasks.org/spec/did-management/account/policy/set/0.1#response",
  "threadId": "urn:uuid:c9d0e1f2-a3b4-4c5d-8e6f-7a8b9c0d1e2f",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:webvh:QmAdminScid:admin.example.com",
  "issuedAt": "2026-12-04T11:00:01Z",
  "payload": {
    "policy": {
      "domain": "did.example.com",
      "enabled": true,
      "admissions": ["invite", "voucher"],
      "trustedIssuers": [
        { "did": "did:webvh:QmVtcScid:community.example", "label": "Example community" }
      ],
      "pathPrefixes": ["g"],
      "maxRegistrations": 1000,
      "maxDidCount": 3,
      "maxTotalSizeBytes": 262144,
      "inactivityDays": 90,
      "graceDays": 30,
      "maxPendingInvites": 1000,
      "updatedAt": "2026-12-04T11:00:01Z",
      "updatedBy": "did:webvh:QmAdminScid:admin.example.com"
    },
    "changed": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-04T11:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z3ZqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The request and response carry the domain's terms, including the operator's labels for the parties it trusts. The labels **SHOULD NOT** hold more than the operator needs to recognise a party.

### Correlation

The terms name the parties the operator trusts; their DIDs become public through did-management/server/info. Trusting a party tells everyone that this service accepts its vouchers, which is the point.

### Retention

The service keeps the terms while they apply, with who last changed them. A record of earlier terms is the operator's audit choice; the service's audit log of each change is the RECOMMENDED one.

### Consent/purpose

The terms exist to decide admission. They carry no personal data about registrants.
