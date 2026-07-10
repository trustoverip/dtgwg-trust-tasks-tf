---
slug: auth/authenticate
version: "0.1"
title: Auth — Authenticate
summary: A subject presents a previously-issued challenge inside a proof-bearing Trust Task document; the proof verifies the subject controls their VID, and the auth service responds with a session + tokens.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - authentication
  - did
  - login
  - challenge-response
  - jwt
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
  rationale: The framework `proof` is the authentication. Without a verified proof binding the document to the subject's VID, the auth service has no basis to issue a session.
sideEffects:
  level: mutating
  rationale: "Establishes an authenticated session and issues tokens; the session is revocable state."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/authenticate:challenge_not_found
    meaning: The `sessionId` does not refer to any challenge the auth service issued, or the challenge was already consumed.
    retryable: false
  - code: auth/authenticate:challenge_expired
    meaning: The challenge's expiresAt is in the past.
    retryable: true
  - code: auth/authenticate:challenge_mismatch
    meaning: The presented `challenge` value does not equal the one the auth service bound to `sessionId`.
    retryable: false
  - code: auth/authenticate:subject_mismatch
    meaning: The `issuer` of the authenticate document does not equal the `subject` the challenge was bound to.
    retryable: false
  - code: auth/authenticate:scope_denied
    meaning: One or more requested scopes were refused by the consumer's authorization policy. `details.refused` MAY enumerate the denied scopes.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        refused:
          type: array
          items: { type: string }
related:
  - auth/challenge
  - auth/refresh
  - auth/revoke-session
  - auth/whoami
  - auth/passkey/login/finish
---

## Abstract

The **Auth — Authenticate** Trust Task is the second half of a challenge-response authentication. The subject signs a document carrying the challenge they received from [`auth/challenge/0.1`](../../challenge/0.1/spec.md); the framework `proof` on that document, verified against the subject's VID, IS the authentication. The auth service replies with a *Session* and *TokenBundle*.

This task does NOT mint tokens itself — it requests them. The auth service applies its own authorization policy (ACL, allowed scopes, AAL ceiling) before responding.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the subject) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/authenticate/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Echo `payload.challenge` and `payload.sessionId` verbatim from the `auth/challenge` response.
3. Include a `proof` member per [SPEC.md §4.7](../../../../SPEC.md#47-proof). The proof's `verificationMethod` MUST resolve via the issuer's DID document.
4. **MAY** request specific `payload.scope` capabilities. The producer MUST be prepared for the consumer to issue a token bundle with a narrower `scope`.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`. The proof being absent or invalid is a hard failure.
2. Look up the server-side binding for `payload.sessionId`. If no binding exists, respond with `auth/authenticate:challenge_not_found`.
3. Compare the binding's stored challenge to `payload.challenge` using a constant-time comparator. Mismatch → `auth/authenticate:challenge_mismatch`.
4. Reject expired bindings with `auth/authenticate:challenge_expired`.
5. When the challenge was issued with a bound `subject`, verify the document's `issuer` equals that subject. Mismatch → `auth/authenticate:subject_mismatch`.
6. Consume the challenge. The same `(sessionId, challenge)` pair MUST NOT be honored a second time, even on identical authenticate documents.
7. Apply the consumer's authorization policy. Refused scopes → `auth/authenticate:scope_denied` with `details.refused`.
8. Issue a `#response` document carrying a freshly-created `Session` (with `amr` containing at least `"did"` and `acr` defaulting to `"aal1"`) and a `TokenBundle`.

## Definitions

* **Subject.** The party authenticating; identified by `issuer` and verified via `proof`.
* **Auth service.** The party verifying the proof and issuing the session; identified by `recipient`.
* **Session.** The logical authentication context the consumer creates on success. Schema: [`_shared/0.1/session.schema.json#Session`](../../_shared/0.1/session.schema.json).
* **TokenBundle.** The access + optional refresh tokens. Schema: [`_shared/0.1/tokens.schema.json#TokenBundle`](../../_shared/0.1/tokens.schema.json).
* **VID.** *Verifiable Identifier* — DID, did:webvh URL, or any other scheme accepted by the consumer's trust framework.

## Payload

`payload.challenge` (REQUIRED) — verbatim echo of the challenge value returned by the prior `auth/challenge` response.

`payload.sessionId` (REQUIRED) — verbatim echo of the sessionId from that response.

`payload.scope` (optional) — capability tags the subject is requesting; consumer-defined vocabulary.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

The full JSON Schema is in [`payload.schema.json`](payload.schema.json).

## Examples

### A subject completes a login

```json
{
  "id": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T10:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Requesting narrower scopes

```json
{
  "id": "2b3c4d5e-6f78-9012-3456-7890abcdef12",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:00:30Z",
  "payload": {
    "challenge": "ZGN3RvOXh0c3JydWxsbmJzcmVxdHJjQVZjbA",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "scope": ["context:project-alpha", "acl:read"]
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/authenticate/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ session, tokens }`. The `session.amr` MUST include `"did"` (the authentication factor that completed). Consumers issuing a fresh session set `session.acr` to `"aal1"` unless the consumer combined this exchange with additional factors at issuance time (e.g. a co-located passkey assertion), in which case higher AAL classes are appropriate.

Failures use `trust-task-error` ([SPEC.md §8](../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful authentication

```json
{
  "id": "3c4d5e6f-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/authenticate/0.1#response",
  "threadId": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:15:31Z",
      "amr": ["did"],
      "acr": "aal1"
    },
    "tokens": {
      "accessToken": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9…",
      "refreshToken": "rt_8f2c1d4e9a7b3056",
      "tokenType": "Bearer",
      "expiresIn": 900,
      "refreshExpiresIn": 86400,
      "scope": ["context:project-alpha", "acl:read"]
    }
  }
}
```

### Challenge already consumed

```json
{
  "id": "4d5e6f78-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "1a2b3c4d-5e6f-7890-1234-567890abcdef",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:00:31Z",
  "payload": {
    "code": "auth/authenticate:challenge_not_found",
    "message": "The sessionId ec5d3c89… does not correspond to an active challenge."
  }
}
```

## Security & Privacy

The framework `proof` carries the entire trust burden. Consumers MUST NOT take any other field as authentication evidence — not the `issuer` claim alone, not the transport-layer identity, and not any header outside the document. The proof's `verificationMethod` MUST be resolvable through the issuer's published DID document at verification time; consumers SHOULD cache resolved DID documents with a TTL that respects the issuer's `nextUpdate` hint if present.

**Replay across challenges.** A successful authenticate consumes its challenge; consumers MUST persist the consumption marker for at least `challenge.expiresAt` so a replay arriving late can't slip through after the binding row would otherwise be cleaned up.

**Binding skew.** If the consumer issued a subject-agnostic challenge, the `issuer` of the authenticate document is whichever VID the producer chose. The consumer's authorization policy decides whether that VID is recognized — `subject_mismatch` is reserved for the bound-subject case.

**Scope downgrade.** The consumer MUST treat `payload.scope` as a *request*, not a grant. Returning a `TokenBundle.scope` narrower than the request is valid and SHOULD NOT trigger an error. Returning broader scope than requested is a policy decision the consumer documents in its trust framework.

**Token confidentiality.** The TokenBundle returned in the response document is bearer-grade material. Consumers operating over transports that don't already provide confidentiality (raw HTTPS POST is fine; broadcast queues are not) MUST encrypt the response to the producer.

The optional `ext` extension is part of the signed surface; producers MUST NOT place secret material in `ext`.
