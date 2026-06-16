---
slug: vta/contexts/did-templates/delete
version: "1.0"
title: VTA Context DID-Template — Delete
summary: A context administrator (or super-admin) removes a context-scoped template by name.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - delete
  - context
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Context administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Template removal is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific context administrator for the audit record, so transport-independent producer identity is required.
errorCodes:
  - code: vta/contexts/did-templates/delete:notFound
    meaning: No template with this name in the context.
    retryable: false
related:
  - vta/contexts/did-templates/create
  - vta/contexts/did-templates/get
  - vta/contexts/did-templates/update
  - vta/contexts/did-templates/list
  - vta/contexts/did-templates/render
  - vta/did-templates/delete
---

## Abstract

The **VTA Context DID-Template — Delete** Trust Task removes a DID template **scoped to a context** on a Verifiable Trust Agent by name. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces. A *context administrator* (or a super-administrator) names the template to remove; the VTA requires the template to exist in the context, deletes it, and confirms the removal.

Context-scoped templates are visible and manageable within a single context and are managed by that context's admin. This contrasts with the **global** family, which is visible to every context on the VTA and managed only by super-administrators. To delete a global template instead, use [`vta/did-templates/delete`](../../../../did-templates/delete/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the context administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/contexts/did-templates/delete/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the context the template is scoped to, and `payload.name` with the resource id of the template to remove.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer is neither an administrator of `payload.contextId` nor a super-administrator, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../../SPEC.md#83-standard-error-codes)).
3. Where no template with `payload.name` exists in `payload.contextId`, respond with `vta/contexts/did-templates/delete:notFound`.
4. On success, remove the template from the context scope and return the confirmation [response](#response) echoing `name` with `deleted: true`.

## Definitions

* **Context administrator.** The party invoking the task; identified by `issuer`. Holds the Admin role for `payload.contextId` on the VTA. A super-administrator MAY also invoke the task.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in the create spec's `payload.schema.json`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/delete/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Remove a context-scoped template

```json
{
  "id": "4e5f6071-8293-74a5-b6c7-d8e9f0a1b2c3",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/delete/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:10:00Z",
  "payload": {
    "contextId": "primary",
    "name": "messaging-bridge"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-16T09:10:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/delete/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload confirms the removal.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/contexts/did-templates/delete:notFound` error.

### The confirmation

Response to the request example:

```json
{
  "id": "5f607182-9394-75b6-c7d8-e9f0a1b2c3d4",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/delete/1.0#response",
  "threadId": "4e5f6071-8293-74a5-b6c7-d8e9f0a1b2c3",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:10:01Z",
  "payload": {
    "name": "messaging-bridge",
    "deleted": true
  }
}
```

## Security & Privacy

**Privileged write, audited.** Removing a template changes what the VTA will mint for future integrations in the context, so the task is restricted to the context's administrators (and super-administrators) and the VTA records the writer DID. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**Scope is enforced server-side.** The VTA authorises the write against `payload.contextId`; a context admin cannot delete a template outside the contexts they administer, and a request that does not name an existing template in the context is rejected rather than silently succeeding.

**Removal is the revocation mechanism.** Deleting a template stops the VTA minting that integration shape going forward; it does not affect integrations already provisioned from it. The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
