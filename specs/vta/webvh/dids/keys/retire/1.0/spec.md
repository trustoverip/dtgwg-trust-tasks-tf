---
slug: vta/webvh/dids/keys/retire
version: "1.0"
title: "VTA WebVH DIDs — Keys Retire"
summary: "An administrator retires one key of a did:webvh by planned retirement — completing a rotation, aborting one, or removing an added key. What the key signed while published stays valid."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - vta
  - webvh
  - key roles
  - retire
  - rotation
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Retiring a key withdraws it from the identity permanently — for the attestation role, the key is destroyed. The VTA must attribute that to one administrator independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Irreversible. Replayed after the operator re-added a key under the same role, a stale retirement is refused by its verificationMethod, but it must still be placeable for the duplicate window of SPEC §7.2 item 11."
sideEffects:
  level: destructive
  rationale: "Irreversible: a retired key never returns, a retired attestation key's private half is destroyed, and under pre-rotation the entry also moves the update key."
consequences:
  - "The key is removed from this DID and can never be used for it again."
  - "A retired attestation key is destroyed. Everything it signed stays valid; it can sign nothing more."
  - "Appending the entry also rotates this DID's update key to its committed successor."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview and the retired key's public record."
retention:
  class: durable
  rationale: "The retired key's record is kept for the DID's lifetime so its past signatures stay attributable."
errorCodes:
  - code: "vta/webvh/dids:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webvh/dids:keyNotFound"
    meaning: "The verification method is not a published key of this DID, not in the stated role, or not part of the stated rotation."
    retryable: false
  - code: "vta/webvh/dids:versionConflict"
    meaning: "The DID's latest entry no longer matches `expectedVersionId`."
    retryable: false
  - code: "vta/webvh/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, already applied, based on an entry the log has moved past, or for a different request."
    retryable: false
  - code: "vta/webvh/dids:stepUpRequired"
    meaning: "The VTA's policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/webvh/dids:wouldEmptyRole"
    meaning: "The key is the role's last active key. Add or rotate in a successor first."
    retryable: false
  - code: "vta/webvh/dids:legacyKeysPresent"
    meaning: "The DID still publishes keys with no role; migrate it first."
    retryable: false
  - code: "vta/webvh/dids/keys/retire:notYetActive"
    meaning: "Completing the rotation would retire the predecessor before its successor has activated — before every cached document can have seen the successor."
    retryable: true
related:
  - vta/webvh/dids/rotate-keys
  - vta/webvh/dids/keys/add
  - vta/webvh/dids/keys/revoke
  - vta/webvh/dids/keys/list
---

## Abstract

**VTA WebVH DIDs — Keys Retire** removes one key from a DID because it is no longer
wanted — not because it is compromised. The key leaves its role's relationship,
`keyRoles` and the document in one entry; what it signed while it was published remains
valid, judged against the DID version current at issuance
([conventions §7](../../../../../_shared/0.3/CONVENTIONS.md#7-judging-material-signed-before-a-change)).

The same act serves three purposes, distinguished by which key is named:

- **Completing a rotation** — retire the predecessor once the overlap has served its
  purpose, earlier than the VTA's `autoRetire` would.
- **Aborting a rotation** — retire the successor; the predecessor, still `active` or
  `retiring`, returns to `active`.
- **Removing an added key** — for example a post-quantum key whose rollout is abandoned.

A key believed compromised **MUST NOT** be retired: a retirement tells every verifier its
past signatures are sound. That is [`keys/revoke`](../../revoke/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../../../_shared/0.3/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse with `vta/webvh/dids:keyNotFound` when `verificationMethod` is not a published
   key of the DID, is not in `role` when `role` is given, or is neither predecessor nor
   successor of `rotationId` when that is given.
2. Refuse with `vta/webvh/dids:wouldEmptyRole` when the key is the role's last `active`
   key. There is no override: a DID whose attestation role is empty can sign no
   credential, and one whose operational role is empty cannot answer a message.
3. Refuse with `vta/webvh/dids/keys/retire:notYetActive` a retirement of a rotation's
   predecessor while its successor is still `staged`. Retiring it then would leave
   verifiers with cached documents unable to check anything the node signs until they
   re-resolve.
4. Remove the key from its relationship, from `keyRoles` and from `verificationMethod` in
   one entry, and mark its record `retired` with the entry's versionId.
5. For an `attestation` key, destroy the private half (VTI-KEY-125) once the entry is
   published, and audit `did.keys.destroy`. For a `messaging` key, keep the private half
   for decryption only, for as long as messages encrypted to it can be in transit
   (conventions §4 item 4).
6. When the key is a rotation's successor, record the rotation `aborted`, and return the
   predecessor to `active`.

## Authorization

As for the role's rotation: context `admin` or `super-admin` for `operational` and
`messaging`; `super-admin` for `attestation`
([conventions §2](../../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).
Update keys are not retired; they move by rotation.

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.verificationMethod` (REQUIRED) — the key to retire.
`payload.role` (OPTIONAL) — guard: the role the caller believes the key holds.
`payload.rotationId` (OPTIONAL) — guard: the rotation this completes or aborts.
`payload.expectedVersionId`, `payload.dryRun`, `payload.previewId`, `payload.reason` — as in [conventions §3](../../../../../_shared/0.3/CONVENTIONS.md#3-approval).
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

### Complete an attestation rotation early

```json
{
  "id": "urn:uuid:3f4e5d6c-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/retire/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-28T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkOldAttestation",
    "role": "attestation",
    "rotationId": "rot-0003",
    "expectedVersionId": "7-QmEntrySeven",
    "previewId": "pv_8a7b6c5d4e3f2a1b0c9d"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-28T09:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:3f4e5d6c-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/retire/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-09-28T09:00:02Z",
  "threadId": "urn:uuid:3f4e5d6c-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "applied",
    "newVersionId": "8-QmEntryEight",
    "keys": [
      {
        "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkOldAttestation",
        "role": "attestation",
        "relationships": [],
        "keyType": "ed25519",
        "publicKeyMultibase": "z6MkOldAttestation",
        "state": "retired",
        "publishedInVersionId": "2-QmEntryTwo",
        "removedInVersionId": "8-QmEntryEight",
        "rotationId": "rot-0003",
        "custody": { "keyId": "vtc-attestation-1", "origin": "internal", "exportable": false, "neverExportable": true, "inBackups": false, "destroyed": true }
      }
    ],
    "rotation": { "rotationId": "rot-0003", "role": "attestation", "kind": "planned", "state": "completed", "endedInVersionId": "8-QmEntryEight" },
    "serverless": false
  }
}
```

## Security & Privacy

### Data carried

The request names a DID and one verification method, with optional guards; the response carries
the retired key's public record and the rotation's new state. No private key material appears in
either direction.

**Retirement is a statement about the past.** It tells verifiers that everything the key signed
while it was published is sound, which is why a key under any suspicion goes to `keys/revoke`
instead; the two produce different records precisely so a verifier can tell them apart.
**Destroying the attestation key is what bounds its history** (VTI-KEY-132): a key that no longer
exists cannot sign after the entry that retired it, so a verifier needs no other evidence of when
a credential it signed was issued. A VTA that kept the key would make that bound false.

### Correlation

The entry publishes when the key left. Nothing about who asked or why is published.

### Retention

The retired key's record is kept, with `destroyed: true` for an attestation key, for as long as
anything it signed may be verified — in practice the DID's lifetime.

### Consent/purpose

The purpose is to end a key's service on the operator's schedule. The consent surface states that
the key can never return and, for an attestation key, that it will be destroyed, and renders the
VTA's preview ([conventions §3](../../../../../_shared/0.3/CONVENTIONS.md#3-approval)).
