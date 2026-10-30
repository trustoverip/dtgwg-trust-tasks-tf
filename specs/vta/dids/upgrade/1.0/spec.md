---
slug: vta/dids/upgrade
version: "1.0"
title: "VTA DIDs — Upgrade"
summary: "A super-administrator raises an identity's key assurance level in place — from a single key to role-separated or recoverable, or from role-separated to recoverable — with fresh keys for every role whose custody tightens, without changing the identifier. A level is never lowered."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - key roles
  - key assurance level
  - upgrade
  - recovery key
parties:
  - role: Super-administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A raise mints the keys that will sign the identity's credentials, moves its change authority and, at level 3, commits the key that can take the identity over. The VTA must attribute it to a super-administrator independently of the transport, and an approval is bound to the request it ratifies."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Replayed, a raise would mint a second set of keys against a version the identity has moved past. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Publishes a history entry that replaces the identity's attestation keys and update authority, declares a level that can never be lowered, and at level 3 commits a recovery key. It cannot be withdrawn."
consequences:
  - "Every role whose custody the new level tightens gets new keys; the keys they replace stop speaking for the identity."
  - "Credentials signed before the raise stay credentials of the lower level: verifiers that require the new level will want them re-issued."
  - "Every status list is re-signed at once under the new attestation keys."
  - "The new level can never be lowered; a lower level needs a new identity."
  - "At level 3, the recovery key you supply becomes the only way to recover the identity if this VTA is lost."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Returns the preview, the new public key records and the re-issuance obligation. The request carries the target level, a layout, and at level 3 the recovery key's public half; no private key material in either direction."
retention:
  class: durable
  rationale: "The raise is part of the identity's key history: its custodian records and audit row are kept for the identity's life."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:versionConflict"
    meaning: "The identity's latest version no longer matches `expectedVersionId`."
    retryable: false
  - code: "vta/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, applied, computed against a superseded version, or for a different request."
    retryable: false
  - code: "vta/dids:stepUpRequired"
    meaning: "A step-up bound to this raise's preview is required; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/dids:unsupportedKeyType"
    meaning: "The layout names an algorithm that cannot serve its role, is not accepted, or that this VTA cannot mint for the role's custody at the new level."
    retryable: false
  - code: "vta/dids:rotationInProgress"
    meaning: "A role of the identity has a change staged, overlapping or awaiting approval or a recovery proof."
    retryable: false
  - code: "vta/dids:recoveryCommitmentRequired"
    meaning: "Raising to level 3 without a recovery key, or with one in this VTA's protection boundary, or whose proof of possession does not verify."
    retryable: false
  - code: "vta/dids:methodNotEligible"
    meaning: "The identity's method has no profile eligible for key roles; `details.missing` lists the criteria it fails. Such an identity reaches a higher level only as a new identity."
    retryable: false
  - code: "vta/dids:methodParameterUnsupported"
    meaning: "A `methodParameters` member is not in the profile's `UpdateParameters`."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "The layout includes `authenticator` or `update`, omits `operational` or `messaging`, or was given for a raise from level 2."
    retryable: false
  - code: "vta/dids:assuranceLevelMismatch"
    meaning: "`toLevel` is not higher than the identity's level, or a recovery commitment was given for a raise to level 2."
    retryable: false
  - code: "vta/dids:witnessRequired"
    meaning: "Raising to level 3 under a method with witnesses, and neither the identity nor `methodParameters` names a witness enforcing CONVENTIONS.md §5.4."
    retryable: false
  - code: "vta/dids/upgrade:compromiseTimeInFuture"
    meaning: "`compromisedSince` is later than the request's `issuedAt`."
    retryable: false
related:
  - vta/dids/create
  - vta/dids/keys/list
  - vta/dids/recover
  - vtc/credentials/reissue
---

## Abstract

**VTA DIDs — Upgrade** raises the [key assurance level](../../_shared/0.1/CONVENTIONS.md#34-key-assurance-levels)
of an identity this VTA holds, in place, by one history entry the identity's current update
authority authorizes (VTI-KEY-147,
[conventions §3.5](../../_shared/0.1/CONVENTIONS.md#35-raising-a-level)). It is the one task
in the family that accepts a level-1 identity — a single key in both `authentication` and
`assertionMethod`, as every DID created before key roles has — and it is how such an
identity reaches level 2 without its members, registries and relying parties having to learn
a new DID.

The entry:

1. **mints fresh keys** for every role whose custody the new level tightens — from level 1,
   every role and a routine update key with its committed successor; from level 2 to 3, the
   `attestation` keys and the routine update key and successor — by the new level's origin
   rules ([§5.1](../../_shared/0.1/CONVENTIONS.md#51-generated-and-derived-roles-and-why));
2. **removes the keys it replaces** from every relationship and from `keyRoles`, retiring
   them, or, with `previousKeys: revoke`, revoking them from `compromisedSince`;
3. **declares the new level** in `keyAssurance`, and at level 3 commits the recovery key and
   names a witness.

No key keeps its place unless its custody already meets the new level. The single key of a
level-1 identity takes no role: it has signed attestation artefacts and been held by the
service process, so making it the `operational` key would let it go on being accepted, by
anyone holding the old document, for what it may no longer sign (VTI-KEY-071).

The raise does not certify the identity's past. What the replaced keys signed stays an
artefact of the lower level (VTI-KEY-201): a verifier that requires the new level refuses it,
and one that accepts the lower level accepts it as it always would have. The node re-signs
every status list at once and re-issues what its relying parties need at the new level
(VTI-KEY-149). A level is never lowered (VTI-KEY-148).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../_shared/0.1/CONVENTIONS.md) and the governing [method profile](../../_shared/0.1/method-profiles/README.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Establish the identity's current level as a verifier would (VTI-KEY-199), and refuse
   `toLevel` not higher than it (`vta/dids:assuranceLevelMismatch`). Refuse an identity whose
   method has no profile eligible for key roles (`vta/dids:methodNotEligible`).
2. From level 1, mint the `layout` (or the default layout) by the new level's origin rules,
   refusing `authenticator` and `update` entries and a layout without `operational` and
   `messaging` (`vta/dids:notApplicableToRole`); from level 2, refuse `layout` and mint
   replacements of the predecessors' algorithms. Never substitute an algorithm
   (`vta/dids:unsupportedKeyType`).
3. Mint a new routine update key and its committed successor by the new level's origin rules,
   and move the update authority to it in the same entry, authorized by the identity's
   current update authority.
4. Remove every replaced key from its relationship and from `keyRoles`: retired, or with
   `previousKeys: revoke` revoked as [§9.1](../../_shared/0.1/CONVENTIONS.md#91-how-a-revocation-is-recorded)
   records, with `compromisedSince` required and not in the future.
5. Render `keyAssurance` with `toLevel`. At level 3: require `recoveryCommitment`, verify its
   proof of possession over the `previewId` and refuse a key in its own boundary
   (`vta/dids:recoveryCommitmentRequired`); name it in `keyAssurance.recoveryCommitment`;
   and require a witness enforcing [§5.4](../../_shared/0.1/CONVENTIONS.md#54-witnesses-and-the-recovery-commitment),
   from the identity's parameters or `methodParameters` (`vta/dids:witnessRequired`). At
   level 2, refuse `recoveryCommitment` and warn `roleKeysInBackups`.
6. At level 3, move every generated key out of the backup partition before the entry is
   published, and report `inBackups: false` from then on.
7. Follow the preview, step-up and approval rules of
   [conventions §6](../../_shared/0.1/CONVENTIONS.md#6-approval) — a bound step-up always, and
   two approvers for a raise to level 3 where two eligible approvers exist (§6.5) — and
   create every custodian record in the same atomic step as the entry.
8. After publishing, re-sign every status list under the new `attestation` keys at once, and
   report in `followUp` what must be re-issued. Audit `did.upgrade`, including refusals.

A conforming consumer **MUST NOT** publish an entry that declares a lower level than the
identity's current one, by this task or any other.

## Authorization

`super-admin`, under the VTA's access-control list
([conventions §7](../../_shared/0.1/CONVENTIONS.md#7-authorization)): a raise mints
`attestation` keys and moves the update authority, which are `super-admin` cells. A caller
that cannot reach the identity's context is answered `vta/dids:notFound`.

## Payload

`payload.did`, `payload.toLevel` and `payload.previousKeys` are REQUIRED.
`payload.recoveryCommitment` is REQUIRED when `toLevel` is 3 and refused otherwise;
`payload.compromisedSince` is REQUIRED when `previousKeys` is `revoke` and refused otherwise.
`layout`, `methodParameters`, `expectedVersionId`, `dryRun`, `previewId` and `reason` are
OPTIONAL.

## Request

### Raise a community's single-key identity to level 2

```json
{
  "id": "urn:uuid:7e7e7e7e-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/upgrade/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-10T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "toLevel": 2,
    "previousKeys": "retire",
    "expectedVersionId": "4-QmEntryFour",
    "dryRun": true,
    "reason": "Stop the community service holding the key that signs credentials"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-10T09:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z5Qa..."
  }
}
```

### Raise it from level 2 to level 3, with a second VTA as recovery custodian

```json
{
  "id": "urn:uuid:7e7e7e7e-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vta/dids/upgrade/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2027-02-01T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "toLevel": 3,
    "previousKeys": "retire",
    "recoveryCommitment": {
      "keyType": "ed25519",
      "publicKeyMultibase": "z6MkRecoveryOnVta2",
      "label": "Recovery VTA at the second site",
      "custodian": { "kind": "vta", "did": "did:webvh:QmVta2Scid:vta2.example" }
    },
    "methodParameters": { "witnesses": { "threshold": 1, "witnesses": ["did:key:z6MkWitnessAtVta2"] } },
    "expectedVersionId": "9-QmEntryNine",
    "dryRun": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-02-01T09:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z2Rb..."
  }
}
```

The recovery custodian here also operates the witness, which is already outside this VTA's
boundary. The apply request repeats the payload without `dryRun`, with the `previewId` and
`recoveryCommitment.proofOfPossession`.

## Response

### The preview of the raise to level 2

```json
{
  "id": "urn:uuid:7e7e7e7e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/upgrade/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-11-10T09:00:01Z",
  "threadId": "urn:uuid:7e7e7e7e-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "preview",
    "fromLevel": 1,
    "assuranceLevel": 2,
    "preview": {
      "previewId": "pv_2b3c4d5e6f708192a3b4",
      "method": "webvh",
      "baseVersionId": "4-QmEntryFour",
      "expiresAt": "2026-11-10T10:00:00Z",
      "updateKeyRotates": true,
      "assuranceLevel": 2,
      "changes": [
        { "op": "retireKey", "role": "operational", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#key-0" },
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkAttest", "keyType": "ed25519", "publicKeyMultibase": "z6MkAttest", "relationships": ["assertionMethod"] },
        { "op": "addKey", "role": "operational", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkOper", "keyType": "ed25519", "publicKeyMultibase": "z6MkOper", "relationships": ["authentication"] },
        { "op": "addKey", "role": "messaging", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6LSMsg", "keyType": "x25519", "publicKeyMultibase": "z6LSMsg", "relationships": ["keyAgreement"] },
        { "op": "rotateUpdateKey", "role": "update" },
        { "op": "commitUpdateKey", "role": "update", "commitment": "QmRoutineNextHash" },
        { "op": "declareAssuranceLevel", "role": "update", "assuranceLevel": 2 }
      ],
      "document": {
        "id": "did:webvh:QmVtcScid:vtc.example",
        "keyRoles": {
          "attestation": ["did:webvh:QmVtcScid:vtc.example#z6MkAttest"],
          "operational": ["did:webvh:QmVtcScid:vtc.example#z6MkOper"],
          "messaging": ["did:webvh:QmVtcScid:vtc.example#z6LSMsg"]
        },
        "keyAssurance": { "level": 2 }
      },
      "warnings": ["roleKeysInBackups", "noWitnesses", "singleKeyRole"]
    },
    "followUp": { "statusListsToResign": 3 }
  }
}
```

(`document` abbreviated. The `retireKey` change names the level-1 key under the role it is
retired from; it was never bound to one, and the preview names it `operational` because it
was in `authentication`.) Failures use `trust-task-error`.

## Security & Privacy

### Data carried

The request carries the target level, a layout, method parameters, and at level 3 a recovery
key's public half with a possession proof and the operator's declaration of its custodian;
the response, the preview, public key records and the re-issuance counts. No private key
material in either direction, or in `ext`. `reason` and the custodian declaration are never
published.

### Correlation

The raise is public: the history shows that every key was replaced and the level declared,
which relying parties need to see. The new keys' fragments are derived from the keys, and
nothing in the document reveals custody, operators or the custodian (VTI-KEY-150). Like an
`attestation` rotation, a raise is a coarse timestamp on everything signed after it
(VTI-KEY-153).

### Retention

The VTA keeps the raise's record, custodian records and audit row for the identity's life.

### Consent/purpose

The purpose is to give an existing identity a higher level of key protection without
changing its identifier. The consent surface renders the VTA's preview and states in words
that the level can never be lowered; that credentials issued before the raise stay at the
old level until re-issued; at level 2, that the new keys are in this VTA's backups; and at
level 3, that they never leave this VTA and that the recovery key is the only way back if it
is lost.
