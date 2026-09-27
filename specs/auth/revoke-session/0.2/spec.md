---
slug: auth/revoke-session
version: "0.2"
title: Auth — Revoke Session
summary: A subject ends one of its sessions or all of them, or an administrator ends every session of a subject whose access it could withdraw.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - logout
  - sign-out
  - revoke
  - session
  - sign-out-everywhere
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer (the subject, or an administrator acting on a subject)
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Revocation signs a subject out of every device or process holding a token for the targeted sessions, and the `subject` form lets one party do that to another. Requiring a verified proof ties the request to the producer's signing key: without it, an attacker holding one captured token could sign the legitimate subject out everywhere (denial of service by revocation), and an administrator's revocation of someone else would leave no attributable record of who acted.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A revocation replayed after the subject has signed in again ends the new sessions instead of the old ones. The issue time is what lets the auth service place the instruction relative to the sessions it was written about, and bound how long it retains the record it needs to absorb a duplicate.
sideEffects:
  level: mutating
  rationale: "Invalidates one or all of a subject's sessions and the tokens bound to them; recoverable by the subject authenticating again."
consequences:
  - "Every device and process signed in with a revoked session is signed out, including the producer's own current session when it targets itself."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a session or a subject; the response is a count. The count reveals how many sessions the targeted subject held, which the producer is entitled to learn only because it could already manage that subject.
retention:
  class: durable
  rationale: >-
    The auth service keeps the revocation marker for at least the longest refresh-token lifetime it issues, so a stolen refresh token replayed after session cleanup still fails; and it keeps the audit record of who revoked whose sessions for as long as its audit policy keeps any other authority-affecting act.
errorCodes:
  - code: auth/revoke-session:sessionNotFound
    meaning: "The named `sessionId` is not a session the producer may revoke — it does not exist, was already revoked, or belongs to a subject outside the producer's authority. The consumer MUST answer all three identically, so the code discloses nothing about sessions the producer does not control. A consumer MAY instead answer all three with `revokedCount: 0`; see Conformance."
    retryable: false
related:
  - auth/sessions/list
  - auth/authenticate
  - auth/refresh
  - auth/whoami
  - acl/revoke
---

## Abstract

The **Auth — Revoke Session** Trust Task ends sessions. The auth service drops its server-side session state, and every subsequent use of an access or refresh token bound to a revoked session **MUST** fail.

The task targets sessions in exactly one of three ways:

- **`sessionId`** — one named session; a sign-out.
- **`all: true`** — every session of the producer itself; sign out everywhere.
- **`subject`** — every session of a named subject. When the subject is the producer this is the same as `all: true`. When it is someone else, the producer is an administrator ending the sessions of a subject whose access it could withdraw — the step that makes an access withdrawal take effect now rather than when the subject's tokens expire.

## Changes from 0.1

`0.2` is a backwards-compatible `MINOR` increment ([SPEC §5.2](/SPEC.md#52-compatibility-rules)): every valid `0.1` payload is a valid `0.2` payload.

- **Adds the `subject` form.** [`0.1`](../0.1/spec.md) allowed an administrative producer in prose but gave it no way to name whose sessions it meant — `all: true` always meant the producer's own. `subject` names them, and the Authorization section states whose sessions a producer may end.
- **Resolves the `0.1` disclosure conflict.** `0.1` declared `notOwner` for a session belonging to someone else while also requiring that the auth service not reveal whether such a session exists. Those cannot both hold. `0.2` drops `notOwner` and requires a missing session and a session outside the producer's authority to be answered identically.
- **Declares the Authorization section** [SPEC §7.3 item 15](/SPEC.md#73-specification-requirements) requires of a consequential task, and the four Security & Privacy sub-headings.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/revoke-session/0.2`, with itself as `issuer` and the auth service as `recipient`.
2. Provide exactly one of `sessionId`, `all: true` or `subject`.
3. Include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof) and an `issuedAt`.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) and verify the `proof`. A document carrying `all: false` targets nothing and is refused with `malformedRequest`.
2. **`sessionId`:** revoke the session when it exists and its subject is one the producer may manage (see Authorization). Otherwise — the session does not exist, was already revoked, or belongs to a subject outside the producer's authority — answer all three cases **identically**: either with a success response carrying `revokedCount: 0` (**RECOMMENDED**, since it makes a retried revocation succeed), or with `auth/revoke-session:sessionNotFound`. A consumer **MUST NOT** answer a session outside the producer's authority differently from a session that does not exist.
3. **`all: true`:** revoke every active session whose subject is the producer.
4. **`subject`:** when `subject` is the producer, behave as for `all: true`. Otherwise establish that the producer may manage `subject` before looking at its sessions, and refuse with `permissionDenied` ([SPEC.md §8.3](/SPEC.md#83-standard-error-codes)) when it may not. The refusal **MUST** be the same whether or not the consumer knows the subject or holds sessions for it. When the producer may manage the subject, revoke every active session whose subject is `subject`.
5. Revoke each targeted session together with the refresh tokens bound to it, and answer with `revokedCount`: the number of sessions this request invalidated. Zero is a success.
6. Persist revocation state for at least the longest refresh-token lifetime it issues, so a late refresh from a stolen token cannot succeed by waiting out session-row cleanup.
7. Record every revocation — and every refusal of the `subject` form — in its audit trail with both the producer and the targeted subject, so an investigation can reconstruct who acted on whose behalf.

`revokedCount: 0` is a valid success outcome. Producers **SHOULD** read it as "the post-state is what you asked for", not as an error.

## Authorization

The authority this task assumes is **authority over the targeted subject's access**: the producer may end a subject's sessions exactly when it could withdraw that subject's access altogether under the consumer's access-control policy. A party that could remove a subject's access entry can also end its sessions; no one else can.

- A subject may always end its own sessions (`sessionId` of its own session, `all: true`, or `subject` naming itself). Ownership is the authority.
- For any other subject, the consumer applies the rule it applies to withdrawing that subject's access. Holding an administrative role is **not sufficient** on its own: a producer whose authority is scoped — to some contexts, tenants or communities — **MUST NOT** be able to end the sessions of a subject whose access lies outside that scope, nor of a subject whose authority exceeds its own. In particular, an administrator scoped to some contexts cannot end the sessions of an unrestricted administrator, whose access belongs to no context it administers.
- A subject the consumer holds no access entry for belongs to no scope, so only a producer with unrestricted authority reaches it.

*This paragraph is non-normative.* The Verifiable Trust Infrastructure specification states this rule as VTI-SES-043 and VTI-ACL-050: session termination of another subject is gated by the same check as removing that subject's ACL entry. Implementations with a different access model apply their own equivalent of "could withdraw this subject's access"; the point the rule makes is that the role alone is the wrong test, because it lets any scoped administrator sign every other administrator out.

Verifying the `proof` establishes who is acting, never that they may act on the subject ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The proof is what lets the consumer apply the rule above to the right party, and what makes the audit record attributable.

## Definitions

* **Producer.** The party requesting the revocation; identified by `issuer`. Either the subject itself or a party with authority over the subject's access.
* **Subject.** The party the targeted sessions belong to — the producer, or the party named in `subject`.
* **Auth service.** The party holding the sessions; identified by `recipient`.
* **Session.** Server-side authenticated state created by a prior authentication (for example [`auth/authenticate`](../../authenticate/0.1/spec.md)), to which access and refresh tokens are bound.

## Request

A *request* document carries `type: https://trusttasks.org/spec/auth/revoke-session/0.2` with a payload that validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

- `sessionId` — revoke this one session.
- `all` — when `true`, revoke every session of the producer.
- `subject` — revoke every session of this subject.
- `reason` — optional human-readable rationale, recorded in the audit trail.
- `ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

### Standard logout

```json
{
  "id": "urn:uuid:4b5c6d7e-8f90-4a1b-8c2d-3e4f5a6b7c81",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-09-27T11:00:00Z",
  "payload": {
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "reason": "logout"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-09-27T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### Sign out everywhere after a device loss

```json
{
  "id": "urn:uuid:5c6d7e8f-9001-4b2c-9d3e-4f5a6b7c8d92",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-09-27T11:00:00Z",
  "payload": {
    "all": true,
    "reason": "device-lost"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-09-27T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### An administrator ends a subject's sessions after withdrawing its access

```json
{
  "id": "urn:uuid:6d7e8f90-a112-4c3d-8e4f-5a6b7c8d9ea3",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.2",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-09-27T11:05:00Z",
  "payload": {
    "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "reason": "access-withdrawn"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-09-27T11:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/revoke-session/0.2#response`, with a payload that validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Its one member, `revokedCount`, is the number of sessions this request invalidated.

Failures — `permissionDenied`, `malformedRequest`, `auth/revoke-session:sessionNotFound` — use `trust-task-error` ([SPEC.md §8](/SPEC.md#8-error-responses)), not the `#response` variant.

### An administrator's revocation ended three sessions

```json
{
  "id": "urn:uuid:7e8f90a1-b223-4d4e-9f5a-6b7c8d9eafb4",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.2#response",
  "threadId": "urn:uuid:6d7e8f90-a112-4c3d-8e4f-5a6b7c8d9ea3",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-27T11:05:01Z",
  "payload": {
    "revokedCount": 3
  }
}
```

### A scoped administrator reaches for an unrestricted administrator's sessions

```json
{
  "id": "urn:uuid:8f90a1b2-c334-4e5f-8a6b-7c8d9eafb0c5",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:6d7e8f90-a112-4c3d-8e4f-5a6b7c8d9ea3",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-09-27T11:05:01Z",
  "payload": {
    "code": "permissionDenied",
    "message": "The named subject is outside your authority.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a session identifier or a subject identifier, and an optional free-text `reason`. The response carries a count. Nothing secret moves in either direction. `reason` is recorded in the audit trail and read by operators; a producer **SHOULD** keep it to a short, non-personal rationale and **MUST NOT** put credentials, token values or third parties' personal data in it or in `ext`.

### Correlation

The `subject` form links an administrator to the subject whose sessions it ended, in the auth service's audit trail — deliberately, because accountability for acting on another party is the point of recording it. The count links a subject to the number of sessions it held, which the producer is entitled to learn only because it could already manage the subject. A refusal is identical whether or not the subject is known, and a `sessionId` outside the producer's authority is answered as a missing one, so neither form lets a producer probe for subjects or sessions it does not control.

### Retention

The auth service keeps two things. The revocation marker, for at least the longest refresh-token lifetime it issues, so a stolen refresh token replayed after session cleanup still fails. And the audit record — producer, subject, count, reason, outcome — for as long as its audit policy keeps other acts that affect a party's access. The request document itself is evidence of who asked; a consumer that retains it for audit keeps it under the same policy.

### Consent/purpose

The purpose is ending access: a subject signing itself out, or an administrator making a withdrawal of a subject's access take effect immediately. The subject identifier and the count serve that purpose and the audit trail that accounts for it, and are not for profiling a subject's sign-in behaviour. Whether a revocation of someone else's sessions warrants any further decision before it executes is the consumer's policy, not this specification's.

### Threats

*Token race.* Between signing the request and the consumer committing it, the targeted tokens are still valid. Consumers **SHOULD** make revocation atomic with respect to their session-lookup path — a single transaction, or a revoked set checked before every token use.

*Revocation as denial of service.* The proof requirement keeps a captured token from being turned against its owner; the authorization rule keeps a scoped administrator from signing out everyone else, including the administrators above it.

*Self-revocation.* A producer targeting itself ends its own current session along with the rest. That is the expected behaviour of "sign out everywhere", and a client **SHOULD** expect to authenticate again afterwards.

The optional `ext` member is part of the producer's signed surface.
