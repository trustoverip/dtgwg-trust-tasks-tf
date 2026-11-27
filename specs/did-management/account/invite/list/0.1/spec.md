---
slug: did-management/account/invite/list
version: "0.1"
title: "DID Management — Account Invite List"
summary: "An administrator of a DID hosting service lists the account invites on the domains they administer — who issued each, its state, and who redeemed it — never their secrets."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, invite, list, audit]
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
    The response says which accounts were admitted on whose authority; only an administrator of the domain may read
    that, and the proof is how the service knows who is asking.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no effect; placing the request in time lets a consumer refuse a stale replay rather than answer it.
sideEffects:
  level: none
  rationale: >-
    Reads invite summaries; nothing is written.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns invite summaries: their domain, state, timestamps, issuing administrator and redeeming DID. Never a token or
    a claim code.
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
  - did-management/account/invite/issue
  - did-management/account/invite/revoke
---

## Abstract

**Account Invite List** is the administrator's view of the invites issued for their domains: which are still pending, which were redeemed and by whom, which were revoked or lapsed. It is the audit trail of who admitted which self-registered account.

**Why there is no `show`.** The registry's default is a list paired with a read-one task ([CONTRIBUTING-SPECS, read-one and read-many](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)). This family does not need one: an invite's summary is small and fully contained in the list, and the one operation where an invite's existence is load-bearing — revoking it — is did-management/account/invite/revoke, which answers a definite `unknownInvite` for an identifier it does not hold.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** return only invites for domains the issuer administers; a `domain` they do not administer is `permissionDenied`.
2. **MUST NOT** return any invite's token or claim code, or their hashes.
3. **MUST** report an unredeemed invite past its `expiresAt` as `expired`, whether or not it has removed it yet.
4. **MUST NOT** accept a `cursor` it did not issue. When `limit` is absent it returns at most 100 invites.

## Authorization

The entitlement is **administrative authority over the domains listed**, as the service's access-control list records it. The proof establishes who is asking; the consumer limits the answer to the domains that party administers.

## Definitions

* **Invite.** An [AccountInvite](../../../../_shared/0.4/hosting-account.schema.json) summary.
* `domain`, `state`, `cursor`, `limit` — as their schema descriptions say.

## Request

An administrator sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The pending invites on one domain

```json
{
  "id": "urn:uuid:c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f",
  "type": "https://trusttasks.org/spec/did-management/account/invite/list/0.1",
  "issuer": "did:webvh:QmAdminScid:admin.example.com",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-02T10:00:00Z",
  "payload": { "domain": "did.example.com", "state": "redeemed", "limit": 20 },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-02T10:00:00Z",
    "verificationMethod": "did:webvh:QmAdminScid:admin.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z5SqPoX3xdRbW"
  }
}
```

## Response

The service answers with one page of invite summaries, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). `nextCursor` is present when more remain. Failures use `trust-task-error`, not a `#response` document.

### One redeemed invite

```json
{
  "id": "urn:uuid:d4e5f6a7-b8c9-4d0e-1f2a-3b4c5d6e7f80",
  "type": "https://trusttasks.org/spec/did-management/account/invite/list/0.1#response",
  "threadId": "urn:uuid:c3d4e5f6-a7b8-4c9d-0e1f-2a3b4c5d6e7f",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:webvh:QmAdminScid:admin.example.com",
  "issuedAt": "2026-12-02T10:00:01Z",
  "payload": {
    "invites": [
      {
        "inviteId": "inv-01J9Z8Y7X6",
        "domain": "did.example.com",
        "label": "openvtc pilot — user 7",
        "state": "redeemed",
        "createdAt": "2026-11-30T10:00:01Z",
        "createdBy": "did:webvh:QmAdminScid:admin.example.com",
        "expiresAt": "2026-12-07T10:00:01Z",
        "redeemedBy": "did:scid:vh:1:QmScidExample",
        "redeemedAt": "2026-12-01T09:00:01Z"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-02T10:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z6TqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The response carries invite summaries: domains, states, timestamps, the issuing administrator's DID and the redeeming account's DID. It never carries a secret. Labels are whatever administrators wrote; see did-management/account/invite/issue on keeping them minimal.

### Correlation

The response links each self-registered account to the administrator who admitted it, which is its purpose. It is disclosed only to administrators of the same domain.

### Retention

Nothing is written. An administrator who keeps a response holds a copy of the domain's admission record and **SHOULD** treat it as such.

### Consent/purpose

The list exists for administering and auditing admissions. It **MUST NOT** be used to contact invitees by any means the invite did not already establish.
