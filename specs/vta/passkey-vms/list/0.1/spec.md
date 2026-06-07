---
slug: vta/passkey-vms/list
version: "0.1"
title: VTA Passkey-VM — List
summary: An administrator enumerates the passkey verificationMethods currently published on a VTA-managed DID.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - vta
  - passkey
  - webauthn
  - verification-method
  - did
  - list
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
  rationale: The listing is admin-gated; the VTA MUST attribute the request to a producer holding the admin role on the target DID's context. Transport-independent producer identity prevents a captured request being replayed or attributed to the wrong party, consistent with the rest of the vta/passkey-vms family.
errorCodes:
  - code: vta/passkey-vms/list:didNotFound
    meaning: The target DID is not managed by this VTA.
    retryable: false
related:
  - vta/passkey-vms/enroll-challenge
  - vta/passkey-vms/enroll-submit
  - vta/passkey-vms/revoke
---

## Abstract

The **VTA Passkey-VM — List** Trust Task enumerates the passkey verificationMethods currently published on a VTA-managed DID. A *DID administrator* asks the *VTA* for every `Multikey` verificationMethod on a DID; the VTA returns them as they appear in the DID document. The task is read-only — it mutates no state.

The entries returned here are the same ones published in the DID document, so the data is public. The task exists as an admin-facing management view (e.g. for the browser plugin's passkey-management UI) and to pair with [`vta/passkey-vms/revoke`](../../revoke/0.1/spec.md), which removes a VM by fragment.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/passkey-vms/list/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.did` with the DID to enumerate.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the admin role on the target DID's context, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where the target DID is not managed by this VTA, respond with `vta/passkey-vms/list:didNotFound`.
4. On success, return every passkey verificationMethod on the DID — an empty array when none are enrolled.

## Definitions

* **DID administrator.** The party invoking the task; identified by `issuer`. Holds the admin role on the target DID's context.
* **VTA.** The Verifiable Trust Agent that manages the DID; identified by `recipient`.
* **PasskeyVerificationMethod.** A published `Multikey` verificationMethod; see [`vta/_shared/0.1/passkey-vm`](../../../_shared/0.1/passkey-vm.schema.json).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/list/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### List passkeys on a DID

```json
{
  "id": "4d0f5g7e-1c65-4e34-d063-7f3h5c1e99g5",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/list/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-16T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmcExampleScid:example.com"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-05-16T11:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/list/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is `{ verificationMethods: PasskeyVerificationMethod[] }`. The array is empty when the DID has no passkey verificationMethods.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### A DID with one passkey

Response to the request example:

```json
{
  "id": "5e1g6h8f-2d76-4f45-e174-8g4i6d2f00h6",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/list/0.1#response",
  "threadId": "4d0f5g7e-1c65-4e34-d063-7f3h5c1e99g5",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-16T11:00:01Z",
  "payload": {
    "verificationMethods": [
      {
        "id": "did:webvh:QmcExampleScid:example.com#passkey-3q2r1s0tUvWxYz",
        "type": "Multikey",
        "controller": "did:webvh:QmcExampleScid:example.com",
        "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "webauthnCredentialId": "AQIDBAUGBwgJCgsMDQ4PEA",
        "webauthnTransports": ["internal", "hybrid"],
        "label": "MacBook Touch ID"
      }
    ]
  }
}
```

## Security & Privacy

**Public data, admin-gated access.** The verificationMethods returned here are published in the DID document and are therefore already public — they carry no secret. The admin gate exists to scope the management surface to the DID's controllers, not to protect the data itself. The **REQUIRED** `proof` lets the VTA attribute the request to a specific admin and keeps the family's authorization model uniform.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload; producers **MUST NOT** place data in `ext` that they would not be comfortable signing.
