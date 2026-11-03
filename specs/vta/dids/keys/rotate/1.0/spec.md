---
slug: vta/dids/keys/rotate
version: "1.0"
title: "VTA DIDs — Keys Rotate"
summary: "An administrator rotates one key role of a key-role identity: successors staged, switched in at the cache horizon, predecessors retired after an overlap. For the update role, the authority moves on or the recovery commitment is replaced."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - vta
  - did
  - key roles
  - rotate
  - pre-rotation
  - overlap
  - recovery key
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A rotation changes which keys speak for an identity. The VTA must attribute the change to a specific administrator independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Replayed after a later rotation, a rotation would retire the keys that rotation installed. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Replaces the keys of a role — for attestation, the key that signs the identity's credentials and status lists; for update, the change authority or its recovery commitment. Published changes cannot be withdrawn, and a retired attestation key is destroyed."
consequences:
  - "A new key takes over this role. From its activation the old key is never used again."
  - "When the overlap ends the old key is removed from the identity; a retired attestation or messaging key is destroyed."
  - "Replacing the recovery commitment changes the only key that can recover the identity if its VTA is lost."
  - "Under did:webvh, every change also moves the identity's update key to its committed successor."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Returns the preview, the rotation record and public key records. A recovery-commitment replacement carries the new recovery key's public half. No private key material."
retention:
  class: durable
  rationale: "The rotation record, custodian records and audit rows are the identity's rotation history."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:keyNotFound"
    meaning: "A key named in `replace` is not an active key of the role."
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
    meaning: "The successor's key type cannot serve the role, is not accepted, or cannot be minted for the role's custody."
    retryable: false
  - code: "vta/dids:rotationInProgress"
    meaning: "The role already has a change staged, overlapping or awaiting approval."
    retryable: false
  - code: "vta/dids:recoveryCommitmentRequired"
    meaning: "The replacement recovery key is in this VTA's boundary, or its proof of possession does not verify."
    retryable: false
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "`authenticator` is not rotated; `replace`, `keyType`, `activatesAt`, `overlapUntil` and `autoRetire` are not for `update`; `recoveryCommitment` is only for `update`."
    retryable: false
  - code: "vta/dids/keys/rotate:overlapTooShort"
    meaning: "`activatesAt` is before the publishing change's cache horizon, or `overlapUntil` is not after `activatesAt`."
    retryable: false
  - code: "vta/dids:assuranceLevelMismatch"
    meaning: "`recoveryCommitment` for an identity below level 3."
    retryable: false
  - code: "vta/dids:recoveryProofInvalid"
    meaning: "With `useRecoveryKey`, the recovery proof does not verify over the previewed change under the committed recovery key."
    retryable: false
related:
  - vta/dids/keys/list
  - vta/dids/keys/retire
  - vta/dids/keys/revoke
  - vta/dids/keys/add
  - auth/step-up/approve-request
---

## Abstract

**VTA DIDs — Keys Rotate** is the **planned** rotation of one role of a key-role identity.
It replaces keys believed sound; everything they signed stays valid. A key believed
compromised is not rotated but revoked, with [`keys/revoke`](../../revoke/1.0/spec.md), a
different act with a different record.

For `attestation`, `operational` and `messaging`, a rotation has the four phases of
[conventions §10](../../../_shared/0.1/CONVENTIONS.md#10-overlap-by-role), and this task starts
the first: the successor is published `staged`; at `activatesAt` — no earlier than the
publishing change's cache horizon — the VTA switches to it; at `overlapUntil` the predecessor
is retired (by the VTA when `autoRetire` holds, or by [`keys/retire`](../../retire/1.0/spec.md)).
Publishing before use is what makes the rotation invisible to relying parties (VTI-KEY-122).

For `update` there is no overlap, and two things can be asked for:

- **Move the routine authority** (no `recoveryCommitment`): the authority moves to its
  committed routine successor and a fresh one is committed, as the method profile says. Under
  did:webvh every change does this anyway; under did:webs this is a rotation event, which
  needs the recovery custodian.
- **Replace the recovery commitment** (`recoveryCommitment` given; level 3 only): the
  identity commits to a new recovery key, with its proof of possession, in place of the old.
  This is the one change that, made by an adversary, silently disarms recovery. So the
  current recovery key authorizes it (`useRecoveryKey: true`): the change reveals that key,
  the custodian checks and signs it as for a recovery
  ([conventions §5.6](../../../_shared/0.1/CONVENTIONS.md#56-recovery-custodians)), and a
  witness can verify it from the entry alone. A threshold custodian's t members each
  check, approve and sign it, which is the two-approver rule counted by the witness. Only where
  the recovery key is lost is the replacement authorized by the routine key, and then the
  VTI specification requires a quorum where two approvers exist (VTI-KEY-181).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../_shared/0.1/CONVENTIONS.md) and the governing [method profile](../../../_shared/0.1/method-profiles/README.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Rotate only the keys of `role` — those in `replace`, or every active key of the role —
   and change nothing else except `keyRoles` and what the method requires of the update
   authority.
2. Mint one successor per predecessor at preview time, of `keyType` or else of the
   predecessor's algorithm, by the role's origin, and refuse rather than substitute.
3. Publish each successor `staged` in the role's relationship and `keyRoles`, under a
   fragment derived from the key, with its custodian record created atomically; report the
   cache horizon as `rotation.cacheHorizonAt`; default `activatesAt` to it and refuse an
   earlier one, or an `overlapUntil` not after it (`overlapTooShort`).
4. At `activatesAt`, switch and audit `did.keys.activate`; for `messaging`, keep both
   private halves and decrypt with either until retirement.
5. When `autoRetire` is not false, retire each predecessor at `overlapUntil` as
   `keys/retire` would, under this request's authorization, destroying an `attestation` or
   `messaging` predecessor, and audit `did.keys.autoRetire`.
6. For `update`, refuse the schedule members, `replace` and `keyType`; move the authority as
   the profile says; and with `recoveryCommitment` — refused below level 3
   (`vta/dids:assuranceLevelMismatch`) — verify its proof of possession over the
   `previewId`, refuse a key in its own boundary, and replace the commitment — never
   otherwise. With `useRecoveryKey: true`, compute the change as one the recovery key
   authorizes, answer `awaitingRecoveryProof` with `preview.recoverySigning`, and verify
   `recoveryProof` over exactly that change before publishing
   (`vta/dids:recoveryProofInvalid`); without it, require the quorum of
   [conventions §6.5](../../../_shared/0.1/CONVENTIONS.md#65-what-each-change-requires).
7. Treat a revocation of any key in this rotation as superseding it (rotation `aborted`).
8. Refuse `authenticator`.

## Authorization

Context `admin` or `super-admin` for `operational` and `messaging`; `super-admin` for
`attestation` and `update` ([conventions §7](../../../_shared/0.1/CONVENTIONS.md#7-authorization)).

## Request

### Apply a previewed rotation of a VTC's attestation key

```json
{
  "id": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/keys/rotate/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-06T08:40:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "attestation",
    "overlapUntil": "2026-11-14T09:00:00Z",
    "expectedVersionId": "5-QmEntryFive",
    "previewId": "pv_1b2c3d4e5f60718293a4",
    "reason": "Annual attestation key rotation"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-06T08:40:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

### Replace the recovery commitment

```json
{
  "id": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/vta/dids/keys/rotate/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-06T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "update",
    "recoveryCommitment": { "keyType": "ed25519", "publicKeyMultibase": "z6MkNewRecovery", "proofOfPossession": "z58Rk...", "label": "Board safe, token 3" },
    "expectedVersionId": "5-QmEntryFive",
    "previewId": "pv_99887766554433221100",
    "reason": "Recovery token custodian changed"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-06T09:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z4Gh..."
  }
}
```

## Response

The VTA's policy requires two approvals for an attestation rotation; the initiator's is
recorded and the plan waits for the second.

```json
{
  "id": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/keys/rotate/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-11-06T08:40:01Z",
  "threadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "pendingApproval",
    "approvals": {
      "required": 2,
      "received": 1,
      "approvals": [ { "approver": "did:key:z6MkSuperAdmin", "evidence": "webauthn", "recordedAt": "2026-11-06T08:39:40Z" } ],
      "expiresAt": "2026-11-06T09:40:00Z"
    },
    "preview": {
      "previewId": "pv_1b2c3d4e5f60718293a4",
      "method": "webvh",
      "baseVersionId": "5-QmEntryFive",
      "expiresAt": "2026-11-06T09:40:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttest", "keyType": "ed25519", "publicKeyMultibase": "z6MkNewAttest", "relationships": ["assertionMethod"] },
        { "op": "rotateUpdateKey", "role": "update" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example", "keyRoles": { "attestation": ["did:webvh:QmVtcScid:vtc.example#z6MkAttest", "did:webvh:QmVtcScid:vtc.example#z6MkNewAttest"] } }
    },
    "rotation": {
      "rotationId": "rot-0007",
      "role": "attestation",
      "kind": "planned",
      "state": "pendingApproval",
      "predecessors": ["did:webvh:QmVtcScid:vtc.example#z6MkAttest"],
      "successors": ["did:webvh:QmVtcScid:vtc.example#z6MkNewAttest"],
      "cacheHorizonAt": "2026-11-07T09:41:00Z",
      "activatesAt": "2026-11-07T09:41:00Z",
      "overlapUntil": "2026-11-14T09:00:00Z",
      "autoRetire": true
    }
  }
}
```

(`document` abbreviated.) Failures use `trust-task-error`.

## Security & Privacy

### Data carried

A role and a schedule in, and for a recovery replacement the new recovery key's public half
and possession proof; the preview, rotation record and public key records out. No private
key material.

### Correlation

The changes publish when the role's keys changed. An operator **SHOULD** rotate attestation
keys on a schedule independent of any member's admission, renewal or departure, and not more
often than policy requires (VTI-KEY-153). The recovery key is committed by hash where the
method allows, so replacing it publishes nothing about its custodian.

### Retention

The rotation record and custodian records are kept for the identity's life.

### Consent/purpose

The purpose is to replace sound keys on the operator's schedule. The consent surface renders
the preview and the schedule in words ("the new key is published now, starts signing on
7 November at 09:41 once every cached copy can have seen it, and the old key is removed on
14 November"); for a recovery replacement it names the custodian label and says the old
recovery key will no longer recover the identity.
