---
slug: vta/dids/recover
version: "1.0"
title: "VTA DIDs — Recover"
summary: "A super-administrator of a replacement VTA takes over a key-role identity whose VTA was lost: the replacement generates new keys, and the identity's recovery custodian signs the one change that reveals the recovery key, publishes the new keys, and commits fresh successors."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - key roles
  - recovery
  - disaster recovery
  - recovery key
parties:
  - role: Super-administrator of the recovering VTA
    requirement: REQUIRED
    member: issuer
  - role: Recovering VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Recovery moves an identity's change authority to a new VTA. The VTA must attribute the request independently of the transport; the recovery custodian's own proof, carried in the payload, is what authorizes the identity's change."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed recovery would attempt a second takeover with a recovery key the first one used. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Publishes a change made with the recovery key that replaces every key the lost VTA held and the recovery commitment itself. It cannot be withdrawn."
consequences:
  - "This VTA becomes the custodian of the identity; the lost VTA's keys stop being the identity's."
  - "The recovery key is used and replaced by the fresh one supplied."
  - "Every status list must be re-signed at once and other credentials re-issued by their renewal."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Ingests the identity's published history and a fresh recovery key's public half; returns the preview, the change to sign and the new public key records. No private key material."
retention:
  class: durable
  rationale: "The recovery is the start of the identity's custody at this VTA, and part of its key history."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "The context does not exist or the caller cannot reach it."
    retryable: false
  - code: "vta/dids:versionConflict"
    meaning: "The identity has moved past the version the plan was computed against — someone else holds its authority."
    retryable: false
  - code: "vta/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, applied, or for a different request."
    retryable: false
  - code: "vta/dids:stepUpRequired"
    meaning: "Policy requires a step-up bound to this recovery's preview."
    retryable: true
  - code: "vta/dids:unsupportedKeyType"
    meaning: "A key the identity's roles require cannot be minted by this VTA for the role's custody."
    retryable: false
  - code: "vta/dids:methodNotEligible"
    meaning: "The identity's method has no eligible profile."
    retryable: false
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity is not a key-role identity, or commits no recovery key; it cannot be recovered this way."
    retryable: false
  - code: "vta/dids:methodParameterUnsupported"
    meaning: "A `methodParameters` member is not in the profile's `UpdateParameters`."
    retryable: false
  - code: "vta/dids:recoveryCommitmentRequired"
    meaning: "The fresh recovery key is in this VTA's boundary, or its proof of possession does not verify."
    retryable: false
  - code: "vta/dids:recoveryProofInvalid"
    meaning: "The recovery proof does not verify over the previewed change under the committed recovery key."
    retryable: false
  - code: "vta/dids/recover:historyUnavailable"
    meaning: "The identity's history could not be obtained or does not verify."
    retryable: true
  - code: "vta/dids/recover:alreadyHeld"
    meaning: "This VTA already holds the identity; use the key-role tasks instead."
    retryable: false
related:
  - vta/dids/keys/revoke
  - vta/dids/keys/list
  - vtc/credentials/reissue
---

## Abstract

**VTA DIDs — Recover** is how an identity survives the loss of its VTA (VTI-KEY-117). Its
`attestation` and `update` keys were generated in the lost VTA and are in no backup
([conventions §5](../../_shared/0.1/CONVENTIONS.md#5-custody)), so nothing restores them. What
survives is the **recovery key**, committed in every version and held by a custodian outside
the lost VTA. This task is sent to the **replacement** VTA:

1. It obtains and verifies the identity's history, and refuses an identity that commits no
   recovery key.
2. It generates new `attestation` and routine `update` keys — and `operational` and
   `messaging` keys, since the lost VTA's seed is not assumed available — and computes **one
   change** that reveals the recovery key, publishes the new keys in their roles, retires the
   lost `attestation` keys (`lostKeys: destroyed`) or revokes every lost key from
   `compromisedSince` (`lostKeys: mayBeExposed`), and commits a fresh routine successor and the
   fresh recovery key supplied.
3. It answers `awaitingRecoveryProof` with the change for the custodian to check and sign. The
   custodian's proof comes back as `recoveryProof`; the VTA verifies it and publishes.

Afterwards the node re-signs every status list at once and re-issues every other attestation
artefact by its renewal (`followUp`). Whether a VTA is "lost" is not something this task can
establish: a recovery against a VTA that is merely unreachable leaves two VTAs each believing
it holds the identity, and the history moving past the plan's base is answered
`versionConflict`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../_shared/0.1/CONVENTIONS.md) and the governing [method profile](../../_shared/0.1/method-profiles/README.md) are part of this specification.

A conforming **consumer** (the recovering VTA) **MUST**:

1. Verify the identity's history in full, whether supplied in `history` or resolved, and
   refuse one that does not verify, is not a key-role identity, or commits no recovery key.
2. Mint every new key by its role's origin, at preview time, and compute the change as above.
3. Require `compromisedSince` with `lostKeys: mayBeExposed` and refuse it otherwise.
4. Verify the fresh recovery key's proof of possession and refuse a key in its own boundary.
5. Verify `recoveryProof` over exactly the previewed change under the committed recovery key,
   and publish only then; refuse with `versionConflict` if the identity has moved past the
   plan's base.
6. Compute `followUp` where it can (it holds no signing record of the lost VTA, so the counts
   are absent unless the operator supplies the record separately), and audit
   `did.keys.recover`, `did.keys.recoveryRequested` and `did.keys.recoveryProof`.

## Authorization

`super-admin` of the recovering VTA, which is the authority to take an identity into its
custody. That authority does not reach the identity: the recovery custodian's proof does. A
request this VTA authorizes but the custodian does not sign publishes nothing.

## Request

```json
{
  "id": "urn:uuid:4d4d4d4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/recover/1.0",
  "issuer": "did:key:z6MkNewVtaSuperAdmin",
  "recipient": "did:webvh:QmNewVtaScid:vta2.example",
  "issuedAt": "2026-12-01T10:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "contextId": "vtc-example",
    "lostKeys": "destroyed",
    "recoveryCommitment": { "keyType": "ed25519", "publicKeyMultibase": "z6MkRecoveryNext" },
    "dryRun": true,
    "reason": "Primary VTA enclave image lost after KMS key deletion"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-01T10:00:00Z",
    "verificationMethod": "did:key:z6MkNewVtaSuperAdmin#z6MkNewVtaSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z2Wx..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:4d4d4d4d-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/recover/1.0#response",
  "issuer": "did:webvh:QmNewVtaScid:vta2.example",
  "recipient": "did:key:z6MkNewVtaSuperAdmin",
  "issuedAt": "2026-12-01T10:00:03Z",
  "threadId": "urn:uuid:4d4d4d4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "preview",
    "preview": {
      "previewId": "pv_ffeeddccbbaa99887766",
      "method": "webvh",
      "baseVersionId": "9-QmEntryNine",
      "expiresAt": "2026-12-02T10:00:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "useRecoveryKey", "role": "update" },
        { "op": "retireKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttest" },
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkRecAttest", "keyType": "ed25519", "publicKeyMultibase": "z6MkRecAttest", "relationships": ["assertionMethod"] },
        { "op": "commitUpdateKey", "role": "update", "commitment": "QmRoutineFreshHash" },
        { "op": "replaceRecoveryCommitment", "role": "update", "commitment": "QmRecoveryNextHash" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example" },
      "warnings": ["recoveryKeyRevealed"]
    }
  }
}
```

(`changes` and `document` abbreviated.) Applying it answers `awaitingRecoveryProof` with
`preview.recoverySigning`, as [`keys/revoke`](../../keys/revoke/1.0/spec.md) does. Failures use
`trust-task-error`.

## Security & Privacy

### Data carried

The identity's public history and a fresh recovery key's public half in; the preview, the change
to sign and public key records out. No private key material. `reason` is never published.

### Correlation

The recovery is public: the history shows the recovery key used and every key replaced. That is
intended — relying parties need to see that the identity changed custody.

### Retention

The recovering VTA keeps the recovery record and audit rows for the identity's life.

### Consent/purpose

The purpose is to restore an identity's custody after its VTA is gone. The consent surface states
that the recovery key will be used and replaced, which keys are retired or revoked, and what the
node must re-issue.
