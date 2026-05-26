---
slug: auth/passkey/enroll/start
version: "0.1"
title: Auth — Passkey Enroll (start)
summary: An authenticated subject asks the auth service to begin a WebAuthn registration ceremony so a new passkey can be bound to their VID.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - enroll
  - registration
  - fido2
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
  - role: Auth service
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: Binding a passkey to a subject is a high-trust assertion. Requiring a verified proof on the start ceremony prevents an opportunistic actor with a captured token from registering a credential they control against the legitimate subject's VID.
errorCodes:
  - code: auth/passkey/enroll/start:max_credentials_reached
    meaning: The subject already has the maximum number of passkeys this auth service is configured to bind. `details.limit` MAY carry the cap.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        limit: { type: integer, minimum: 0 }
  - code: auth/passkey/enroll/start:enrollment_not_supported
    meaning: This auth service does not accept passkey enrollment (for example, a deployment that mandates an external IdP).
    retryable: false
related:
  - auth/passkey/enroll/finish
  - auth/passkey/login/start
  - auth/passkey/login/finish
---

## Abstract

The **Auth — Passkey Enroll (start)** Trust Task is the first leg of a WebAuthn registration ceremony. The subject — already authenticated via `auth/authenticate` or another mechanism — asks the auth service for `PublicKeyCredentialCreationOptions`. The subject's user agent hands the options to `navigator.credentials.create({ publicKey })`, and the resulting attestation is returned via [`auth/passkey/enroll/finish/0.1`](../finish/0.1/spec.md).

The framework `proof` is REQUIRED. The auth service binds the issued `enrollmentId` to the producer's VID server-side; a finish call carrying a credential from a different subject MUST fail.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/enroll/start/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Include a verified `proof`.
3. **MAY** include a `payload.deviceLabel` so the credential is identifiable in subsequent listing / revocation flows.

A conforming **consumer** **MUST**:

1. Verify the `proof` and identify the producer's VID.
2. Generate a fresh server-side `enrollmentId` and bind it to:
   - The producer's VID.
   - The challenge embedded in `options.challenge`.
   - An expiry (RECOMMENDED 5 minutes).
3. Return `PublicKeyCredentialCreationOptions` with:
   - `rp.id` set to the auth service's relying-party identifier.
   - `user.id` set to a stable opaque user handle for the producer's VID (NOT the VID itself, per WebAuthn's privacy guidance — `user.id` SHOULD be a per-RP-per-subject hash or random-mapped identifier).
   - `pubKeyCredParams` containing at least one algorithm the consumer accepts. Ed25519 (`alg: -8`) is RECOMMENDED.
4. Refuse with `auth/passkey/enroll/start:max_credentials_reached` if the subject is at the consumer's credential cap.

## Definitions

* **Subject.** The party enrolling a passkey; identified by `issuer`.
* **Auth service.** The WebAuthn relying party; identified by `recipient`.
* **enrollmentId.** Opaque correlation handle between the start and finish ceremonies; produced by the consumer.
* **PublicKeyCredentialCreationOptions.** WebAuthn dictionary; see [`_shared/0.1/webauthn.schema.json#CredentialCreationOptions`](../../../_shared/0.1/webauthn.schema.json).

## Payload

`payload.deviceLabel` — optional human-readable credential name.

`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Authenticated subject enrolls a new device

```json
{
  "id": "11111111-aaaa-bbbb-cccc-222222222222",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/start/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T12:00:00Z",
  "payload": {
    "deviceLabel": "Alice's MacBook Pro"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/enroll/start/0.1#response`. Payload: `{ enrollmentId, options }`.

### Successful start

```json
{
  "id": "33333333-dddd-eeee-ffff-444444444444",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/start/0.1#response",
  "threadId": "11111111-aaaa-bbbb-cccc-222222222222",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T12:00:01Z",
  "payload": {
    "enrollmentId": "enr_1a2b3c4d5e6f7890",
    "options": {
      "challenge": "Zm9vYmFyYmF6cXV4",
      "rp": { "id": "auth.example", "name": "Auth Example" },
      "user": {
        "id": "dXNyXzhmMmMxZDRlOWE3YjMwNTY",
        "name": "alice",
        "displayName": "Alice"
      },
      "pubKeyCredParams": [
        { "type": "public-key", "alg": -8 },
        { "type": "public-key", "alg": -7 }
      ],
      "timeout": 60000,
      "attestation": "none",
      "authenticatorSelection": {
        "residentKey": "preferred",
        "userVerification": "preferred"
      }
    }
  }
}
```

## Security & Privacy

**user.id privacy.** WebAuthn's `user.id` is visible to roaming authenticators. Consumers MUST NOT use the producer's VID directly here; use a per-RP opaque mapping so a credential exported from the authenticator doesn't leak the VID outside the relying party.

**Challenge binding.** The `options.challenge` MUST be bound server-side to the `enrollmentId` and the producer's VID. A finish call presenting a different challenge MUST fail; this prevents replay of harvested attestations from other ceremonies.

**Algorithm policy.** The consumer's `pubKeyCredParams` ordering signals preference. Consumers SHOULD include only algorithms they accept; producers' authenticators will pick the first match.

**Attestation.** `attestation: "none"` is appropriate for most deployments — privacy-preserving and good enough for binding. Consumers operating under regulatory regimes that require attestation (FIDO2 enterprise contexts) MAY set `"direct"`, but should be aware this reveals authenticator-make/model.

The optional `ext` extension is part of the signed surface.
