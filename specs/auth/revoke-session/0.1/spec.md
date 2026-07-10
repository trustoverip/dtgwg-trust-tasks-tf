---
slug: auth/revoke-session
version: "0.1"
title: Auth — Revoke Session
summary: A subject (or an administrator acting on their behalf) tells an auth service to invalidate a specific session or every session bound to the subject.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - logout
  - sign-out
  - revoke
  - session
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Revocation is an evidentiary event that affects every device or process holding a token for this subject. Requiring a verified proof prevents an attacker who has captured one token from invalidating other sessions (denial-of-service via revocation) without controlling the subject's signing key.
sideEffects:
  level: mutating
  rationale: "Invalidates one or all of the subject's sessions; recoverable by re-authenticating."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/revoke-session:session_not_found
    meaning: The named `sessionId` does not exist (already revoked, or never belonged to this subject).
    retryable: false
  - code: auth/revoke-session:not_owner
    meaning: The named `sessionId` exists but belongs to a different subject than the producer. The auth service MUST NOT reveal whether the session exists at all when the producer is not its owner.
    retryable: false
related:
  - auth/authenticate
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Revoke Session** Trust Task invalidates one or all of a subject's active sessions. The auth service drops its server-side session state; any subsequent use of access or refresh tokens bound to a revoked session MUST fail.

The framework `proof` is REQUIRED. A bearer-token-only revocation would let any holder of a captured access token deny service to the legitimate subject by revoking everything they have; requiring a DID signature ties the revoke to the subject's signing key.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/revoke-session/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Provide exactly one of `payload.sessionId` (revoke a single session) or `payload.all: true` (revoke every session the consumer holds for the producer's subject).
3. Include a verified `proof` per [SPEC.md §4.7](../../../../SPEC.md#47-proof).

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. When `sessionId` is provided:
   - Look up the session; if absent, respond with `auth/revoke-session:session_not_found`.
   - Verify the session's `subject` equals the document `issuer`. Mismatch → `auth/revoke-session:not_owner`.
   - Mark the session and its refresh tokens revoked.
3. When `all: true` is provided, enumerate every session for the issuer's subject and revoke each. The response's `revokedCount` carries the total.
4. Persist revocation state for at least the longest issued refresh-token lifetime, so a late-arriving refresh from a stolen token cannot succeed by waiting out the consumer's session-row cleanup.

A consumer **MAY** also accept revoke requests from an *administrative* issuer whose `subject` differs from the targeted session's subject. The consumer's authorization policy MUST validate the administrator's authority to revoke others' sessions; this framework deliberately does not constrain administrative-revoke semantics — that's an ACL/permissions concern, not a wire-format one.

## Definitions

* **Subject.** The party owning the session(s); identified by the document's `issuer`.
* **Auth service.** The party holding the sessions; identified by `recipient`.
* **Session.** A `Session` row created by a prior `auth/authenticate` or `auth/passkey/login/finish`.

## Payload

`payload.sessionId` — exclusive with `all`; revoke this specific session.

`payload.all` — exclusive with `sessionId`; when `true`, revoke every session for the subject.

`payload.reason` — optional human-readable rationale.

`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Standard logout

```json
{
  "id": "abcd1234-5678-90ab-cdef-1234567890ab",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T11:00:00Z",
  "payload": {
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "reason": "logout"
  },
  "proof": { "…": "…" }
}
```

### Sign out everywhere (e.g. after a device loss)

```json
{
  "id": "bcde2345-6789-01bc-def2-345678901bcd",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T11:00:00Z",
  "payload": {
    "all": true,
    "reason": "device-lost"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/revoke-session/0.1#response`. The payload is `{ revokedCount }`.

`revokedCount: 0` is a valid success outcome — the named session was already revoked, or `all: true` was sent when no active sessions existed. Producers SHOULD treat zero as "the post-state is what you asked for", not as an error.

### Successful single revoke

```json
{
  "id": "cdef3456-7890-12cd-ef34-56789012cdef",
  "type": "https://trusttasks.org/spec/auth/revoke-session/0.1#response",
  "threadId": "abcd1234-5678-90ab-cdef-1234567890ab",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T11:00:01Z",
  "payload": {
    "revokedCount": 1
  }
}
```

## Security & Privacy

**Token race.** Between the moment a producer signs the revoke and the moment the consumer commits, the access token is still valid. Consumers SHOULD make the revoke atomic with respect to their session-lookup path (a single transaction, or a "revoked" set checked before every token use).

**Cleanup horizon.** A consumer cleaning up old session rows MUST retain the revocation marker for at least the longest issued refresh-token lifetime. Otherwise a stolen refresh token replayed after cleanup would succeed.

**Information leakage.** `not_owner` deliberately does NOT distinguish "exists but yours not" from "doesn't exist" beyond the error code itself. The error message MUST NOT reveal the actual owner of a session the producer doesn't control.

**Administrative revoke.** When the producer is not the session's subject (the administrative case), the consumer's policy decides whether to allow it. Audit logs MUST record both the actual issuer and the targeted subject so an incident investigation can reconstruct who acted on whose behalf.

The optional `ext` extension is part of the producer's signed surface.
