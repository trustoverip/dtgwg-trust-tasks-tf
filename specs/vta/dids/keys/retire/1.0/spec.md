---
slug: vta/dids/keys/retire
version: "1.0"
title: "VTA DIDs — Keys Retire"
summary: "An administrator retires a key of a key-role identity on schedule under any eligible DID method — completing a rotation by retiring the predecessor or aborting it by retiring the successor; a retired attestation or messaging key is destroyed."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - vta
  - did
  - key roles
  - retire
  - rotation
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Retiring a key removes it from an identity and, for the attestation and messaging roles, destroys it. The VTA must attribute that independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed retirement could remove a successor that a later rotation published. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Removes a key from a public identity in an append-only history and destroys a retired attestation or messaging key, which can never sign or decrypt again."
consequences:
  - "The key is removed from the identity and never used again."
  - "A retired attestation or messaging key is destroyed; what an attestation key signed stays valid."
  - "Under did:webvh, the change also moves the identity's update key to its committed successor."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview and the affected key records. No key material."
retention:
  class: durable
  rationale: "The retirement is part of the identity's key history, and bounds what a destroyed attestation key can have signed (VTI-KEY-132)."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:keyNotFound"
    meaning: "The key is not a published key of the identity, not in the stated role, or not in the named rotation."
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
  - code: "vta/dids:wouldEmptyRole"
    meaning: "Retiring the key would leave its role with no active key."
    retryable: false
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "`update` keys move by rotation and `authenticator` keys by the passkey ceremony; neither is retired here."
    retryable: false
  - code: "vta/dids/keys/retire:notYetActive"
    meaning: "Retiring the predecessor now would end a planned overlap before the cache horizon has passed and the successor has switched in."
    retryable: true
related:
  - vta/dids/keys/rotate
  - vta/dids/keys/list
  - vta/dids/keys/revoke
---

## Abstract

**VTA DIDs — Keys Retire** removes a sound key from its role on schedule. It completes a
rotation when it retires the predecessor — no earlier than the cache horizon of the change
that published the successor, and after the successor has switched in
([conventions §10.2](../../../_shared/0.1/CONVENTIONS.md#102-phases-of-a-planned-rotation)) —
and aborts one when it retires the successor, which never has to wait because nobody has been
asked to rely on that key alone. It also removes a key `keys/add` added.

A retired key leaves its relationship, `keyRoles` and `verificationMethod`. A retired
`attestation` or `messaging` key is **destroyed** when the change is published (VTI-KEY-125):
for an attestation key that is what lets the retirement bound the issuance time of
everything it signed (VTI-KEY-132). A key believed compromised is never retired; it is revoked
with [`keys/revoke`](../../revoke/1.0/spec.md), which records the compromise time — a retirement
records none, and a verifier will go on accepting what the key signed.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../_shared/0.1/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse `update` and `authenticator` keys, and a key not in the stated `role` or
   `rotationId`.
2. Refuse to retire a predecessor before the overlap rules allow (`notYetActive`), and refuse
   to leave a role empty (`wouldEmptyRole`).
3. Remove the key from its relationship, `keyRoles` and `verificationMethod`; it **MUST NOT**
   set `revoked` on it ([conventions §9.1](../../../_shared/0.1/CONVENTIONS.md#91-how-a-revocation-is-recorded)).
4. Destroy a retired `attestation` or `messaging` key's private half when the change is
   published, keep its record with `destroyed: true`, and audit `did.keys.retire` and
   `did.keys.destroy`.
5. Follow [conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-approval).

## Authorization

As [`keys/rotate`](../../rotate/1.0/spec.md): context `admin` for `operational` and
`messaging`, `super-admin` for `attestation`.

## Request

```json
{
  "id": "urn:uuid:0f1e2d3c-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/keys/retire/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-14T09:05:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkAttest",
    "role": "attestation",
    "rotationId": "rot-0007",
    "expectedVersionId": "6-QmEntrySix",
    "previewId": "pv_aa00bb11cc22dd33ee44"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-14T09:05:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z2Lm..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:0f1e2d3c-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/keys/retire/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-11-14T09:05:02Z",
  "threadId": "urn:uuid:0f1e2d3c-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "applied",
    "newVersionId": "7-QmEntrySeven",
    "keys": [
      { "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkAttest", "role": "attestation", "relationships": [], "keyType": "ed25519", "publicKeyMultibase": "z6MkAttest", "state": "retired", "removedInVersionId": "7-QmEntrySeven" }
    ],
    "publication": { "selfPublish": false }
  }
}
```

Failures use `trust-task-error`.

## Security & Privacy

### Data carried

A key identifier in; key records out. No key material.

### Correlation

The retiring change publishes when a key left the identity, which the history reveals anyway.

### Retention

The key's record survives its destruction, so its past signatures stay attributable.

### Consent/purpose

The purpose is to end a key's service on schedule. The consent surface states that a retired
attestation key is destroyed and that what it signed stays valid.
