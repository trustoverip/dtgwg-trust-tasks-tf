---
slug: did-management/account/invite/issue
version: "0.1"
title: "DID Management — Account Invite Issue"
summary: "An administrator of a DID hosting service issues a single-use invite — a token and a separately delivered claim code — that lets a party with no account register one."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, invite, onboarding, claim-code, self-registration]
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
    An invite admits a party the service has no other relationship with, so it carries the issuing administrator's
    authority; the administrator's signature is what ties each account admitted by invite back to someone accountable
    for it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Issuing creates a credential that admits an account. A replayed issue would mint a second invite the administrator
    never asked for — and the response, carrying its secrets, would go to whoever replayed it if the transport allowed.
sideEffects:
  level: mutating
  rationale: >-
    Creates a pending invite. Undone by did-management/account/invite/revoke, or by expiry.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The response carries the invite's token and claim code, which together admit an account; they are returned once
    and the service keeps only hashes of them.
retention:
  class: durable
  rationale: >-
    The invite's summary is kept until it is redeemed, revoked or expired, and afterwards as the record of who admitted
    which account.
errorCodes:
  - code: did-management/account/invite/issue:invitesNotAccepted
    meaning: The domain does not accept self-registration by invite.
    retryable: false
  - code: did-management/account/invite/issue:tooManyPending
    meaning: The domain already holds the most pending invites the service allows. Revoke unused ones, or wait for them to expire.
    retryable: false
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
  - did-management/account/register
  - did-management/account/invite/list
  - did-management/account/invite/revoke
---

## Abstract

A party that has no relationship with a DID hosting service cannot be given an account in advance: until it registers, the service does not know its DID, and it may not have minted one yet. **Account Invite Issue** gives an administrator something to hand it instead — an invite, bound to a domain and optionally a path prefix, that the party redeems with did-management/account/register when it is ready.

The invite has two halves. The **token** and the **claim code** are both needed to redeem it, and the administrator is expected to deliver them separately, so that intercepting one channel is not enough. The invite is single-use and expires.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** refuse with `permissionDenied` unless the issuer administers the target domain.
2. **MUST** refuse with `invitesNotAccepted` unless the domain accepts self-registration by invite.
3. **MUST** generate a token of at least 128 bits of entropy and a claim code, store only slow hashes of each, and **MUST NOT** log either.
4. **MUST** return the token and claim code in this response and never again: no other task in this family returns them.
5. **MAY** cap `ttlSeconds` lower than requested, and **MAY** bound the number of pending invites per domain (`tooManyPending`).

Issuing is a *creating something* operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): every request creates a new invite.

## Authorization

The entitlement is **administrative authority over the target domain** on this service, as the service's access-control list records it. The proof establishes who is asking; the consumer then checks that party's ACL entry. The invite carries that administrator's authority forward: every account admitted by it was admitted on their say-so, and `createdBy` records it.

## Definitions

* **Invite.** A single-use, expiring admission for one account on one domain, summarised by an [AccountInvite](../../../../_shared/0.4/hosting-account.schema.json).
* **Token, claim code.** The invite's two secrets; both are needed to redeem it.
* `domain`, `pathPrefix`, `label`, `ttlSeconds` — as their schema descriptions say.

## Request

An administrator sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An invite for one new agent, valid for a week

```json
{
  "id": "urn:uuid:a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
  "type": "https://trusttasks.org/spec/did-management/account/invite/issue/0.1",
  "issuer": "did:webvh:QmAdminScid:admin.example.com",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-11-30T10:00:00Z",
  "payload": {
    "domain": "did.example.com",
    "label": "openvtc pilot — user 7",
    "ttlSeconds": 604800
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-30T10:00:00Z",
    "verificationMethod": "did:webvh:QmAdminScid:admin.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z3QqPoX3xdRbW"
  }
}
```

## Response

The service answers with the invite's summary and its two secrets, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The invite is issued

```json
{
  "id": "urn:uuid:b2c3d4e5-f6a7-4b8c-9d0e-1f2a3b4c5d6e",
  "type": "https://trusttasks.org/spec/did-management/account/invite/issue/0.1#response",
  "threadId": "urn:uuid:a1b2c3d4-e5f6-4a7b-8c9d-0e1f2a3b4c5d",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:webvh:QmAdminScid:admin.example.com",
  "issuedAt": "2026-11-30T10:00:01Z",
  "payload": {
    "invite": {
      "inviteId": "inv-01J9Z8Y7X6",
      "domain": "did.example.com",
      "label": "openvtc pilot — user 7",
      "state": "pending",
      "createdAt": "2026-11-30T10:00:01Z",
      "createdBy": "did:webvh:QmAdminScid:admin.example.com",
      "expiresAt": "2026-12-07T10:00:01Z"
    },
    "token": "inv_4b1f9c2e8a7d6e5f",
    "claimCode": "K7QP-29XD"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-30T10:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z4RqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The response carries two secrets that together admit an account. The transport carrying it **MUST** be confidential, and the administrator **SHOULD** deliver the token and claim code to the invitee over different channels. `label` is operator-facing; it **SHOULD NOT** hold more about the invitee than the operator needs to recognise the invite later.

### Correlation

The invite carries no identifier of the invitee: the service learns who redeemed it only at registration, and records the link then (`redeemedBy`). An administrator's DID is recorded as `createdBy` on every invite they issue.

### Retention

The service keeps the invite's summary and the hashes of its secrets until it is redeemed, revoked or expires, and **SHOULD** keep the summary afterwards as the record of who admitted which account, for as long as that account exists. The secrets themselves are never kept.

### Consent/purpose

The invite exists to admit one account. The service **MUST NOT** use an invite's label or its redemption for any other purpose.
