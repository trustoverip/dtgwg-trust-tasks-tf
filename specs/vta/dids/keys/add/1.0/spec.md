---
slug: vta/dids/keys/add
version: "1.0"
title: "VTA DIDs — Keys Add"
summary: "An administrator adds a newly minted key to the attestation, operational or messaging role of a key-role identity — a post-quantum attestation key beside a classical one — under any eligible DID method."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - vta
  - did
  - key roles
  - add key
  - post-quantum
  - proof set
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Adding a key widens what can speak for the identity; for the attestation role, what can sign its credentials. The VTA must attribute the addition independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed addition would add a second key. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Publishes a new key in a role of a public identity, in an append-only history; under did:webvh the change also moves the update key."
consequences:
  - "A new key is published in the role and can sign (or, for messaging, receive) for the identity."
  - "Under did:webvh, the change also moves the identity's update key to its committed successor."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview and the new key's public record. No key material."
retention:
  class: durable
  rationale: "The key's custodian record and the audit row are part of the identity's key history."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:versionConflict"
    meaning: "The identity's latest version no longer matches `expectedVersionId`."
    retryable: false
  - code: "vta/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, applied, based on a superseded version, or for a different request."
    retryable: false
  - code: "vta/dids:stepUpRequired"
    meaning: "Policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/dids:unsupportedKeyType"
    meaning: "The key type cannot serve the role, is not in the VTA's accepted set, or cannot be minted for the role's custody."
    retryable: false
  - code: "vta/dids:rotationInProgress"
    meaning: "The role already has a change staged, overlapping or awaiting approval."
    retryable: false
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "`update` and `authenticator` keys are not added by this task."
    retryable: false
related:
  - vta/dids/keys/list
  - vta/dids/keys/retire
  - vta/dids/keys/rotate
---

## Abstract

**VTA DIDs — Keys Add** puts a newly minted key into the `attestation`, `operational` or
`messaging` role of a key-role identity, beside the keys already there. Its main use is
algorithm agility: an ML-DSA-44 `attestation` key beside an Ed25519 one, so the identity's
credentials carry a proof set a post-quantum verifier can require (VTI-KEY-103, VTI-KEY-104)
while a classical verifier checks the Ed25519 proof
([conventions §14](../../../_shared/0.1/CONVENTIONS.md#14-algorithm-agility-and-post-quantum-keys)).

The key is minted by the role's origin — generated for `attestation`, derived for
`operational` and `messaging` — when the preview is computed, so the approval binds the exact
public key. It is published `active`: an added key replaces nothing, so no verifier relies
on its absence, and the cache horizon does not apply.

`update` keys move by rotation, and `authenticator` keys are enrolled by the passkey
ceremony; both are refused here (`vta/dids:notApplicableToRole`).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../_shared/0.1/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse an identity that is not a key-role identity, and refuse `update` or
   `authenticator`.
2. Refuse a `keyType` that does not suit the role, is not accepted, or that it cannot mint
   for the role's custody (`vta/dids:unsupportedKeyType`), and never substitute one.
3. Mint the key at preview time by the role's origin
   ([§5.1](../../../_shared/0.1/CONVENTIONS.md#51-generated-and-derived-roles-and-why)), publish
   it in exactly the role's relationship and in `keyRoles` under a fragment derived from the
   key, with its custodian record created atomically.
4. Follow [conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-approval), and audit
   `did.keys.add`, including refusals.

## Authorization

Context `admin` or `super-admin` for `operational` and `messaging`; `super-admin` for
`attestation` ([conventions §7](../../../_shared/0.1/CONVENTIONS.md#7-authorization)).

## Request

```json
{
  "id": "urn:uuid:a1d2d000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/keys/add/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-05T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "attestation",
    "keyType": "mldsa44",
    "expectedVersionId": "4-QmEntryFour",
    "dryRun": true,
    "reason": "Post-quantum attestation proofs alongside Ed25519"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-05T09:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z5Ab..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:a1d2d000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/keys/add/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-11-05T09:00:01Z",
  "threadId": "urn:uuid:a1d2d000-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "preview",
    "preview": {
      "previewId": "pv_11aa22bb33cc44dd55ee",
      "method": "webvh",
      "baseVersionId": "4-QmEntryFour",
      "expiresAt": "2026-11-05T10:00:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z2SyPq", "keyType": "mldsa44", "publicKeyMultibase": "z2SyPq", "relationships": ["assertionMethod"] },
        { "op": "rotateUpdateKey", "role": "update" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example", "keyRoles": { "attestation": ["did:webvh:QmVtcScid:vtc.example#z6MkAttest", "did:webvh:QmVtcScid:vtc.example#z2SyPq"] } }
    },
    "keys": [
      { "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z2SyPq", "role": "attestation", "relationships": ["assertionMethod"], "keyType": "mldsa44", "publicKeyMultibase": "z2SyPq", "state": "pending" }
    ]
  }
}
```

(`document` abbreviated.) Failures use `trust-task-error`.

## Security & Privacy

### Data carried

A role and an algorithm in; the preview and the new key's public record out. No key material.

### Correlation

The new key is public. Its fragment is derived from the key, so it reveals nothing about the
deployment.

### Retention

The custodian record and the audit row are kept for the identity's life.

### Consent/purpose

The purpose is to widen a role, usually for algorithm agility. The consent surface renders the
preview, including the update-key movement under did:webvh.
