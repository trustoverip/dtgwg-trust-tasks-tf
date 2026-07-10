---
slug: auth/passkey/login/start
version: "0.2"
title: Auth — Passkey Login (start)
summary: A party asks an auth service to begin a WebAuthn authentication ceremony — issuing PublicKeyCredentialRequestOptions for either an initial login or an AAL step-up against an existing session.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - login
  - step-up
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
  rationale: A login-start needs no evidentiary value — the cryptographic gate is the assertion submitted at finish. For step-up purposes a proof MAY be required by consumer policy so the assertion is bindable to a specific session, but the framework treats this as a consumer concern.
sideEffects:
  level: none
  rationale: "Begins a WebAuthn authentication ceremony and returns options; no state change."
subjectPath: /subject
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/passkey/login/start:subjectNotRecognized
    meaning: A named `subject` is not registered with this auth service.
    retryable: false
  - code: auth/passkey/login/start:noCredentials
    meaning: The named subject has no enrolled passkeys.
    retryable: false
  - code: auth/passkey/login/start:rateLimited
    meaning: The producer has exceeded the issuer's login-start budget.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/passkey/login/finish
  - auth/passkey/enroll/start
  - auth/challenge
  - auth/step-up/approve-request
---

## Abstract

The **Auth — Passkey Login (start)** Trust Task is the first leg of a WebAuthn assertion ceremony. The producer asks the auth service for `PublicKeyCredentialRequestOptions`; the user agent hands them to `navigator.credentials.get({ publicKey })`, and the resulting assertion is returned via [`auth/passkey/login/finish/0.1`](../finish/0.1/spec.md).

This task serves two semantically-distinct flows, distinguished by `payload.purpose`:

- **`login`** — issue a fresh session at AAL ≥ 2 on the matching finish.
- **`stepUp`** — elevate the producer's existing session's `acr` on the matching finish, without rotating its `id` or `subject`.

A consumer that does not support step-up MAY refuse `purpose: "stepUp"` and respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/login/start/0.2`, with itself as `issuer` and the auth service as `recipient`.
2. **MAY** populate `payload.subject` — when present, the consumer SHOULD scope `allowCredentials` to that subject's enrolled credentials. When omitted, the consumer SHOULD issue a *discoverable-credentials* assertion (empty or absent `allowCredentials`).
3. **MAY** populate `payload.purpose` (defaulting to `"login"`).

A conforming **consumer** **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements).
2. Generate a fresh server-side `authId` and bind it to:
   - The challenge embedded in `options.challenge`.
   - The named subject (when present).
   - The declared `purpose`.
   - An expiry (RECOMMENDED 5 min).
3. Return `PublicKeyCredentialRequestOptions` with:
   - `rpId` set to the auth service's relying-party identifier.
   - `challenge` containing ≥128 bits of entropy.
   - `allowCredentials` populated with the named subject's enrolled credentials, OR empty when the producer requested a discoverable flow.
4. Refuse with `subjectNotRecognized` / `noCredentials` / `rateLimited` per the table above.

## Definitions

* **Producer.** The user agent initiating login; may be the subject themselves or an agent acting on their behalf. The actual subject is determined at finish time by `userHandle` in the WebAuthn assertion.
* **Auth service.** The WebAuthn relying party; identified by `recipient`.
* **authId.** Opaque correlation handle between start and finish.

## Payload

`payload.subject` (optional) — known target VID.

`payload.purpose` (optional, default `"login"`) — `login` or `stepUp`.

`payload.ext` (optional) — extension slot.

## Examples

### Username-first login

```json
{
  "id": "aaaaaaaa-1111-2222-3333-444444444444",
  "type": "https://trusttasks.org/spec/auth/passkey/login/start/0.2",
  "issuer": "did:web:client.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T13:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "purpose": "login"
  }
}
```

### Discoverable-credential login (no subject)

```json
{
  "id": "bbbbbbbb-2222-3333-4444-555555555555",
  "type": "https://trusttasks.org/spec/auth/passkey/login/start/0.2",
  "issuer": "did:web:client.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T13:00:00Z",
  "payload": {
    "purpose": "login"
  }
}
```

### Step-up on an existing session

```json
{
  "id": "cccccccc-3333-4444-5555-666666666666",
  "type": "https://trusttasks.org/spec/auth/passkey/login/start/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T13:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "purpose": "stepUp"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/login/start/0.2#response`. Payload: `{ authId, options }`.

### Successful start

```json
{
  "id": "dddddddd-4444-5555-6666-777777777777",
  "type": "https://trusttasks.org/spec/auth/passkey/login/start/0.2#response",
  "threadId": "aaaaaaaa-1111-2222-3333-444444444444",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:client.example",
  "issuedAt": "2026-05-23T13:00:01Z",
  "payload": {
    "authId": "auth_3c4d5e6f7890abcd",
    "options": {
      "challenge": "Q2hhbExlbmdlVmFsdWVCYXNlNjQ",
      "timeout": 60000,
      "rpId": "auth.example",
      "allowCredentials": [
        { "type": "public-key", "id": "Y3JlZF8xYTJiM2M" }
      ],
      "userVerification": "preferred"
    }
  }
}
```

## Security & Privacy

**Subject enumeration.** Returning `noCredentials` reveals that a subject is registered but has no passkeys. Consumers operating in environments where membership is sensitive SHOULD substitute a generic `rateLimited` error for both `subjectNotRecognized` and `noCredentials`, distinguishing only in audit logs.

**Discoverable-credential UX.** A start with no `subject` causes the authenticator to surface its account picker. This is the recommended UX for first-time users but leaks the set of registered VIDs to anyone with platform-authenticator access (e.g. a shared device). Operators MAY default to username-first when their threat model includes hostile co-tenants.

**Step-up trust boundary.** When `purpose: "stepUp"` is declared, the consumer MUST verify on the matching finish that the assertion came from a credential bound to the same subject as the existing session. A `userHandle` mismatch is a hard failure.

The optional `ext` extension is part of the signed surface when a proof is included.
