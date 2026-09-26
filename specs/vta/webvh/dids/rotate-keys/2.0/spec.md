---
slug: vta/webvh/dids/rotate-keys
version: "2.0"
title: "VTA WebVH DIDs — Rotate Keys"
summary: "An administrator rotates one key role of a did:webvh: the successor is staged, activated once every cached document can have seen it, and the predecessor retired at the end of an overlap. For the update role, the log's update key moves to its committed successor."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - rotate
  - keys
  - key roles
  - pre-rotation
  - overlap
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
  rationale: "A rotation changes which keys speak for an identity. The VTA must attribute the change to a specific administrator independently of the transport that carried it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Replayed after a later rotation, a rotation would retire the keys that rotation installed. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Replaces the keys of a role — for the attestation role, the key that signs the community's credentials and status lists; for the update role, the sole authority over the DID's log. The published entries cannot be withdrawn, and a retired attestation key is destroyed."
consequences:
  - "A new key takes over this role. From its activation the old key is never used again."
  - "When the overlap ends the old key is removed from the DID; a retired attestation key is destroyed and can never sign again."
  - "Appending the entry also rotates this DID's update key to its committed successor."
  - "Appends to a public, append-only log; the entry cannot be withdrawn, only superseded."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview, the rotation record and the public records of the keys involved. No private key material in either direction."
retention:
  class: durable
  rationale: "The rotation record, the keys' custodian records and the audit rows are the DID's rotation history — what a verifier's question about a past signature, and an incident review, are answered from."
errorCodes:
  - code: "vta/webvh/dids:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/webvh/dids:keyNotFound"
    meaning: "A key named in `replace` is not an active key of the role."
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
    meaning: "The successor's key type cannot serve the role, or is not in the VTA's accepted set."
    retryable: false
  - code: "vta/webvh/dids:rotationInProgress"
    meaning: "The role already has a change staged, overlapping or awaiting approval."
    retryable: false
  - code: "vta/webvh/dids:preRotationRequired"
    meaning: "The rotation would leave a durable node identity with no committed successor update key."
    retryable: false
  - code: "vta/webvh/dids:notKeyRoleIdentity"
    meaning: "The DID was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/webvh/dids/rotate-keys:overlapTooShort"
    meaning: "`activatesAt` is earlier than the publishing entry's cache horizon (publication + max(document TTL, verifier cache cap)), or `overlapUntil` is not after `activatesAt`."
    retryable: false
  - code: "vta/webvh/dids/rotate-keys:notApplicableToRole"
    meaning: "A member was given that the role does not take — `replace`, `keyType`, `activatesAt`, `overlapUntil` or `autoRetire` for `role: update`."
    retryable: false
related:
  - vta/webvh/dids/keys/list
  - vta/webvh/dids/keys/retire
  - vta/webvh/dids/keys/revoke
  - vta/webvh/dids/keys/add
  - vta/webvh/dids/update
  - auth/step-up/approve-request
---

## Abstract

**VTA WebVH DIDs — Rotate Keys** is the **planned** rotation of one
[key role](../../../../_shared/0.3/CONVENTIONS.md#1-roles-and-the-relationships-they-own)
of a DID. It replaces keys believed sound; everything they signed stays valid.
A key believed compromised is not rotated but revoked, with
[`keys/revoke`](../../keys/revoke/1.0/spec.md), which is a different act with a different
record.

A planned rotation has four phases ([conventions §11](../../../../_shared/0.3/CONVENTIONS.md#11-overlap-by-role)),
and this task starts the first:

| Phase | What happens | Successor | Predecessor | Earliest |
|---|---|---|---|---|
| **1. Publish** (this task) | One entry publishes the successor in the role | `staged` | `active` | — |
| **2. Wait** | Nothing is published; caches catch up | `staged` | `active` | until the **cache horizon**: publication + max(document TTL, verifier cache cap — 24 h proposed, VTI-KEY-134) |
| **3. Switch** (`activatesAt`, the VTA) | The VTA begins signing with the successor and never again with the predecessor | `active` | `retiring` | ≥ cache horizon |
| **4. Retire** (`overlapUntil`, the VTA or [`keys/retire`](../../keys/retire/1.0/spec.md)) | A second entry removes the predecessor | `active` | `retired` | > `activatesAt` |

Publishing before switching is what makes a rotation invisible to relying parties
(VTI-KEY-122): a verifier holding a cached copy learns of the successor only when it
re-resolves, and a signature by a key it has never seen looks exactly like a forgery. The
overlap after the switch keeps what the predecessor signed verifiable against the current
document while caches catch up.

What each role does during the overlap:

| Role | Overlap |
|---|---|
| `attestation` | Both keys in `assertionMethod`; new artefacts signed only by the active key. At retirement the predecessor is destroyed, and what it signed still verifies by the DID version at issuance. |
| `operational` | Both keys in `authentication`; new traffic signed only by the active key. |
| `messaging` | Both keys in `keyAgreement`; the VTA **holds both private halves for the whole overlap**, and senders may encrypt to either (preferring the newer). The predecessor is destroyed at retirement. |
| `update` | **No overlap**: the pre-rotation handover moves the update key in one entry and commits a fresh next key. |

A revocation for compromise has no overlap in any role; it is [`keys/revoke`](../../keys/revoke/1.0/spec.md).

The `update` role is different in kind: a did:webvh's update key moves to the successor
the previous entry committed to, in one entry, with no overlap, and every entry under
pre-rotation does it anyway ([conventions §8](../../../../_shared/0.3/CONVENTIONS.md#8-every-entry-under-pre-rotation-rotates-the-update-key)).
A rotation of `update` is that move requested on its own, with nothing else changing.

## What changed from 1.0, and why it is a new major version

**Breaking.** Version 1.0 rotated *every* verification method of the DID at once, with no
notion of role, and was defective in four ways this version closes:

1. **It rotated everything.** 1.0 replaced every key in one entry, so rotating the
   operational key also replaced the attestation key and the messaging key. `role` is now
   REQUIRED, and nothing outside the role changes except the update key.
2. **It derived every successor as Ed25519.** An `x25519` key-agreement method received
   Ed25519 bytes, and an `mldsa44` key silently became classical. A successor now keeps its
   predecessor's algorithm unless `keyType` names another
   ([conventions §1 item 6](../../../../_shared/0.3/CONVENTIONS.md#1-roles-and-the-relationships-they-own)),
   and attestation successors are generated, not derived.
3. **It rewrote only three relationships.** Methods referenced from
   `capabilityInvocation` or `capabilityDelegation` kept pointing at the old key. Both are
   now reserved and must be empty (conventions §1 item 2), and a rotation rewrites the role
   everywhere it appears, including `keyRoles`.
4. **It left the new keys without custodian records.** The successors consumed derivation
   paths nobody recorded, so the next rotation, `realign-keys` and every `keys/*` task could
   not find them. Every published key now has its record, created atomically with the
   entry (conventions §1 item 5).

It also replaced keys instantly, with no staging and no overlap, and accepted
`preRotationCount: 0` on a node's durable DID.

1.0 remains deployed. A VTA implementing 2.0 **SHOULD** refuse 1.0 for any DID that has key
roles, with the framework's `unsupportedVersion`
([SPEC §8.3](/SPEC.md#83-standard-error-codes)), since 1.0 cannot rotate one role without
rotating the others. **Client impact:** no shipped client sends 1.0 today, so the break
falls on the SDK method (`rotate_keys`) and its tests, not on a user-facing command.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../../_shared/0.3/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Rotate only the keys of `role` — those named in `replace`, or every active key of the
   role when it is absent — and change nothing else in the document except `keyRoles` and
   the update key.
2. Generate one successor per predecessor, of `keyType` or else of the predecessor's own
   algorithm, when the preview is computed (conventions §3.1): generated inside its
   protection boundary for `attestation`; derived as the context's other keys for
   `operational` and `messaging`.
3. Publish each successor in the role's relationship and `keyRoles`, under a fragment
   derived from the key, with its custodian record created in the same atomic step, in
   state `staged`.
4. Compute the publishing entry's cache horizon (conventions §11.1) and report it as
   `rotation.cacheHorizonAt`. Set `activatesAt` no earlier than it (defaulting to it), and
   `overlapUntil` later than `activatesAt`; refuse a requested value that breaks either with
   `vta/webvh/dids/rotate-keys:overlapTooShort`. At `activatesAt`, begin signing with the
   successor and stop signing with the predecessor, and audit `did.keys.activate`. For
   `messaging`, keep both private halves and decrypt with either until retirement.
5. When `autoRetire` is not false, retire the predecessor at `overlapUntil` by appending
   the entry [`keys/retire`](../../keys/retire/1.0/spec.md) would, under this request's
   authorization, and audit `did.keys.autoRetire`. Never retire the predecessor before
   `activatesAt` has passed. Destroy the predecessor's private half when the retiring entry
   is published (VTI-KEY-125 for `attestation`).
6. For `role: update`, move the update key to a committed successor in one entry, commit
   fresh successors, and refuse any member the schema forbids for that role.
7. Refuse to leave a durable node identity with no committed successor
   (`vta/webvh/dids:preRotationRequired`).
8. Treat a revocation for compromise of a key in this rotation as superseding it
   (conventions §3.3): the rotation is recorded `aborted` and the revocation proceeds.

A conforming **consumer** **MUST NOT** change the DID's SCID, and **MUST** refuse a DID that is
not a key-role identity with `vta/webvh/dids:notKeyRoleIdentity`
([conventions §10](../../../../_shared/0.3/CONVENTIONS.md#10-only-key-role-identities)).

## Authorization

Context `admin` or `super-admin` for `operational` and `messaging`; `super-admin` for
`attestation` and `update` ([conventions §2](../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.role` (REQUIRED) — the one role to rotate.
`payload.replace` (OPTIONAL) — which of the role's active keys to rotate.
`payload.keyType` (OPTIONAL) — the successors' algorithm.
`payload.activatesAt`, `payload.overlapUntil`, `payload.autoRetire` (OPTIONAL) — the schedule.
`payload.preRotationCount` (OPTIONAL) — successor update keys to commit; at least 1.
`payload.label` (OPTIONAL) — operator-facing label; never published.
`payload.expectedVersionId`, `payload.dryRun`, `payload.previewId`, `payload.reason` — as in [conventions §3](../../../../_shared/0.3/CONVENTIONS.md#3-approval).
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

None of the schedule members, `replace` or `keyType` may be given for `role: update`; a VTA
**MUST** refuse such a request with `vta/webvh/dids/rotate-keys:notApplicableToRole`. (The rule is
stated here rather than as a schema conditional because the generated bindings cannot express one.)

## Request

### Apply a previewed rotation of a VTC's attestation key

```json
{
  "id": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/rotate-keys/2.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-25T08:40:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "role": "attestation",
    "overlapUntil": "2026-10-03T09:00:00Z",
    "expectedVersionId": "6-QmEntrySix",
    "previewId": "pv_1b2c3d4e5f60718293a4",
    "reason": "Annual attestation key rotation"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-25T08:40:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

The VTA's policy requires two approvals for an attestation rotation; the initiator's is
recorded, and the plan waits for the second.

```json
{
  "id": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/rotate-keys/2.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-09-25T08:40:01Z",
  "threadId": "urn:uuid:9e8d7c6b-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "pendingApproval",
    "approvals": { "required": 2, "received": 1, "approvers": ["did:key:z6MkSuperAdmin"], "expiresAt": "2026-09-25T09:40:00Z" },
    "preview": {
      "previewId": "pv_1b2c3d4e5f60718293a4",
      "baseVersionId": "6-QmEntrySix",
      "expiresAt": "2026-09-25T09:40:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttestation", "keyType": "ed25519", "publicKeyMultibase": "z6MkNewAttestation", "relationships": ["assertionMethod"] },
        { "op": "rotateUpdateKey", "role": "update" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example", "assertionMethod": ["#z6MkOldAttestation", "#z6MkNewAttestation"], "keyRoles": { "attestation": ["#z6MkOldAttestation", "#z6MkNewAttestation"] } }
    },
    "keys": [
      { "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkOldAttestation", "role": "attestation", "relationships": ["assertionMethod"], "keyType": "ed25519", "publicKeyMultibase": "z6MkOldAttestation", "state": "active" },
      { "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkNewAttestation", "role": "attestation", "relationships": ["assertionMethod"], "keyType": "ed25519", "publicKeyMultibase": "z6MkNewAttestation", "state": "pending", "activatesAt": "2026-09-26T09:41:00Z" }
    ],
    "rotation": {
      "rotationId": "rot-0003",
      "role": "attestation",
      "kind": "planned",
      "state": "pendingApproval",
      "predecessors": ["did:webvh:QmVtcScid:vtc.example#z6MkOldAttestation"],
      "successors": ["did:webvh:QmVtcScid:vtc.example#z6MkNewAttestation"],
      "overlapUntil": "2026-10-03T09:00:00Z",
      "autoRetire": true,
      "cacheHorizonAt": "2026-09-26T09:41:00Z",
      "activatesAt": "2026-09-26T09:41:00Z"
    }
  }
}
```

(`document` abbreviated.) When the second approver's bound approve-response arrives the VTA
publishes the entry itself; `keys/list` then shows the successor `staged` and later `active`.

## Security & Privacy

### Data carried

The request carries a DID, a role and a schedule; the response, the preview, the rotation record and
public key records. Successors are generated or derived inside the VTA; an attestation predecessor is
destroyed at retirement. No key material in either direction, and none in `ext`.

**Least change.** A rotation touches one role, so a compromise of the operational key is never an
occasion to disturb the attestation key, and the approval asked for describes exactly the role that
moves. **The update key moves every time**: the preview states `updateKeyRotates`, and a client
**MUST** show it ([conventions §8](../../../../_shared/0.3/CONVENTIONS.md#8-every-entry-under-pre-rotation-rotates-the-update-key)).

### Correlation

The entries publish when the role's keys changed. The `staged`/`retiring` distinction and the
schedule are custody data ([conventions §5](../../../../_shared/0.3/CONVENTIONS.md#5-what-a-caller-sees)).
An operator **SHOULD** rotate attestation keys on a schedule independent of any member's admission,
renewal or departure (VTI-KEY-153), and **SHOULD NOT** rotate them more often than policy requires.

### Retention

The rotation record and the keys' custodian records are kept for the DID's lifetime; they are its
rotation history.

### Consent/purpose

The purpose is to replace sound keys on the operator's schedule. The consent surface renders the
VTA's preview and the schedule in words ("the new key is published now, starts signing on 26 September at
09:41 once every cached copy of the DID can have seen it, and the old key is removed on
3 October"), and an approval is bound to the preview.
