---
slug: did-management/account/policy/list
version: "0.1"
title: "DID Management — Account Policy List"
summary: "An administrator reads each hosting domain's self-registration terms: open or closed, the authorising parties whose vouchers it trusts, where self-registered DIDs may sit, and the limits they are held to."
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
    The response carries operator-only detail — the labels the operator gives the parties it trusts — so only an
    administrator may read it, and the proof is how the service knows who is asking.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no effect; placing the request in time lets a consumer refuse a stale replay rather than answer it.
sideEffects:
  level: none
  rationale: >-
    Reads the stored terms; nothing is written.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns each domain's terms, including the operator's labels for the parties it trusts, which
    did-management/server/info does not publish.
retention:
  class: transient
  rationale: >-
    Nothing is written, and the request carries nothing to keep.
errorCodes:
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
  - did-management/account/policy/set
  - did-management/account/register
  - did-management/server/info
---

## Abstract

Whether a hosting domain admits parties with no account (did-management/account/register), and on what terms, is the operator's decision, and one it changes while the service runs: opening a domain to a pilot, trusting a new community, closing it again. **Account Policy List** is how an administrator reads those terms; did-management/account/policy/set is how they change them.

The public part of the terms — what a registrant must know before it mints a DID — is also published, without the operator's labels, by did-management/server/info/0.2. This task is the administrator's full view, closed domains included.

**Why there is no `get`.** The collection is one entry per hosting domain, keyed by the domain name, and small; filtering by `domain` returns the one entry or none, and a domain without terms is simply closed, which is what an empty answer says.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** refuse with `permissionDenied` unless the issuer is an administrator of the domains it answers for.
2. **MUST** answer with the terms it applies, closed domains included, and **MUST NOT** answer with terms it does not apply.

## Authorization

The entitlement is **administrative authority over the service's hosting domains**, as its access-control list records it. The proof establishes who is asking; the consumer checks that party's ACL entry.

## Definitions

* **Terms.** A [SelfRegistrationPolicy](../../../../_shared/0.4/self-registration-policy.schema.json).
* **Authorising party.** A [TrustedIssuer](../../../../_shared/0.4/self-registration-policy.schema.json): a party whose `DidHostingVoucher` admits a registrant.

## Request

An administrator sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Every domain's terms

```json
{
  "id": "urn:uuid:a7b8c9d0-e1f2-4a3b-8c4d-5e6f7a8b9c0d",
  "type": "https://trusttasks.org/spec/did-management/account/policy/list/0.1",
  "issuer": "did:webvh:QmAdminScid:admin.example.com",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-04T10:00:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-04T10:00:00Z",
    "verificationMethod": "did:webvh:QmAdminScid:admin.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z9WqPoX3xdRbW"
  }
}
```

## Response

The service answers with every matching domain's terms, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### One open domain

```json
{
  "id": "urn:uuid:b8c9d0e1-f2a3-4b4c-9d5e-6f7a8b9c0d1e",
  "type": "https://trusttasks.org/spec/did-management/account/policy/list/0.1#response",
  "threadId": "urn:uuid:a7b8c9d0-e1f2-4a3b-8c4d-5e6f7a8b9c0d",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:webvh:QmAdminScid:admin.example.com",
  "issuedAt": "2026-12-04T10:00:01Z",
  "payload": {
    "policies": [
      {
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
        "updatedAt": "2026-12-01T08:00:00Z",
        "updatedBy": "did:webvh:QmAdminScid:admin.example.com"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-04T10:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z1XqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The response carries each domain's terms, including the operator's labels for the parties it trusts. The labels are operator-facing; an operator **SHOULD NOT** put in them anything about a party it would not show its other administrators.

### Correlation

The response names the authorising parties the operator trusts. They are public in did-management/server/info already; only their labels are added here.

### Retention

Nothing is written. An administrator who keeps a response holds a copy of the service's admission policy and **SHOULD** treat it as such.

### Consent/purpose

The response exists for administering admission. It carries no personal data about registrants.
