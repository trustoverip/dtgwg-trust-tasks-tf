---
slug: vta/webvh/dids/keys/migrate
version: "1.0"
title: "VTA WebVH DIDs — Keys Migrate"
summary: "A super-admin moves a legacy did:webvh — one key signing everything and holding the log's update authority — onto key roles in one log entry, with a published grace period for what the legacy key already signed."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - vta
  - webvh
  - key roles
  - migration
  - legacy identity
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
  rationale: "Migration replaces the key that signs the identity's credentials and transfers the authority over its log. The VTA must attribute that to one administrator independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Appends an unretractable entry that changes every role at once; it must be placeable in time."
sideEffects:
  level: destructive
  rationale: "Authority-shifting in every role at once: new attestation and operational keys, the legacy key withdrawn from assertionMethod, and the update authority transferred to a generated update key. A published grace period bounds how long anything the legacy key signed is accepted."
consequences:
  - "New attestation keys, generated inside the VTA and never exportable, become the only keys that can sign this identity's credentials and status lists."
  - "The legacy key stops being trusted for credentials; everything it signed is accepted only until the grace period ends, and status lists signed by it stop being accepted at once."
  - "Authority over the DID's log moves to a generated update key."
  - "Every status list must be re-signed now, and every other credential still in force re-issued before the grace period ends."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the preview and the public records of every key involved."
retention:
  class: durable
  rationale: "The migration entry and its grace period are what verifiers judge every legacy signature against."
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
    meaning: "A requested key type cannot serve its role, or is not in the VTA's accepted set."
    retryable: false
  - code: "vta/webvh/dids/keys/migrate:alreadyMigrated"
    meaning: "The DID already carries key roles and publishes no unassigned key."
    retryable: false
  - code: "vta/webvh/dids/keys/migrate:gracePeriodTooLong"
    meaning: "`gracePeriodEnd` is later than the latest expiry of any artefact the legacy key signed, or than the community's maximum membership period."
    retryable: false
related:
  - vta/webvh/dids/keys/list
  - vta/webvh/dids/keys/add
  - vta/webvh/dids/keys/retire
  - vta/webvh/dids/rotate-keys
---

## Abstract

A DID created before key roles typically has one Ed25519 key in both `authentication` and
`assertionMethod`, held by the service process, signing everything the node signs, and
authorizing its own log entries. It is a **legacy identity** (VTI-KEY-140).

**VTA WebVH DIDs — Keys Migrate** moves it onto key roles **in one entry**: it adds one or
more generated `attestation` keys and a new `operational` key, adds `keyRoles`, withdraws
the legacy key from `assertionMethod`, moves the update authority to a generated `update`
key (through the committed successor, where there is one), and records the end of a grace
period for what the legacy key already signed.

It has to be one entry. Assembled from separate `keys/add` and `keys/retire` entries, the
migration would publish — for as long as the steps take — a document with the legacy key
and the new attestation key both in `assertionMethod` and nothing to tell a verifier which
is meant, which is exactly the ambiguity key roles exist to end.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../../../_shared/0.3/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**, in one entry:

1. Generate one `attestation` key per `attestationKeyTypes` entry, inside its protection
   boundary, non-exportable and excluded from backups, and publish them in
   `assertionMethod`.
2. Publish a new `operational` key in `authentication`. The legacy key **MUST NOT** be bound
   to `attestation` (VTI-KEY-141); it **MAY** stay in `authentication`, bound to
   `operational`, until `legacyOperationalUntil`, after which the VTA retires it.
3. Bind existing `keyAgreement` keys to `messaging` when they are distinct from the legacy
   signing key, and otherwise publish a new messaging key.
4. Remove every method from `capabilityInvocation` and `capabilityDelegation` (conventions §1 item 2).
5. Add `keyRoles` agreeing with all of the above.
6. Move the update authority to a generated `update` key, by way of the committed successor
   where one exists, and commit fresh successors.
7. Record `gracePeriodEnd` in the entry, refusing one the rules of VTI-KEY-143 forbid
   (`vta/webvh/dids/keys/migrate:gracePeriodTooLong`). Until the VTI specification fixes the
   encoding, the VTA records it as a `gracePeriodEnd` member of the legacy key's
   verification method.

and then:

8. Refuse every other key-role task for a DID that is not yet migrated
   (`vta/webvh/dids:legacyKeysPresent`).
9. Tell the node, through the response and the audit row, that it must re-sign every status
   list at once (VTI-KEY-142) and re-issue every other attestation artefact in force before
   `gracePeriodEnd` (VTI-KEY-143).

A VTA **SHOULD** migrate its own identity before, or together with, the identities of the
nodes provisioned on it (VTI-KEY-145).

## Authorization

`super-admin` ([conventions §2](../../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.attestationKeyTypes` (REQUIRED) — one attestation key per entry.
`payload.operationalKeyType` (OPTIONAL) — default `ed25519`.
`payload.legacyOperationalUntil` (OPTIONAL) — how long the legacy key may stay operational.
`payload.gracePeriodEnd` (REQUIRED) — the end of the grace period for legacy attestation artefacts.
`payload.expectedVersionId`, `payload.dryRun`, `payload.previewId`, `payload.reason` — as in [conventions §3](../../../../../_shared/0.3/CONVENTIONS.md#3-approval).
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

```json
{
  "id": "urn:uuid:c0ffee00-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/migrate/1.0",
  "issuer": "did:key:z6MkSuperAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-25T12:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "attestationKeyTypes": ["ed25519", "mldsa44"],
    "legacyOperationalUntil": "2026-10-02T12:00:00Z",
    "gracePeriodEnd": "2027-03-31T00:00:00Z",
    "dryRun": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-25T12:00:00Z",
    "verificationMethod": "did:key:z6MkSuperAdmin#z6MkSuperAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:c0ffee00-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/migrate/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkSuperAdmin",
  "issuedAt": "2026-09-25T12:00:02Z",
  "threadId": "urn:uuid:c0ffee00-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "preview",
    "preview": {
      "previewId": "pv_0a1b2c3d4e5f6a7b8c9d",
      "baseVersionId": "3-QmEntryThree",
      "expiresAt": "2026-09-25T12:30:00Z",
      "updateKeyRotates": true,
      "changes": [
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkAttEd", "keyType": "ed25519", "publicKeyMultibase": "z6MkAttEd", "relationships": ["assertionMethod"] },
        { "op": "addKey", "role": "attestation", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z2SyAttPq", "keyType": "mldsa44", "publicKeyMultibase": "z2SyAttPq", "relationships": ["assertionMethod"] },
        { "op": "addKey", "role": "operational", "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkOps", "keyType": "ed25519", "publicKeyMultibase": "z6MkOps", "relationships": ["authentication"] },
        { "op": "setKeyRoles", "role": "attestation" },
        { "op": "rotateUpdateKey", "role": "update" },
        { "op": "setGracePeriod", "role": "attestation" }
      ],
      "document": { "id": "did:webvh:QmVtcScid:vtc.example" },
      "warnings": ["attestationReissuanceRequired"]
    },
    "keys": [],
    "rotation": { "rotationId": "rot-0001", "role": "attestation", "kind": "migration", "state": "pendingApproval", "gracePeriodEnd": "2027-03-31T00:00:00Z" }
  }
}
```

(`document` and `keys` abbreviated.)

## Security & Privacy

### Data carried

The request carries a DID, algorithms and two dates; the response, the preview and public records.
No key material in either direction; the attestation and update keys generated here never leave the
VTA ([conventions §4](../../../../../_shared/0.3/CONVENTIONS.md#4-custody-and-exportability)).

The legacy key has lived in the service process, so its custody cannot be retrofitted and it is not
destroyed at migration: VTI-KEY-132 cannot bound what it signs. The grace period is a stated,
bounded acceptance of risk that already existed, not a judgement that the legacy signatures are
sound — which is why it is published, capped (VTI-KEY-143), and why status lists get none.

### Correlation

The migration entry publishes that the node moved to key roles and when, which every resolver learns
anyway. Fragments derived from the keys (VTI-KEY-150) reveal nothing about the node's layout.

### Retention

The migration entry and its grace period are permanent parts of the log.

### Consent/purpose

The purpose is to end a key's dual use. The consent surface renders the full preview and states the
re-signing obligation and the grace period's end in words.
