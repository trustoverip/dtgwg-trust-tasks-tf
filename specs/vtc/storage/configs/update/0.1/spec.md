---
slug: vtc/storage/configs/update
version: "0.1"
title: "VTC Storage — Configs — Update"
summary: A room-hosting administrator changes a storage config's label, capacity, default flag, authentication or the settings that do not locate stored files. Its kind and location are fixed while files live there. Parked for another administrator's consent.
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
    The request changes how the community reaches a backend holding members' files, or which account it uses. It is parked for other administrators' consent and must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Members are replaced wholesale, so a replayed earlier update would silently revert a config to settings administrators have since changed.
sideEffects:
  level: mutating
  rationale: >-
    Replaces the given members of one config. Reversible by another update; nothing stored on the config moves or is deleted.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries settings, an authentication mode and an external account name — never a secret. The response echoes the stored config.
retention:
  class: durable
  rationale: >-
    The change persists; the accepted document is the record of who changed how the community reaches a store and why.
errorCodes:
  - code: vtc/storage/configs/update:notFound
    meaning: "The community has no storage config with this identifier."
    retryable: false
  - code: vtc/storage/configs/update:kindImmutable
    meaning: "`settings.kind` differs from the config's kind. A config's kind never changes; create a new config and migrate rooms to it."
    retryable: false
  - code: vtc/storage/configs/update:locationImmutable
    meaning: "The update changes a setting that locates stored blobs — a `local` root; an `s3` endpoint, region, bucket or prefix; a `gcs` bucket or prefix — while blobs name this config. Changing it would strand them. Create a new config and migrate."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [members]
      properties:
        members:
          type: array
          maxItems: 8
          items:
            type: string
            maxLength: 64
  - code: vtc/storage/configs/update:invalidSettings
    meaning: "The settings are well-formed but unusable. `details` names the member and the problem."
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
  - code: vtc/storage/configs/update:authNotAllowed
    meaning: "`auth` does not fit the config's kind; see vtc/storage/configs/create."
    retryable: false
  - code: vtc/storage/configs/update:unknownAccount
    meaning: "`auth.account` names no external account at the community's VTA that the community is bound to use."
    retryable: false
  - code: vtc/storage/configs/update:notActive
    meaning: "The config is `draining` or `retired`; only its label can be changed."
    retryable: false
related:
  - vtc/storage/configs/create
  - vtc/storage/configs/get
  - vtc/storage/credentials/set
  - vtc/admin/actions/show
---

## Abstract

The **VTC Storage — Configs — Update** Trust Task changes a storage config. Each member given replaces the stored one wholesale; a member absent is unchanged. Two things it will not change while files live on the config: its **kind**, ever, and the settings that **locate** stored blobs — a directory, a bucket, a prefix, a region or endpoint — because a blob is found by its config and its digest, and moving the config under it would strand it. To move files, create another config and [migrate](../../../../../vtc/rooms/storage/migrate/0.1/spec.md).

What it does change, such as Walrus endpoints, the epochs bought or the extension margin, a capacity, the default flag, or the authentication, applies from the next operation. Changing `auth` from `sealed` to another mode discards the stored credential.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.admin` with `permissionDenied`, and an unknown config with `notFound`.
2. **MUST** refuse a `settings.kind` other than the config's with `kindImmutable`, and a change to a locating setting while any blob names the config with `locationImmutable`, naming the members.
3. **MUST** refuse any change but `label` to a config that is not `active` with `notActive`.
4. **MUST** validate as [`vtc/storage/configs/create`](../../../../../vtc/storage/configs/create/0.1/spec.md) does (`invalidSettings`, `authNotAllowed`, `unknownAccount`).
5. **MUST** discard, not retain, a stored `sealed` credential when `auth` changes away from `sealed`.
6. **MUST** rebuild its connection to the backend with the new settings before the next operation, and **MUST NOT** accept a credential through this task in any member.
7. **MUST** record `updatedBy` and `updatedAt`, and audit the previous and new values with `reason`.

## Authorization

The authority this task presupposes is the **`vtc.rooms.admin` capability at the community**, a capability introduced with the files-in-data-rooms work so that a community can hand room hosting — storage, limits and usage — to someone who is not a full administrator ([`vtc/roles/define`](../../../../../vtc/roles/define/0.1/spec.md) can place it in a role's ceiling). It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

This is the **host's** authority, not a room's. Like [`vtc/rooms/list`](../../../../../vtc/rooms/list/0.1/spec.md), and for the reason that specification gives, it lives under `vtc/` rather than `rooms/`: every `rooms/*` task is authorized by credentials the room issued and never by the host's access control, and this one is the opposite. No room-issued authority is required or accepted, and nothing here needs, or could use, a room key.

**Second-administrator consent.** This operation changes where the community keeps its members' files or what it uses to reach them, so a community implementing the administrator action list **parks** it rather than executing it: it answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../../../vtc/admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked payload itself, exactly once, when its threshold is met, re-running every check in this specification against the community as it is then, and answers on the original `threadId` with this task's `#response` or a `trust-task-error`. An approval is signed by the approver's own DID; nobody approves their own request. Where the community has no other administrator to consent and runs in a single-administrator mode, it may waive the consent on its own stated terms. Whether, and by how many, an operation must be approved is the community's policy; this specification describes the model it is designed for.

## Definitions

- **Locating setting** — `root` (`local`); `endpoint`, `region`, `bucket`, `prefix` (`s3`); `bucket`, `prefix` (`gcs`). Walrus has none: a Walrus blob is located by its blob identifier, which does not depend on the relay or aggregator used to reach it.
- `StorageSettings`, `StorageAuth` — see [`vtc/_shared/0.1/room-storage.schema.json`](../../../../../vtc/_shared/0.1/room-storage.schema.json).

## Request

A room-hosting administrator (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Move from an ambient role to a VTA account, and raise the capacity

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-10T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000009ff",
  "payload": {
    "configId": "eu-s3-primary",
    "auth": {
      "mode": "vta-account",
      "account": "eu-s3-primary"
    },
    "capacityBytes": 2199023255552,
    "reason": "Stop relying on the instance role; credentials now come from the VTA."
  }
}
```

## Response

When the update executes, the community answers on the request's `threadId` with the sub-schema reachable via `$anchor: "response"`: the stored config. Before that, a parking community answers with `trust-task-next-step` (see [Authorization](#authorization)). A refusal is a `trust-task-error`.

### Updated

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/storage/configs/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000009ff",
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
      "capacityBytes": 2199023255552,
      "state": "active",
      "isDefault": false,
      "createdBy": "did:example:administrator",
      "createdAt": "2026-10-01T09:00:00Z",
      "updatedBy": "did:example:administrator",
      "updatedAt": "2026-10-10T11:20:00Z"
    }
  }
}
```

### Refused: the bucket holds files

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-10T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000009fe",
  "payload": {
    "code": "vtc/storage/configs/update:locationImmutable",
    "message": "eu-s3-primary holds 4210 blobs; its bucket cannot change. Create a new config and migrate.",
    "retryable": false,
    "details": {
      "members": [
        "bucket"
      ]
    }
  }
}
```

## Security & Privacy

### Data carried

A config identifier, the members to change and a reason. No secret.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered.

### Retention

The change persists. A credential discarded by a change of `auth` is destroyed, not archived.

### Consent/purpose

The purpose is operating the community's file storage.
