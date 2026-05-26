---
slug: auth/whoami
version: "0.1"
title: Auth — Whoami
summary: A subject asks an auth service to introspect the current session — returning the Session object, role assignments, and effective scopes — so the client can reconcile state with the server's view.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - whoami
  - introspect
  - session
  - claims
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
  - role: Auth service
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: The response carries the consumer's view of "who the producer is" — roles, scopes, session metadata. A proof on the request ties the introspection to the subject's signing key, preventing a token-bearing intermediary from harvesting a different subject's claims.
errorCodes:
  - code: auth/whoami:no_session
    meaning: The producer's subject has no active session with the auth service.
    retryable: false
related:
  - auth/authenticate
  - auth/refresh
  - auth/revoke-session
---

## Abstract

The **Auth — Whoami** Trust Task asks an auth service "what do you know about me right now?". The framework `proof` identifies the producer; the response is the auth service's current view: the active `Session`, role assignments, and effective scopes.

This task replaces ad-hoc `/me` REST endpoints. It serves three concrete needs:

- **Client state reconciliation** — after a refresh, after a step-up, or on a fresh tab.
- **Audit confirmation** — what does the server actually think I am? Useful when an authorization decision surprises the client.
- **Self-debugging** — operators inspecting their own session for support.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/whoami/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Provide an empty payload (only the optional `ext` slot is permitted).
3. Include a verified `proof` per [SPEC.md §4.7](../../../../SPEC.md#47-proof).

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Look up the most recently-issued active session whose `subject` equals the document `issuer`. If none, respond with `auth/whoami:no_session`.
3. Return a `#response` document carrying `{ session, roles?, scopes? }`. Consumers MAY omit `roles` / `scopes` when their model has no concept of either; the `session` field is REQUIRED.

A consumer **MAY** return information about a session distinct from the *most recently-issued* one when its policy is more specific (e.g. tying the introspection to whichever access token was used at the transport layer). The framework deliberately does not pin a multi-session selection policy.

## Definitions

* **Subject.** The party introspecting; identified by `issuer` and verified via `proof`.
* **Session.** The `Session` object the consumer holds for the producer.
* **Roles, scopes.** Consumer-defined vocabularies surfaced for client reconciliation.

## Payload

The payload carries no required members. The `proof` is the entire request.

`payload.ext` — optional extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Producer introspects

```json
{
  "id": "ef456789-0123-4567-89ab-cdef01234567",
  "type": "https://trusttasks.org/spec/auth/whoami/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T10:15:00Z",
  "payload": {},
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/whoami/0.1#response`. Payload: `{ session, roles?, scopes? }`.

### Successful whoami

```json
{
  "id": "f5678901-2345-6789-0abc-def012345678",
  "type": "https://trusttasks.org/spec/auth/whoami/0.1#response",
  "threadId": "ef456789-0123-4567-89ab-cdef01234567",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:15:01Z",
  "payload": {
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T10:30:31Z",
      "amr": ["did", "passkey"],
      "acr": "aal2"
    },
    "roles": ["admin"],
    "scopes": ["context:project-alpha", "acl:read", "acl:write"]
  }
}
```

### No active session

```json
{
  "id": "06789012-3456-789a-bcde-f01234567890",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "ef456789-0123-4567-89ab-cdef01234567",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T10:15:01Z",
  "payload": {
    "code": "auth/whoami:no_session",
    "message": "No active session for did:web:alice.example."
  }
}
```

## Security & Privacy

**Why proof-required.** The response reveals what the consumer believes about the producer — roles can be sensitive (signalling membership in private cohorts, regulatory rosters, employment status, etc.). A bearer-token introspection would let any holder of a captured token harvest these claims. The DID-signed proof binds the introspection to the subject's signing key.

**Stale responses.** Consumers SHOULD NOT cache whoami responses across clients; the response is a snapshot of *server* state at the moment of issuance and the consumer's authoritative view MAY have changed by the time the client renders it. Clients SHOULD re-fetch whoami after any policy-affecting action (acl/grant, acl/swap-key, refresh that may have rotated scope).

**Confidentiality.** The response is privacy-sensitive (roles + scopes). Consumers MUST require transport-level confidentiality.

The optional `ext` extension is part of the producer's signed surface.
