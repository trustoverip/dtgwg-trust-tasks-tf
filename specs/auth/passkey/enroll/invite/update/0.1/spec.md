---
slug: auth/passkey/enroll/invite/update
version: "0.1"
title: Auth — Passkey Invite Update
summary: An administrator changes an outstanding passkey enrolment invite's role or expiry, identified by its invite handle, without re-issuing it.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - invite
  - enrolment
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
  rationale: "Changing an invite's role changes what its redeemer will be allowed to do, and extending it keeps a credential-binding offer open longer. The auth service authorises on the proven issuer, and the signed request is the record of who changed the offer."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A mutating task. A replayed update would reset a role or expiry an administrator has since changed again."
sideEffects:
  level: mutating
  rationale: "Rewrites the invite's role and/or expiry. The token and claim code are unchanged."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a handle, a role and an expiry; the response, the invite's summary."
retention:
  class: durable
  rationale: "The signed request records who changed the role or lifetime of an offer to bind a credential to someone, which an audit of enrolments needs."
errorCodes:
  - code: auth/passkey/enroll/invite/update:notFound
    meaning: "No outstanding invite within the administrator's authority has this `inviteId` — never issued, already redeemed, revoked, or outside the caller's scope, all alike."
    retryable: false
  - code: auth/passkey/enroll/invite/update:inviteLapsed
    meaning: "The invite has expired. An expired invite is not revived; revoke it and issue a new one."
    retryable: false
  - code: auth/passkey/enroll/invite/update:roleNotAllowed
    meaning: "The role is not one the invite's purpose may carry (a stepUp invite carries none), or not one this administrator may grant."
    retryable: false
related:
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/invite/list
  - auth/passkey/enroll/invite/revoke
---

## Abstract

**Passkey Invite Update** lets an administrator correct an outstanding invite — the wrong role, or an expiry the invitee will miss — without revoking it and sending a new token and claim code. The invite is named by the `inviteId` [`auth/passkey/enroll/invite/list`](../../list/0.1/spec.md) returns, never by its token.

An expired invite is not revived. Lengthening an offer that has already lapsed would amount to issuing a new one without the issuance record; the administrator issues a new one instead.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). The payload names at least one of `role`, `expiresAt` and `extendBy`, and not both `expiresAt` and `extendBy`; the schema enforces both. A conforming auth service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`), and one with no `issuedAt` with `malformedRequest`.
2. Refuses a producer without administrator standing with `permissionDenied`.
3. Answers `notFound` for an `inviteId` that names no outstanding invite within the caller's authority, and `inviteLapsed` for one whose expiry has passed.
4. Answers `roleNotAllowed` for a role the invite's purpose may not carry or the administrator may not grant.
5. **SHOULD** refuse, with `malformedRequest`, an `expiresAt` in the past or beyond its own maximum invite lifetime, and clamp an `extendBy` to that maximum.
6. Otherwise applies the change, leaves the token and claim code as they were, and answers with the invite's summary — never its token.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on the auth service, with authority over the invite's subject and to grant the requested role**, read from the service's records at execution time. Being able to name an invite by `inviteId` is not standing: the handle identifies, it does not authorise. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Outstanding invite** — as in [`auth/passkey/enroll/invite/list`](../../list/0.1/spec.md).

## Request

```json
{
  "id": "urn:uuid:8d3956ca-4a68-49bc-8fb0-9c0d1e2f3a01",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/update/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "inviteId": "inv_01J8Z6Q4M2", "extendBy": 86400 }
}
```

## Response

```json
{
  "id": "urn:uuid:8d3956ca-4a68-49bc-8fb0-9c0d1e2f3a02",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/update/0.1#response",
  "threadId": "urn:uuid:8d3956ca-4a68-49bc-8fb0-9c0d1e2f3a01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "invite": {
      "inviteId": "inv_01J8Z6Q4M2",
      "subject": "did:webvh:QmBobScid8:did.example.com:bob",
      "purpose": "session",
      "role": "owner",
      "createdAt": "2026-09-26T15:00:00Z",
      "expiresAt": "2026-09-28T09:00:01Z",
      "expired": false
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a handle, and a role or an expiry. The response carries the invite's summary: subject, purpose, role, instants. No token, URL or claim code travels in either direction.

### Correlation

The request links the administrator to the invite's subject, at an auth service that already holds that link. Both parties declare `identifierScope: public`, and must: the administrator's DID is what the auth service's access-control entry is keyed on, and the auth service's DID is the recipient every administrator addresses.

### Retention

The signed request is worth keeping with the service's enrolment audit trail: a change of role or lifetime on an offer to bind a credential is exactly what an investigation of an unexpected enrolment needs to see.

### Consent/purpose

The task exists to correct an invite in flight. It **MUST NOT** be used to re-point an invite at a different subject — there is no member for it, and an invite for someone else is a new invite.
