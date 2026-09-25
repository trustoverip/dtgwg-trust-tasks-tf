---
slug: vta/webvh/dids/keys/list
version: "1.0"
title: "VTA WebVH DIDs — Keys List"
summary: "List a did:webvh's keys by role — attestation, operational, messaging, update — with each key's state, and optionally the rotation history."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - vta
  - webvh
  - key roles
  - rotation history
  - custody
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: "A read that changes nothing, but whose custody projection names pending keys, approvers and the reasons for past revocations. The projection is chosen by who the caller is, so the VTA must be able to rely on that; over a transport that authenticates the sender the transport suffices, and over one that does not the proof is what does."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "Read-only. A replay returns the current state to the party that is entitled to it, which is no disclosure."
sideEffects:
  level: none
  rationale: "Reads the VTA's key-role records and the DID's log. Writes nothing."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Public keys, verification-method identifiers and state — in the public projection exactly what the DID's log already reveals; in the custody projection also the VTA's own custody records, which are metadata about keys and never the keys."
retention:
  class: transient
  rationale: "Answered from state the VTA already holds; nothing about the request needs keeping beyond the audit trail's access record."
errorCodes:
  - code: "vta/webvh/dids:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context. See [conventions](../../../../../_shared/0.3/CONVENTIONS.md#9-family-error-codes)."
    retryable: false
  - code: "vta/webvh/dids:notKeyRoleIdentity"
    meaning: "The DID was not created with key roles. Create a new identity with key roles instead."
    retryable: false
related:
  - vta/webvh/dids/keys/add
  - vta/webvh/dids/rotate-keys
  - vta/webvh/dids/keys/retire
  - vta/webvh/dids/keys/revoke
  - vta/webvh/dids/get
  - keys/list
---

## Abstract

**VTA WebVH DIDs — Keys List** answers, for one DID the VTA holds keys for, *which key
does what*: the keys in each of the DID's [key roles](../../../../../_shared/0.3/CONVENTIONS.md#1-roles-and-the-relationships-they-own)
— `attestation`, `operational`, `messaging` and `update` — with each key's state,
and on request the history of every change to them.

It is the screen every client opens first, and the one it re-reads to show progress:
a rotation in its overlap window, a change waiting for a second approval, a key revoked
for compromise.

[`keys/list`](../../../../../../keys/list/0.1/spec.md) cannot answer this. It lists a
custodian's key records, which know nothing about DIDs, relationships or roles; and it
is custodian-wide, where this is scoped to one DID and so to the one context whose
authorization governs it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../../../_shared/0.3/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Answer `vta/webvh/dids:notFound` for a DID it does not hold **and** for one whose
   context the caller cannot reach, indistinguishably.
2. Choose the projection from the caller's authority
   ([conventions §5](../../../../../_shared/0.3/CONVENTIONS.md#5-what-a-caller-sees)),
   state it in `projection`, and **MUST NOT** include in a `public` answer anything
   §5 reserves to `custody`. The caller does not ask for a projection: a request member
   choosing one would be a request member to escalate with.
3. Derive every key's role and relationships from its own records *and* check them
   against the document published at `versionId`. A DID whose document does not bind
   every key to exactly one role, in exactly its role's relationship, is not a key-role
   identity: refuse it with `vta/webvh/dids:notKeyRoleIdentity`
   ([conventions §10](../../../../../_shared/0.3/CONVENTIONS.md#10-only-key-role-identities))
   rather than report what its records say it should be.
4. Return every role asked for, including one with no keys.
5. Report the key record behind each key (`custody.keyId`) in the custody projection,
   and state `exportable` and `neverExportable` explicitly — never by omission — so an
   operator can confirm from this answer alone that the `attestation` and
   `update` keys cannot leave the VTA.

A conforming **producer** that intends to change a role on the strength of this answer
**SHOULD** pass `versionId` as the change's `expectedVersionId`.

## Authorization

Any ACL role that reaches the DID's context may read the public projection; the custody
projection requires context `admin` or `super-admin`
([conventions §2](../../../../../_shared/0.3/CONVENTIONS.md#2-authorization--who-may-change-which-role)).

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.roles` (OPTIONAL) — restrict the answer to these roles.
`payload.includeEnded` (OPTIONAL) — include retired and revoked keys.
`payload.includeHistory` (OPTIONAL) — include the rotation history.
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

```json
{
  "id": "urn:uuid:5a1e0c2b-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/list/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-25T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "includeHistory": true
  }
}
```

## Response

A VTC's DID in the middle of an attestation rotation, answered in the custody projection.
The predecessor is still published and `retiring`; the successor is `active`.

```json
{
  "id": "urn:uuid:5a1e0c2b-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/keys/list/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkAdmin",
  "issuedAt": "2026-09-25T09:00:01Z",
  "threadId": "urn:uuid:5a1e0c2b-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "versionId": "7-QmEntrySeven",
    "projection": "custody",
    "roles": [
      {
        "role": "attestation",
        "relationships": ["assertionMethod"],
        "keys": [
          {
            "verificationMethod": "did:webvh:QmVtcScid:vtc.example#key-4",
            "role": "attestation",
            "relationships": ["assertionMethod"],
            "keyType": "ed25519",
            "publicKeyMultibase": "z6MkOldAttestation",
            "state": "retiring",
            "publishedInVersionId": "2-QmEntryTwo",
            "retireAfter": "2026-10-03T09:00:00Z",
            "rotationId": "rot-0003",
            "custody": { "keyId": "vtc-attestation-1", "origin": "internal", "exportable": false, "neverExportable": true, "inBackups": false }
          },
          {
            "verificationMethod": "did:webvh:QmVtcScid:vtc.example#key-9",
            "role": "attestation",
            "relationships": ["assertionMethod"],
            "keyType": "ed25519",
            "publicKeyMultibase": "z6MkNewAttestation",
            "state": "active",
            "publishedInVersionId": "7-QmEntrySeven",
            "rotationId": "rot-0003",
            "custody": { "keyId": "vtc-attestation-2", "origin": "internal", "exportable": false, "neverExportable": true, "inBackups": false }
          }
        ]
      },
      {
        "role": "operational",
        "relationships": ["authentication"],
        "keys": [
          {
            "verificationMethod": "did:webvh:QmVtcScid:vtc.example#key-5",
            "role": "operational",
            "relationships": ["authentication"],
            "keyType": "ed25519",
            "publicKeyMultibase": "z6MkOperational",
            "state": "active",
            "publishedInVersionId": "2-QmEntryTwo",
            "custody": { "keyId": "vtc-operational-1", "origin": "derived", "exportable": false, "neverExportable": false, "inBackups": true }
          }
        ]
      },
      {
        "role": "messaging",
        "relationships": ["keyAgreement"],
        "keys": [
          {
            "verificationMethod": "did:webvh:QmVtcScid:vtc.example#key-6",
            "role": "messaging",
            "relationships": ["keyAgreement"],
            "keyType": "x25519",
            "publicKeyMultibase": "z6LSMessaging",
            "state": "active",
            "publishedInVersionId": "2-QmEntryTwo",
            "custody": { "keyId": "vtc-messaging-1", "origin": "derived", "exportable": false, "neverExportable": false, "inBackups": true }
          }
        ]
      },
      {
        "role": "update",
        "relationships": [],
        "preRotationCommitments": 2,
        "keys": [
          {
            "role": "update",
            "relationships": [],
            "keyType": "ed25519",
            "publicKeyMultibase": "z6MkUpdateKey",
            "state": "active",
            "publishedInVersionId": "7-QmEntrySeven",
            "custody": { "keyId": "vtc-update-7", "origin": "internal", "exportable": false, "neverExportable": true, "inBackups": false }
          }
        ]
      }
    ],
    "rotations": [
      {
        "rotationId": "rot-0003",
        "role": "attestation",
        "kind": "planned",
        "state": "overlapping",
        "predecessors": ["did:webvh:QmVtcScid:vtc.example#key-4"],
        "successors": ["did:webvh:QmVtcScid:vtc.example#key-9"],
        "startedInVersionId": "7-QmEntrySeven",
        "startedAt": "2026-09-25T08:40:00Z",
        "cacheHorizonAt": "2026-09-26T08:41:00Z",
        "activatesAt": "2026-09-26T08:41:00Z",
        "overlapUntil": "2026-10-03T09:00:00Z",
        "autoRetire": true,
        "initiatedBy": "did:key:z6MkAdmin",
        "approvals": { "required": 2, "received": 2, "approvers": ["did:key:z6MkAdmin", "did:key:z6MkSecondAdmin"], "expiresAt": "2026-09-25T09:40:00Z" },
        "reason": "Annual attestation rotation"
      }
    ]
  }
}
```

The same DID in the public projection shows both attestation keys as `active`, and
carries no `retireAfter`, `custody`, `label`, `overlapUntil`, `autoRetire`,
`initiatedBy`, `approvals` or `reason`.

## Security & Privacy

### Data carried

The request carries a DID and three filters. The public projection returns only what
resolving the DID and reading its log already reveals. The custody projection adds the
VTA's custody records — key-record identifiers, whether each key can leave, pending
keys, deadlines, initiators, approvers, reasons — which is operationally sensitive
metadata and never key material. No private key material is carried in either
projection, and a conforming VTA **MUST NOT** place any in `ext`.

### Correlation

Answering `notFound` alike for "not held" and "not yours" keeps the task from being an
oracle for which DIDs a VTA holds keys for. The custody projection names
administrators' VIDs; that is why it is reserved to administrators of the same context.

### Why `staged` and `retiring` are custody-only

A published key that is `staged` or `retiring` is one the VTA is not signing with.
Publishing that distinction tells an observer which key a stolen copy
would be least likely to be noticed misusing, and gives a verifier nothing it needs —
both keys are valid for the overlap.

### Retention

The VTA keeps nothing about the request beyond the audit trail's access record. A caller that keeps
a custody-projection answer holds administrators' VIDs and revocation reasons, and **SHOULD** keep it
no longer than the task it was read for.

### Consent/purpose

The purpose is to show an operator the state of a DID's keys so they can decide on a change. A
caller **MUST NOT** use repeated reads across a VTA's DIDs to build an inventory of its custody.
