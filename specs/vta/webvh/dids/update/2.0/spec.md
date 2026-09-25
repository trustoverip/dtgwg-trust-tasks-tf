---
slug: vta/webvh/dids/update
version: "2.0"
title: WebVH DID Update
summary: A caller asks a Verifiable Trust Agent to publish a new entry in a did:webvh log whose update key the agent holds — services, witnesses, watchers, TTL — with a preview of exactly what the entry will do. Keys are not changed here; they change only through the key-role tasks.
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - did-webvh
  - did-management
  - update
  - preview
  - delegated-execution
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: Verifiable Trust Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The agent signs a log entry that is thereafter part of a public, append-only identity history, using a key the caller does not hold and cannot be given. It must be able to attribute the request to a party authorized over the subject, and a bearer token proves only who opened the channel.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An update appends to the DID's log, and appended entries cannot be withdrawn. An update that cannot be placed in time therefore writes an unretractable entry at a moment nobody chose.
sideEffects:
  level: destructive
  rationale: "Every entry under pre-rotation moves the DID's update key to its committed successor — rotation of the sole controlling key, and so authority-shifting, whether or not the caller asked for it. The published entry is also permanent."
consequences:
  - "Appending the entry rotates this DID's update key to its committed successor. The current update key stops being able to authorize further changes."
  - "Refreshes the pre-rotation commitments that will authorize the next rotation."
  - "Appends to a public, append-only log. The new state resolves immediately and the entry cannot be withdrawn."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: "The request carries a proposed DID document, public by construction; the response carries the preview or the published entry."
retention:
  class: durable
  rationale: "The published entry is permanent; the preview and audit row record who proposed it and who approved it."
errorCodes:
  - code: vta/webvh/dids/update:notFound
    meaning: The agent holds no update key for this DID, or the caller cannot reach its context.
    retryable: false
  - code: vta/webvh/dids/update:versionConflict
    meaning: The DID's latest entry no longer matches `expectedVersionId`. The caller SHOULD re-read and re-apply its edits.
    retryable: false
  - code: vta/webvh/dids/update:invalidDocument
    meaning: The document is not a valid DID document for this subject (for example, its `id` does not match `did`).
    retryable: false
  - code: vta/webvh/dids/update:keyMembersManaged
    meaning: "The document changes `verificationMethod`, a verification relationship or `keyRoles`. Keys change only through the key-role tasks; `details.members` names the members that differ."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        members:
          type: array
          items: { type: string }
  - code: "vta/webvh/dids:previewStale"
    meaning: "The `previewId` is unknown, expired, already applied, based on an entry the log has moved past, or for a different request."
    retryable: false
  - code: "vta/webvh/dids:stepUpRequired"
    meaning: "The VTA's policy requires a step-up bound to this change's preview; `details.stepUpRequest` carries it and `details.preview` the plan."
    retryable: true
  - code: "vta/webvh/dids:preRotationRequired"
    meaning: "The update would leave a durable node identity with no committed successor update key."
    retryable: false
  - code: "vta/webvh/dids:notKeyRoleIdentity"
    meaning: "The DID was not created with key roles. Create a new identity with key roles instead."
    retryable: false
related:
  - vta/webvh/dids/keys/list
  - vta/webvh/dids/keys/add
  - vta/webvh/dids/rotate-keys
  - task-consent/request
  - policy/evaluate
---

## Abstract

The **WebVH DID Update** Trust Task is how a caller who does **not** hold a `did:webvh`
update key asks the agent that does to publish a change to anything **but the DID's keys**:
services, witnesses, watchers, TTL, `alsoKnownAs`.

The caller proposes; it cannot sign a log entry, is never given the key, and nothing it sends
carries authority. The agent validates the request, previews it, decides whether its owner's
policy permits it, signs, and publishes.

## What changed from 1.0, and why it is a new major version

**Breaking.** 1.0 accepted any document, so a caller permitted to add a service endpoint could
also rewrite `verificationMethod` and `assertionMethod` — adding a key of its own choosing to the
role that signs the community's credentials, with no role check, no custodian record and none of
the staging, approval or audit the key-role tasks require. 2.0 closes that:

1. **Key members are managed.** A document whose `verificationMethod`, verification
   relationships or `keyRoles` differ from the current entry's is refused with
   `vta/webvh/dids/update:keyMembersManaged`. Keys change through
   [`keys/add`](../../keys/add/1.0/spec.md), [`rotate-keys/2.0`](../../rotate-keys/2.0/spec.md),
   [`keys/retire`](../../keys/retire/1.0/spec.md) and [`keys/revoke`](../../keys/revoke/1.0/spec.md).
2. **Preview and bound approval.** `dryRun` returns the executor-computed preview 1.0's prose
   demanded but its payload could not carry, and `previewId` binds an approval to it
   ([conventions §3](../../../../_shared/0.3/CONVENTIONS.md#3-approval)).
3. **Pre-rotation cannot be switched off** for a durable node identity.

**Client impact.** `pnm did-mgmt dids edit` sends 1.0 today and lets an operator edit the whole
document, including keys. Under 2.0 the editor must lock the key members (showing them read-only,
with a pointer to the key-role commands) and render the VTA's preview before saving.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/vta/webvh/dids/update/2.0`, with the agent as `recipient` and a
verifiable `proof`, and **MUST** populate `expectedVersionId` whenever the edit was authored
against something the caller read.

A conforming **consumer** (the agent) **MUST**:

1. Verify the `proof` and that the issuer is authorized over `did` (context `admin` or
   `super-admin`).
2. Reject a payload carrying members this schema does not define.
3. Refuse the update when `expectedVersionId` is present and no longer matches →
   `versionConflict`.
3a. Refuse a DID that is not a key-role identity with `vta/webvh/dids:notKeyRoleIdentity`
   ([conventions §10](../../../../_shared/0.3/CONVENTIONS.md#10-only-key-role-identities)).
   A pre-role DID has no key-role tasks to change its keys through, so accepting updates to it
   would leave it permanently outside them; the remedy is a new identity.
4. Refuse a document whose key members differ from the current entry's →
   `keyMembersManaged`, naming them in `details.members`. Member order within a relationship
   is not a difference.
5. On `dryRun`, compute the preview by running the handler it would run, and on `previewId`
   apply exactly that plan ([conventions §3.1](../../../../_shared/0.3/CONVENTIONS.md#31-preview-first)).
6. Derive the task's side-effect class from the handler it is about to invoke, not from this
   specification's declaration (SPEC §7.3 item 13).

## Payload

`payload.did` (REQUIRED) — the subject.
`payload.document` (OPTIONAL) — the new DID document, with its key members unchanged.
`payload.preRotationCount` (OPTIONAL) — commitments to publish.
`payload.witnesses`, `payload.watchers`, `payload.ttl` (OPTIONAL) — as in 1.0.
`payload.label` (OPTIONAL) — operator-facing audit label.
`payload.expectedVersionId` (OPTIONAL) — optimistic-concurrency precondition.
`payload.dryRun`, `payload.previewId` (OPTIONAL) — preview and bound apply.
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Request

```json
{
  "id": "urn:uuid:2f7c1a90-4b6e-4d21-9a55-1c3e8b7d0f43",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/2.0",
  "issuer": "did:key:z6MkCallerExample",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-09-25T10:12:00Z",
  "payload": {
    "did": "did:webvh:QmSCIDExample:example.com:acme",
    "document": {
      "@context": ["https://www.w3.org/ns/did/v1"],
      "id": "did:webvh:QmSCIDExample:example.com:acme",
      "service": [
        { "id": "#files", "type": "FileStore", "serviceEndpoint": "https://files.example.com/acme" }
      ]
    },
    "expectedVersionId": "3-QmPriorEntryHashExample",
    "dryRun": true,
    "label": "add file store"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-09-25T10:12:00Z",
    "verificationMethod": "did:key:z6MkCallerExample#z6MkCallerExample",
    "proofPurpose": "authentication",
    "proofValue": "z4Xq7WExampleProofValueForWebvhUpdateRequest"
  }
}
```

(The document is abbreviated; a real request carries the key members unchanged.)

## Response

```json
{
  "id": "urn:uuid:8d1b6e34-7f92-4c05-b3a1-6e0d29c4f8b8",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/update/2.0#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkCallerExample",
  "issuedAt": "2026-09-25T10:12:01Z",
  "payload": {
    "did": "did:webvh:QmSCIDExample:example.com:acme",
    "outcome": "preview",
    "preview": {
      "previewId": "pv_5e6f7a8b9c0d1e2f3a4b",
      "baseVersionId": "3-QmPriorEntryHashExample",
      "expiresAt": "2026-09-25T10:42:01Z",
      "updateKeyRotates": true,
      "changes": [ { "op": "rotateUpdateKey", "role": "update" } ],
      "document": { "id": "did:webvh:QmSCIDExample:example.com:acme" }
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a proposed DID document — public by construction — and control members; the
response, the preview or the published entry. Closing the key members is the security point of this
version: the least-privileged task that can publish an entry can no longer change who speaks for the
identity.

### Correlation

Everything in the document is published to every resolver; a caller **SHOULD NOT** add service
endpoints or `alsoKnownAs` values that reveal more about the node's deployment than its peers need
(VTI-KEY-150).

### Retention

The entry is permanent; the audit row records who proposed and who approved it.

### Consent/purpose

The consent surface renders the executor's preview — including the update-key rotation the entry
carries — instead of a document diff that hides it.
