---
slug: vta/dids/create
version: "1.1"
title: "VTA DIDs — Create"
summary: "An administrator has a VTA create a key-role identity at key assurance level 2 or 3, under any eligible DID method, from a key-role DID template: every key minted fresh for its role and level, and at level 3 an external recovery key and a witness."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - key roles
  - create
  - did template
  - pre-rotation
  - recovery key
  - key assurance level
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Creating a node's durable identity mints the keys that will sign its credentials and govern its identifier. The VTA must attribute the creation to an administrator independently of the transport, and an approval is bound to the request it ratifies."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed creation would mint a second identity and a second set of keys. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Publishes the first version of a public, append-only identifier history and generates non-exportable keys that no backup can restore. Neither can be withdrawn, only superseded or deactivated."
consequences:
  - "A new identity is published and cannot be unpublished, only deactivated."
  - "At level 2, its attestation and update keys are held only by this VTA and are in its backups: a restore brings them back, and a stolen backup is a copy of them."
  - "At level 3, its attestation and update keys are generated inside the VTA and can never leave it or be restored from a backup, and the recovery key you supply becomes the only way to recover the identity if this VTA is lost."
  - "Its level can later be raised, never lowered."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Returns the preview, the new identity's public key records and its first published version. The request carries a template name, variables, the level and at level 3 the recovery key's public half; no private key material in either direction."
retention:
  class: durable
  rationale: "The creation record, the keys' custodian records and the audit row are the start of the identity's key history."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "The context does not exist or the caller cannot reach it."
    retryable: false
  - code: "vta/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, applied, or for a different request."
    retryable: false
  - code: "vta/dids:stepUpRequired"
    meaning: "Policy requires a step-up bound to this creation's preview; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/dids:unsupportedKeyType"
    meaning: "The template's layout names an algorithm that cannot serve its role, is not accepted, or that this VTA cannot mint for the role's custody."
    retryable: false
  - code: "vta/dids:methodNotEligible"
    meaning: "The method has no profile, or its profile is ineligible for key roles; `details.missing` lists the criteria it fails."
    retryable: false
  - code: "vta/dids:methodParameterUnsupported"
    meaning: "A `methodParameters` member is not defined by the method profile's `CreateParameters`."
    retryable: false
  - code: "vta/dids:recoveryCommitmentRequired"
    meaning: "At level 3, the recovery key is missing, is in this VTA's protection boundary, or its proof of possession does not verify."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "The template's layout includes a role that is not created (`authenticator`) or omits one that is required."
    retryable: false
  - code: "vta/dids:assuranceLevelMismatch"
    meaning: "A recovery commitment was given for a level-2 identity."
    retryable: false
  - code: "vta/dids/create:assuranceLevelBelowTemplate"
    meaning: "`assuranceLevel` is below the floor the template sets in `defaults.keyAssurance.minimum` (CONVENTIONS.md §4 item 7)."
    retryable: false
  - code: "vta/dids:witnessRequired"
    meaning: "A level-3 identity under a method with witnesses, and `methodParameters` names no witness enforcing CONVENTIONS.md §5.4."
    retryable: false
  - code: "vta/dids/create:templateNotFound"
    meaning: "No template of that name is visible in the requested scope."
    retryable: false
  - code: "vta/dids/create:templateNotKeyRole"
    meaning: "The template is not a key-role template: not `schemaVersion` 3, or its `document` carries key members."
    retryable: false
related:
  - vta/dids/upgrade
  - vta/dids/keys/list
  - vta/dids/update
  - vta/dids/keys/rotate
  - vta/did-templates/create
  - auth/step-up/approve-request
---

## Abstract

**VTA DIDs — Create** is how a key-role identity comes into existence at key assurance
level 2 or 3 ([conventions §3.4](../../_shared/0.1/CONVENTIONS.md#34-key-assurance-levels)).
The other way in is [`upgrade`](../../upgrade/1.0/spec.md), which raises an existing
identity in place; the rest of the family refuses an identity at level 1
([§3.1](../../_shared/0.1/CONVENTIONS.md#31-only-key-role-identities)).

The caller chooses the level. **Level 2** separates the roles and keeps the `attestation`
and `update` keys in the VTA, which may derive them and back them up, so a restore brings
the identity back; it protects against compromise of the node's service process. **Level 3**
also generates those keys and keeps them out of every backup, commits a recovery key held by
a custodian outside the VTA, and requires a witness; it protects against theft of a backup,
and against compromise or loss of the VTA itself.

The caller names a DID method, a **key-role DID template** and the template's variables. The
template's `keyRoles` layout says which keys the identity has in which role; its `document`
supplies the non-key members — services, `alsoKnownAs`. The VTA mints every key fresh by its
role's custody rules at the chosen level — at level 3, `attestation` and `update` keys
generated inside its protection boundary; at level 2, derived or generated at its choice;
`operational` and `messaging` keys derived under the context — renders every key member of
the document from the layout, renders `keyAssurance`, and commits a routine update
successor, beside which, at level 3, it commits the **recovery key** the caller supplies
from a custodian outside the VTA
([§5.3](../../_shared/0.1/CONVENTIONS.md#53-the-change-authority-a-routine-key-and-a-recovery-key),
[§5.6](../../_shared/0.1/CONVENTIONS.md#56-recovery-custodians)).

What the template cannot do is the point of the design. It cannot name a relationship, so it
cannot put an operational key in `assertionMethod`; it cannot supply a key, so no key with
unknown custody enters the identity; and at level 3 it cannot omit the recovery commitment.

The method-specific part — where the identity is hosted, its witnesses, its presented form —
is `methodParameters`, validated against the [method profile](../../_shared/0.1/method-profiles/README.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../_shared/0.1/CONVENTIONS.md) and the governing [method profile](../../_shared/0.1/method-profiles/README.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse a method with no eligible profile (`vta/dids:methodNotEligible`), and validate
   `methodParameters` against the profile's `CreateParameters`.
2. Resolve `template` in `templateContext`'s scope (else the global scope) and refuse it
   unless it is a key-role template ([conventions §4](../../_shared/0.1/CONVENTIONS.md#4-creation)
   items 1–2). Refuse `assuranceLevel` below the template's `defaults.keyAssurance.minimum`
   (conventions §4 item 7, `vta/dids/create:assuranceLevelBelowTemplate`). Render its
   `document` with the VTA's template renderer, which checks the variables, and with nothing
   that names a key.
3. Mint one key per algorithm per role in the layout, by the role's origin
   ([§5.1](../../_shared/0.1/CONVENTIONS.md#51-generated-and-derived-roles-and-why)),
   **when the preview is computed**, held `pending`. Refuse an algorithm it cannot mint for
   the role's custody — for example an X25519 or ML-DSA key it cannot generate as an
   internal key — with `vta/dids:unsupportedKeyType`, and never substitute one.
4. Render `verificationMethod`, the relationships, `keyRoles` and `keyAssurance` from the
   minted keys, §2 and §3.4.2, with fragments derived from the keys, and commit the routine
   update successor — and at level 3 the recovery key — as the method profile says.
5. At level 3, require `recoveryCommitment`, verify its `proofOfPossession` over the
   `previewId` on the apply request, refuse a recovery key it generated or can derive
   (`vta/dids:recoveryCommitmentRequired`), record `custodian` in custody and nowhere else,
   and refuse a method with witnesses when `methodParameters` names none
   (`vta/dids:witnessRequired`). At level 2, refuse `recoveryCommitment`
   (`vta/dids:assuranceLevelMismatch`), and warn `roleKeysInBackups` and, where no witness
   is named, `noWitnesses`.
6. Follow the preview, step-up and approval rules of
   [conventions §6](../../_shared/0.1/CONVENTIONS.md#6-approval), and create every custodian
   record in the same atomic step as the first version.
7. Audit `did.create`, including refusals.

A conforming consumer **MUST NOT** accept a key, a key identifier, a document carrying keys,
or an existing history in any member or in `ext`. Importing an identity minted elsewhere is
not creation; it is [`recover`](../../recover/1.0/spec.md), or a method-specific task for a
level-1 identity.

## Authorization

The caller's context `admin` (or `super-admin`) authority over `contextId`, under the
VTA's access-control list ([conventions §7](../../_shared/0.1/CONVENTIONS.md#7-authorization)).
The request's proof establishes who the caller is; the ACL entry is the authority. A context
the caller cannot reach is answered `vta/dids:notFound`. The VTA's policy decides whether a
step-up or quorum is required; the VTI specification sets a bound step-up as the floor at
level 3, and recommends one at level 2 (VTI-KEY-180).

## Payload

`payload.contextId`, `payload.method`, `payload.template` and `payload.assuranceLevel` are
REQUIRED; `payload.recoveryCommitment` is REQUIRED at level 3 and refused at level 2.
`templateContext`, `templateVars`, `methodParameters`, `label`, `dryRun`, `previewId` and
`reason` are OPTIONAL. There is no `expectedVersionId`: nothing exists yet to have moved.

## Request

### Preview the creation of a VTC's identity at level 3, with a threshold custodian

```json
{
  "id": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/create/1.1",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "vtc-example",
    "method": "webvh",
    "template": "vtc-host-key-roles",
    "templateVars": { "VTC_URL": "https://vtc.example" },
    "assuranceLevel": 3,
    "methodParameters": { "serverId": "webvh-host-1", "ttlSeconds": 3600, "witnesses": { "threshold": 1, "witnesses": ["did:key:z6MkWitnessOne"] } },
    "recoveryCommitment": {
      "threshold": 2,
      "label": "Recovery group: three member VTAs",
      "members": [
        { "keyType": "ed25519", "publicKeyMultibase": "z6MkMemberA", "custodian": { "kind": "vta", "did": "did:webvh:QmA:vta-a.example" } },
        { "keyType": "ed25519", "publicKeyMultibase": "z6MkMemberB", "custodian": { "kind": "vta", "did": "did:webvh:QmB:vta-b.example" } },
        { "keyType": "ed25519", "publicKeyMultibase": "z6MkMemberC", "custodian": { "kind": "vta", "did": "did:webvh:QmC:vta-c.example" } }
      ]
    },
    "dryRun": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-02T09:00:00Z",
    "verificationMethod": "did:key:z6MkContextAdmin#z6MkContextAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z4Fy..."
  }
}
```

The apply request repeats the payload without `dryRun`, with the `previewId` the preview
returned and `recoveryCommitment.proofOfPossession` added. For a threshold custodian, every
member adds its own `proofOfPossession` over the same `previewId`, after checking what it
signs for ([conventions §5.7](../../_shared/0.1/CONVENTIONS.md#57-threshold-custodians)).

### Preview the creation of a VTC's identity at level 2

```json
{
  "id": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vta/dids/create/1.1",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "vtc-small",
    "method": "webvh",
    "template": "vtc-host-key-roles",
    "templateVars": { "VTC_URL": "https://small.example" },
    "assuranceLevel": 2,
    "methodParameters": { "serverId": "webvh-host-1" },
    "dryRun": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-02T09:00:00Z",
    "verificationMethod": "did:key:z6MkContextAdmin#z6MkContextAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3Hk..."
  }
}
```

No recovery commitment and no witness; the preview warns `roleKeysInBackups` and
`noWitnesses`.

## Response

### The preview

```json
{
  "id": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/create/1.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkContextAdmin",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmNewScid:vtc.example",
    "contextId": "vtc-example",
    "method": "webvh",
    "outcome": "preview",
    "assuranceLevel": 3,
    "preview": {
      "previewId": "pv_7c1d2e3f40516273a8b9",
      "method": "webvh",
      "expiresAt": "2026-11-02T10:00:00Z",
      "updateKeyRotates": false,
      "assuranceLevel": 3,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmNewScid:vtc.example#z6MkAttest", "keyType": "ed25519", "publicKeyMultibase": "z6MkAttest", "relationships": ["assertionMethod"] },
        { "op": "addKey", "role": "operational", "verificationMethod": "did:webvh:QmNewScid:vtc.example#z6MkOper", "keyType": "ed25519", "publicKeyMultibase": "z6MkOper", "relationships": ["authentication"] },
        { "op": "addKey", "role": "messaging", "verificationMethod": "did:webvh:QmNewScid:vtc.example#z6LSMsg", "keyType": "x25519", "publicKeyMultibase": "z6LSMsg", "relationships": ["keyAgreement"] },
        { "op": "commitUpdateKey", "role": "update", "commitment": "QmRoutineNextHash" },
        { "op": "replaceRecoveryCommitment", "role": "update", "commitment": "QmRecoveryHash" },
        { "op": "declareAssuranceLevel", "role": "update", "assuranceLevel": 3 }
      ],
      "document": {
        "id": "did:webvh:QmNewScid:vtc.example",
        "assertionMethod": ["did:webvh:QmNewScid:vtc.example#z6MkAttest"],
        "authentication": ["did:webvh:QmNewScid:vtc.example#z6MkOper"],
        "keyAgreement": ["did:webvh:QmNewScid:vtc.example#z6LSMsg"],
        "keyRoles": {
          "attestation": ["did:webvh:QmNewScid:vtc.example#z6MkAttest"],
          "operational": ["did:webvh:QmNewScid:vtc.example#z6MkOper"],
          "messaging": ["did:webvh:QmNewScid:vtc.example#z6LSMsg"]
        },
        "keyAssurance": { "level": 3, "recoveryCommitment": "QmRecoveryHash" }
      },
      "warnings": ["singleKeyRole"]
    }
  }
}
```

(`document` abbreviated.) Failures use `trust-task-error`.

## Security & Privacy

### Data carried

The request carries a template name, its variables, method parameters, the level, and at
level 3 the recovery key's public half with a possession proof and the operator's
declaration of its custodian; the response, the preview and public key records. The
custodian declaration is custody data: the VTA never publishes it. No
private key material in either direction, or in `ext`. Template variables are rendered into a
public document: a producer **MUST NOT** put anything in them it would not publish.

### Correlation

The identity is public by design (VTI-KEY-007). Fragments are derived from keys, and nothing
in the document reveals the node's layout, custody or operators (VTI-KEY-150). The document
declares the level, which a verifier needs, and at level 3 names which commitment is the
recovery commitment, which a witness needs; neither says who holds what. The recovery key is
committed by hash where the method allows, so its public half is not published until used;
its custodian's label and kind are custody data only.

### Retention

The VTA keeps the creation record, custodian records and audit row for the identity's life.

### Consent/purpose

The purpose is to establish an identity whose keys are separated by role from its first
version. The consent surface renders the VTA's preview — every key, the document, the
warnings — and states in words what the level means: at level 2, that the attestation and
update keys are in this VTA's backups and nowhere else; at level 3, that they can never
leave this VTA and that the recovery key is the only way back if it is lost.

## Changes from 1.0

- **`assuranceLevel` is REQUIRED**, `2` or `3` (VTI-KEY-194). Version 1.0 always created
  what is now level 3.
- **`recoveryCommitment` is REQUIRED only at level 3**, and refused at level 2
  (`vta/dids:assuranceLevelMismatch`). The recovery key was mandatory because generated
  keys are in no backup; at level 2 they may be derived or backed up, so a restore recovers a
  lost VTA and no recovery key is needed.
- `RecoveryCommitment.custodian` declares the custodian's kind — a separately held key, a
  second VTA, or a threshold group of VTAs — for custody and routing; it is never published.
- At level 3, a method with witnesses needs a witness enforcing CONVENTIONS.md §5.4
  (`vta/dids:witnessRequired`); version 1.0 only warned `noWitnesses`.
- The document carries `keyAssurance` (CONVENTIONS.md §3.4.2), and the response and preview
  carry `assuranceLevel`.
