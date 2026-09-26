---
slug: auth/passkey/enroll/invite
version: "0.2"
title: Auth — Passkey Enroll (invite)
summary: An administrator issues a single-use invite, redeemed with a separately delivered claim code, binding a passkey to a subject who cannot authorise it themselves — a first passkey, or a step-up-only passkey that never opens a session.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - enrollment
  - invite
  - step-up
  - bootstrap
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
  - role: Invitee
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: An invite lets someone who has never authenticated bind a credential to a VID. The administrator's signed proof is the entire trust chain — without it, a token-stealing attacker could mint invites pointing at VIDs they want to act as.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An enrolment invite is an offer to bind an authenticator, and an offer with no issue time never lapses. Requiring it gives the auth service an outer bound on how long a captured invite stays usable, independent of `ttl`.
sideEffects:
  level: mutating
  rationale: "Issues a single-use enrollment invite; consumable, expiring state. Nothing is bound until the invite is redeemed."
subjectPath: /subject
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: "The response carries the invite token and the claim code, which together authorise binding a credential to the subject until the invite expires or is redeemed."
retention:
  class: durable
  rationale: "The auth service keeps who invited whom, when, for what purpose, and whether and when the invite was redeemed, as the audit record of how the credential came to exist."
errorCodes:
  - code: auth/passkey/enroll/invite:subjectAlreadyEnrolled
    meaning: "For `purpose: session`, the invitee VID already has a session credential on file. Use auth/passkey/enroll/start with the existing credential instead."
    retryable: false
  - code: auth/passkey/enroll/invite:roleNotPermitted
    meaning: "The administrator's authority does not allow assigning the requested role, or issuing an invite of this purpose."
    retryable: false
  - code: auth/passkey/enroll/invite:purposeNotSupported
    meaning: "This auth service does not issue credentials of the requested purpose."
    retryable: false
  - code: auth/passkey/enroll/invite:subjectUnknown
    meaning: "For `purpose: stepUp`, the auth service does not recognise the subject — it holds no standing a step-up could ever be asked of."
    retryable: false
related:
  - auth/passkey/enroll/redeem/start
  - auth/passkey/enroll/redeem/finish
  - auth/passkey/enroll/start
  - auth/passkey/revoke/start
  - auth/passkey/login/start
  - auth/step-up/approve-request
  - auth/step-up/approve-response
---

## Abstract

The **Auth — Passkey Enroll (invite)** Trust Task closes the gap where a subject has no factor the auth service trusts with which to authorise binding a passkey to their VID. An administrator asks the auth service for a single-use invite; the service returns a URL and a separate **claim code**; the administrator delivers the two over different channels; the invitee opens the URL, types the code and creates a passkey through [`auth/passkey/enroll/redeem/start`](../../redeem/start/0.1/spec.md) and [`finish`](../../redeem/finish/0.1/spec.md).

0.2 adds **`purpose`**. A `session` credential, as in 0.1, signs its subject in. A **`stepUp`** credential never does: it is accepted only as the user-verification evidence of a step-up **bound to one operation** — the passkey gesture a VTC asks for before it lets a signed document confer authority, such as `git-ns/right/break-glass`. It exists for subjects who act through signed documents and have no session at all, but whose acts a service wants a human gesture for.

## Changes from 0.1

- **`purpose`** (`session` | `stepUp`, default `session`), echoed in the response. For `stepUp`, `role` and `scopes` **MUST** be absent — the credential confers nothing — and the auth service **MUST** already recognise the subject (`subjectUnknown` otherwise). `subjectAlreadyEnrolled` applies only to `session`: a subject may hold several step-up credentials, and inviting another requires, at redemption, a user-verified assertion from one they already hold.
- **`claimCode`**, returned once in the response, **REQUIRED** at redemption, and never carried in the URL. 0.1's token was the entire redemption factor, so one intercepted message was enough; 0.2 needs two channels.
- **Redemption has a wire**: [`auth/passkey/enroll/redeem/start`](../../redeem/start/0.1/spec.md) and [`finish`](../../redeem/finish/0.1/spec.md). 0.1 said only that the token "replaces the producer-side proof" of `enroll/{start,finish}`, which both still require.
- **The rules of a `stepUp` credential**, below, bind every auth service that issues one.

A 0.1 invite is a 0.2 invite of `purpose: session` except that its response lacks `claimCode`; the framework target moves to 0.6.0. Released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/enroll/invite/0.2`, with itself as `issuer` and the auth service as `recipient`, carrying a verified `proof`.
2. Populate `payload.subject` with the VID being invited, and `payload.purpose` where it is not `session`.
3. Deliver `invite.url` and `claimCode` to the invitee over **different** channels.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document and verify the `proof`.
2. Authorize the administrator: their authority MUST permit issuing invites for the requested purpose, role and scopes. Refuse with `roleNotPermitted` otherwise, and with `purposeNotSupported` for a purpose it does not issue.
3. For `session`, refuse a subject already holding a session credential with `subjectAlreadyEnrolled`. For `stepUp`, refuse a `role` or `scopes` as `malformedRequest`, and a subject it does not recognise with `subjectUnknown`.
4. Generate a single-use `token` with at least 128 bits of entropy, a `url` carrying it, and a `claimCode` with at least 40 bits of entropy. Store the token and the code only as hashes — the code as a slow, salted one, since it is short.
5. Bind the invite to: the subject, the purpose, the role and scopes, the administrator, and an expiry from `payload.ttl` (default 1 h).
6. Record an audit event naming the administrator, the subject, the purpose and the expiry — never the token or the code.
7. Return `{ invite, subject, purpose, expiresAt, claimCode }`. `claimCode` is returned only here.

### A `stepUp` credential

An auth service that issues `stepUp` credentials **MUST**:

1. **Never** offer one in `auth/passkey/login/start`, and refuse it in `auth/passkey/login/finish` — for sign-in and for session step-up (`purpose: stepUp` there) alike. The simplest way to hold this is to keep step-up credentials in a store the login ceremony does not read.
2. Accept one **only** as user verification bound to one operation of its own subject, and nowhere else: the `webauthn` evidence of an `auth/step-up/approve-response` answering a step-up that is **bound to one operation** (`boundTo`, no `sessionId`) and was issued to the credential's subject, in **self** mode — a step-up credential never makes its holder a delegated approver for anyone else; the `uvCredential` of an [`auth/passkey/enroll/redeem/finish`](../../redeem/finish/0.1/spec.md) binding another step-up credential to the same subject; and the `uvCredential` of an [`auth/passkey/revoke/finish`](../../../revoke/finish/0.2/spec.md) 0.2 revoking one of the subject's own step-up credentials.
3. Treat the credential as confirming a human gesture from its subject and nothing else: it authorises no act by itself, and the operation it is bound to is still decided on the operation's own terms.
4. Let its subject revoke it with [`auth/passkey/revoke`](../../../revoke/start/0.2/spec.md) 0.2, and let an administrator revoke it for them (`subject` there). The last-credential refusal does not protect a step-up credential: losing one costs the subject a gesture, not their account.
5. Record an audit event for its redemption and its revocation, naming the subject and, for a revocation, who revoked it.

## Definitions

* **Administrator.** The party issuing the invite; identified by `issuer`. Who counts is the consumer's policy.
* **Invitee.** The party redeeming the invite. The ceremony establishes that they hold the URL, the claim code and a working authenticator. It does **not** establish that they are the human behind the subject VID; that trust flows from the administrator, which is why the two channels matter.
* **Purpose.** What a credential may authenticate: `session` or `stepUp`.

## Payload

`payload.subject` (REQUIRED) — invitee VID. `payload.purpose` — `session` (default) or `stepUp`. `payload.role`, `payload.scopes` — for `session` only. `payload.deviceLabel`, `payload.ttl` — optional. `payload.ext` — extension slot.

## Examples

### A community administrator invites a namespace admin to enrol a step-up passkey

```json
{
  "id": "urn:uuid:5d1e7c2a-3b4f-4e8a-9c61-0f2a7b3c9d01",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/0.2",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T10:00:00Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "purpose": "stepUp",
    "deviceLabel": "Carol's laptop",
    "ttl": 3600
  },
  "proof": { "…": "…" }
}
```

## Response

### Issued invite

```json
{
  "id": "urn:uuid:5d1e7c2a-3b4f-4e8a-9c61-0f2a7b3c9d02",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/0.2#response",
  "threadId": "urn:uuid:5d1e7c2a-3b4f-4e8a-9c61-0f2a7b3c9d01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-25T10:00:01Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "purpose": "stepUp",
    "expiresAt": "2026-09-25T11:00:01Z",
    "claimCode": "7KQ4-MX2P-9TDA",
    "invite": {
      "token": "inv_8f2c1d4e9a7b30568f2c1d4e9a7b3056a1b2c3d4",
      "url": "https://acme-vtc.example/admin/enroll-passkey?token=inv_8f2c1d4e9a7b30568f2c1d4e9a7b3056a1b2c3d4"
    }
  }
}
```

## Security & Privacy

**Why a subject cannot enrol their own first step-up passkey.** A step-up exists to add a human gesture to the possession of a signing key. If the key alone could enrol the passkey, whoever stole the key would enrol their own and the step-up would add nothing. So the first binding is anchored outside the subject's key: in an administrator's signed invite, delivered over two channels. Later ones additionally need a user-verified assertion from a step-up credential the subject already holds.

**Two channels.** Neither the URL nor the claim code alone redeems the invite. An administrator who sends both in one message has thrown away half the control; a surface that issues invites **SHOULD** say so and **SHOULD** present the two separately.

**Token and code strength.** The token is at least 128 bits (192 RECOMMENDED). The claim code is short enough to type, so the consumer **MUST** rate-limit redemption attempts per invite and **SHOULD** invalidate an invite after a small number of wrong codes.

**Purpose separation.** A step-up credential that could open a session would turn every member into a console user the moment they enrolled one. The rules above keep the two kinds apart; keeping them in separate stores makes the separation hold by construction rather than by a check each login path must remember.

**Audit.** Issuance, redemption and revocation are each logged with the administrator and the subject; an incident review needs to reconstruct who let whom bind which credential.

**Free text.** `deviceLabel` is free text bounded at 256 characters, authored by the inviter; it is a suggestion the invitee MAY override, and a surface rendering it before redemption SHOULD attribute it to the inviter.

### Data carried

The request carries the subject's VID, the purpose, and for `session` a role and scopes. The response carries the invite token and URL and the claim code — together a bearer authorisation to bind a credential to the subject until the invite is redeemed or expires. The consumer keeps only hashes of the token and code.

### Correlation

The invite links the administrator to the subject and to the moment they chose to let the subject enrol. The URL reveals the auth service's origin to anyone who sees it; it names no subject, so a URL seen alone does not say who was invited.

### Retention

The token and code hashes are kept until the invite is redeemed, invalidated or expires, and then discarded. The audit record of issuance — administrator, subject, purpose, expiry, never the token or code — is durable.

### Consent/purpose

The invite exists to bind one credential of one purpose to one subject. Its token and code **MUST NOT** be accepted for anything else, and a `stepUp` invite **MUST NOT** yield a credential that opens a session.
