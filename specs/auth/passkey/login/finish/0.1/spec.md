---
slug: auth/passkey/login/finish
version: "0.1"
title: Auth — Passkey Login (finish)
summary: A party submits the WebAuthn assertion completing a passkey login or step-up; the auth service verifies the assertion and either issues a fresh session (login) or elevates an existing session's AAL (step-up).
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - login
  - step-up
  - aal
  - fido2
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Producer
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: The cryptographic gate is the WebAuthn assertion itself; the framework proof adds value only when the producer is signing on behalf of an existing session (step-up against a session held by a different VID). Consumers MAY require it for the step-up flow.
sideEffects:
  level: mutating
  rationale: "Issues a fresh session or elevates an existing session's AAL."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/passkey/login/finish:auth_not_found
    meaning: The `authId` does not refer to any active login ceremony.
    retryable: false
  - code: auth/passkey/login/finish:auth_expired
    meaning: The login's start-time expiry has elapsed.
    retryable: true
  - code: auth/passkey/login/finish:credential_unknown
    meaning: The asserted credential id is not registered with this auth service.
    retryable: false
  - code: auth/passkey/login/finish:assertion_invalid
    meaning: The WebAuthn assertion failed verification. `details.reason` carries a machine-readable hint.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum:
            ["challenge_mismatch", "origin_mismatch", "rp_id_mismatch", "signature_invalid", "counter_regressed", "user_handle_mismatch"]
  - code: auth/passkey/login/finish:step_up_session_not_found
    meaning: A step-up finish referenced a session id that the consumer does not hold or that has expired.
    retryable: false
related:
  - auth/passkey/login/start
  - auth/passkey/enroll/finish
  - auth/authenticate
  - auth/step-up/approve-request
---

## Abstract

The **Auth — Passkey Login (finish)** Trust Task completes the WebAuthn assertion ceremony started by [`auth/passkey/login/start/0.1`](../../start/0.1/spec.md). The producer submits the `AuthenticatorAssertionResponse` returned by `navigator.credentials.get`; the auth service verifies the assertion per WebAuthn Level 2 §7.2 and dispatches on the start ceremony's recorded `purpose`:

- `purpose: "login"` — issue a new `Session` and `TokenBundle` with `amr` containing at least `"passkey"`, `acr` defaulting to `"aal2"`.
- `purpose: "step-up"` — locate the producer's existing session, rotate its `amr`/`acr` to reflect the new factor (`amr` adds `"passkey"`, `acr` raises to `"aal2"` or `"aal3"` per consumer policy). The session id and existing tokens persist.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/login/finish/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Echo `payload.authId` from the start response.
3. Populate `payload.credential` with the unmodified `AuthenticatorAssertionResponse`; binary fields base64url-encoded.
4. For `purpose: "step-up"` ceremonies, the producer's transport context MUST carry an authenticated existing session — typically the bearer access token in transport headers, or a framework `proof` whose `verificationMethod` resolves to the existing session's subject. Consumers determine the existing session by either mechanism.

A conforming **consumer** **MUST**:

1. Look up the login ceremony via `payload.authId`. Unknown → `auth_not_found`. Expired → `auth_expired`.
2. Resolve `credential.id` to a registered credential. Unknown → `credential_unknown`.
3. Perform full WebAuthn Level 2 §7.2 assertion verification:
   - Decode `clientDataJSON`; verify `type === "webauthn.get"`, `challenge` matches the bound challenge, `origin` matches the consumer's expected origin.
   - Verify `rpIdHash` in `authenticatorData` matches the consumer's RP ID.
   - Verify the assertion signature with the credential's stored public key.
   - Verify the signature counter is strictly greater than the previously-stored value (`counter_regressed` on regression).
4. For `purpose: "login"`:
   - Identify the subject via `credential.response.userHandle` (or the stored credential→subject mapping when `userHandle` is null).
   - Mint a fresh `Session` with `amr: ["passkey"]`, `acr: "aal2"`. Issue a `TokenBundle`.
5. For `purpose: "step-up"`:
   - Identify the existing session per the transport-context rule above.
   - Verify the assertion's subject (from `userHandle` / credential mapping) equals the existing session's subject. Mismatch → `assertion_invalid:user_handle_mismatch`.
   - Update the session's `amr` to include `"passkey"` (if not already present) and `acr` to `"aal2"` (or higher per consumer policy).
   - Do NOT issue a new `TokenBundle`; the response carries only the updated `session`.
6. Persist the counter update and consume the `authId` so the same assertion cannot be replayed.

## Definitions

* **AuthenticatorAssertionResponse.** WebAuthn dictionary; see [`_shared/0.1/webauthn.schema.json#AssertionResponse`](../../../../_shared/0.1/webauthn.schema.json).
* **Existing session.** For step-up purposes, the session the producer is elevating. Identified by transport-context: `Authorization: Bearer` token, or the document `issuer` when a framework proof is present.

## Payload

`payload.authId` (REQUIRED) — echoed start handle.

`payload.credential` (REQUIRED) — the assertion response.

`payload.ext` (optional) — extension slot.

## Examples

### Successful login finish

```json
{
  "id": "eeeeeeee-5555-6666-7777-888888888888",
  "type": "https://trusttasks.org/spec/auth/passkey/login/finish/0.1",
  "issuer": "did:web:client.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T13:00:30Z",
  "payload": {
    "authId": "auth_3c4d5e6f7890abcd",
    "credential": {
      "id": "Y3JlZF8xYTJiM2M",
      "rawId": "Y3JlZF8xYTJiM2M",
      "type": "public-key",
      "response": {
        "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiUTJoaGJFeGxibWRsVmtGc2RXVkNZWE5sTmpRIn0",
        "authenticatorData": "TXltSXNUaGVBdXRoRGF0YQ",
        "signature": "U2lnbmF0dXJlVmFsdWVCYXNlNjQ",
        "userHandle": "dXNyXzhmMmMxZDRlOWE3YjMwNTY"
      }
    }
  }
}
```

### Successful step-up finish

```json
{
  "id": "ffffffff-6666-7777-8888-999999999999",
  "type": "https://trusttasks.org/spec/auth/passkey/login/finish/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T13:00:30Z",
  "payload": {
    "authId": "auth_step-up_handle",
    "credential": { "…": "…" }
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/login/finish/0.1#response`. The payload includes `purpose` echoed back so the producer can branch unambiguously.

### Successful login response

```json
{
  "id": "00000000-7777-8888-9999-aaaaaaaaaaaa",
  "type": "https://trusttasks.org/spec/auth/passkey/login/finish/0.1#response",
  "threadId": "eeeeeeee-5555-6666-7777-888888888888",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-05-23T13:00:31Z",
  "payload": {
    "purpose": "login",
    "session": {
      "id": "fa7d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T13:00:31Z",
      "expiresAt": "2026-05-23T13:30:31Z",
      "amr": ["passkey"],
      "acr": "aal2"
    },
    "tokens": {
      "accessToken": "eyJhbGciOi…",
      "refreshToken": "rt_passkey_abc",
      "tokenType": "Bearer",
      "expiresIn": 1800
    }
  }
}
```

### Successful step-up response

```json
{
  "id": "11111100-8888-9999-aaaa-bbbbbbbbbbbb",
  "type": "https://trusttasks.org/spec/auth/passkey/login/finish/0.1#response",
  "threadId": "ffffffff-6666-7777-8888-999999999999",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T13:00:31Z",
  "payload": {
    "purpose": "step-up",
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T13:30:31Z",
      "amr": ["did", "passkey"],
      "acr": "aal2"
    }
  }
}
```

## Security & Privacy

**Counter regression.** WebAuthn §6.1.1: an authenticator's signature counter MUST strictly increase. A regressed counter is the canonical "cloned authenticator" indicator. Consumers MUST refuse with `assertion_invalid:counter_regressed` and SHOULD revoke the credential when this fires.

**Cross-flow conflation.** A login ceremony's assertion MUST NOT be accepted on a step-up authId, and vice versa. The consumer's enforcement is via the `purpose` it recorded at start time — re-checking on finish prevents an attacker from harvesting an assertion intended for one purpose and submitting it for the other.

**Step-up session binding.** For step-up, the consumer MUST verify the asserted subject equals the existing session's subject. A user with two registered subjects on the same auth service MUST NOT be able to step-up subject A's session by asserting against subject B's credential.

**Token-bundle issuance scope.** A passkey-login completion issues a new bundle. The consumer's authorization policy decides the scope; if the subject had narrower scopes on a previous (DID-only) session, the new bundle MAY be broader because aal2 unlocks additional capabilities — but it MUST NOT exceed what the consumer's ACL grants for this subject.

The optional `ext` extension is part of the signed surface when a proof is included.
