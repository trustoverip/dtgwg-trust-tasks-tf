---
slug: did-management/account/invite/revoke
version: "0.1"
title: "DID Management — Account Invite Revoke"
summary: "An administrator of a DID hosting service withdraws a pending account invite so that it can no longer be redeemed."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, invite, revoke, revocation]
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
    Revoking an invite withdraws a party's admission; only an administrator of the invite's domain may do that, and
    the proof is how the service knows who is asking.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A consequential state change; the duplicate-execution record of SPEC §7.2 item 11 is bounded by this timestamp.
sideEffects:
  level: mutating
  rationale: >-
    Marks a pending invite revoked, so it can never be redeemed. Revocation cannot be undone: an administrator who
    changes their mind issues a new invite.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Returns the revoked invite's summary, never its secrets.
retention:
  class: durable
  rationale: >-
    The revoked invite's summary stays in the domain's admission record.
errorCodes:
  - code: did-management/account/invite/revoke:unknownInvite
    meaning: No invite with this identifier exists on a domain the issuer administers.
    retryable: false
  - code: did-management/account/invite/revoke:notPending
    meaning: The invite was already redeemed or has expired, so there is nothing to revoke. Revoking an invite does not affect an account it admitted.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        state: { type: string, enum: [redeemed, expired] }
related:
  - did-management/account/invite/issue
  - did-management/account/invite/list
---

## Abstract

An invite that was sent to the wrong party, leaked, or is simply no longer wanted can be withdrawn before it is used. **Account Invite Revoke** marks it revoked; did-management/account/register then refuses it.

Revocation acts on the invite only. An account an invite has already admitted is not affected: removing an account is an account-management decision, taken through the service's administration.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** answer `unknownInvite` for an identifier it does not hold **and** for one on a domain the issuer does not administer, so that an outsider cannot learn which identifiers exist.
2. **MUST** revoke a `pending` invite and answer `changed: true`.
3. **MUST** answer an already-`revoked` invite with `changed: false`: revocation is a *setting a state* operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)).
4. **MUST** answer `notPending` for a `redeemed` or `expired` invite, and change nothing.
5. **MUST** make revocation effective before answering: a registration that begins after the response **MUST** be refused.

## Authorization

The entitlement is **administrative authority over the invite's domain**, as the service's access-control list records it — not authorship of the invite: any administrator of the domain may revoke any of its invites, so that a departed administrator's invites can be withdrawn. The proof establishes who is asking; the consumer checks that party's ACL entry against the invite's domain.

## Definitions

* `inviteId` — as its schema description says.

## Request

An administrator sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdrawing an unused invite

```json
{
  "id": "urn:uuid:e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091",
  "type": "https://trusttasks.org/spec/did-management/account/invite/revoke/0.1",
  "issuer": "did:webvh:QmAdminScid:admin.example.com",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-03T10:00:00Z",
  "payload": { "inviteId": "inv-01J9Z8Y7X7" },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-03T10:00:00Z",
    "verificationMethod": "did:webvh:QmAdminScid:admin.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z7UqPoX3xdRbW"
  }
}
```

## Response

The service answers with the invite after the request and whether it changed, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The invite is revoked

```json
{
  "id": "urn:uuid:f6a7b8c9-d0e1-4f2a-3b4c-5d6e7f8091a2",
  "type": "https://trusttasks.org/spec/did-management/account/invite/revoke/0.1#response",
  "threadId": "urn:uuid:e5f6a7b8-c9d0-4e1f-2a3b-4c5d6e7f8091",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:webvh:QmAdminScid:admin.example.com",
  "issuedAt": "2026-12-03T10:00:01Z",
  "payload": {
    "invite": {
      "inviteId": "inv-01J9Z8Y7X7",
      "domain": "did.example.com",
      "state": "revoked",
      "createdAt": "2026-11-30T10:05:00Z",
      "createdBy": "did:webvh:QmAdminScid:admin.example.com",
      "expiresAt": "2026-12-07T10:05:00Z"
    },
    "changed": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-03T10:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z8VqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The request carries an invite identifier, which is not a secret; the response carries the invite's summary, never its token or claim code.

### Correlation

The task links the revoking administrator to the invite, which is recorded in the domain's admission record. It carries nothing about the invitee.

### Retention

The service keeps the revoked invite's summary in the admission record and **SHOULD** delete the hashes of its secrets, which can no longer be used.

### Consent/purpose

Revocation exists to stop an invite being used. It carries no data collected for any other purpose.
