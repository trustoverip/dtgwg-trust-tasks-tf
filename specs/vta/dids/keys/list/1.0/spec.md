---
slug: vta/dids/keys/list
version: "1.0"
title: "VTA DIDs — Keys List"
summary: "Read the key roles of a key-role identity under any eligible DID method: each role's keys and states, the update authority and whether a recovery key is committed, and the rotation history — in a public or a custody projection."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - vta
  - did
  - key roles
  - list
  - rotation history
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: "A read; the transport authenticates the caller, and the projection returned depends only on the caller's ACL entry."
sideEffects:
  level: none
  rationale: "Reads custodian and rotation records; changes nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Public key records, and for an administrator the custody state: which boundary holds a key, overlap deadlines, initiators, approvers and reasons. No key material."
retention:
  class: transient
  rationale: "The answer describes a version that the next change supersedes; nothing in it is evidence."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/dids:methodNotEligible"
    meaning: "The identity's method has no eligible profile."
    retryable: false
related:
  - vta/dids/keys/add
  - vta/dids/keys/rotate
  - vta/dids/keys/retire
  - vta/dids/keys/revoke
---

## Abstract

**VTA DIDs — Keys List** answers what a key-role identity's keys are, role by role, and what
has happened to them. It is the read every client shows before and after a change: the keys
in each role with their states, the update authority — including whether a recovery key is
committed (`recoveryCommitted`), which a client **SHOULD** show prominently when it is
`false` — and, on request, the rotation history.

The answer is in one of two projections
([conventions §11](../../../_shared/0.1/CONVENTIONS.md#11-what-a-caller-sees)): `public`, which
says nothing the published history does not, or `custody`, which adds pending keys, the
boundary and protection behind each key, schedules, initiators, approvers and reasons. The
response states which.

This task does not offer a single-key `get`. A key is identified by its verification method
within one identity, and every question about a key is a question about its role, so a list
filtered by `roles` answers it; a key that is not there is a statement the caller can act on,
not an existence oracle, because only callers who can reach the identity's context get any
answer at all (`vta/dids:notFound` otherwise).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../../_shared/0.1/CONVENTIONS.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse an identity that is not a key-role identity, without guessing roles
   (`vta/dids:notKeyRoleIdentity`).
2. Choose the projection from the caller's ACL entry
   ([§7](../../../_shared/0.1/CONVENTIONS.md#7-authorization)), and withhold from the public
   projection everything §11 lists.
3. Return one `KeyRoleSummary` per role asked for, including an empty one, and for `update`
   report `recoveryCommitted` truthfully, and `assuranceLevel` as a verifier would establish it
   (VTI-KEY-199) — below the level the document declares, if its structure does not support
   it, which is a defect the VTA reports rather than hides.
4. Always include a `revoked` key the document still lists.

## Authorization

Any ACL role reaching the identity's context sees the public projection; context `admin` and
`super-admin` see the custody projection.

## Request

```json
{
  "id": "urn:uuid:3c2b1a00-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/keys/list/1.0",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-04T08:00:00Z",
  "payload": { "did": "did:webvh:QmVtcScid:vtc.example", "roles": ["attestation", "update"] }
}
```

## Response

```json
{
  "id": "urn:uuid:3c2b1a00-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/keys/list/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkContextAdmin",
  "issuedAt": "2026-11-04T08:00:01Z",
  "threadId": "urn:uuid:3c2b1a00-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "method": "webvh",
    "versionId": "4-QmEntryFour",
    "projection": "custody",
    "roles": [
      {
        "role": "attestation",
        "relationships": ["assertionMethod"],
        "keys": [
          {
            "verificationMethod": "did:webvh:QmVtcScid:vtc.example#z6MkAttest",
            "role": "attestation",
            "relationships": ["assertionMethod"],
            "keyType": "ed25519",
            "publicKeyMultibase": "z6MkAttest",
            "state": "active",
            "publishedInVersionId": "1-QmEntryOne",
            "custody": { "keyId": "vtc-example:attest-1", "origin": "internal", "exportable": false, "neverExportable": true, "inBackups": false, "protection": "enclave", "boundary": "vta-primary" }
          }
        ]
      },
      {
        "role": "update",
        "relationships": [],
        "recoveryCommitted": true,
        "keys": [
          { "role": "update", "updateKeyClass": "routine", "keyType": "ed25519", "publicKeyMultibase": "z6MkRoutineFour", "state": "active", "publishedInVersionId": "4-QmEntryFour" },
          { "role": "update", "updateKeyClass": "routine", "keyType": "ed25519", "commitment": "QmRoutineFiveHash", "state": "committed", "publishedInVersionId": "4-QmEntryFour" },
          { "role": "update", "updateKeyClass": "recovery", "keyType": "ed25519", "commitment": "QmRecoveryHash", "state": "committed", "publishedInVersionId": "1-QmEntryOne", "custody": { "exportable": false, "neverExportable": true, "inBackups": false, "protection": "externalCustodian", "boundary": "board-safe" } }
        ]
      }
    ]
  }
}
```

Failures use `trust-task-error`.

## Security & Privacy

### Data carried

Public key records; for administrators, custody state. No key material, in any member or
`ext`. Boundary labels are opaque and VTA-assigned.

### Correlation

The public projection adds nothing to what resolving the identity reveals. The custody
projection reveals the deployment's custody layout, which is why it is limited to the
identity's administrators; `staged` and `retiring` are withheld from everyone else because
they tell an observer which published key is not in use.

### Retention

Transient: a caller **SHOULD NOT** retain the custody projection beyond the session that
asked for it.

### Consent/purpose

The purpose is to show an identity's key state to those who manage or rely on it.
