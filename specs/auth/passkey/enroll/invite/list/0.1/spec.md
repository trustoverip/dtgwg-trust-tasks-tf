---
slug: auth/passkey/enroll/invite/list
version: "0.1"
title: Auth — Passkey Invite List
summary: An administrator lists the passkey enrolment invites they have issued and that have not been redeemed — who each binds a passkey to, for what, and when it lapses — never the tokens themselves.
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
  rationale: "The answer is the set of subjects someone could be enrolled as right now. It is disclosed to administrators only, and the auth service authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads pending invites; nothing is issued, changed or revoked."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns, per invite, an opaque handle, the subject VID, the purpose, the role and two instants. Never the token, its URL or the claim code."
retention:
  class: transient
  rationale: "The request carries at most one flag."
errorCodes:
  - code: auth/passkey/enroll/invite/list:notAdministrator
    meaning: "The producer holds no administrator standing at the auth service."
    retryable: false
related:
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/invite/update
  - auth/passkey/enroll/invite/revoke
  - auth/passkey/admin-list
---

## Abstract

An administrator issues passkey enrolment invites with [`auth/passkey/enroll/invite`](../../0.2/spec.md). **Passkey Invite List** shows the ones still outstanding, so the administrator can see who is yet to enrol, change an invite's role or expiry with [`update`](../../update/0.1/spec.md), or withdraw it with [`revoke`](../../revoke/0.1/spec.md).

Each invite is identified by an `inviteId` — a handle for managing it, never the token. An unredeemed token is a credential: whoever holds it (with the claim code, under invite 0.2) binds a passkey to the invite's subject. It is disclosed once, when the invite is issued, and this listing never discloses it again. An administrator who has lost a token revokes the invite and issues a new one.

A list without a read-one sibling suffices: invites are few and short-lived, and the tasks that act on one by `inviteId` return their own `notFound`, so whether an invite exists is never answered by an empty page.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming auth service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a producer without administrator standing with `auth/passkey/enroll/invite/list:notAdministrator`.
3. Returns every unredeemed invite within the administrator's authority — including expired ones only when `includeExpired` is true — newest first, as `InviteSummary` entries.
4. **MUST NOT** return, in any member including `ext`, an invite token, a URL carrying one, a claim code, or anything from which one can be derived. The `inviteId` **MUST NOT** be the token or derivable into it.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on the auth service**, read from its own records at execution time; a consumer whose administrators are scoped lists only invites within the caller's scope. The `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Outstanding invite** — one that has been issued and neither redeemed nor revoked; it may have expired.

## Request

```json
{
  "id": "urn:uuid:7c2845b9-3957-48ab-9eaf-8b9c0d1e2f01",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/list/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:7c2845b9-3957-48ab-9eaf-8b9c0d1e2f02",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/list/0.1#response",
  "threadId": "urn:uuid:7c2845b9-3957-48ab-9eaf-8b9c0d1e2f01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "invites": [
      {
        "inviteId": "inv_01J8Z6Q4M2",
        "subject": "did:webvh:QmBobScid8:did.example.com:bob",
        "purpose": "session",
        "role": "owner",
        "createdAt": "2026-09-26T15:00:00Z",
        "expiresAt": "2026-09-28T15:00:00Z",
        "expired": false
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a flag. The response carries, per outstanding invite, a handle, a subject VID, a purpose, an optional role and two instants. The members that would let a reader enrol — token, URL, claim code — are excluded by rule 4, because this is the kind of listing that ends up on a screen or in a log.

### Correlation

The listing shows which subjects are yet to enrol and with what role — to an administrator who issued, or has authority over, those invites. Both parties declare `identifierScope: public`, and must: the administrator's DID is what the auth service's access-control entry is keyed on, and the auth service's DID is the recipient every administrator addresses.

### Retention

Nothing of the request needs keeping beyond the exchange.

### Consent/purpose

The listing exists so an administrator can manage outstanding invites. It **MUST NOT** be used as a directory of pending members for any other purpose.
