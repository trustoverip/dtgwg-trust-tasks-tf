---
slug: auth/sessions/list
version: "0.1"
title: Auth — Sessions List
summary: A subject lists every active session the auth service holds for them — typical "where am I signed in?" UX for users managing multi-device authentication.
status: draft
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - auth
  - sessions
  - list
  - device-management
  - signed-in-devices
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
  - role: Auth service
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Enumerating active sessions reveals where a subject is signed in (device labels, geographic hints if surfaced via ext). A bearer-token list would let any token-holder harvest a different subject's device fingerprint; requiring a DID-signed proof keeps the enumeration tied to the subject's signing key.
related:
  - auth/whoami
  - auth/revoke-session
  - auth/authenticate
---

## Abstract

The **Auth — Sessions List** Trust Task is the multi-session counterpart to [`auth/whoami/0.1`](../../whoami/0.1/spec.md). Whoami answers "what's my current session?"; this task answers "what sessions do you hold for me?" — typical for the "Signed in on these devices" UX or for an operator auditing where their identity is presented.

The proof on the document identifies the subject; the response is an array of `Session` objects the consumer currently holds. The subject MAY follow up with [`auth/revoke-session/0.1`](../../revoke-session/0.1/spec.md) (targeting a specific `id` or `all: true`) to invalidate any session in the list.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/sessions/list/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Provide an empty payload (only the optional `ext` slot).
3. Include a verified `proof`.

A conforming **consumer** **MUST**:

1. Validate the document and verify the `proof`.
2. Enumerate every active (non-revoked, non-expired) session whose `subject` equals the document `issuer`.
3. Return a `#response` document carrying `{ sessions }`. The array MAY be empty if no active sessions exist (the subject has never authenticated, or every session has expired).

A consumer **MAY** include the subject's own current session in the list. Consumers SHOULD return the array sorted by `issuedAt` descending so the most recent sessions surface first.

## Payload

The request payload carries no required members. The proof is the request.

`payload.ext` (optional) — extension slot.

## Examples

### Subject lists their sessions

```json
{
  "id": "list-sess-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/auth/sessions/list/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T16:00:00Z",
  "payload": {},
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/sessions/list/0.1#response`.

### Successful list with two sessions

```json
{
  "id": "list-resp-2345-6789-01bc-def234567890",
  "type": "https://trusttasks.org/spec/auth/sessions/list/0.1#response",
  "threadId": "list-sess-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T16:00:01Z",
  "payload": {
    "sessions": [
      {
        "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
        "subject": "did:web:alice.example",
        "issuedAt": "2026-05-23T15:30:00Z",
        "expiresAt": "2026-05-23T16:30:00Z",
        "amr": ["did", "passkey"],
        "acr": "aal2"
      },
      {
        "id": "fa7d3c89-aaaa-bbbb-cccc-dddddddddddd",
        "subject": "did:web:alice.example",
        "issuedAt": "2026-05-23T10:00:00Z",
        "expiresAt": "2026-05-23T22:00:00Z",
        "amr": ["did"],
        "acr": "aal1"
      }
    ]
  }
}
```

## Security & Privacy

**Why proof-required.** The session list reveals identity-attribute breadcrumbs (typical login times, AAL distribution, device labels surfaced via ext). A bearer-only introspection invites token-stealing attackers to enumerate the legitimate user's footprint as a preparatory step for an attack.

**Inactive-session policy.** The "active" filter is consumer-defined; whether it includes revoked-but-not-cleaned-up sessions, expired-but-still-in-DB sessions, or only fully-live sessions is up to the consumer's audit policy. RECOMMENDED: only fully-live sessions in the default list, and a `payload.includeExpired: true` extension via `ext` for the audit-grade variant.

**Confidentiality.** The response is privacy-sensitive. Consumers MUST require transport-level confidentiality.

The optional `ext` extension is part of the producer's signed surface.
