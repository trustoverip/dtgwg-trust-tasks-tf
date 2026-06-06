---
slug: auth/passkey/enroll/finish
version: "0.1"
title: Auth — Passkey Enroll (finish)
summary: A subject submits the WebAuthn attestation that completes a passkey enrollment. The auth service verifies the attestation and binds the public credential to the subject's VID.
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
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The producer is asserting "I, $subject, control a fresh passkey that should be bound to my VID for future authentication." The framework proof ties that assertion to the same key that signed the matching start.
errorCodes:
  - code: auth/passkey/enroll/finish:enrollment_not_found
    meaning: The `enrollmentId` does not refer to any active enrollment ceremony.
    retryable: false
  - code: auth/passkey/enroll/finish:enrollment_expired
    meaning: The enrollment's start-time expiry has elapsed.
    retryable: true
  - code: auth/passkey/enroll/finish:subject_mismatch
    meaning: The producer's VID differs from the VID the start ceremony was issued to.
    retryable: false
  - code: auth/passkey/enroll/finish:attestation_invalid
    meaning: The WebAuthn attestation failed verification (challenge mismatch, signature failure, unsupported algorithm, etc.). `details.reason` carries a machine-readable hint.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum:
            ["challenge_mismatch", "origin_mismatch", "rp_id_mismatch", "signature_invalid", "algorithm_unsupported", "attestation_format_invalid"]
related:
  - auth/passkey/enroll/start
  - auth/passkey/login/start
  - auth/passkey/login/finish
---

## Abstract

The **Auth — Passkey Enroll (finish)** Trust Task completes the WebAuthn registration ceremony started by [`auth/passkey/enroll/start/0.1`](../start/0.1/spec.md). The subject submits the `AuthenticatorAttestationResponse` returned by `navigator.credentials.create`; the auth service verifies the attestation per WebAuthn Level 2 §7.1 and binds the resulting public credential to the subject's VID.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/enroll/finish/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Echo `payload.enrollmentId` verbatim from the start response.
3. Populate `payload.credential` with the unmodified `AuthenticatorAttestationResponse` returned by `navigator.credentials.create`; binary fields are base64url-encoded.
4. Include a verified `proof` whose `verificationMethod` resolves to the same VID the start ceremony was bound to.

A conforming **consumer** **MUST**:

1. Verify the document's `proof`.
2. Look up the enrollment server-side via `payload.enrollmentId`. Unknown → `enrollment_not_found`. Expired → `enrollment_expired`. Mismatched subject → `subject_mismatch`.
3. Perform full WebAuthn Level 2 §7.1 attestation verification:
   - Decode `clientDataJSON`; verify `type === "webauthn.create"`, `challenge` matches the bound challenge, `origin` matches the consumer's expected origin.
   - Verify `rpIdHash` in `authData` matches the consumer's RP ID.
   - Verify the attestation signature per the format declared in `attestationObject`.
   - Verify the credential public key algorithm is in the start ceremony's accepted `pubKeyCredParams`.
4. On any verification step failure, respond with `attestation_invalid` and `details.reason` set to the specific gate that failed.
5. Persist the credential (id, public key, counter, subject VID, deviceLabel) and consume the enrollment record so the same `enrollmentId` cannot be replayed.

## Definitions

* **AuthenticatorAttestationResponse.** WebAuthn dictionary; see [`_shared/0.1/webauthn.schema.json#AttestationResponse`](../../../_shared/0.1/webauthn.schema.json).
* **credentialId.** The opaque per-credential identifier the consumer persists and surfaces in later list / revoke operations.

## Payload

`payload.enrollmentId` (REQUIRED) — echoed start handle.

`payload.credential` (REQUIRED) — the attestation response.

`payload.deviceLabel` (optional) — overrides the start-time label.

`payload.ext` (optional) — extension slot.

## Examples

### Successful finish submission

```json
{
  "id": "55555555-eeee-ffff-aaaa-666666666666",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/finish/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T12:01:00Z",
  "payload": {
    "enrollmentId": "enr_1a2b3c4d5e6f7890",
    "credential": {
      "id": "Y3JlZF8xYTJiM2M",
      "rawId": "Y3JlZF8xYTJiM2M",
      "type": "public-key",
      "response": {
        "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uY3JlYXRlIiwiY2hhbGxlbmdlIjoiWm05dlltRnlZbUY2Y1hWNCJ9",
        "attestationObject": "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YVijSZYN…"
      }
    }
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/enroll/finish/0.1#response`. Payload: `{ credentialId, subject, deviceLabel?, registeredAt }`.

### Successful enrollment

```json
{
  "id": "77777777-aaaa-bbbb-cccc-888888888888",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/finish/0.1#response",
  "threadId": "55555555-eeee-ffff-aaaa-666666666666",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T12:01:01Z",
  "payload": {
    "credentialId": "Y3JlZF8xYTJiM2M",
    "subject": "did:web:alice.example",
    "deviceLabel": "Alice's MacBook Pro",
    "registeredAt": "2026-05-23T12:01:01Z"
  }
}
```

## Security & Privacy

**Verification completeness.** Skipping any WebAuthn §7.1 step is a real-world vulnerability. The `attestation_invalid:reason` enum exists so a consumer can be explicit about which gate refused — implementations SHOULD log the specific gate (not surface it to producers when the info would help a phishing operator).

**Counter handling.** The consumer SHOULD record the authenticator's signature counter at registration time and require it to strictly increase on subsequent assertions, per WebAuthn §6.1.1. A non-increasing counter is a strong cloning indicator; consumers SHOULD treat it as cause to revoke the credential.

**Origin pinning.** `clientDataJSON.origin` MUST match the consumer's expected origin (or one of a configured allow-list for multi-tenant deployments). Mismatch indicates a relay attack — the credential was likely created against a phishing site.

**Replay.** Consuming the `enrollmentId` server-side prevents the same attestation from being submitted twice. A consumer that allows multi-submit "for idempotency" opens a window for a stale attestation to bind a credential after the legitimate ceremony was abandoned.

The optional `ext` extension is part of the signed surface.
