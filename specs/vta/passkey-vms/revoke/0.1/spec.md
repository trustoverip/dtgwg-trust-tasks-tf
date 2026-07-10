---
slug: vta/passkey-vms/revoke
version: "0.1"
title: VTA Passkey-VM — Revoke
summary: An administrator removes a passkey verificationMethod from a VTA-managed DID document, identified by its URL fragment, via a WebVH log entry.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - vta
  - passkey
  - webauthn
  - verification-method
  - did
  - revoke
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
  rationale: Revocation mutates a DID document (the verificationMethod is removed via a WebVH log entry) and is admin-gated. The VTA MUST attribute the change to a producer holding the admin role on the target DID's context; transport-independent producer identity is required so the removal is non-repudiable and auditable.
sideEffects:
  level: destructive
  rationale: "Removes a passkey verificationMethod from a live DID document via a log entry."
consequences:
  - "Anything relying on that passkey to authenticate stops working once the change resolves."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/passkey-vms/revoke:didNotFound
    meaning: The target DID is not managed by this VTA.
    retryable: false
  - code: vta/passkey-vms/revoke:fragmentNotFound
    meaning: No verificationMethod with the given `fragment` exists on the DID. Nothing was removed.
    retryable: false
related:
  - vta/passkey-vms/enroll-challenge
  - vta/passkey-vms/enroll-submit
  - vta/passkey-vms/list
---

## Abstract

The **VTA Passkey-VM — Revoke** Trust Task removes a passkey `Multikey` verificationMethod from a VTA-managed DID document. A *DID administrator* names the verificationMethod by its URL `fragment`; the *VTA* removes it from the DID document via a WebVH log entry and returns an empty success body. After revocation the DID no longer publishes that key, so any WebAuthn assertion against it will fail to verify for resolvers that have picked up the update.

Use [`vta/passkey-vms/list`](../../list/0.1/spec.md) to discover the `fragment` values currently on a DID.

This task is **idempotent in effect** but not in outcome: revoking a fragment that does not exist yields `vta/passkey-vms/revoke:fragmentNotFound` rather than a silent success, so a caller can tell whether it actually removed anything.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the DID administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/passkey-vms/revoke/0.1`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.did` and `payload.fragment`. `fragment` is the verificationMethod id with the leading `#` removed.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the admin role on the target DID's context, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where the target DID is not managed by this VTA, respond with `vta/passkey-vms/revoke:didNotFound`.
4. Where no verificationMethod with the given `fragment` exists on the DID, respond with `vta/passkey-vms/revoke:fragmentNotFound`.
5. On success, remove the verificationMethod via a WebVH log entry and return the empty success body.

## Definitions

* **DID administrator.** The party invoking the task; identified by `issuer`. Holds the admin role on the target DID's context.
* **VTA.** The Verifiable Trust Agent that manages the DID and applies the WebVH update; identified by `recipient`.
* **Fragment.** The verificationMethod URL fragment — the portion of the VM id after `#` (e.g. `passkey-3q2r1s0tUvWxYz`).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/revoke/0.1` with a payload that validates against the top-level schema in `payload.schema.json`.

### Revoke a passkey by fragment

```json
{
  "id": "6f2h7i9g-3e87-4g56-f285-9h5j7e3g11i7",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/revoke/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-05-16T12:00:00Z",
  "payload": {
    "did": "did:webvh:QmcExampleScid:example.com",
    "fragment": "passkey-3q2r1s0tUvWxYz"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-05-16T12:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/passkey-vms/revoke/0.1#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

The response payload is an **empty object** (`{}`). It is modelled as an object — rather than no payload — so that future additive fields (for example, the resulting WebVH version) can be introduced as a backwards-compatible `MINOR` change.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### Successful revocation

Response to the request example:

```json
{
  "id": "7g3i8j0h-4f98-4h67-g396-0i6k8f4h22j8",
  "type": "https://trusttasks.org/spec/vta/passkey-vms/revoke/0.1#response",
  "threadId": "6f2h7i9g-3e87-4g56-f285-9h5j7e3g11i7",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-16T12:00:01Z",
  "payload": {}
}
```

## Security & Privacy

**Revocation is a security-critical operation.** Removing a passkey verificationMethod is how a compromised or retired authenticator is taken out of service. Because it changes who can authenticate as the DID, it is admin-gated and the **REQUIRED** `proof` makes the removal non-repudiable. The resulting WebVH log entry is the durable audit record.

**Propagation is not instantaneous.** Revocation removes the key from the DID document, but verifiers that have cached an earlier DID-document version may continue to accept assertions against the removed key until they re-resolve. Relying parties **SHOULD** resolve the current DID-document version before honouring a high-value WebAuthn assertion. Revocation is necessary but not by itself sufficient for immediate cut-off.

**Distinguish "not found" from "removed".** A revoke against an absent fragment returns `vta/passkey-vms/revoke:fragmentNotFound` rather than a silent success, so an operator cannot mistake a typo'd fragment for a successful revocation.

The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload; producers **MUST NOT** place data in `ext` that they would not be comfortable signing.
