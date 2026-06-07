---
slug: vta/passkey-vms/enroll-submit
version: "0.1"
title: VTA Passkey-VM — Enroll Submit
summary: An administrator submits the WebAuthn registration result for an open ceremony; the VTA re-derives the public key from the attestation, rejects browser tampering, and publishes the passkey as a verificationMethod on the DID.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - vta
  - passkey
  - webauthn
  - verification-method
  - did
  - enrolment
  - attestation
  - webvh
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: DID administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Submission mutates a DID document (a passkey verificationMethod is appended via a WebVH log entry) and is admin-gated. The VTA MUST attribute the change to a producer holding the admin role on the target DID's context; transport-independent producer identity is required so the published change is non-repudiable and auditable.
errorCodes:
  - code: vta/passkey-vms/enroll-submit:unknownCeremony
    meaning: The `ceremonyId` is unknown, has expired, or has already been consumed. Re-running this submission will not succeed; the producer must obtain a fresh challenge.
    retryable: false
  - code: vta/passkey-vms/enroll-submit:ceremonyDidMismatch
    meaning: The submitted `did` does not match the DID bound to the ceremony at challenge time — a cross-DID replay.
    retryable: false
  - code: vta/passkey-vms/enroll-submit:invalidAttestation
    meaning: The WebAuthn attestation could not be parsed or verified, or its credential key could not be converted to a Multikey.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        reason:
          type: string
          enum:
            - unparseable
            - webauthnVerificationFailed
            - unsupportedAlgorithm
  - code: vta/passkey-vms/enroll-submit:publicKeyMismatch
    meaning: The browser-supplied `publicKeyMultibase` does not match the key the VTA re-derived from `attestationObject.authData`. The browser tampered with (or miscomputed) the public key; the submission is rejected.
    retryable: false
  - code: vta/passkey-vms/enroll-submit:alreadyEnrolled
    meaning: A passkey with this `credentialId` is already enrolled on the DID (the derived verificationMethod fragment already exists).
    retryable: false
related:
  - vta/passkey-vms/enroll-challenge
  - vta/passkey-vms/list
  - vta/passkey-vms/revoke
---

## Abstract

The **VTA Passkey-VM — Enroll Submit** Trust Task is step 2 of the two-step ceremony that publishes a WebAuthn passkey as a `Multikey` verificationMethod (purpose `authentication`) on a VTA-managed DID. After running `navigator.credentials.create` against the challenge from [`vta/passkey-vms/enroll-challenge`](../../enroll-challenge/0.1/spec.md), the *DID administrator* submits the WebAuthn registration result. The *VTA* verifies the ceremony, **re-derives the public key from the attestation**, rejects any mismatch with the browser-claimed value, and — on success — appends the verificationMethod to the DID document via a WebVH log entry. It returns the published verificationMethod and the WebVH version that recorded it.

The browser's `publicKeyMultibase` is **advisory only**. The authoritative public key is the one the VTA re-derives from `attestationObject.authData`; a divergence is treated as tampering and rejected (`vta/passkey-vms/enroll-submit:publicKeyMismatch`).

This task is **not idempotent**: it consumes a single-use ceremony. A retry of the same document fails with `vta/passkey-vms/enroll-submit:unknownCeremony` once the ceremony is consumed; to enrol again the producer obtains a fresh challenge.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Echo the `ceremonyId` from [`enroll-challenge`](../../enroll-challenge/0.1/spec.md) unchanged, and set `payload.did` to the same DID used at challenge time.
3. Populate the WebAuthn registration result: `credentialId`, `publicKeyMultibase`, `coseAlgorithm`, `attestationObject`, `clientDataJson`, `authenticatorData`. All byte-valued members are base64url-encoded with no padding.
4. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the admin role on the target DID's context, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where the `ceremonyId` is unknown, expired, or already consumed, respond with `vta/passkey-vms/enroll-submit:unknownCeremony`.
4. Where `payload.did` does not match the DID bound to the ceremony, respond with `vta/passkey-vms/enroll-submit:ceremonyDidMismatch`.
5. Verify the WebAuthn registration against the ceremony `challenge`. On a parse failure, verification failure, or unsupported credential algorithm, respond with `vta/passkey-vms/enroll-submit:invalidAttestation` and set `details.reason`.
6. **Re-derive** the credential public key from `attestationObject.authData` and compare it to the submitted `publicKeyMultibase`. On mismatch, respond with `vta/passkey-vms/enroll-submit:publicKeyMismatch`. The re-derived key — never the submitted one — is published.
7. Where a passkey with this `credentialId` is already enrolled on the DID, respond with `vta/passkey-vms/enroll-submit:alreadyEnrolled`.
8. On success, append the verificationMethod to the DID document via a WebVH log entry and return the published `verificationMethod` and the `webvhVersion`.

## Definitions

* **DID administrator.** The party invoking the task; identified by `issuer`. Holds the admin role on the target DID's context.
* **VTA.** The Verifiable Trust Agent that verifies the ceremony and mutates the DID document; identified by `recipient`.
* **Ceremony.** The single-use server-side registration state opened by [`enroll-challenge`](../../enroll-challenge/0.1/spec.md), identified by `ceremonyId`.
* **PasskeyVerificationMethod.** The published `Multikey` verificationMethod; see [`vta/_shared/0.1/passkey-vm`](../../../_shared/0.1/passkey-vm.schema.json).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Submit a completed WebAuthn registration

```json
{
  "id": "2b8d3e5c-9a43-4c12-be41-5d1f3a9c77e3",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-16T10:00:30Z",
  "threadId": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "payload": {
    "did": "did:webvh:QmcExampleScid:example.com",
    "ceremonyId": "cer_8Q1m2n3o4p5q6r7s",
    "credentialId": "AQIDBAUGBwgJCgsMDQ4PEA",
    "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "coseAlgorithm": -7,
    "attestationObject": "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YVjF...",
    "clientDataJson": "eyJ0eXBlIjoid2ViYXV0aG4uY3JlYXRlIiwiY2hhbGxlbmdlIjoiazlYeTJaIn0",
    "authenticatorData": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MFAAAAAA",
    "transports": ["internal", "hybrid"],
    "label": "MacBook Touch ID"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-05-16T10:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ verificationMethod, webvhVersion }`. The `verificationMethod` is the entry exactly as it now appears in the DID document — its `publicKeyMultibase` is the server-re-derived (authoritative) key. The producer **SHOULD** treat this as the source of truth, since the VTA may have rejected or corrected the browser-supplied value.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful enrolment

Response to the request example:

```json
{
  "id": "3c9e4f6d-0b54-4d23-cf52-6e2g4b0d88f4",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-submit/0.1#response",
  "threadId": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-16T10:00:31Z",
  "payload": {
    "verificationMethod": {
      "id": "did:webvh:QmcExampleScid:example.com#passkey-3q2r1s0tUvWxYz",
      "type": "Multikey",
      "controller": "did:webvh:QmcExampleScid:example.com",
      "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "webauthnCredentialId": "AQIDBAUGBwgJCgsMDQ4PEA",
      "webauthnTransports": ["internal", "hybrid"],
      "label": "MacBook Touch ID"
    },
    "webvhVersion": "3-QmExampleLogEntryHash"
  }
}
```

## Security & Privacy

**Server-side re-derivation is the trust anchor.** The browser-supplied `publicKeyMultibase` is never trusted as authoritative. The VTA re-derives the public key from `attestationObject.authData` and publishes that. A divergence (`vta/passkey-vms/enroll-submit:publicKeyMismatch`) means the browser-side value was tampered with or miscomputed — publishing it would let an attacker bind a key they do not control to the DID. This check is mandatory.

**Single-use ceremony.** `ceremonyId` is consumed by exactly one submission and bound to one DID. A re-used or expired ceremony (`vta/passkey-vms/enroll-submit:unknownCeremony`) and a cross-DID submission (`vta/passkey-vms/enroll-submit:ceremonyDidMismatch`) are both rejected. The WebAuthn `challenge` from the ceremony is bound into `clientDataJSON`, so a replayed registration cannot be retargeted.

**Admin authority and auditability.** Only an administrator of the DID's context may enrol a passkey. The **REQUIRED** `proof` makes the published change non-repudiable; the resulting WebVH log entry is the durable audit record of who added which key and when.

**Published key is public.** The verificationMethod is written into the DID document and is therefore public. It carries no secret. `webauthnCredentialId` and `label` are likewise public; producers **SHOULD NOT** place sensitive information in `label`.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload; producers **MUST NOT** place data in `ext` that they would not be comfortable signing.
