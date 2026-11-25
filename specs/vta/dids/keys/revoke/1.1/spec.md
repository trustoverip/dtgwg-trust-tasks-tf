---
slug: vta/dids/keys/revoke
version: "1.1"
title: "VTA DIDs — Keys Revoke"
summary: "An administrator revokes a key of a key-role identity for compromise: no overlap, the compromise time published, a replacement where needed, and for the update role a move to the routine successor or, if the VTA may be compromised, to the recovery key."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - vta
  - did
  - key roles
  - revoke
  - compromise
  - incident response
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
  rationale: "A revocation tells every verifier to stop trusting a key's signatures from a stated instant, and for an attestation key obliges the node to re-issue its credentials. An unattributable revocation is a denial of service with no record of who caused it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Irreversible, and `compromisedSince` is bounded by it: a compromise time later than the request's own issuedAt is refused, which is only checkable if issuedAt is present."
sideEffects:
  level: destructive
  rationale: "Irreversible and authority-shifting: the key is withdrawn with no overlap, verifiers are told to distrust what it signed from the compromise time, replacements may be published, and for the update role the change authority moves, possibly to the recovery key."
consequences:
  - "The key stops being trusted immediately — there is no overlap."
  - "Verifiers are told to distrust anything the key signed from the compromise time onward, unless there is independent evidence it was signed earlier."
  - "Revoking an attestation key obliges re-signing every status list at once and re-issuing every credential still in force that it signed."
  - "Revoking the update authority with scope `boundary` needs the recovery custodian to sign, and replaces the recovery key."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Returns the preview, the affected key records, and for a boundary compromise the change the recovery custodian must sign. The request may carry a fresh recovery key's public half and the custodian's proof. No private key material."
retention:
  class: durable
  rationale: "The revocation and its compromise time are what every later judgement of the key's signatures rests on (VTI-KEY-130 to VTI-KEY-133)."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:keyNotFound"
    meaning: "The key is not a published key of the identity in the stated role."
    retryable: false
  - code: "vta/dids:versionConflict"
    meaning: "The identity's latest version no longer matches `expectedVersionId`."
    retryable: false
  - code: "vta/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, applied, based on a superseded version, or for a different request."
    retryable: false
  - code: "vta/dids:unsupportedKeyType"
    meaning: "The replacement's key type cannot serve the role, is not accepted, or cannot be minted."
    retryable: false
  - code: "vta/dids:recoveryCommitmentRequired"
    meaning: "`scope: boundary` without a fresh recovery key, or one in this VTA's boundary, or whose proof of possession does not verify."
    retryable: false
  - code: "vta/dids:recoveryProofInvalid"
    meaning: "The recovery proof does not verify over the previewed change under the committed recovery key."
    retryable: false
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "A member was given that the role does not take: `verificationMethod` or `replacement` for `update`, `scope` or `recoveryCommitment` for any other role."
    retryable: false
  - code: "vta/dids/keys/revoke:compromiseTimeInFuture"
    meaning: "`compromisedSince` is later than the request's `issuedAt`."
    retryable: false
  - code: "vta/dids/keys/revoke:noSoundSuccessor"
    meaning: "`scope: key` was asked for, but no uncompromised routine successor is committed; use `scope: boundary`, or deactivate the identity if the recovery key is also lost."
    retryable: false
  - code: "vta/dids:assuranceLevelMismatch"
    meaning: "`scope: boundary` for an identity below level 3, which has no recovery key: deactivate it and create a successor identity."
    retryable: false
related:
  - vta/dids/keys/list
  - vta/dids/keys/retire
  - vta/dids/recover
  - vtc/credentials/reissue
  - auth/step-up/approve-request
  - auth/step-up/approve-response
  - trust-task-next-step
---

## Abstract

**VTA DIDs — Keys Revoke** withdraws a key believed to be in someone else's hands. It differs
from [`keys/retire`](../../retire/1.1/spec.md) in everything a verifier relies on
([conventions §9](../../../_shared/0.1/CONVENTIONS.md#9-compromise)):

- **No overlap.** The key leaves its relationship and `keyRoles` in one change
  (VTI-KEY-123), and the VTA stops using it on receipt of an authorized request.
- **The compromise time is published.** The verification method stays in
  `verificationMethod` with `revoked` set to `compromisedSince`, for the identity's life, so a
  verifier resolving the current document sees that the key was withdrawn for cause and from
  when. A planned retirement removes the method instead, which is how the two are told apart.
- **Never blocked.** A revocation supersedes any pending, staged or overlapping change to the
  same role, and the VTI specification forbids a policy that makes it wait for a quorum
  (VTI-KEY-182).
- **Emergency replacement.** Where the role would otherwise be empty, or the caller asks, a
  replacement is minted and published `active` in the same change.

For the `update` role, `scope` says how far the compromise reaches
([§9.4](../../../_shared/0.1/CONVENTIONS.md#94-compromise-of-the-change-authority)):

- `key` — one routine key; the authority moves to the committed routine successor.
- `boundary` — level 3 only. The VTA itself may be compromised, so its routine successor is
  too. (Below level 3 there is no recovery key: the request is refused with
  `vta/dids:assuranceLevelMismatch`, and the identity is to be deactivated and replaced,
  [conventions §9.4](../../../_shared/0.1/CONVENTIONS.md#94-compromise-of-the-change-authority).) The VTA
  computes one change revealing the recovery key, revoking the routine keys, replacing every
  key the boundary held, and committing a fresh routine successor and the fresh recovery key
  the caller supplies; it answers `awaitingRecoveryProof` with the change for the recovery
  custodian to check and sign. The signed proof comes back as `recoveryProof`.

An `attestation` revocation's response carries `followUp`: the status lists to re-sign at
once and the artefacts to re-issue (VTI-KEY-133; [`vtc/credentials/reissue`](../../../../../vtc/credentials/reissue/0.1/spec.md)).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../_shared/0.1/CONVENTIONS.md) and the governing [method profile](../../../_shared/0.1/method-profiles/README.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Stop using the key on receipt of an authorized request, before the change is published.
2. Refuse a `compromisedSince` later than `issuedAt`.
3. Remove the key from its relationship and `keyRoles`, keep it in `verificationMethod` with
   `revoked` set to `compromisedSince`, and keep it there in every later version.
4. Mint a replacement where required, never from a path derivable from what the compromise
   exposed; where the compromise may have exposed the seed, generate replacements of every
   derived role (§9.3).
5. For `update` with `scope: key`, move to the routine successor, or refuse with
   `noSoundSuccessor` when none is sound. With `scope: boundary`, require
   `recoveryCommitment` or, for a threshold custodian, `thresholdRecoveryCommitment` —
   exactly one ([conventions §5.7](../../../_shared/0.1/CONVENTIONS.md#57-threshold-custodians)) — compute the change, answer `awaitingRecoveryProof` with
   `preview.recoverySigning`, and on the apply request verify `recoveryProof` over exactly
   that change under the committed recovery key before publishing
   (`vta/dids:recoveryProofInvalid` otherwise).
6. Abort any change to the same role in progress, and never wait on a quorum.
7. For `attestation`, compute `followUp` from its signing record and warn
   `attestationReissuanceRequired`; never destroy a revoked key's record.
8. Audit `did.keys.revoke` (and `did.keys.recoveryRequested`, `did.keys.recoveryProof`),
   including refusals.

### A step-up hold is a next step

This version answers a step-up hold as [SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs)
requires: with a *next step*, never an *error response*. It replaces the second paragraph of
[conventions §6.2](../../../_shared/0.1/CONVENTIONS.md#62-step-up-is-bound-to-the-preview), which refused the request
with `vta/dids:stepUpRequired`; the rest of §6.2 applies unchanged, so the step-up is still
bound, and bound to the `previewId`. A conforming consumer (the VTA):

1. **MUST** answer a request whose `previewId` names a live plan, and which its policy holds
   for a bound step-up it does not yet have, with a
   [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md) document whose `continuation` is `resubmit` and
   whose `expects` has one entry: `typeUri`
   `https://trusttasks.org/spec/auth/step-up/approve-response/0.7`, and as its `hint` an
   [`auth/step-up/approve-request/0.6`](../../../../../auth/step-up/approve-request/0.6/spec.md) payload delivered as that specification's
   *Inline delivery* says — `boundTo` a digest of the task type and `previewId` salted with
   the step-up's `challenge`, `sessionId` absent, and a `reason` naming the identity, the role
   and the operation in words (§6.2).
2. **MUST** answer a request with no `previewId`, which its policy would hold for a step-up,
   by computing and storing the preview as for a dry run (§6.1) and answering with a next
   step whose `continuation` is `proceed` and whose `expects` has one entry: `typeUri` this
   task's own Type URI, and as its `hint` the `#response` payload the dry run would have
   returned — `outcome: "preview"` with the `preview`, its `previewId` included — valid
   against this specification's response schema. Nothing is applied, and no step-up is
   issued, until a request carries that `previewId` back.
3. **MUST NOT** answer either hold with `permissionDenied`, `taskFailed` or any other
   *error response*. A step-up the approver declines ends the hold: the VTA answers the
   request as refused (`permissionDenied`) when the producer next sends it.

The producer renders the VTA's preview, never one it computed (§6.1), obtains the
[`approve-response/0.7`](../../../../../auth/step-up/approve-response/0.7/spec.md) from its human, and on a `recorded` acknowledgement sends
the request again as `continuation: resubmit` directs — a new document in the same thread,
with the same payload and the same `previewId`. The approval is bound to that `previewId`, so
the VTA applies exactly the previewed plan or nothing, and a plan that has expired or been
moved past is `vta/dids:previewStale` as before. Where quorum is required, the VTA then
answers `pendingApproval` (§6.4).

## Authorization

Context `admin` or `super-admin` for `operational`, `messaging` and `attestation`;
`super-admin` for `update`. For an `authenticator` key, also the person whose identity lists
it ([conventions §7](../../../_shared/0.1/CONVENTIONS.md#7-authorization)). Revocation is
deliberately easier to reach than addition: it removes authority and cannot be used to gain
it.

## Request

### Revoke a VTC's operational key

```json
{
  "id": "urn:uuid:ee11dd22-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/keys/revoke/1.1",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-20T14:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "operational",
    "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkOper",
    "compromisedSince": "2026-11-19T00:00:00Z",
    "previewId": "pv_5566778899aabbccddee",
    "reason": "Service host intrusion detected"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-20T14:00:00Z",
    "verificationMethod": "did:key:z6MkContextAdmin#z6MkContextAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3Rv..."
  }
}
```

### Recover the update authority after a VTA compromise

```json
{
  "id": "urn:uuid:ee11dd22-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/vta/dids/keys/revoke/1.1",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-20T15:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "update",
    "scope": "boundary",
    "compromisedSince": "2026-11-18T00:00:00Z",
    "recoveryCommitment": { "keyType": "ed25519", "publicKeyMultibase": "z6MkRecoveryNext", "proofOfPossession": "z4Pq..." },
    "previewId": "pv_0011223344556677aabb"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-20T15:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z5Tu..."
  }
}
```

## Response

### Waiting for the recovery custodian

```json
{
  "id": "urn:uuid:ee11dd22-0000-4000-8000-000000000012",
  "type": "https://trusttasks.org/spec/vta/dids/keys/revoke/1.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-11-20T15:00:02Z",
  "threadId": "urn:uuid:ee11dd22-0000-4000-8000-000000000011",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "awaitingRecoveryProof",
    "preview": {
      "previewId": "pv_0011223344556677aabb",
      "method": "webvh",
      "baseVersionId": "7-QmEntrySeven",
      "expiresAt": "2026-11-21T15:00:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "useRecoveryKey", "role": "update" },
        { "op": "commitUpdateKey", "role": "update", "commitment": "QmRoutineFreshHash" },
        { "op": "replaceRecoveryCommitment", "role": "update", "commitment": "QmRecoveryNextHash" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example" },
      "warnings": ["recoveryKeyRevealed", "attestationReissuanceRequired"],
      "recoverySigning": {
        "commitment": "QmRecoveryHash",
        "keyType": "ed25519",
        "unsignedChange": "{\"versionId\":\"{SCID}\",\"parameters\":{\"updateKeys\":[\"z6MkRecoveryKey\"]}}",
        "proofOptions": { "type": "DataIntegrityProof", "cryptosuite": "eddsa-jcs-2022", "verificationMethod": "did:key:z6MkRecoveryKey#z6MkRecoveryKey", "proofPurpose": "assertionMethod", "created": "2026-11-20T15:00:02Z" },
        "expiresAt": "2026-11-21T15:00:00Z"
      }
    },
    "followUp": { "statusListsToResign": 3, "artefactsToReissue": 412 }
  }
}
```

(`document` and `unsignedChange` abbreviated.) Failures use `trust-task-error`.

## Security & Privacy

### Data carried

A key, a compromise time, and for a boundary compromise a fresh recovery key's public half and
the custodian's proof, in; the preview, key records and the change to sign, out. No private key
material in either direction. `reason` is never published.

### Correlation

The revocation and its compromise time are public by design: they are what a verifier needs.
The re-issuance counts are custody data.

### Retention

The revocation is kept for the identity's life, in the document itself and in the history.

### Consent/purpose

The purpose is to stop trusting a key that may be someone else's. The consent surface states in
words that verifiers will refuse what the key signed from the compromise time, what must be
re-issued, and — for a boundary compromise — that the recovery custodian must now sign.

## Changes from 1.0

- **A step-up hold is answered with a next step, not an error.** 1.0 refused a request
  needing a step-up with the error code `vta/dids:stepUpRequired`, carrying the step-up
  request in `details.stepUpRequest` and the plan in `details.preview`. A hold is not a
  refusal, and [SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs) forbids answering one
  with an error response, so 1.1 drops the code and answers with a `trust-task-next-step`
  ([A step-up hold is a next step](#a-step-up-hold-is-a-next-step)): the step-up request
  travels as the `hint` of an entry naming `auth/step-up/approve-response`, as
  [`approve-request/0.6`](../../../../../auth/step-up/approve-request/0.6/spec.md) defines, and the plan as the `hint` of an entry naming this
  task, in the form a dry run returns it. The binding is unchanged: the step-up is bound to the
  `previewId`, and the plan applied is exactly the one previewed. The decision is recorded in
  [trustoverip/dtgwg-trust-tasks-spec#36](https://github.com/trustoverip/dtgwg-trust-tasks-spec/issues/36).
- The payload schema is 1.0's unchanged. The version does not declare `extends: "1.0"`,
  because it drops a declared error code, which the declared-extension rule does not admit.
- `1.0` remains published, and a client **MAY** accept both forms of the hold while the VTAs
  it talks to move to 1.1.
