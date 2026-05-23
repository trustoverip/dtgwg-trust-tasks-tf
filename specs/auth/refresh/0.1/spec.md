---
slug: auth/refresh
version: "0.1"
title: Auth — Refresh
summary: Exchange a refresh token for a new access token, without re-running the challenge-response handshake.
status: draft
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - auth
  - refresh
  - token
  - session
  - jwt
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
  - role: Auth service
    requirement: REQUIRED
proofRequirement:
  requirement: OPTIONAL
  rationale: The refreshToken itself is the secret; a proof on the document is redundant when the transport binds the producer's identity end-to-end (the typical case for refresh, which is called by an already-authenticated client). Consumers retaining refresh exchanges for audit MAY require a proof.
errorCodes:
  - code: auth/refresh:token_not_found
    meaning: The refreshToken does not refer to any session the auth service issued.
    retryable: false
  - code: auth/refresh:token_expired
    meaning: The refreshToken's refreshExpiresIn has elapsed.
    retryable: false
  - code: auth/refresh:token_revoked
    meaning: The refreshToken was explicitly invalidated (typically via auth/revoke-session, or by a step-up event that rotated all session refresh tokens).
    retryable: false
  - code: auth/refresh:scope_widening_refused
    meaning: The requested scope exceeds the original session's scope. Refresh MUST NOT broaden privilege.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        offending:
          type: array
          items: { type: string }
related:
  - auth/authenticate
  - auth/revoke-session
  - auth/whoami
---

## Abstract

The **Auth — Refresh** Trust Task exchanges a long-lived *refresh token* for a fresh short-lived *access token*, without re-running the challenge-response handshake. The refresh token serves as bearer authentication for this exchange; the consumer verifies it against its own session state.

This task does not change the session's *AAL* — refresh preserves whatever `amr` and `acr` the original authentication established. To elevate AAL, use [`auth/passkey/login/finish/0.1`](../../passkey/login/finish/0.1/spec.md) against the existing session, or run a [`auth/step-up/approve-request`](../../step-up/approve-request/0.1/spec.md) handshake.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the subject) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/refresh/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Populate `payload.refreshToken` with the value previously received in a `TokenBundle`.
3. **MAY** include a `payload.scope` request, which MUST be a (non-strict) subset of the session's current scope. A consumer that detects widening MUST respond with `auth/refresh:scope_widening_refused`.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements). If a `proof` is present it is verified; if absent it is not an error.
2. Look up the session associated with `payload.refreshToken`. Unknown → `auth/refresh:token_not_found`. Expired → `auth/refresh:token_expired`. Revoked → `auth/refresh:token_revoked`.
3. Issue a fresh access token. The consumer's policy decides whether to also rotate the refresh token: rotation is RECOMMENDED for tokens older than 24 h or after any suspicious-activity signal.
4. Preserve `session.amr` and `session.acr` across the refresh — refresh does not elevate or downgrade AAL.
5. Refuse with `auth/refresh:scope_widening_refused` when `payload.scope` ⊄ session scope.

## Definitions

* **Refresh token.** A long-lived opaque string issued in a prior `TokenBundle`. Consumer-internal correlation to a session.
* **Session.** The `Session` object the consumer holds for the subject; see [`_shared/0.1/session.schema.json`](../../_shared/0.1/session.schema.json).
* **Access token.** The short-lived bearer credential. Consumers SHOULD pick lifetimes between 5 min and 1 h.

## Payload

`payload.refreshToken` (REQUIRED) — the refresh token value.

`payload.scope` (optional) — narrower scope request; MUST NOT broaden.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Standard refresh

```json
{
  "id": "5e6f7890-1234-5678-9012-3456abcdef78",
  "type": "https://trusttasks.org/spec/auth/refresh/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:14:00Z",
  "payload": {
    "refreshToken": "rt_8f2c1d4e9a7b3056"
  }
}
```

### Refresh with narrower scope

```json
{
  "id": "6f789012-3456-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/refresh/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:14:00Z",
  "payload": {
    "refreshToken": "rt_8f2c1d4e9a7b3056",
    "scope": ["acl:read"]
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/refresh/0.1#response`. The payload is `{ tokens, session? }`. Consumers SHOULD include the `session` snapshot so the client can reconcile state without a separate `auth/whoami` call.

### Successful refresh with rotated refresh token

```json
{
  "id": "78901234-5678-9012-3456-7890abcdef12",
  "type": "https://trusttasks.org/spec/auth/refresh/0.1#response",
  "threadId": "5e6f7890-1234-5678-9012-3456abcdef78",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:14:01Z",
  "payload": {
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:29:01Z",
      "amr": ["did"],
      "acr": "aal1"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJFZERTQSI…",
      "refreshToken": "rt_9b3e2c5fa8d41067",
      "tokenType": "Bearer",
      "expiresIn": 900,
      "refreshExpiresIn": 86400
    }
  }
}
```

## Security & Privacy

**Token theft.** A stolen refresh token grants the attacker access until detected. Consumers SHOULD implement *refresh-token rotation*: each refresh consumes the presented token and issues a new one, and seeing the original token used again after a successful refresh is a strong signal of theft. The recommended response is to revoke the entire session.

**Scope monotonicity.** Refresh MUST NOT broaden scope; widening is reserved for re-authentication. The `scope_widening_refused` error makes this explicit.

**Transport confidentiality.** The refresh token is bearer material — both in the request and in the response. Consumers MUST require transport-level confidentiality (TLS, DIDComm authcrypt) for this exchange.

**Audit.** Refresh events are usually too frequent for individual audit logging, but rotation events and `token_revoked` responses MUST be logged — they signal session lifecycle changes that a future incident investigation will need.

The optional `ext` extension is part of the producer's signed surface when a proof is included.
