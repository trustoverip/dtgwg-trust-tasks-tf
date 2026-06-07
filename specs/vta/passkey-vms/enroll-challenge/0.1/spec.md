---
slug: vta/passkey-vms/enroll-challenge
version: "0.1"
title: VTA Passkey-VM — Enroll Challenge
summary: An administrator of a VTA-managed DID requests a fresh WebAuthn registration challenge so a browser can create a passkey to be published as a verificationMethod on that DID.
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
  - challenge
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
  rationale: The challenge initiates a mutation of a DID document (a passkey verificationMethod will be published on success) and is admin-gated. The VTA MUST attribute the request to a producer holding the admin role on the target DID's context; transport-independent producer identity is required so the request cannot be replayed or attributed to the wrong party.
errorCodes:
  - code: vta/passkey-vms/enroll-challenge:didNotFound
    meaning: The target DID is not managed by this VTA, so no challenge can be issued.
    retryable: false
related:
  - vta/passkey-vms/enroll-submit
  - vta/passkey-vms/list
  - vta/passkey-vms/revoke
---

## Abstract

The **VTA Passkey-VM — Enroll Challenge** Trust Task is step 1 of the two-step ceremony that publishes a WebAuthn passkey as a `Multikey` verificationMethod (purpose `authentication`) on a VTA-managed DID. A *DID administrator* asks the *VTA* for a fresh WebAuthn registration challenge bound to a specific DID; the VTA returns the challenge and the relying-party / user parameters the browser needs to call `navigator.credentials.create`. The administrator's browser runs the WebAuthn ceremony and then completes enrolment with [`vta/passkey-vms/enroll-submit`](../../enroll-submit/0.1/spec.md), echoing the `ceremonyId` returned here.

Once enrolment completes, any verifier that resolves the DID can validate a WebAuthn assertion against the embedded public key — there is no callback to the VTA and no shared secret.

This task is **not idempotent**: each invocation mints a fresh, single-use ceremony. The returned `ceremonyId` is consumed by exactly one [`enroll-submit`](../../enroll-submit/0.1/spec.md) and cannot be reused.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.did` with the DID the new verificationMethod is to be added to.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).
4. Treat the returned `ceremonyId` as a single-use, short-lived secret and present it unchanged to [`enroll-submit`](../../enroll-submit/0.1/spec.md).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the admin role on the target DID's context, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where the target DID is not managed by this VTA, respond with `vta/passkey-vms/enroll-challenge:didNotFound`.
4. Where the passkey feature is not configured on this VTA, respond with the framework's `unavailable` (retryable).
5. On success, mint a fresh single-use ceremony bound to the target DID and the issued `challenge`, and return the WebAuthn registration parameters.

## Definitions

* **DID administrator.** The party invoking the task; identified by `issuer`. Holds the admin role on the target DID's context.
* **VTA.** The Verifiable Trust Agent that manages the DID and runs the WebAuthn ceremony; identified by `recipient`.
* **Ceremony.** A single-use server-side registration state that binds a `challenge`, the target DID, and the eventual [`enroll-submit`](../../enroll-submit/0.1/spec.md) together. Identified by `ceremonyId`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Request a challenge for a DID

```json
{
  "id": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-16T10:00:00Z",
  "payload": {
    "did": "did:webvh:QmcExampleScid:example.com",
    "label": "MacBook Touch ID"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-05-16T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response carries the WebAuthn registration parameters. All byte-valued members (`challenge`, `userHandle`) are base64url-encoded with no padding; the browser decodes them and passes them to `navigator.credentials.create`. The producer **MUST** echo `ceremonyId` unchanged into the subsequent [`enroll-submit`](../../enroll-submit/0.1/spec.md).

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Issued challenge

Response to the request example:

```json
{
  "id": "1a7c2d4b-8f32-4b01-ad30-4c0e2f8b66d2",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/enroll-challenge/0.1#response",
  "threadId": "0d6b1c3a-7e21-4a90-9c2f-3b9d1e7a55c1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-16T10:00:00Z",
  "payload": {
    "ceremonyId": "cer_8Q1m2n3o4p5q6r7s",
    "challenge": "k9Xy2Zr1Tq8sV3wB7nM0pL4eR6uA1cD2fG3hJ5kN8o",
    "rpId": "example.com",
    "rpName": "Example",
    "userHandle": "dXNlci1oYW5kbGUtZXhhbXBsZQ",
    "userName": "did:webvh:QmcExampleScid:example.com",
    "userDisplayName": "MacBook Touch ID",
    "timeoutMs": 120000
  }
}
```

## Security & Privacy

**Single-use, short-lived ceremony.** The `ceremonyId` and `challenge` bind one WebAuthn registration attempt to one DID. The VTA **MUST** reject a re-used or expired ceremony at [`enroll-submit`](../../enroll-submit/0.1/spec.md) (`vta/passkey-vms/enroll-submit:unknownCeremony`). The `challenge` **MUST** carry sufficient entropy (at least 32 random bytes) to make replay infeasible.

**Admin authority is the gate.** The security of the whole family rests on the admin-role check: only an administrator of the target DID's context may add a verificationMethod. The **REQUIRED** `proof` lets the VTA attribute the request to a specific admin key and prevents a captured request being replayed against a different DID.

**Label is producer-supplied.** `label` is informational only and is not authenticated as belonging to any particular device. Consumers **MUST NOT** make trust decisions based on it.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload; producers **MUST NOT** place data in `ext` that they would not be comfortable signing.
