---
slug: vtc/roles/delete
version: "0.1"
title: VTC Roles — Delete
summary: A community administrator deletes a custom administrative role from a Verifiable Trust Community — refused for built-in roles and for any role an ACL entry still holds.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - vtc
  - role
  - custom-role
  - administration
parties:
  - role: Community administrator
    requirement: REQUIRED
    member: issuer
  - role: VTC
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Deleting a role removes a term from the community's authority vocabulary and is typically executed only after other administrators approve it; it must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A replayed deletion would remove a role re-defined since, under a request its approvers judged against the earlier vocabulary. Placing the document in time bounds the duplicate-execution window.
sideEffects:
  level: mutating
  rationale: "Removes a custom role record that no entry holds; recoverable by vtc/roles/define with the same name. Because a held role cannot be deleted, no entry's authority changes."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The request names one role and a reason; the response echoes the name. A refusal for a role in use discloses a holder count, never a subject.
retention:
  class: durable
  rationale: The accepted document is the record that the community withdrew a role from its vocabulary, and on whose authority.
errorCodes:
  - code: vtc/roles/delete:builtInRole
    meaning: The name is a built-in role, which cannot be deleted.
    retryable: false
  - code: vtc/roles/delete:notFound
    meaning: No custom role of this name exists.
    retryable: false
  - code: vtc/roles/delete:inUse
    meaning: At least one ACL entry holds the role. Move each holder to another role (acl/change-role) or revoke it first; a held role is never deleted out from under its holders.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [holders]
      properties:
        holders:
          type: integer
          minimum: 1
related:
  - vtc/roles/define
  - vtc/roles/list
  - vtc/roles/show
  - acl/change-role
  - acl/list
---

## Abstract

The **VTC Roles — Delete** Trust Task removes a custom administrative role from a Verifiable Trust Community (VTC). It is refused for a built-in role and for any role that an ACL entry still holds: deleting a held role would leave entries naming a role the VTC no longer recognises, which confers no authority — a silent revocation by a task that does not look like one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTC) **MUST**:

1. Refuse a built-in role with `builtInRole`, and an unknown name with `notFound`.
2. Refuse a role held by any ACL entry — including an expired entry it still stores, and one with a pending grant of the role — with `inUse`, reporting the holder count in `details.holders`. There is no override.
3. Check the holder count and delete the role atomically with respect to grants of it, so that a grant racing the deletion either completes first (and the deletion is refused) or is refused as `roleNotRecognized`.
4. On acceptance, remove the role record and return its name.

## Authorization

**Authority: the same authority that defines roles.** Deleting a custom role administers the community's authority vocabulary exactly as defining one does, so it rests on the capabilities that [`vtc/roles/define`](../../define/0.1/spec.md) rests on — at a VTC implementing the administrative-roles model, the capability that assigns roles together with the one that administers approvals — and such a VTC executes it once an N-of-M approval among its community administrators has been satisfied. The threshold and approver set are the VTC's policy, not declared here. Because a held role cannot be deleted, the task never reduces any subject's authority, which is why it needs no bound against the requester's own capabilities beyond that.

The `proof` identifies the requester so its stored entry can be read and the deletion attributed; it is not the authorization.

## Definitions

* **Holder.** An ACL entry whose `role` is the named role.
* **Custom role.** A community-defined role (`builtIn: false`); only these are deletable.

## Request

The community administrator sends the VTC a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Delete an unused role

```json
{
  "id": "urn:uuid:4e5f6a7b-8c9d-4e0f-8a1b-3c4d5e6f7a8b",
  "type": "https://trusttasks.org/spec/vtc/roles/delete/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T16:00:00Z",
  "payload": {
    "name": "events-team",
    "reason": "Events moved to a separate community."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-02T16:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z8dEExampleProofValue"
  }
}
```

## Response

The VTC answers with a document of type `https://trusttasks.org/spec/vtc/roles/delete/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json).

* `deleted` — the name of the deleted role.

Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### Role deleted

```json
{
  "id": "urn:uuid:5f6a7b8c-9d0e-4f1a-9b2c-4d5e6f7a8b9c",
  "type": "https://trusttasks.org/spec/vtc/roles/delete/0.1#response",
  "threadId": "urn:uuid:4e5f6a7b-8c9d-4e0f-8a1b-3c4d5e6f7a8b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T16:45:00Z",
  "payload": {
    "deleted": "events-team"
  }
}
```

### Refused: the role is still held

```json
{
  "id": "urn:uuid:6a7b8c9d-0e1f-4a2b-8c3d-5e6f7a8b9c0d",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:4e5f6a7b-8c9d-4e0f-8a1b-3c4d5e6f7a8b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-02T16:00:01Z",
  "payload": {
    "code": "vtc/roles/delete:inUse",
    "message": "events-team is held by 3 entries; change or revoke them first.",
    "retryable": false,
    "details": { "holders": 3 }
  }
}
```

## Security & Privacy

### Data carried

The request names one role and an optional reason; the response echoes the name. The `inUse` refusal discloses how many entries hold the role — a count, never the subjects, which a requester entitled to see them obtains through `acl/list/0.2` filtered by role. A producer **MUST NOT** put personal data in `reason` or `ext`.

### Correlation

The task concerns community vocabulary rather than any person, so there is little to correlate beyond the requester's identity and timing. The holder count in a refusal reveals the approximate size of a role's membership to the requester.

### Retention

The accepted document is **durable**: it records that the community withdrew a role, on whose request and under which approval. A VTC **SHOULD** retain it for the audit life of the community's authority records, so that a later reader can tell why a role referenced by historical entries no longer exists.

### Consent/purpose

The data is collected to maintain the community's authority vocabulary. It **SHOULD NOT** be reused for other purposes. Whether a deletion requires approval by other administrators is the VTC's policy and is not declared here.
