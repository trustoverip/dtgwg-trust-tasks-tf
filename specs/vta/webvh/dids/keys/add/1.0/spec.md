---
slug: vta/webvh/dids/keys/add
version: "1.0"
title: "VTA WebVH DIDs — Keys Add"
summary: "An administrator asks the VTA to generate a key and add it to one role of a did:webvh — a post-quantum attestation key beside a classical one, a second operational key — without removing any."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - vta
  - webvh
  - key roles
  - post-quantum
  - ml-dsa
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
  rationale: "Adding a key extends who can speak for the identity — for the attestation role, who can sign the community's credentials and status lists. The VTA must attribute that to one administrator independently of the transport that carried it, and a bearer token proves only who opened the channel."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The change appends an entry to a public, append-only log. Replayed after the key it added was retired, it would add another — so it must be placeable in time for the duplicate window of SPEC §7.2 item 11 to absorb it."
sideEffects:
  level: destructive
  rationale: "Authority-shifting and unretractable: the new key can sign for its role from the moment it activates, the published entry cannot be withdrawn, and under pre-rotation the entry also moves the DID's update key to its committed successor."
consequences:
  - "Adds a new key that can sign for this role. For the attestation role, that key can sign membership credentials and status lists."
  - "Appending the entry also rotates this DID's update key to its committed successor."
  - "Appends to a public, append-only log; the entry cannot be withdrawn, only superseded."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview and the new key's public record. The private half is generated inside the VTA and never leaves; for the attestation role it can never leave."
retention:
  class: durable
  rationale: "The key, its custodian record, the rotation record and the audit row are kept for the DID's lifetime: they are what a verifier's later question about this key is answered from."
errorCodes:
  - code: "vta/webvh/dids:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
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
  - code: "vta/webvh/dids:unsupportedKeyType"
    meaning: "The key type cannot serve the role, or is not in the VTA's accepted set."
    retryable: false
  - code: "vta/webvh/dids:rotationInProgress"
    meaning: "The role already has a change staged, overlapping or awaiting approval."
    retryable: false
  - code: "vta/webvh/dids:legacyKeysPresent"
    meaning: "The DID still publishes keys with no role; migrate it first."
    retryable: false
related:
  - vta/webvh/dids/keys/list
  - vta/webvh/dids/rotate-keys
  - vta/webvh/dids/keys/retire
  - vta/webvh/dids/keys/migrate
  - auth/step-up/approve-request
---

## Abstract

**VTA WebVH DIDs — Keys Add** asks the VTA to generate one key and publish it in one
[key role](../../../../../_shared/0.3/CONVENTIONS.md#1-roles-and-the-relationships-they-own)
of a DID, alongside the keys already there. Nothing leaves the role.

The case it exists for is algorithm agility: adding an ML-DSA-44 `attestation` key to a
community that signs with Ed25519, so that the two sign every attestation artefact as a
proof set (VTI-KEY-103) and a verifier with post-quantum support can require the
post-quantum proof, while one without it still checks the classical one. Replacing a key
is [`rotate-keys`](../../../rotate-keys/2.0/spec.md); removing an added one is
[`keys/retire`](../../retire/1.0/spec.md).

The request carries no key material and names no identifier for the new key: the VTA
generates the key and derives the fragment from it. A request that could supply either
would be a request that could plant a key of the caller's choosing in the identity.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../../../_shared/0.3/CONVENTIONS.md) are part of this specification; in particular §1 (roles), §2 (authorization), §3 (preview, step-up, quorum), §4 (custody), §6 (audit) and §8 (the update-key rotation every entry carries).

A conforming **consumer** (the VTA) **MUST**:

1. Refuse `role: update` — the schema does, and the VTA **MUST** as well; update keys move
   only by rotation.
2. Generate the key when the preview is computed (conventions §3.1): for `attestation`, from its CSPRNG inside its protection boundary,
   non-exportable and excluded from backups; for `operational` and `messaging`, as it
   derives the context's other keys.
3. Publish it in exactly the role's relationship and in `keyRoles`, under a fragment
   derived from the key, and create its custodian record in the same atomic step.
4. Stage it: the key is `staged` on publication and becomes `active` no sooner than the
   document's validity period later (VTI-KEY-122), so no verifier holding a cached
   document sees a signature by a key it has not seen. The response's `keys[0].activatesAt`
   says when.
5. Compute and return the preview on `dryRun`, and apply exactly a previewed plan on
   `previewId` (conventions §3.1).

A conforming **producer** **SHOULD** dry-run first and apply with the `previewId`, and
**MUST** do so when the change will be shown to a human for approval.

## Authorization

Context `admin` or `super-admin` for `operational` and `messaging`; `super-admin` for
`attestation` ([conventions §2](../../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.role` (REQUIRED) — `attestation`, `operational` or `messaging`.
`payload.keyType` (REQUIRED) — the algorithm to generate.
`payload.label` (OPTIONAL) — operator-facing label; never published.
`payload.expectedVersionId`, `payload.dryRun`, `payload.previewId`, `payload.reason` — as in [conventions §3](../../../../../_shared/0.3/CONVENTIONS.md#3-approval).
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

### Preview adding a post-quantum attestation key

```json
{
  "id": "urn:uuid:7c2d0e11-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/add/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-25T10:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "attestation",
    "keyType": "mldsa44",
    "expectedVersionId": "7-QmEntrySeven",
    "dryRun": true,
    "reason": "Sign credentials with an ML-DSA-44 proof alongside Ed25519"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-25T10:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:7c2d0e11-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/add/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-09-25T10:00:01Z",
  "threadId": "urn:uuid:7c2d0e11-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "preview",
    "preview": {
      "previewId": "pv_4f9c1e0a2b7d4c8e9a1f",
      "baseVersionId": "7-QmEntrySeven",
      "expiresAt": "2026-09-25T10:30:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z2SyPqAttestation", "keyType": "mldsa44", "publicKeyMultibase": "z2SyPqAttestation", "relationships": ["assertionMethod"] },
        { "op": "rotateUpdateKey", "role": "update" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example", "keyRoles": { "attestation": ["#z6MkNewAttestation", "#z2SyPqAttestation"], "operational": ["#z6MkOperational"], "messaging": ["#z6LSMessaging"] } },
      "warnings": ["algorithmNotInAcceptedSet"]
    },
    "keys": [
      { "role": "attestation", "keyType": "mldsa44", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z2SyPqAttestation", "publicKeyMultibase": "z2SyPqAttestation", "relationships": ["assertionMethod"], "state": "pending" }
    ]
  }
}
```

The `document` above is abbreviated; a VTA returns the complete document. Applying is the
same request with `dryRun` removed and `"previewId": "pv_4f9c1e0a2b7d4c8e9a1f"` added; where
policy requires a step-up the first apply is answered `vta/webvh/dids:stepUpRequired`.

## Security & Privacy

### Data carried

The request carries a DID, a role, an algorithm and control members — no key material, and no
identifier for the new key: the VTA generates the key and derives its fragment, so a request
cannot plant a key of the caller's choosing. The response carries the preview and the new key's
public record. An attestation key generated here can never be exported, backed up or re-derived
([conventions §4](../../../../../_shared/0.3/CONVENTIONS.md#4-custody-and-exportability)); it ends only by `keys/retire` (destroyed) or
`keys/revoke`. A conforming VTA **MUST NOT** place key material in `ext`.

The preview hides nothing the entry does: `updateKeyRotates` is always stated, and
`algorithmNotInAcceptedSet` tells the approver that some verifiers the VTA knows of cannot yet check
the new key — harmless for a proof-set addition, and exactly what they need to know before
rotating to it.

### Correlation

Adding a key publishes, to every resolver, when the node's key set changed. Attestation keys are a
coarse timestamp on everything they sign (VTI-KEY-153): an operator **SHOULD** add them on a
schedule independent of any member's admission, renewal or departure. The label and reason are
never published.

### Retention

The key's custodian record, the rotation record and the audit row are kept for the DID's lifetime.
A preview that is never applied is discarded at `expiresAt`, and the key generated for it is
destroyed.

### Consent/purpose

The purpose is to extend one role with one key. The consent surface renders the VTA's preview —
the exact key, the exact document, the update-key rotation — and an approval is bound to its
`previewId` ([conventions §3](../../../../../_shared/0.3/CONVENTIONS.md#3-approval)), so it cannot be spent on anything else.
