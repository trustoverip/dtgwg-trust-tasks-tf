---
slug: auth/passkey/enroll/invite/revoke
version: "0.1"
title: Auth — Passkey Invite Revoke
summary: An administrator withdraws an outstanding passkey enrolment invite, identified by its invite handle, so it can never be redeemed.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - invite
  - revoke
  - administrator
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Auth service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Revocation destroys an offer an invitee may be about to act on. The auth service authorises on the proven issuer on every transport, and the signed request records who withdrew it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A destructive task; an issue time bounds how long a captured revocation can be spent."
sideEffects:
  level: destructive
  rationale: "Removes the invite. Its token and claim code can never be redeemed, and nothing restores it."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The request carries a handle; the response echoes it with an instant."
retention:
  class: durable
  rationale: "The signed request records who withdrew an offer to bind a credential, which an audit of enrolments needs."
errorCodes:
  - code: auth/passkey/enroll/invite/revoke:notFound
    meaning: "No outstanding invite within the administrator's authority has this `inviteId`. An invite already redeemed or revoked is answered the same way."
    retryable: false
related:
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/invite/list
  - auth/passkey/enroll/invite/update
  - auth/passkey/revoke/start
---

## Abstract

**Passkey Invite Revoke** withdraws an invite before it is redeemed: it was sent to the wrong address, the token may have leaked, or the invitee no longer needs access. The invite is named by the `inviteId` [`auth/passkey/enroll/invite/list`](../../list/0.1/spec.md) returns, never by its token — an administrator revoking a leaked token should not have to send that token anywhere again.

A passkey the invite already bound is revoked with [`auth/passkey/revoke`](../../../../revoke/start/0.2/spec.md), not this task; a redeemed invite no longer exists to revoke.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming auth service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`), and one with no `issuedAt` with `malformedRequest`.
2. Refuses a producer without administrator standing with `permissionDenied`.
3. Answers `notFound` for an `inviteId` that names no outstanding invite within the caller's authority, including one already redeemed or revoked.
4. Otherwise removes the invite so that its token and claim code are refused at redemption from this moment, and answers with `revokedAt`. A redemption ceremony already in progress on the invite **MUST** fail at its finish step.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on the auth service, with authority over the invite's subject**, read from the service's records at execution time. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Outstanding invite** — as in [`auth/passkey/enroll/invite/list`](../../list/0.1/spec.md).

## Request

```json
{
  "id": "urn:uuid:9e4a67db-5b79-4acd-9ac1-0d1e2f3a4b01",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/revoke/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "inviteId": "inv_01J8Z6Q4M2" }
}
```

## Response

```json
{
  "id": "urn:uuid:9e4a67db-5b79-4acd-9ac1-0d1e2f3a4b02",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/revoke/0.1#response",
  "threadId": "urn:uuid:9e4a67db-5b79-4acd-9ac1-0d1e2f3a4b01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "inviteId": "inv_01J8Z6Q4M2", "revokedAt": "2026-09-27T09:00:01Z" }
}
```

## Security & Privacy

### Data carried

A handle and an instant. The token is deliberately not a member: a leaked token is the commonest reason to revoke, and the task must not ask for it to be transmitted again.

### Correlation

The request links the administrator to an invite whose subject the auth service already knows. Both parties declare `identifierScope: public`, and must: the administrator's DID is what the auth service's access-control entry is keyed on, and the auth service's DID is the recipient every administrator addresses.

### Retention

The signed request belongs in the enrolment audit trail next to the invite's issuance.

### Consent/purpose

The task exists to withdraw an offer before it is used. It does not affect credentials already bound.
