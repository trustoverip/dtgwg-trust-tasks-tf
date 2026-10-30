---
slug: vta/dids/create
version: "1.0"
title: "VTA DIDs — Create"
summary: "An administrator has a VTA create a key-role identity under any eligible DID method from a key-role DID template: every key minted fresh by its role's custody rules, a routine update successor and an external recovery key committed from the first version."
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
  - "Its attestation and update keys are generated inside the VTA and can never leave it or be restored from a backup."
  - "The recovery key you supply becomes the only way to recover the identity if this VTA is lost."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Returns the preview, the new identity's public key records and its first published version. The request carries a template name, variables and the recovery key's public half; no private key material in either direction."
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
    meaning: "The recovery key is missing, is in this VTA's protection boundary, or its proof of possession does not verify."
    retryable: false
  - code: "vta/dids:notApplicableToRole"
    meaning: "The template's layout includes a role that is not created (`authenticator`) or omits one that is required."
    retryable: false
  - code: "vta/dids/create:templateNotFound"
    meaning: "No template of that name is visible in the requested scope."
    retryable: false
  - code: "vta/dids/create:templateNotKeyRole"
    meaning: "The template is not a key-role template: not `schemaVersion` 3, or its `document` carries key members."
    retryable: false
related:
  - vta/dids/keys/list
  - vta/dids/update
  - vta/dids/keys/rotate
  - vta/did-templates/create
  - auth/step-up/approve-request
---

## Abstract

**VTA DIDs — Create** is how a key-role identity comes into existence. It is the creation
path the [key-role conventions](../../_shared/0.1/CONVENTIONS.md) depend on: the other tasks
in the family refuse any identity that was not created with key roles
([§3.1](../../_shared/0.1/CONVENTIONS.md#31-only-key-role-identities)), so this is the only
way in.

The caller names a DID method, a **key-role DID template** and the template's variables. The
template's `keyRoles` layout says which keys the identity has in which role; its `document`
supplies the non-key members — services, `alsoKnownAs`. The VTA mints every key fresh by its
role's custody rules — `attestation` and `update` keys generated inside its protection
boundary, `operational` and `messaging` keys derived under the context — renders every key
member of the document from the layout, and commits a routine update successor beside the
**recovery key** the caller supplies from a custodian outside the VTA
([§5.3](../../_shared/0.1/CONVENTIONS.md#53-the-change-authority-a-routine-key-and-a-recovery-key)).

What the template cannot do is the point of the design. It cannot name a relationship, so it
cannot put an operational key in `assertionMethod`; it cannot supply a key, so no key with
unknown custody enters the identity; and it cannot omit the recovery commitment.

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
   items 1–2). Render its `document` with the VTA's template renderer, which checks the
   variables, and with nothing that names a key.
3. Mint one key per algorithm per role in the layout, by the role's origin
   ([§5.1](../../_shared/0.1/CONVENTIONS.md#51-generated-and-derived-roles-and-why)),
   **when the preview is computed**, held `pending`. Refuse an algorithm it cannot mint for
   the role's custody — for example an X25519 or ML-DSA key it cannot generate as an
   internal key — with `vta/dids:unsupportedKeyType`, and never substitute one.
4. Render `verificationMethod`, the relationships and `keyRoles` from the minted keys and
   §2, with fragments derived from the keys, and commit the routine update successor and the
   recovery key as the method profile says.
5. Verify the recovery key's `proofOfPossession` over the `previewId` on the apply request,
   and refuse a recovery key it generated or can derive
   (`vta/dids:recoveryCommitmentRequired`).
6. Follow the preview, step-up and approval rules of
   [conventions §6](../../_shared/0.1/CONVENTIONS.md#6-approval), and create every custodian
   record in the same atomic step as the first version.
7. Audit `did.create`, including refusals.

A conforming consumer **MUST NOT** accept a key, a key identifier, a document carrying keys,
or an existing history in any member or in `ext`. Importing an identity minted elsewhere is
not creation; it is [`recover`](../../recover/1.0/spec.md), or a method-specific task for a
pre-role identity.

## Authorization

The caller's context `admin` (or `super-admin`) authority over `contextId`, under the
VTA's access-control list ([conventions §7](../../_shared/0.1/CONVENTIONS.md#7-authorization)).
The request's proof establishes who the caller is; the ACL entry is the authority. A context
the caller cannot reach is answered `vta/dids:notFound`. The VTA's policy decides whether a
step-up or quorum is required; the VTI specification sets a bound step-up as the floor
(VTI-KEY-180).

## Payload

`payload.contextId`, `payload.method`, `payload.template` and `payload.recoveryCommitment` are
REQUIRED. `templateContext`, `templateVars`, `methodParameters`, `label`, `dryRun`, `previewId`
and `reason` are OPTIONAL. There is no `expectedVersionId`: nothing exists yet to have moved.

## Request

### Preview the creation of a VTC's identity

```json
{
  "id": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/create/1.0",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "vtc-example",
    "method": "webvh",
    "template": "vtc-host-key-roles",
    "templateVars": { "VTC_URL": "https://vtc.example" },
    "methodParameters": { "serverId": "webvh-host-1", "ttlSeconds": 3600 },
    "recoveryCommitment": { "keyType": "ed25519", "publicKeyMultibase": "z6MkRecoveryKey", "label": "Board safe, token 2" },
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
returned and `recoveryCommitment.proofOfPossession` added.

## Response

### The preview

```json
{
  "id": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/create/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkContextAdmin",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0c6c1e-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmNewScid:vtc.example",
    "contextId": "vtc-example",
    "method": "webvh",
    "outcome": "preview",
    "preview": {
      "previewId": "pv_7c1d2e3f40516273a8b9",
      "method": "webvh",
      "expiresAt": "2026-11-02T10:00:00Z",
      "updateKeyRotates": false,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmNewScid:vtc.example#z6MkAttest", "keyType": "ed25519", "publicKeyMultibase": "z6MkAttest", "relationships": ["assertionMethod"] },
        { "op": "addKey", "role": "operational", "verificationMethod": "did:webvh:QmNewScid:vtc.example#z6MkOper", "keyType": "ed25519", "publicKeyMultibase": "z6MkOper", "relationships": ["authentication"] },
        { "op": "addKey", "role": "messaging", "verificationMethod": "did:webvh:QmNewScid:vtc.example#z6LSMsg", "keyType": "x25519", "publicKeyMultibase": "z6LSMsg", "relationships": ["keyAgreement"] },
        { "op": "commitUpdateKey", "role": "update", "commitment": "QmRoutineNextHash" },
        { "op": "replaceRecoveryCommitment", "role": "update", "commitment": "QmRecoveryHash" }
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
        }
      },
      "warnings": ["noWitnesses", "singleKeyRole"]
    }
  }
}
```

(`document` abbreviated.) Failures use `trust-task-error`.

## Security & Privacy

### Data carried

The request carries a template name, its variables, method parameters and the recovery key's
public half with a possession proof; the response, the preview and public key records. No
private key material in either direction, or in `ext`. Template variables are rendered into a
public document: a producer **MUST NOT** put anything in them it would not publish.

### Correlation

The identity is public by design (VTI-KEY-007). Fragments are derived from keys, and nothing
in the document reveals the node's layout, custody or operators (VTI-KEY-150). The recovery
key is committed by hash where the method allows, so its public half is not published until
used; its custodian's label is custody data only.

### Retention

The VTA keeps the creation record, custodian records and audit row for the identity's life.

### Consent/purpose

The purpose is to establish an identity whose keys are separated by role from its first
version. The consent surface renders the VTA's preview — every key, the document, the
warnings — and states in words that the attestation and update keys can never leave this VTA
and that the recovery key is the only way back if it is lost.
