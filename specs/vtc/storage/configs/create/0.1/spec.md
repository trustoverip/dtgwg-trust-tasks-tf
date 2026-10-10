---
slug: vtc/storage/configs/create
version: "0.1"
title: "VTC Storage — Configs — Create"
summary: A room-hosting administrator adds a storage config — a named local, S3, GCS or Walrus backend and how the community authenticates to it — that data rooms can then be assigned to. Parked for another administrator's consent.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request decides where members' files may be sent and which account receives them. It is parked for other administrators' consent and must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Bounding replay keeps a captured request from re-creating a config that administrators have since retired under another identifier.
sideEffects:
  level: mutating
  rationale: >-
    Creates one storage config. Nothing is stored on it until a room is assigned to it; it can be retired.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries backend settings, an authentication mode and the name of an external account — never a secret; a `sealed` credential is set separately with vtc/storage/credentials/set. The response echoes the stored config.
retention:
  class: durable
  rationale: >-
    The config persists until retired, and is never deleted while a blob names it; the accepted document is the record of who added a place members' files can go.
errorCodes:
  - code: vtc/storage/configs/create:exists
    meaning: "A config with this identifier exists, or existed and was retired; identifiers are never reused, because blobs name their config by it."
    retryable: false
  - code: vtc/storage/configs/create:invalidSettings
    meaning: "The settings are well-formed but unusable — a `local` root the community may not write, an endpoint it will not reach, a bucket name the backend rejects. `details` names the member and the problem."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [member, problem]
      properties:
        member:
          type: string
          maxLength: 64
        problem:
          type: string
          maxLength: 500
  - code: vtc/storage/configs/create:authNotAllowed
    meaning: "`auth` does not fit the kind: present on a `local` config, absent on any other, or `ambient` on a `walrus` config (Walrus storage is paid for by a signer, which no ambient cloud identity provides)."
    retryable: false
  - code: vtc/storage/configs/create:unknownAccount
    meaning: "`auth.account` names no external account at the community's VTA that the community is bound to use."
    retryable: false

related:
  - vtc/storage/configs/update
  - vtc/storage/configs/retire
  - vtc/storage/configs/probe
  - vtc/storage/credentials/set
  - vtc/rooms/storage/assign
  - vtc/admin/actions/show
---

## Abstract

The **VTC Storage — Configs — Create** Trust Task adds a storage config: one named backend, with its settings and how the community authenticates to it. Rooms are then assigned to it by the community's rooms policy at creation, by a room's creator choosing among the configs that policy allows, or by [`vtc/rooms/storage/assign`](../../../../../vtc/rooms/storage/assign/0.1/spec.md).

**The request never carries a secret.** Authentication is one of three modes:

- **`vta-account`** (RECOMMENDED) names an external account at the community's VTA. The VTA holds the long-lived authority, and issues the community credentials scoped to one room's prefix for at most 15 minutes, every issuance audited there. The community holds nothing an administrator could take home. Creating and changing that account is consented at the VTA, by approvers signing with their own DIDs.
- **`ambient`** uses the cloud identity of the machine the community runs on, which the community downscopes to one room's prefix itself.
- **`sealed`** (discouraged) stores a long-lived credential on the community, set afterwards with [`vtc/storage/credentials/set`](../../../../../vtc/storage/credentials/set/0.1/spec.md). Until then the config reports its credential as `required` and cannot be assigned.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`.
2. **MUST** refuse an identifier in use, or ever used, with `exists`.
3. **MUST** refuse with `authNotAllowed` an `auth` that does not fit the kind (see the error code).
4. **MUST** validate `settings` beyond the schema — that a `local` root is a directory it can create owner-only files in, that an endpoint is reachable — and refuse with `invalidSettings` naming the member.
5. With `vta-account`, **SHOULD** confirm with its VTA that the account exists and that the community is bound to use it, and **MUST** refuse with `unknownAccount` when the VTA says it does not.
6. **MUST** store the config `active`, with `createdBy` and `createdAt`; and, when `isDefault` is true, **MUST** clear the flag on the previous default in the same write.
7. **MUST NOT** accept a credential through this task in any member, `ext` included.
8. **SHOULD** run [`vtc/storage/configs/probe`](../../../../../vtc/storage/configs/probe/0.1/spec.md) once the config exists and record the result in its health, and **SHOULD** let approvers see a probe of the proposed settings before they decide.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

**Second-administrator consent.** This operation changes where the community keeps its members' files or what it uses to reach them, so a community implementing the administrator action list **parks** it rather than executing it: it answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../../../vtc/admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked payload itself, exactly once, when its threshold is met, re-running every check in this specification against the community as it is then, and answers on the original `threadId` with this task's `#response` or a `trust-task-error`. An approval is signed by the approver's own DID; nobody approves their own request. Where the community has no other administrator to consent and runs in a single-administrator mode, it may waive the consent on its own stated terms. Whether, and by how many, an operation must be approved is the community's policy; this specification describes the model it is designed for.

## Definitions

- `StorageConfig`, `StorageSettings`, `StorageAuth` — see [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).
- **External account** — an account the community's VTA holds at a cloud or third-party service, and lets bound integrations use through short-lived credentials (the `external/accounts/*` family).

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An S3 bucket on a VTA account

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/create/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000008ff",
  "payload": {
    "id": "eu-s3-primary",
    "label": "EU — S3 (main account)",
    "settings": {
      "kind": "s3",
      "region": "eu-west-1",
      "bucket": "northwind-rooms-eu"
    },
    "auth": {
      "mode": "vta-account",
      "account": "eu-s3-primary"
    },
    "capacityBytes": 1099511627776,
    "reason": "Board rooms must stay in the EU."
  }
}
```

### Walrus, paid for by the VTA's Sui signer

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/create/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000008fe",
  "payload": {
    "id": "walrus-main",
    "label": "Walrus — public ciphertext",
    "settings": {
      "kind": "walrus",
      "relayUrl": "https://relay.walrus.example",
      "aggregatorUrl": "https://aggregator.walrus.example",
      "epochs": 26,
      "extendBeforeEpochs": 2
    },
    "auth": {
      "mode": "vta-account",
      "account": "walrus-signer"
    },
    "reason": "For public-interest rooms that want replicated storage."
  }
}
```

## Response

When the creation executes, the community answers on the request's `threadId` with the sub-schema reachable via `$anchor: "response"`: the stored config. Before that, a parking community answers with `trust-task-next-step` (see [Authorization](#authorization)). A refusal is a `trust-task-error`.

### Created

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/create/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000008ff",
  "payload": {
    "config": {
      "id": "eu-s3-primary",
      "label": "EU — S3 (main account)",
      "settings": {
        "kind": "s3",
        "region": "eu-west-1",
        "bucket": "northwind-rooms-eu"
      },
      "auth": {
        "mode": "vta-account",
        "account": "eu-s3-primary"
      },
      "capacityBytes": 1099511627776,
      "state": "active",
      "isDefault": false,
      "createdBy": "did:example:administrator",
      "createdAt": "2026-10-01T09:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A config identifier, a label, backend settings, an authentication mode with an external account name, an optional capacity and a reason. No secret: a producer **MUST NOT** place a credential in any member, and a community refuses one rather than storing it.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The config links the community to a cloud account and bucket, or to a Sui address on a public ledger in the case of Walrus — where every storage payment is publicly visible and attributable to that address.

### Retention

The config persists until retired, and is kept while any blob names it.

### Consent/purpose

The purpose is to offer the community's rooms a place to keep their files. A `walrus` config makes every file stored on it public ciphertext, permanently, and removal from it cryptographic rather than physical; a community **SHOULD** make that visible wherever a room's creator or owner chooses or is shown the config.
