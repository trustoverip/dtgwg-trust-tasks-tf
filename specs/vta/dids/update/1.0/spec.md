---
slug: vta/dids/update
version: "1.0"
title: "VTA DIDs — Update"
summary: "An administrator changes the non-key members of a key-role identity — services, aliases, method parameters such as witnesses — under any eligible DID method. Keys change only through the key-role tasks."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - key roles
  - update
  - services
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "An update publishes a new version of a public identity. The VTA must attribute it to an administrator independently of the transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Replayed after a later change, an update would restore the members that change replaced. Only a window in which the document is too old to execute prevents that."
sideEffects:
  level: destructive
  rationale: "Appends to a public, append-only history; under a method where every change moves the update authority (did:webvh), it also rotates the identity's update key."
consequences:
  - "A new version of the identity is published and cannot be withdrawn, only superseded."
  - "Under did:webvh, appending it also moves the identity's update key to its committed successor."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "Returns the preview and the published version. The request carries public document members and method parameters; no key material in either direction."
retention:
  class: durable
  rationale: "The audit row and the published version are part of the identity's history."
errorCodes:
  - code: "vta/dids:notFound"
    meaning: "No such identity is held by this VTA, or the caller cannot reach its context."
    retryable: false
  - code: "vta/dids:versionConflict"
    meaning: "The identity's latest version no longer matches `expectedVersionId`."
    retryable: false
  - code: "vta/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, applied, based on a superseded version, or for a different request."
    retryable: false
  - code: "vta/dids:stepUpRequired"
    meaning: "Policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/dids:notKeyRoleIdentity"
    meaning: "The identity was not created with key roles. Create a new identity with key roles instead."
    retryable: false
  - code: "vta/dids:methodNotEligible"
    meaning: "The identity's method has no eligible profile."
    retryable: false
  - code: "vta/dids:methodParameterUnsupported"
    meaning: "A `methodParameters` member is not in the profile's `UpdateParameters`, or is fixed at creation."
    retryable: false
  - code: "vta/dids/update:keyMembersManaged"
    meaning: "The document changes `verificationMethod`, a verification relationship or `keyRoles`; keys change only through the key-role tasks."
    retryable: false
  - code: "vta/dids/update:documentNotWritable"
    meaning: "`document` was given for a method whose document is derived rather than written by the controller."
    retryable: false
related:
  - vta/dids/create
  - vta/dids/keys/list
  - vta/dids/keys/rotate
---

## Abstract

**VTA DIDs — Update** publishes a new version of a key-role identity that changes anything
but its keys: services, `alsoKnownAs`, and the method's own parameters (did:webvh witnesses,
watchers, TTL). Keys, relationships and `keyRoles` are refused here
(`vta/dids/update:keyMembersManaged`) and change only through
[`keys/add`](../../keys/add/1.0/spec.md), [`keys/rotate`](../../keys/rotate/1.0/spec.md),
[`keys/retire`](../../keys/retire/1.0/spec.md) and [`keys/revoke`](../../keys/revoke/1.0/spec.md),
whose authority and approval rules are stricter: otherwise anyone allowed to add a service
endpoint could add a key to `assertionMethod`.

The update still changes the change authority where the method requires it — under
did:webvh, every entry moves the update key to its committed successor and carries the
recovery commitment forward — and the preview says so (`updateKeyRotates`).

This task is for key-role identities only. A pre-role did:webvh identity keeps
`vta/webvh/dids/update/1.0` until it is replaced
([conventions §3.1](../../_shared/0.1/CONVENTIONS.md#31-only-key-role-identities)).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply. The [key-role conventions](../../_shared/0.1/CONVENTIONS.md) and the governing [method profile](../../_shared/0.1/method-profiles/README.md) are part of this specification.

A conforming **consumer** (the VTA) **MUST**:

1. Refuse an identity that is not a key-role identity (`vta/dids:notKeyRoleIdentity`).
2. Refuse a `document` whose `verificationMethod`, verification relationships or `keyRoles`
   differ from the current version's (`vta/dids/update:keyMembersManaged`), and a `document`
   for a method whose document is derived (`vta/dids/update:documentNotWritable`).
3. Validate `methodParameters` against the profile's `UpdateParameters`.
4. Carry the recovery commitment forward unchanged, and move the update authority only as
   the profile requires
   ([§5.3](../../_shared/0.1/CONVENTIONS.md#53-the-change-authority-a-routine-key-and-a-recovery-key)).
5. Follow the preview, step-up and approval rules of
   [conventions §6](../../_shared/0.1/CONVENTIONS.md#6-approval), and audit `did.update`.

## Authorization

Context `admin` or `super-admin` over the identity's context
([conventions §7](../../_shared/0.1/CONVENTIONS.md#7-authorization)). The proof establishes the
caller; the ACL entry is the authority.

## Payload

`payload.did` is REQUIRED. `document`, `methodParameters`, `label`, `expectedVersionId`,
`dryRun`, `previewId` and `reason` are OPTIONAL. A request with neither `document` nor
`methodParameters` changes nothing but the update authority, and is refused with
`malformedRequest`: rotating the update key is [`keys/rotate`](../../keys/rotate/1.0/spec.md).

## Request

### Add witnesses to a VTC's identity

```json
{
  "id": "urn:uuid:7b1e0d55-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/update/1.0",
  "issuer": "did:key:z6MkContextAdmin",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "methodParameters": { "witnesses": { "threshold": 1, "witnesses": ["did:key:z6MkWitnessOne"] } },
    "expectedVersionId": "3-QmEntryThree",
    "previewId": "pv_0a1b2c3d4e5f60718293"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-03T10:00:00Z",
    "verificationMethod": "did:key:z6MkContextAdmin#z6MkContextAdmin",
    "proofPurpose": "authentication",
    "proofValue": "z2Hq..."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:7b1e0d55-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/update/1.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkContextAdmin",
  "issuedAt": "2026-11-03T10:00:02Z",
  "threadId": "urn:uuid:7b1e0d55-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmVtcScid:vtc.example",
    "outcome": "applied",
    "newVersionId": "4-QmEntryFour",
    "publication": { "selfPublish": false }
  }
}
```

Failures use `trust-task-error`.

## Security & Privacy

### Data carried

Public document members and method parameters in; the preview and version identifier out. No
key material. A producer **MUST NOT** put anything in a service entry or alias it would not
publish.

### Correlation

The new version is public. A service endpoint can reveal hosting and operator; the
document's privacy rules (VTI-KEY-150) still bind what a producer adds.

### Retention

The audit row is kept for the identity's life.

### Consent/purpose

The purpose is to keep the identity's reachability and method configuration current. The
consent surface renders the VTA's preview, including any movement of the update key.
