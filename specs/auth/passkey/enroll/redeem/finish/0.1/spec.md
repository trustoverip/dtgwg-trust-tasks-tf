---
slug: auth/passkey/enroll/redeem/finish
version: "0.1"
title: Auth — Passkey Enroll (redeem, finish)
summary: The invitee returns the new credential's attestation — and an assertion from an existing credential of the same purpose, where the subject has one — and the auth service binds the credential to the invite's subject, with the invite's purpose, and consumes the invite.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - enrollment
  - invite
  - redeem
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Invitee
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "As for the start: the authorisation is the invite, bound server-side to the enrollmentId, and — where the subject already has a credential of the purpose — a user-verified assertion from it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed finish would try to bind an authenticator after the invite was meant to be spent. The enrollmentId is consumed on success; placing the request in time bounds the rest."
sideEffects:
  level: mutating
  rationale: "Binds a new passkey credential to the invite's subject and consumes the invite; revocable with auth/passkey/revoke."
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: "The request carries an attestation and possibly an assertion; the response returns the credential id, subject, purpose and label."
retention:
  class: durable
  rationale: "The credential is kept until revoked, and the redemption is part of the audit record of how it came to exist."
errorCodes:
  - code: auth/passkey/enroll/redeem/finish:enrollmentNotFound
    meaning: "The enrollmentId refers to no active redemption ceremony, or its invite was consumed or invalidated meanwhile."
    retryable: false
  - code: auth/passkey/enroll/redeem/finish:enrollmentExpired
    meaning: "The ceremony's expiry has elapsed. Start again while the invite is still valid."
    retryable: true
  - code: auth/passkey/enroll/redeem/finish:attestationInvalid
    meaning: "The attestation failed WebAuthn registration verification."
    retryable: false
  - code: auth/passkey/enroll/redeem/finish:userVerificationFailed
    meaning: "The start required an assertion from an existing credential and it was missing, did not verify, or did not carry the UV flag."
    retryable: true
related:
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/redeem/start
  - auth/passkey/revoke/start
---

## Abstract

The second leg of redeeming an [`auth/passkey/enroll/invite`](../../../invite/0.2/spec.md). The invitee returns the result of `navigator.credentials.create` over the start's `options`, and of `navigator.credentials.get` over its `uvOptions` when there were any. The auth service binds the credential to the invite's subject with the invite's purpose, and consumes the invite.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorisation evidence is **the redemption ceremony named by `enrollmentId`, and the still-valid invite it was started for**, plus — exactly when the start returned `uvOptions` — a user-verified assertion from one of the subject's existing credentials of the same purpose.

A conforming **consumer** **MUST**:

1. Look up the ceremony by `enrollmentId`. Unknown, or its invite consumed or invalidated since → `enrollmentNotFound`; past its expiry → `enrollmentExpired`.
2. Where the start returned `uvOptions`, verify `uvCredential` against them, requiring the UV flag and a credential of the subject's of the same purpose → `userVerificationFailed` otherwise, and a missing `uvCredential` is a failure, never consent.
3. Verify the attestation as WebAuthn Level 2 §7.1 requires → `attestationInvalid` otherwise.
4. Persist the credential bound to the invite's subject with the invite's purpose — a `stepUp` credential where a login ceremony never reads it ([`enroll/invite` 0.2](../../../invite/0.2/spec.md#a-stepup-credential)) — and consume the invite and the ceremony, **atomically** with the persist.
5. Record an audit event naming the subject, the purpose, the credential id and the inviting administrator.
6. **SHOULD** tell the subject, over a channel it already has with them, that a credential was bound to their VID, so an enrolment they did not make does not go unnoticed.

## Payload

`payload.enrollmentId` (REQUIRED), `payload.credential` (REQUIRED), `payload.uvCredential` (REQUIRED when the start returned `uvOptions`), `payload.deviceLabel`, `payload.ext`.

## Response

`{ credentialId, subject, purpose, deviceLabel?, registeredAt }`, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

## Security & Privacy

**Consumption is atomic with binding.** An invite that stayed redeemable after a credential was bound would let a second authenticator follow the first on the same two messages.

**Counter and origin.** As for [`auth/passkey/enroll/finish`](../../../finish/0.2/spec.md): record the signature counter, and require `clientDataJSON.origin` to be the auth service's own.

**Free text.** `deviceLabel` is free text bounded at 256 characters, authored by the invitee and untrusted; it is retained with the credential and shown to whoever may list or revoke it.

### Data carried

The request carries the new credential's attestation and, where required, an assertion from an existing credential. The response names the credential id, subject, purpose and label.

### Correlation

The credential id is stable and names the subject's authenticator to this auth service; it is not disclosed to anyone else. A `none` attestation is RECOMMENDED so the authenticator's make and model are not collected without need.

### Retention

The credential is kept until revoked; the redemption's audit record is durable.

### Consent/purpose

The credential is used only for its purpose — for `stepUp`, only as the operation-bound gesture of [`enroll/invite` 0.2](../../../invite/0.2/spec.md#a-stepup-credential), rule 2.
